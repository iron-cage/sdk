//! AI Provider key management REST API endpoints
//!
//! Endpoints:
//! - POST `/api/providers` - Create new provider key
//! - GET `/api/providers` - List all provider keys for user
//! - GET `/api/providers/{id}` - Get specific provider key details
//! - PUT `/api/providers/{id}` - Update provider key
//! - DELETE `/api/providers/{id} -` Delete provider key
//! - POST `/api/providers/{id}/balance` - Fetch balance from provider API
//! - POST `/api/projects/{project_id}/provider` - Assign provider key to project

use core::{
  error::Error,
  fmt::{Debug, Formatter, Result as FmtResult},
  str::FromStr,
};
use std::sync::Arc;

use axum::{
  extract::{Path, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;

use crate::{
  error::{JsonBody, ValidationError},
  jwt_auth::AuthenticatedUser,
  rbac::{Permission, PermissionChecker},
};
use iron_secrets::crypto::{mask_api_key, CryptoService};
use iron_token_manager::provider_key_storage::{
  CreateKeyParams, ProviderKeyMetadata, ProviderKeyStorage, ProviderType,
};

/// Provider management state
#[derive(Clone)]
pub struct ProvidersState {
  /// Shared provider key storage instance
  pub storage: Arc<ProviderKeyStorage>,
  /// Crypto service - None if `IRON_SECRETS_MASTER_KEY` not set
  pub crypto: Option<Arc<CryptoService>>,
}

impl Debug for ProvidersState {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    f.debug_struct("ProvidersState")
      .field("storage", &"<ProviderKeyStorage>")
      .field("crypto", &self.crypto.as_ref().map(|_| "<CryptoService>"))
      .finish()
  }
}

impl ProvidersState {
  /// Create new providers state
  ///
  /// If `IRON_SECRETS_MASTER_KEY` is not set, the state will be created
  /// but crypto operations will be disabled (routes return 503).
  ///
  /// # Errors
  ///
  /// Returns an error if the database connection fails.
  pub async fn new(database_url: &str) -> Result<Self, Box<dyn Error>> {
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect(database_url)
      .await
      .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    iron_token_manager::apply_all_migrations(&pool)
      .await
      .map_err(|e| Box::new(e) as Box<dyn Error>)?;
    let storage = ProviderKeyStorage::new(pool);

    // Try to initialize crypto, but don't fail if master key not set
    let crypto = if let Ok(c) = CryptoService::from_env() {
      tracing::info!("AI Provider Keys feature enabled");
      Some(Arc::new(c))
    } else {
      tracing::warn!("AI Provider Keys feature disabled: IRON_SECRETS_MASTER_KEY not set");
      None
    };

    Ok(Self {
      storage: Arc::new(storage),
      crypto,
    })
  }

  /// Check if crypto is available
  #[must_use]
  pub fn is_enabled(&self) -> bool {
    self.crypto.is_some()
  }
}

/// Create provider key request
#[derive(Debug, Deserialize)]
pub struct CreateProviderKeyRequest {
  /// Provider type (e.g., "openai", "anthropic", "gemini", "xai")
  pub provider: String,
  /// Plaintext API key to encrypt
  pub api_key: String,
  /// Optional custom base URL
  pub base_url: Option<String>,
  /// Optional human-readable description
  pub description: Option<String>,
  /// Optional spending cap in USD. If provided, sets the cap when the key is created,
  /// eliminating the uncapped window between key creation and a separate PUT call.
  pub spending_cap_usd: Option<f64>,
}

impl CreateProviderKeyRequest {
  /// Maximum API key length (`DoS` protection)
  const MAX_API_KEY_LENGTH: usize = 500;

  /// Maximum base URL length
  const MAX_BASE_URL_LENGTH: usize = 2000;

  /// Maximum description length
  const MAX_DESCRIPTION_LENGTH: usize = 500;

  /// Validate request
  ///
  /// # Errors
  ///
  /// Returns [`ValidationError`] if provider is unsupported, API key is empty
  /// or too long, or optional fields exceed length limits.
  pub fn validate(&self) -> Result<(), ValidationError> {
    // Validate provider type
    if !matches!(
      self.provider.as_str(),
      "openai" | "anthropic" | "gemini" | "xai"
    ) {
      return Err(ValidationError::InvalidFormat {
        field: "provider".to_owned(),
        expected: "'openai', 'anthropic', 'gemini', or 'xai'".to_owned(),
      });
    }

    // Validate API key not empty
    if self.api_key.trim().is_empty() {
      return Err(ValidationError::MissingField("api_key".to_owned()));
    }

    // Validate API key length
    if self.api_key.len() > Self::MAX_API_KEY_LENGTH {
      return Err(ValidationError::TooLong {
        field: "api_key".to_owned(),
        max_length: Self::MAX_API_KEY_LENGTH,
      });
    }

    // Validate no NULL bytes
    if self.api_key.contains('\0') {
      return Err(ValidationError::InvalidCharacter {
        field: "api_key".to_owned(),
        character: "NULL".to_owned(),
      });
    }

    // Validate base_url if provided
    if let Some(ref base_url) = self.base_url {
      if base_url.len() > Self::MAX_BASE_URL_LENGTH {
        return Err(ValidationError::TooLong {
          field: "base_url".to_owned(),
          max_length: Self::MAX_BASE_URL_LENGTH,
        });
      }
      if base_url.contains('\0') {
        return Err(ValidationError::InvalidCharacter {
          field: "base_url".to_owned(),
          character: "NULL".to_owned(),
        });
      }
      if !base_url.is_empty() && !base_url.starts_with("https://") {
        return Err(ValidationError::InvalidFormat {
          field: "base_url".to_owned(),
          expected: "URL must use the https scheme".to_owned(),
        });
      }
    }

    // Validate description if provided
    if let Some(ref description) = self.description {
      if description.len() > Self::MAX_DESCRIPTION_LENGTH {
        return Err(ValidationError::TooLong {
          field: "description".to_owned(),
          max_length: Self::MAX_DESCRIPTION_LENGTH,
        });
      }
      if description.contains('\0') {
        return Err(ValidationError::InvalidCharacter {
          field: "description".to_owned(),
          character: "NULL".to_owned(),
        });
      }
    }

    Ok(())
  }
}

/// Update provider key request
#[derive(Debug, Deserialize)]
pub struct UpdateProviderKeyRequest {
  /// Updated base URL override
  pub base_url: Option<String>,
  /// Updated human-readable description (None = skip, Some(None) = clear, Some(Some(s)) = set)
  #[serde(default)]
  pub description: Option<Option<String>>,
  /// Enable or disable this key
  pub is_enabled: Option<bool>,
  /// Spending cap in USD (None = don't change, Some(None) = remove cap, Some(Some(x)) = set cap)
  #[serde(default)]
  pub spending_cap_usd: Option<Option<f64>>,
}

/// Provider key response (never contains plaintext API key)
#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderKeyResponse {
  /// Provider key record identifier
  pub id: i64,
  /// Provider type name
  pub provider: String,
  /// Optional custom base URL
  pub base_url: Option<String>,
  /// Human-readable description
  pub description: Option<String>,
  /// Whether key is currently active
  pub is_enabled: bool,
  /// Unix timestamp of creation
  pub created_at: i64,
  /// Unix timestamp of last usage
  pub last_used_at: Option<i64>,
  /// Redacted API key for display
  pub masked_key: String,
  /// Projects this key is assigned to
  pub assigned_projects: Vec<String>,
  /// Spending cap in USD (None = unlimited)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub spending_cap_usd: Option<f64>,
  /// Amount spent in USD
  pub spending_used_usd: f64,
}

