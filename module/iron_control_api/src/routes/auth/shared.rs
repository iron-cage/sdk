//! Shared types and state for authentication endpoints

use crate::error::ValidationError;
use crate::jwt_auth::JwtSecret;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;

/// Shared authentication state
#[derive(Clone)]
pub struct AuthState {
  pub jwt_secret: Arc<JwtSecret>,
  pub db_pool: Pool<Sqlite>,
  pub rate_limiter: crate::rate_limiter::LoginRateLimiter,
}

impl AuthState {
  /// Create new auth state
  ///
  /// # Arguments
  ///
  /// * `jwt_secret_key` - Secret key for JWT signing
  /// * `database_url` - Database connection string
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new(jwt_secret_key: String, database_url: &str) -> Result<Self, sqlx::Error> {
    let db_pool = SqlitePool::connect(database_url).await?;

    // Run migration 003 (users table) if not already applied
    let migration_003_completed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_003_completed'",
    )
    .fetch_one(&db_pool)
    .await?;

    if migration_003_completed == 0 {
      let migration_003 =
        include_str!("../../../../iron_token_manager/migrations/003_create_users_table.sql");
      sqlx::raw_sql(migration_003).execute(&db_pool).await?;
    }

    // Migration 006: Create user audit log table
    let migration_006_completed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_006_completed'",
    )
    .fetch_one(&db_pool)
    .await?;

    if migration_006_completed == 0 {
      let migration_006 =
        include_str!("../../../../iron_token_manager/migrations/006_create_user_audit_log.sql");
      sqlx::raw_sql(migration_006).execute(&db_pool).await?;
    }

    // Migration 007: Create token blacklist table
    let migration_007_completed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_007_completed'",
    )
    .fetch_one(&db_pool)
    .await?;

    if migration_007_completed == 0 {
      let migration_007 =
        include_str!("../../../../iron_token_manager/migrations/007_create_blacklist_table.sql");
      sqlx::raw_sql(migration_007).execute(&db_pool).await?;
    }

    // Migration 019: Add account lockout fields
    let migration_019_completed: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_019_completed'",
    )
    .fetch_one(&db_pool)
    .await?;

    if migration_019_completed == 0 {
      let migration_019 =
        include_str!("../../../../iron_token_manager/migrations/019_add_account_lockout_fields.sql");
      sqlx::raw_sql(migration_019).execute(&db_pool).await?;
    }

    Ok(Self {
      jwt_secret: Arc::new(JwtSecret::new(jwt_secret_key)),
      db_pool,
      rate_limiter: crate::rate_limiter::LoginRateLimiter::new(),
    })
  }
}

/// Login request body
///
/// Per Protocol 007:
/// ```json
/// {
///   "email": "developer@example.com",
///   "password": "secure_password_123"
/// }
/// ```
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
  pub email: String,
  pub password: String,
}

impl LoginRequest {
  /// Maximum email length for DoS prevention
  const MAX_EMAIL_LENGTH: usize = 255;

  /// Maximum password length for DoS prevention
  const MAX_PASSWORD_LENGTH: usize = 1000;

  /// Validate login request parameters
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - email is empty or whitespace-only
  /// - password is empty or whitespace-only
  /// - email exceeds 255 characters
  /// - password exceeds 1000 characters
  pub fn validate(&self) -> Result<(), ValidationError> {
    // Validate email is not empty
    if self.email.trim().is_empty() {
      return Err(ValidationError::MissingField("email".to_string()));
    }

    // Validate password is not empty
    if self.password.trim().is_empty() {
      return Err(ValidationError::MissingField("password".to_string()));
    }

    // Validate email length
    if self.email.len() > Self::MAX_EMAIL_LENGTH {
      return Err(ValidationError::TooLong {
        field: "email".to_string(),
        max_length: Self::MAX_EMAIL_LENGTH,
      });
    }

    // Validate password length
    if self.password.len() > Self::MAX_PASSWORD_LENGTH {
      return Err(ValidationError::TooLong {
        field: "password".to_string(),
        max_length: Self::MAX_PASSWORD_LENGTH,
      });
    }

    Ok(())
  }
}

/// User information in login response
///
/// Per Protocol 007:
/// ```json
/// {
///   "id": "user_abc123",
///   "email": "developer@example.com",
///   "role": "developer",
///   "name": "John Doe"
/// }
/// ```
#[derive(Debug, Serialize, Deserialize )]
pub struct UserInfo {
  pub id: String,
  pub email: String,
  pub role: String,
  pub name: String,
}

impl UserInfo {
  /// Create UserInfo from JWT claims and database user
  ///
  /// # Arguments
  ///
  /// * `claims` - JWT access token claims
  /// * `user` - Database user record
  pub fn from_claims_and_user(claims: &crate::jwt_auth::AccessTokenClaims, user: &crate::user_auth::User) -> Self {
    Self {
      id: user.id.to_string(),
      email: user.username.clone(),
      role: claims.role.clone(),
      name: user.name.clone().unwrap_or_else( || user.username.clone() ),
    }
  }

  /// Create UserInfo from database user only
  ///
  /// # Arguments
  ///
  /// * `user` - Database user record
  pub fn from_user(user: &crate::user_auth::User) -> Self {
    Self {
      id: user.id.to_string(),
      email: user.email.clone(),
      role: user.role.clone(),
      name: user.name.clone().unwrap_or_else( || user.username.clone() ),
    }
  }
}


/// Login response body
///
/// Per Protocol 007:
/// ```json
/// {
///   "user_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer",
///   "expires_in": 2592000,
///   "expires_at": "2026-01-08T09:00:00Z",
///   "refresh_token": "refresh_abc123def456...",
///   "user": { ... }
/// }
/// ```
#[derive(Debug, Serialize, Deserialize )]
pub struct LoginResponse {
  pub user_token: String,
  pub token_type: String,
  pub expires_in: u64,
  pub expires_at: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refresh_token: Option<String>,
  pub user: UserInfo,
}

/// Error response body
///
/// Per Protocol 007:
/// ```json
/// {
///   "error": {
///     "code": "AUTH_INVALID_CREDENTIALS",
///     "message": "Invalid email or password",
///     "details": { ... }
///   }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
  pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
  pub code: String,
  pub message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub details: Option<serde_json::Value>,
}

/// Refresh response body
///
/// Per Protocol 007:
/// ```json
/// {
///   "user_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
///   "token_type": "Bearer",
///   "expires_in": 2592000,
///   "expires_at": "2026-01-08T15:00:00Z",
///   "user": { ... }
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
  pub user_token: String,
  pub token_type: String,
  pub expires_in: u64,
  pub expires_at: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  pub refresh_token: Option<String>,
  pub user: UserInfo,
}

/// Validate response body
///
/// Per Protocol 007:
/// ```json
/// {
///   "valid": true,
///   "user": { ... },
///   "expires_at": "2026-01-08T09:00:00Z",
///   "expires_in": 2500000
/// }
/// ```
///
/// Or for invalid token:
/// ```json
/// {
///   "valid": false,
///   "reason": "TOKEN_EXPIRED",
///   "expired_at": "2025-12-09T09:00:00Z"
/// }
/// ```
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum ValidateResponse {
  Valid {
    valid: bool,
    user: UserInfo,
    expires_at: String,
    expires_in: u64,
  },
  Invalid {
    valid: bool,
    reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expired_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    revoked_at: Option<String>,
  },
}
