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
use serde::{de::Deserializer, Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;

use crate::{
  error::{JsonBody, ValidationError},
  jwt_auth::AuthenticatedUser,
  rbac::{Permission, PermissionChecker},
};
use iron_secrets::crypto::{mask_api_key, CryptoService};
use iron_token_manager::provider_key_storage::{
  ProviderKeyMetadata, ProviderKeyStorage, ProviderType,
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
  /// Provider type (e.g., "openai", "anthropic")
  pub provider: String,
  /// Plaintext API key to encrypt
  pub api_key: String,
  /// Optional custom base URL
  pub base_url: Option<String>,
  /// Optional human-readable description
  pub description: Option<String>,
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
    if self.provider != "openai" && self.provider != "anthropic" {
      return Err(ValidationError::InvalidFormat {
        field: "provider".to_owned(),
        expected: "'openai' or 'anthropic'".to_owned(),
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
#[allow(clippy::option_option)] // Three-state semantics for spending_cap_usd
pub struct UpdateProviderKeyRequest {
  /// Updated base URL override
  pub base_url: Option<String>,
  /// Updated human-readable description (None = skip, Some(None) = clear, Some(Some(s)) = set)
  pub description: Option<Option<String>>,
  /// Enable or disable this key
  pub is_enabled: Option<bool>,
  /// Spending cap in USD (None = don't change, Some(None) = remove cap, Some(Some(x)) = set cap)
  ///
  /// JSON mapping:
  /// - field absent → `None` (don't change)
  /// - `"spending_cap_usd": null` → `Some(None)` (remove cap / unlimited)
  /// - `"spending_cap_usd": 10.0` → `Some(Some(10.0))` (set cap)
  #[serde(default, deserialize_with = "deserialize_nullable_f64")]
  pub spending_cap_usd: Option<Option<f64>>,
}

/// Deserialize a JSON field into `Option<Option<f64>>` with three-state semantics.
///
/// - Field absent (handled by `#[serde(default)]`) → `None`
/// - Field present as `null` → `Some(None)`
/// - Field present as number → `Some(Some(value))`
#[allow(clippy::option_option)] // Three-state semantics: absent vs null vs value
fn deserialize_nullable_f64<'de, D>(deserializer: D) -> Result<Option<Option<f64>>, D::Error>
where
  D: Deserializer<'de>,
{
  // If this function is called, the field was present in JSON.
  // Deserialize its value: null becomes None, number becomes Some(x).
  let value: Option<f64> = Option::deserialize(deserializer)?;
  Ok(Some(value))
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

  //qqq: [Low] 503 implies transient failure — 501 Not Implemented is more accurate for a missing configuration
  let crypto = state.crypto.as_ref().ok_or_else(|| {
    ApiError::ServiceUnavailable(
      "AI Provider Keys feature is disabled. Set IRON_SECRETS_MASTER_KEY to enable.".into(),
    )
  })?;

  request.validate().map_err(|e| ApiError::BadRequest(e.to_string()))?;

  let provider = match request.provider.as_str() {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    _ => return Err(ApiError::BadRequest("Invalid provider type".into())),
  };

  let masked_key = mask_api_key(&request.api_key);

  let encrypted = crypto
    .encrypt(&request.api_key)
    .map_err(|_| ApiError::Internal("Failed to encrypt API key".into()))?;

  // Atomic count + insert inside BEGIN IMMEDIATE to prevent TOCTOU quota race.
  let key_id = state
    .storage
    .create_key_within_quota(
      provider,
      &encrypted.ciphertext_base64(),
      &encrypted.nonce_base64(),
      request.base_url.as_deref(),
      request.description.as_deref(),
      &claims.sub,
      MAX_KEYS_PER_USER_PER_PROVIDER,
    )
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
  let mut all_projects = state.storage.get_all_key_projects(&key_ids).await.unwrap_or_default();
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

  // Validate base_url scheme if provided
  if let Some(ref url) = request.base_url {
    if !url.is_empty() && !url.starts_with("https://") {
      return crate::error::ApiError::BadRequest("base_url must use HTTPS".into()).into_response();
    }
  }

  // Validate spending_cap_usd is non-negative if provided
  if let Some(Some(cap)) = request.spending_cap_usd {
    if !cap.is_finite() {
      return crate::error::ApiError::BadRequest(
        "spending_cap_usd must be a finite number".into(),
      )
      .into_response();
    }
    if cap < 0.0 {
      return crate::error::ApiError::BadRequest(
        "spending_cap_usd must be greater than or equal to 0".into(),
      )
      .into_response();
    }
  }

  // Apply all field updates atomically in a single transaction.
  // description is Option<Option<String>>: None = skip, Some(None) = clear, Some(Some(s)) = set.
  let description = request.description.as_ref().map(|opt| opt.as_deref());
  //qqq: [Low] empty string is a sentinel to clear base_url — undocumented and inconsistent with description field semantics
  let base_url = request.base_url.as_deref().map(|u| if u.is_empty() { None } else { Some(u) });
  let spending_cap = request
    .spending_cap_usd
    .map(|cap| cap.map(usd_to_microdollars));

  if state
    .storage
    .update_key_fields(key_id, description, base_url, request.is_enabled, spending_cap)
    .await
    .is_err()
  {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to update provider key"
      })),
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
  let Ok(metadata) = state
    .storage
    .get_key_metadata(request.provider_key_id)
    .await
  else {
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

  // Verify project ownership — query api_tokens to confirm the caller owns this project.
  // Return 404 (not 403) to avoid leaking whether the project exists.
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

  //qqq: [Medium] no UNIQUE(project_id) constraint — a project can accumulate multiple key assignments; active key is resolved by most-recent assigned_at
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

  //qqq: [Low] returns 403 on wrong owner; assign returns 404 — inconsistent; 404 is preferable to not leak key existence
  if metadata.user_id != claims.sub {
    return (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({
        "error": "You do not own the key assigned to this project"
      })),
    )
      .into_response();
  }

  // qqq: BOLA — no project ownership check (same as assign_provider_to_project).
  // Both Admin and Manager have ManageProviderKeys. Add ownership guard
  // once projects are first-class entities.
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