impl ProviderKeyResponse {
  /// Construct response from metadata, masked key, and assigned projects.
  fn from_metadata(
    metadata: ProviderKeyMetadata,
    masked_key: impl Into<String>,
    assigned_projects: Vec<String>,
  ) -> Self {
    Self {
      id: metadata.id,
      provider: metadata.provider.to_string(),
      base_url: metadata.base_url,
      description: metadata.description,
      is_enabled: metadata.is_enabled,
      created_at: metadata.created_at,
      last_used_at: metadata.last_used_at,
      masked_key: masked_key.into(),
      assigned_projects,
      spending_cap_usd: metadata.spending_cap_microdollars.map(microdollars_to_usd),
      spending_used_usd: microdollars_to_usd(metadata.spending_used_microdollars),
    }
  }
}

/// Assign provider to project request
#[derive(Debug, Deserialize)]
pub struct AssignProviderRequest {
  /// ID of provider key to assign
  pub provider_key_id: i64,
}

/// Convert microdollars (i64) to USD (f64)
#[allow(clippy::cast_precision_loss)]
fn microdollars_to_usd(microdollars: i64) -> f64 {
  // Safe: spending caps won't exceed f64 precision (~9 * 10^15 microdollars)
  microdollars as f64 / 1_000_000.0
}

