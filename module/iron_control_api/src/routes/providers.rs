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
  /// Updated human-readable description
  pub description: Option<String>,
  /// Enable or disable this key
  pub is_enabled: Option<bool>,
  /// Spending cap in USD (None = don't change, Some(None) = remove cap, Some(Some(x)) = set cap)
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

/// Check if user has `ManageProviderKeys` permission
fn check_manage_provider_keys(role_str: &str) -> Result<(), impl IntoResponse> {
  let role = iron_types::Role::from_str(role_str).map_err(|_| {
    (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "error": format!("Invalid role: {role_str}") })),
    )
  })?;
  let checker = PermissionChecker::new();
  if checker.has_permission(role, Permission::ManageProviderKeys) {
    Ok(())
  } else {
    Err((
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({
        "error": "Insufficient permissions: ManageProviderKeys required"
      })),
    ))
  }
}

/// Error response for disabled feature
fn feature_disabled_response() -> impl IntoResponse {
  (
    StatusCode::SERVICE_UNAVAILABLE,
    Json(serde_json::json!({
      "error": "AI Provider Keys feature is disabled. Set IRON_SECRETS_MASTER_KEY environment variable to enable."
    })),
  )
}

/// POST /api/providers
///
/// Create new AI provider key
pub async fn create_provider_key(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
  JsonBody(request): JsonBody<CreateProviderKeyRequest>,
) -> impl IntoResponse {
  // RBAC: require ManageProviderKeys permission
  if let Err(resp) = check_manage_provider_keys(&claims.role) {
    return resp.into_response();
  }

  // Check if crypto is enabled
  let Some(crypto) = &state.crypto else {
    return feature_disabled_response().into_response();
  };

  // Validate request
  if let Err(validation_error) = request.validate() {
    return (
      StatusCode::BAD_REQUEST,
      Json(serde_json::json!({
        "error": validation_error.to_string()
      })),
    )
      .into_response();
  }

  // Parse provider type
  let provider = match request.provider.as_str() {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    _ => {
      return (
        StatusCode::BAD_REQUEST,
        Json(serde_json::json!({
          "error": "Invalid provider type"
        })),
      )
        .into_response();
    }
  };

  // Encrypt API key
  let Ok(encrypted) = crypto.encrypt(&request.api_key) else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to encrypt API key"
      })),
    )
      .into_response();
  };

  // Create masked key for response
  let masked_key = mask_api_key(&request.api_key);

  let Ok(keys) = state.storage.get_keys_by_provider(provider).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to get provider keys"
      })),
    )
      .into_response();
  };
  let key_id = if keys.is_empty() {
    match state
      .storage
      .create_key(
        provider,
        &encrypted.ciphertext_base64(),
        &encrypted.nonce_base64(),
        request.base_url.as_deref(),
        request.description.as_deref(),
        &claims.sub,
      )
      .await
    {
      Ok(id) => id,
      Err(_) => {
        return (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({
            "error": "Failed to create provider key"
          })),
        )
          .into_response();
      }
    }
  } else {
    match state
      .storage
      .update_key(
        keys[0],
        provider,
        &encrypted.ciphertext_base64(),
        &encrypted.nonce_base64(),
        request.base_url.as_deref(),
        request.description.as_deref(),
        &claims.sub,
      )
      .await
    {
      Ok(id) => id,
      Err(_) => {
        return (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({
            "error": "Failed to update provider key"
          })),
        )
          .into_response();
      }
    }
  };

  // Get metadata for response
  let Ok(metadata) = state.storage.get_key_metadata(key_id).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to retrieve provider key metadata"
      })),
    )
      .into_response();
  };

  (
    StatusCode::CREATED,
    Json(ProviderKeyResponse::from_metadata(
      metadata,
      masked_key,
      vec![],
    )),
  )
    .into_response()
}

/// GET /api/providers
///
/// List all provider keys for authenticated user
pub async fn list_provider_keys(
  State(state): State<ProvidersState>,
  AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
  let Ok(keys) = state.storage.list_keys(&claims.sub).await else {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({
        "error": "Failed to fetch provider keys"
      })),
    )
      .into_response();
  };

  // For each key, fetch assigned projects and build response
  let mut responses: Vec<ProviderKeyResponse> = Vec::with_capacity(keys.len());

  for meta in keys {
    // Fetch projects assigned to this key
    let assigned_projects = state
      .storage
      .get_key_projects(meta.id)
      .await
      .unwrap_or_default();

    responses.push(ProviderKeyResponse::from_metadata(
      meta,
      "***",
      assigned_projects,
    ));
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

  // Update fields if provided
  if let Some(ref description) = request.description {
    if state
      .storage
      .update_description(key_id, Some(description))
      .await
      .is_err()
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to update description"
        })),
      )
        .into_response();
    }
  }

  if let Some(ref base_url) = request.base_url {
    let url = if base_url.is_empty() {
      None
    } else {
      Some(base_url.as_str())
    };
    if state.storage.update_base_url(key_id, url).await.is_err() {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to update base_url"
        })),
      )
        .into_response();
    }
  }

  if let Some(is_enabled) = request.is_enabled {
    if state.storage.set_enabled(key_id, is_enabled).await.is_err() {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to update is_enabled"
        })),
      )
        .into_response();
    }
  }

  if let Some(spending_cap) = request.spending_cap_usd {
    let cap_microdollars = spending_cap.map(usd_to_microdollars);
    if state
      .storage
      .set_spending_cap(key_id, cap_microdollars)
      .await
      .is_err()
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "Failed to update spending_cap"
        })),
      )
        .into_response();
    }
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

  // Unassign from project
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
