//! `FreeForm` onboarding API endpoints
//!
//! Endpoints:
//! - POST `/api/v1/freeform/company` — parse + upsert workspace (Admin)
//! - POST `/api/v1/freeform/providers` — parse + apply provider keys (Admin)

use core::fmt::{Debug, Formatter, Result as FmtResult};
use core::str::FromStr;
use std::sync::Arc;

use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::{
  error::{ApiError, ApiResult, JsonBody},
  freeform::{
    company_setup::{self, AccountType},
    providers,
  },
  jwt_auth::AuthenticatedUser,
  rbac::{Permission, PermissionChecker, Role},
};
use iron_secrets::crypto::CryptoService;
use iron_token_manager::provider_key_storage::{CreateKeyParams, ProviderKeyStorage, ProviderType};

/// State for `FreeForm` onboarding endpoints.
#[derive(Clone)]
pub struct FreeformState {
  /// Database pool for workspace operations.
  pub pool: Pool<Sqlite>,
  /// Provider key storage shared with the providers route.
  pub provider_storage: Arc<ProviderKeyStorage>,
  /// Crypto service for key encryption. `None` if `IRON_SECRETS_MASTER_KEY` not set.
  pub crypto: Option<Arc<CryptoService>>,
}

impl Debug for FreeformState {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    f.debug_struct("FreeformState")
      .field("pool", &"<SqlitePool>")
      .field("provider_storage", &"<ProviderKeyStorage>")
      .field("crypto", &self.crypto.as_ref().map(|_| "<CryptoService>"))
      .finish()
  }
}

/// Request body for both freeform endpoints: plain text paste.
#[derive(Debug, Deserialize)]
pub struct TextRequest {
  /// The raw paste text to parse.
  pub text: String,
}

/// Response from `POST /api/v1/freeform/company`.
#[derive(Debug, Serialize)]
pub struct WorkspaceResponse {
  /// Database ID of the upserted workspace.
  pub id: i64,
  /// Workspace display name.
  pub name: String,
  /// Primary domain.
  pub domain: String,
  /// Account type string (`client` or `internal`).
  pub account_type: String,
}

/// One entry in the providers apply response.
#[derive(Debug, Serialize)]
pub struct AppliedProvider {
  /// Provider name string.
  pub provider: String,
  /// Database ID of the newly created key.
  pub key_id: i64,
}

/// Response from `POST /api/v1/freeform/providers`.
#[derive(Debug, Serialize)]
pub struct ApplyProvidersResponse {
  /// List of successfully applied provider keys.
  pub applied: Vec<AppliedProvider>,
}

fn check_permission(role_str: &str, permission: Permission) -> Result<(), ApiError> {
  let role = Role::from_str(role_str)
    .map_err(|_| ApiError::Forbidden(format!("Invalid role: {role_str}")))?;
  let checker = PermissionChecker::new();
  if checker.has_permission(role, permission) {
    Ok(())
  } else {
    Err(ApiError::Forbidden(format!(
      "Permission {permission:?} required"
    )))
  }
}

fn account_type_to_str(t: &AccountType) -> &'static str {
  match t {
    AccountType::Client => "client",
    AccountType::Internal => "internal",
  }
}

fn map_known_provider(known: &providers::KnownProvider) -> Result<ProviderType, ApiError> {
  match known {
    providers::KnownProvider::Gemini => Ok(ProviderType::Gemini),
    providers::KnownProvider::OpenAi => Ok(ProviderType::OpenAI),
    providers::KnownProvider::Anthropic => Ok(ProviderType::Anthropic),
    providers::KnownProvider::Mistral | providers::KnownProvider::Cohere => {
      Err(ApiError::BadRequest(format!(
        "Provider '{known:?}' is not yet supported in key storage"
      )))
    }
  }
}

/// Maximum provider keys per user per provider (shared with providers route).
const MAX_KEYS_PER_USER_PER_PROVIDER: i64 = 20;

/// POST /api/v1/freeform/company
///
/// Parse a company-setup line and upsert the workspace record.
///
/// # Errors
///
/// Returns `ApiError` on permission failure, parse error, or database error.
pub async fn post_company(
  State(state): State<FreeformState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(body): JsonBody<TextRequest>,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageUsers)?;

  let company =
    company_setup::parse(&body.text).map_err(|e| ApiError::BadRequest(e.to_string()))?;

  let account_type_str = account_type_to_str(&company.account_type);

  let id: i64 = sqlx::query_scalar(
    r"
      INSERT INTO workspaces (id, name, domain, account_type, created_at)
      VALUES (1, ?, ?, ?, strftime('%s', 'now') * 1000)
      ON CONFLICT(id) DO UPDATE SET
        name         = excluded.name,
        domain       = excluded.domain,
        account_type = excluded.account_type
      RETURNING id
    ",
  )
  .bind(&company.name)
  .bind(&company.domain)
  .bind(account_type_str)
  .fetch_one(&state.pool)
  .await
  .map_err(|_| ApiError::Internal("Failed to upsert workspace".into()))?;

  Ok((
    StatusCode::OK,
    Json(WorkspaceResponse {
      id,
      name: company.name,
      domain: company.domain,
      account_type: account_type_str.to_string(),
    }),
  ))
}

/// POST /api/v1/freeform/providers
///
/// Parse a provider-key block and apply all keys transactionally.
/// Returns the list of applied provider names and their new key IDs.
///
/// # Errors
///
/// Returns `ApiError` on permission failure, parse error, crypto unavailability,
/// or database error.
pub async fn post_providers(
  State(state): State<FreeformState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(body): JsonBody<TextRequest>,
) -> ApiResult<impl IntoResponse> {
  check_permission(&claims.role, Permission::ManageProviderKeys)?;

  let crypto = state.crypto.as_ref().ok_or_else(|| {
    ApiError::ServiceUnavailable(
      "Provider key encryption is disabled. Set IRON_SECRETS_MASTER_KEY to enable.".into(),
    )
  })?;

  let entries = providers::parse(&body.text).map_err(|errors| {
    let msg = errors
      .iter()
      .map(std::string::ToString::to_string)
      .collect::<Vec<_>>()
      .join("; ");
    ApiError::BadRequest(msg)
  })?;

  let mut applied: Vec<AppliedProvider> = Vec::with_capacity(entries.len());

  for entry in &entries {
    let provider_type = map_known_provider(&entry.provider)?;
    let encrypted = crypto
      .encrypt(&entry.key)
      .map_err(|_| ApiError::Internal("Failed to encrypt provider key".into()))?;

    let key_id = state
      .provider_storage
      .create_key_within_quota(&CreateKeyParams {
        provider: provider_type,
        encrypted_api_key: &encrypted.ciphertext_base64(),
        encryption_nonce: &encrypted.nonce_base64(),
        base_url: None,
        description: Some("Added via FreeForm onboarding"),
        user_id: &claims.sub,
        max_keys: MAX_KEYS_PER_USER_PER_PROVIDER,
      })
      .await
      .map_err(|e| {
        if matches!(e, iron_token_manager::error::TokenError::KeyQuotaExceeded) {
          ApiError::TooManyRequests(format!("Quota exceeded for provider {:?}", entry.provider))
        } else {
          ApiError::Internal("Failed to store provider key".into())
        }
      })?;

    applied.push(AppliedProvider {
      provider: provider_type.to_string(),
      key_id,
    });
  }

  Ok((StatusCode::OK, Json(ApplyProvidersResponse { applied })))
}