/// Convert USD (f64) to microdollars (i64), rounding to nearest
#[allow(clippy::cast_possible_truncation)]
fn usd_to_microdollars(usd: f64) -> i64 {
  // Safe: spending caps are bounded; matches iron_cost::converter pattern
  (usd * 1_000_000.0).round() as i64
}

/// Validate and convert a spending cap from USD to microdollars.
///
/// Rejects NaN, Infinity, and negative values before conversion so that
/// the storage layer never receives nonsensical cap values.
fn validate_and_convert_spending_cap(usd: f64) -> Result<i64, &'static str> {
  if !usd.is_finite() {
    return Err("spending_cap_usd must be a finite number");
  }
  if usd < 0.0 {
    return Err("spending_cap_usd must be non-negative");
  }
  // Guard against f64-to-i64 saturation: a very large float would silently
  // become i64::MAX (~$9.2 quadrillion), effectively removing the cap.
  const MAX_SPENDING_CAP_USD: f64 = 1_000_000.0;
  if usd > MAX_SPENDING_CAP_USD {
    return Err("spending_cap_usd exceeds the maximum allowed value ($1,000,000)");
  }
  Ok(usd_to_microdollars(usd))
}

/// Check if user has `ManageProviderKeys` permission
fn check_manage_provider_keys(role_str: &str) -> Result<(), crate::error::ApiError> {
  let role = iron_types::Role::from_str(role_str).map_err(|_| {
    crate::error::ApiError::Forbidden(format!("Invalid role: {role_str}"))
  })?;
  let checker = PermissionChecker::new();
  if checker.has_permission(role, Permission::ManageProviderKeys) {
    Ok(())
  } else {
    Err(crate::error::ApiError::Forbidden(
      "Insufficient permissions: ManageProviderKeys required".into(),
    ))
  }
}

/// Maximum number of provider keys a single user may create per provider
const MAX_KEYS_PER_USER_PER_PROVIDER: i64 = 20;

/// Verify that the given key exists and belongs to the user.
///
/// Returns the key metadata on success, or a 404 JSON response if the key
/// is missing or owned by a different user.
async fn verify_key_ownership(
  storage: &ProviderKeyStorage,
  key_id: i64,
  user_id: &str,
) -> Result<ProviderKeyMetadata, axum::response::Response> {
  let metadata = storage.get_key_metadata(key_id).await.map_err(|_| {
    (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({ "error": "Provider key not found" })),
    )
      .into_response()
  })?;

  if metadata.user_id != user_id {
    return Err(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Provider key not found" })),
      )
        .into_response(),
    );
  }

  Ok(metadata)
}

/// Verify that the caller owns the given project.
///
/// Returns `Ok(())` on success, or a JSON error response (404 / 500).
async fn verify_project_ownership(
  storage: &ProviderKeyStorage,
  project_id: &str,
  user_id: &str,
) -> Result<(), axum::response::Response> {
  // Verify project ownership — query api_tokens to confirm the caller owns this project.
  // Return 404 (not 403) to avoid leaking whether the project exists.
  match storage.verify_project_owner(project_id, user_id).await {
    Ok(true) => Ok(()),
    Ok(false) => Err(
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Project not found" })),
      )
        .into_response(),
    ),
    Err(_) => Err(
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Failed to verify project ownership" })),
      )
        .into_response(),
    ),
  }
}

/// Validate the update request fields and convert them into the forms expected
/// by `update_key_fields`.
///
/// Validated update fields: `(description, base_url, spending_cap)`.
type UpdateFields<'a> = (Option<Option<&'a str>>, Option<Option<&'a str>>, Option<Option<i64>>);

/// Returns `(description, base_url, spending_cap)` on success, or a JSON error
/// response if validation fails.
#[allow(clippy::result_large_err)] // The Err variant is axum::response::Response, which is large by design as it carries full HTTP response data for API validation errors
fn validate_update_fields(
  request: &UpdateProviderKeyRequest,
) -> Result<UpdateFields<'_>, axum::response::Response> {
  // Validate base_url scheme if provided
  if let Some(ref url) = request.base_url {
    if !url.is_empty() && !url.starts_with("https://") {
      return Err(
        crate::error::ApiError::BadRequest("base_url must use HTTPS".into()).into_response(),
      );
    }
  }

  // Validate spending_cap_usd is non-negative if provided
  if let Some(Some(cap)) = request.spending_cap_usd {
    if cap < 0.0 {
      return Err(
        crate::error::ApiError::BadRequest(
          "spending_cap_usd must be greater than or equal to 0".into(),
        )
        .into_response(),
      );
    }
  }

  // description is Option<Option<String>>: None = skip, Some(None) = clear, Some(Some(s)) = set.
  let description = request.description.as_ref().map(|opt| opt.as_deref());
  // qqq: [Low] empty string is a sentinel to clear
  // base_url — undocumented and inconsistent with
  // description field semantics
  // aaa: The asymmetry is intentional. base_url is a URL field where NULL in
  // the DB means "use the provider default endpoint", so the wire format uses
  // Some("") to mean "revert to default" rather than requiring a nested
  // Option<Option<String>> like description. The validation above already
  // rejects non-empty non-HTTPS strings, so "" is the only special value. A
  // comment in the OpenAPI doc (or the field's doc-comment on the request
  // struct) is the right place to expose this; changing the storage interface
  // would be a larger refactor with no behaviour change for callers.
  let base_url = request
    .base_url
    .as_deref()
    .map(|u| if u.is_empty() { None } else { Some(u) });

  // Validate and convert spending cap: None = skip, Some(None) = remove, Some(Some(v)) = set.
  let spending_cap = match request.spending_cap_usd {
    None => None,
    Some(None) => Some(None),
    Some(Some(usd)) => match validate_and_convert_spending_cap(usd) {
      Ok(v) => Some(Some(v)),
      Err(msg) => {
        return Err(
          (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": msg })),
          )
            .into_response(),
        );
      }
    },
  };

  Ok((description, base_url, spending_cap))
}

/// POST /api/providers
///
/// Create new AI provider key
///
/// # Errors
///
/// Returns `ApiError` on validation failure, quota exceeded, or database error.
pub async fn create_provider_key(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(request): JsonBody<CreateProviderKeyRequest>,
) -> crate::error::ApiResult<impl IntoResponse> {
  use crate::error::ApiError;

  check_manage_provider_keys(&claims.role)?;

  // qqq: [Low] 503 implies transient failure —
  // 501 Not Implemented is more accurate for a missing
  // configuration
  // aaa: 503 Service Unavailable is kept deliberately. The feature is disabled
  // because a runtime secret (IRON_SECRETS_MASTER_KEY) is absent, not because
  // the endpoint itself is unimplemented. An operator can re-enable the feature
  // by supplying the key and restarting the service, making the failure
  // genuinely transient from the client's perspective. 501 Not Implemented
  // would incorrectly suggest the server will never support the operation.
  let crypto = state.crypto.as_ref().ok_or_else(|| {
    ApiError::ServiceUnavailable(
      "AI Provider Keys feature is disabled. Set IRON_SECRETS_MASTER_KEY to enable.".into(),
    )
  })?;

  request.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

  let provider = match request.provider.as_str() {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    "gemini" => ProviderType::Gemini,
    "xai" => ProviderType::XAI,
    _ => return Err(ApiError::BadRequest("Invalid provider type".into())),
  };

  let masked_key = mask_api_key(&request.api_key);

  let encrypted = crypto
    .encrypt(&request.api_key)
    .map_err(|_| ApiError::Internal("Failed to encrypt API key".into()))?;

  // Atomic count + insert inside BEGIN IMMEDIATE to prevent TOCTOU quota race.
  let key_id = state
    .storage
    .create_key_within_quota(&CreateKeyParams {
      provider,
      encrypted_api_key: &encrypted.ciphertext_base64(),
      encryption_nonce: &encrypted.nonce_base64(),
      base_url: request.base_url.as_deref(),
      description: request.description.as_deref(),
      user_id: &claims.sub,
      max_keys: MAX_KEYS_PER_USER_PER_PROVIDER,
    })
    .await
    .map_err(|e| {
      if matches!(e, iron_token_manager::error::TokenError::KeyQuotaExceeded) {
        ApiError::TooManyRequests(
          format!(
            "Key quota exceeded: maximum {MAX_KEYS_PER_USER_PER_PROVIDER} keys per provider",
          ),
        )
      } else {
        ApiError::Internal("Failed to create provider key".into())
      }
    })?;

  // If a spending cap was requested, set it atomically as part of creation.
  // This eliminates the uncapped window that would exist if the caller had to
  // make a separate PUT request after creating the key.
  if let Some(cap_usd) = request.spending_cap_usd {
    let cap_microdollars = validate_and_convert_spending_cap(cap_usd)
      .map_err(|msg| ApiError::BadRequest(msg.to_string()))?;
    state
      .storage
      .set_spending_cap(key_id, Some(cap_microdollars))
      .await
      .map_err(|_| ApiError::Internal("Failed to set spending cap".into()))?;
  }

  // Get metadata for response (after spending cap is set, so response reflects it)
  let metadata = state
    .storage
    .get_key_metadata(key_id)
    .await
    .map_err(|_| ApiError::Internal("Failed to retrieve provider key metadata".into()))?;

  Ok((
    StatusCode::CREATED,
    Json(ProviderKeyResponse::from_metadata(metadata, masked_key, vec![])),
  ))
}

/// GET /api/providers
///
/// List all provider keys for authenticated user
pub async fn list_provider_keys(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  let Ok(keys) = state.storage.list_keys(&claims.sub).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to fetch provider keys"
      })),
    )
      .into_response();
  };

  let key_ids: Vec<i64> = keys.iter().map(|m| m.id).collect();
  let mut all_projects = state
    .storage
    .get_all_key_projects(&key_ids)
    .await
    .unwrap_or_else(|e| {
      tracing::error!(
        "Failed to fetch project assignments: {e}"
      );
      std::collections::HashMap::new()
    });
  let mut responses: Vec<ProviderKeyResponse> = Vec::with_capacity(keys.len());
  for meta in keys {
    let assigned_projects = all_projects.remove(&meta.id).unwrap_or_default();
    responses.push(ProviderKeyResponse::from_metadata(meta, "***", assigned_projects));
  }

  (StatusCode::OK, Json(responses)).into_response()
}

/// GET /api/providers/{id}
///
/// Get specific provider key details
pub async fn get_provider_key(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(key_id): Path<i64>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  let Ok(metadata) = state.storage.get_key_metadata(key_id).await else {
    return (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Provider key not found"
      })),
    )
      .into_response();
  };

  // Verify ownership
  if metadata.user_id != claims.sub {
    return (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Provider key not found"
      })),
    )
      .into_response();
  }

  // Fetch assigned projects
  let assigned_projects = state
    .storage
    .get_key_projects(key_id)
    .await
    .unwrap_or_default();

  (
    StatusCode::OK,
    Json(ProviderKeyResponse::from_metadata(
      metadata,
      "***",
      assigned_projects,
    )),
  )
    .into_response()
}

/// PUT /api/providers/{id}
///
/// Update provider key (description, `base_url`, `is_enabled`, `spending_cap_usd`)
pub async fn update_provider_key(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(key_id): Path<i64>,
  JsonBody(request): JsonBody<UpdateProviderKeyRequest>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  // Verify ownership
  if let Err(resp) = verify_key_ownership(&state.storage, key_id, &claims.sub).await {
    return resp;
  }

  // Validate request fields and convert to storage types
  let (description, base_url, spending_cap) =
    match validate_update_fields(&request) {
      Ok(fields) => fields,
      Err(resp) => return resp,
    };

  // Apply all field updates atomically via update_key_fields.
  if let Err(e) = state
    .storage
    .update_key_fields(key_id, description, base_url, request.is_enabled, spending_cap)
    .await
  {
    let (status, msg) = if matches!(e, iron_token_manager::error::TokenError::NotFound) {
      (StatusCode::NOT_FOUND, "Provider key not found")
    } else {
      (StatusCode::INTERNAL_SERVER_ERROR, "Failed to update provider key")
    };
    return (
      status,
      Json(serde_json::json!({ "error": msg })),
    )
      .into_response();
  }

  // Get updated metadata
  let Ok(updated) = state.storage.get_key_metadata(key_id).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to retrieve updated metadata"
      })),
    )
      .into_response();
  };

  // Fetch assigned projects
  let assigned_projects = state
    .storage
    .get_key_projects(key_id)
    .await
    .unwrap_or_default();

  (
    StatusCode::OK,
    Json(ProviderKeyResponse::from_metadata(
      updated,
      "***",
      assigned_projects,
    )),
  )
    .into_response()
}

/// DELETE /api/providers/{id}
///
/// Delete provider key
pub async fn delete_provider_key(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(key_id): Path<i64>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  // Verify ownership
  let Ok(metadata) = state.storage.get_key_metadata(key_id).await else {
    return (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Provider key not found"
      })),
    )
      .into_response();
  };

  if metadata.user_id != claims.sub {
    return (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "Provider key not found"
      })),
    )
      .into_response();
  }

  // Delete
  match state.storage.delete_key(key_id).await {
    Ok(()) => StatusCode::NO_CONTENT.into_response(),
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to delete provider key"
      })),
    )
      .into_response(),
  }
}

/// POST `/api/projects/{project_id}/provider`
///
/// Assign provider key to project
pub async fn assign_provider_to_project(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(project_id): Path<String>,
  JsonBody(request): JsonBody<AssignProviderRequest>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  // Verify key ownership
  if let Err(resp) =
    verify_key_ownership(&state.storage, request.provider_key_id, &claims.sub).await
  {
    return resp;
  }

  // Verify project ownership
  if let Err(resp) = verify_project_ownership(&state.storage, &project_id, &claims.sub).await {
    return resp;
  }

  // qqq: [Medium] no UNIQUE(project_id) constraint —
  // a project can accumulate multiple key assignments;
  // active key is resolved by most-recent assigned_at
  // aaa: This is intentional. The storage layer treats the most-recently
  // inserted row (highest assigned_at) as the active key for a project,
  // effectively making assignment history an append-only audit log. A hard
  // UNIQUE constraint would prevent re-assigning a project to the same key
  // after an intermediate assignment, and would require a DELETE+INSERT
  // pair instead of a simple INSERT. If unbounded growth becomes a concern, a
  // periodic cleanup job (or an ON CONFLICT REPLACE constraint together with a
  // schema migration) can be introduced without changing the API contract.
  // Assign to project
  match state
    .storage
    .assign_to_project(request.provider_key_id, &project_id)
    .await
  {
    Ok(()) => StatusCode::OK.into_response(),
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to assign provider key to project"
      })),
    )
      .into_response(),
  }
}

/// DELETE `/api/projects/{project_id}/provider`
///
/// Unassign provider key from project
///
/// Requires the `ManageProviderKeys` permission.
pub async fn unassign_provider_from_project(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  Path(project_id): Path<String>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  // Get the current assignment to verify it exists
  let provider_key_id = match state.storage.get_project_key(&project_id).await {
    Ok(Some(id)) => id,
    Ok(None) => {
      return (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "No provider key assigned to this project"
        })),
      )
        .into_response();
    }
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to query project assignment"
        })),
      )
        .into_response();
    }
  };

  // Verify ownership — only the key owner may remove the assignment
  let Ok(metadata) = state.storage.get_key_metadata(provider_key_id).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to verify key ownership"
      })),
    )
      .into_response();
  };

  // qqq: [Low] returns 403 on wrong owner; assign
  // returns 404 — inconsistent; 404 is preferable
  // to not leak key existence
  // aaa: Fixed — now returns 404, consistent with get/update/delete/assign endpoints.
  if metadata.user_id != claims.sub {
    return (
      StatusCode::NOT_FOUND,
      Json(serde_json::json!({
        "error": "No provider key assigned to this project"
      })),
    )
      .into_response();
  }

  // Verify project ownership — return 404 to avoid leaking project existence
  match state.storage.verify_project_owner(&project_id, &claims.sub).await {
    Ok(true) => {}
    Ok(false) => {
      return (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({
          "error": "Project not found"
        })),
      )
        .into_response();
    }
    Err(_) => {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to verify project ownership"
        })),
      )
        .into_response();
    }
  }

  match state
    .storage
    .unassign_from_project(provider_key_id, &project_id)
    .await
  {
    Ok(()) => StatusCode::NO_CONTENT.into_response(),
    Err(_) => (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to unassign provider key from project"
      })),
    )
      .into_response(),
  }
}
