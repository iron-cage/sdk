//! Authentication REST API endpoints - Protocol 007 Implementation
//!
//! **Status:** Specification
//! **Version:** 1.0.0
//! **Last Updated:** 2025-12-10
//!
//! REST API endpoints for User authentication and User Token lifecycle management.
//!
//! Endpoints:
//! - POST /api/v1/auth/login - User login (email/password → User Token)
//! - POST /api/v1/auth/logout - User logout (invalidate User Token)
//! - POST /api/v1/auth/refresh - User Token refresh (extend expiration)
//! - POST /api/v1/auth/validate - User Token validation (check if valid)
//!
//! # Token Types
//!
//! - **User Token (JWT)**: For Control Panel access (30 days)
//! - **NOT IC Token**: IC Tokens are for agents (see Protocol 005)
//!
//! # Security
//!
//! - JWT signed with HS256 (HMAC SHA-256)
//! - Password hashing with bcrypt (cost factor 12)
//! - Rate limiting: 5 attempts per 5 minutes per IP
//! - Token blacklisting for logout
//! - Account lockout after 10 failed attempts

use crate::jwt_auth::{AuthenticatedUser, JwtSecret};
use crate::user_auth;
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use chrono::{DateTime, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite, SqlitePool};
use std::sync::Arc;
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader
};

/// Shared authentication state
#[derive(Clone)]
pub struct AuthState {
  pub jwt_secret: Arc<JwtSecret>,
  pub db_pool: Pool<Sqlite>,
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
        include_str!("../../../iron_token_manager/migrations/003_create_users_table.sql");
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
        include_str!("../../../iron_token_manager/migrations/006_create_user_audit_log.sql");
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
        include_str!("../../../iron_token_manager/migrations/007_create_blacklist_table.sql");
      sqlx::raw_sql(migration_007).execute(&db_pool).await?;
    }

    Ok(Self {
      jwt_secret: Arc::new(JwtSecret::new(jwt_secret_key)),
      db_pool,
    })
  }
}

// ============================================================================
// Login Endpoint - POST /api/v1/auth/login
// ============================================================================

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
  pub fn validate(&self) -> Result<(), String> {
    // Validate email is not empty
    if self.email.trim().is_empty() {
      return Err("email cannot be empty".to_string());
    }

    // Validate password is not empty
    if self.password.trim().is_empty() {
      return Err("password cannot be empty".to_string());
    }

    // Validate email length
    if self.email.len() > Self::MAX_EMAIL_LENGTH {
      return Err(format!(
        "email too long (max {} characters)",
        Self::MAX_EMAIL_LENGTH
      ));
    }

    // Validate password length
    if self.password.len() > Self::MAX_PASSWORD_LENGTH {
      return Err(format!(
        "password too long (max {} characters)",
        Self::MAX_PASSWORD_LENGTH
      ));
    }

    Ok(())
  }
}

/// User information in login response
///
/// Per Protocol 007:
/// ```json
/// {
///   "id": "user-abc123",
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
      id: format!("user-{}", user.id),
      email: user.username.clone(),
      role: claims.role.clone(),
      name: user.username.clone(), // TODO: Add name field to users table
    }
  }

  /// Create UserInfo from database user only
  ///
  /// # Arguments
  ///
  /// * `user` - Database user record
  pub fn from_user(user: &crate::user_auth::User) -> Self {
    Self {
      id: format!("user-{}", user.id),
      email: user.email.clone(),
      role: user.role.clone(),
      name: user.username.clone(), // TODO: Add name field to users table
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

/// POST /api/v1/auth/login
///
/// Authenticate user and return User Token (JWT, 30 days)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `request` - Login credentials (email, password)
///
/// # Returns
///
/// - 200 OK with User Token if authentication successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if credentials invalid
/// - 403 Forbidden if account disabled
/// - 429 Too Many Requests if rate limit exceeded
/// - 500 Internal Server Error if token generation or database query fails
///
/// # Security
///
/// - Password never logged or exposed in responses
/// - Rate limiting: 5 attempts per 5 minutes per IP
/// - Failed attempts logged for security monitoring
/// - Account lockout after 10 failed attempts (manual unlock by admin)
pub async fn login(
  State(state): State<AuthState>,
  Json(request): Json<LoginRequest>,
) -> impl IntoResponse {
  // Validate request
  if let Err(validation_error) = request.validate() {
    return (
      StatusCode::BAD_REQUEST,
      Json(ErrorResponse {
        error: ErrorDetail {
          code: "VALIDATION_ERROR".to_string(),
          message: validation_error,
          details: None,
        },
      }),
    )
      .into_response();
  }

  // TODO: Rate limiting check (5 attempts per 5 minutes per IP)
  // SELECT COUNT(*) FROM login_attempts WHERE ip = ? AND timestamp > NOW() - INTERVAL 5 MINUTE

  // Authenticate user against database
  // Note: Using username field for email (database schema uses username)
  let user = match user_auth::authenticate_user(&state.db_pool, &request.email, &request.password)
    .await
  {
    Ok(Some(user)) => user,
    Ok(None) => {
      // Invalid credentials - return 401
      // TODO: Log failed attempt for security monitoring
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_CREDENTIALS".to_string(),
            message: "Invalid email or password".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
    Err(err) => {
      // Database error - return 500
      tracing::error!("Database error during authentication: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "INTERNAL_ERROR".to_string(),
            message: "Authentication service unavailable".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  // Check if account is active
  if !user.is_active {
    return (
      StatusCode::FORBIDDEN,
      Json(ErrorResponse {
        error: ErrorDetail {
          code: "AUTH_ACCOUNT_DISABLED".to_string(),
          message: "Account has been disabled".to_string(),
          details: Some(serde_json::json!({
            "user_id": format!("user-{}", user.id)
          })),
        },
      }),
    )
      .into_response();
  }

  let user_id = user.id;
  let user_role = &user.role;

  // Generate User Token (30 days expiration)
  // Generate unique token ID for blacklist tracking
  let access_token_id = format!("access_{}_{}", user_id, chrono::Utc::now().timestamp());
  let user_token = match state.jwt_secret.generate_access_token(user_id, user_role, &access_token_id) {
    Ok(token) => token,
    Err(err) => {
      tracing::error!("Failed to generate user token: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_GENERATION_ERROR".to_string(),
            message: "Failed to generate access token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  // Generate refresh token (optional, future feature)
  // Per Protocol 007: refresh_token is optional
  let refresh_token_id = format!("refresh_{}_{}", user_id, chrono::Utc::now().timestamp());
  let refresh_token = match state
    .jwt_secret
    .generate_refresh_token(user_id, &refresh_token_id)
  {
    Ok(token) => Some(token),
    Err(err) => {
      tracing::warn!("Failed to generate refresh token: {}", err);
      None
    }
  };

  // Calculate expiration (30 days from now)
  let expires_in = 2592000u64; // 30 days in seconds
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

  (
    StatusCode::OK,
    Json(LoginResponse {
      user_token,
      token_type: "Bearer".to_string(),
      expires_in,
      expires_at: expires_at.to_rfc3339(),
      refresh_token,
      user: UserInfo::from_user(&user),
    }),
  )
    .into_response()
}

// ============================================================================
// Logout Endpoint - POST /api/v1/auth/logout
// ============================================================================

/// Logout request (User Token in Authorization header)
///
/// Per Protocol 007:
/// ```http
/// POST /api/v1/auth/logout
/// Authorization: Bearer <USER_TOKEN>
/// ```
///
/// No request body required.

/// POST /api/v1/auth/logout
///
/// Logout user by blacklisting User Token
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - User Token from Authorization header (extracted by middleware)
///
/// # Returns
///
/// - 204 No Content if logout successful
/// - 401 Unauthorized if token invalid or expired
///
/// # Implementation
///
/// - User Token added to blacklist (redis/database)
/// - Blacklist checked on every authenticated request
/// - Token remains blacklisted until original expiration time
/// - Multiple User Tokens per user supported (logout only invalidates current token)
///
/// # Side Effects
///
/// - Logged out User Token immediately invalid
/// - All subsequent requests with logged out token return 401 Unauthorized
/// - Other User Tokens for same user remain valid (if user has multiple sessions)
pub async fn logout(
  State(state): State<AuthState>,
  AuthenticatedUser( claims ): AuthenticatedUser
) -> impl IntoResponse {
//   let claims = match state.jwt_secret.verify_access_token(&bearer.token()) {
//     Ok(claims) => claims,
//     Err(_) => {
//       return (
//         StatusCode::UNAUTHORIZED,
//         Json(ErrorResponse {
//           error: ErrorDetail {
//             code: "AUTH_INVALID_TOKEN".to_string(),
//             message: "Invalid or expired authentication token".to_string(),
//             details: None,
//           },
//         }),
//       )
//         .into_response();
//     }
//   };

  let jti = claims.jti;
  let user_id = match i64::from_str_radix(&claims.sub, 10) {
    Ok(id) => id,
    Err(_) => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_TOKEN".to_string(),
            message: "Invalid or expired authentication token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  // INSERT INTO token_blacklist (jti, blacklisted_at, expires_at) VALUES (?, ?, ?)
  // - jti: Token ID from JWT claims
  // - blacklisted_at: Current timestamp
  // - expires_at: Original token expiration (for cleanup)
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(claims.exp as i64);
  match user_auth::add_token_to_blacklist(&state.db_pool, &jti, user_id, expires_at).await {
    Ok(()) => {},
    Err(err) => {
      tracing::error!("Failed to add token to blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to add token to blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  }

  // TODO: Log logout event for security monitoring
  // INSERT INTO user_audit_log (user_id, action, timestamp) VALUES (?, 'logout', ?)
  
  StatusCode::NO_CONTENT.into_response()
}

// ============================================================================
// Refresh Endpoint - POST /api/v1/auth/refresh
// ============================================================================

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
  pub user: UserInfo,
}

/// POST /api/v1/auth/refresh
///
/// Refresh User Token (extend expiration)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - Current User Token from Authorization header
///
/// # Returns
///
/// - 200 OK with new User Token
/// - 401 Unauthorized if token expired or invalid
///
/// # Refresh Window
///
/// - Can refresh anytime before expiration
/// - Recommended: Refresh when < 7 days remaining
/// - Old token invalidated when new token issued
/// - Atomic operation (old invalidated, new generated)
///
/// # CLI Behavior
///
/// - `iron_cli` automatically refreshes token when < 7 days remaining
/// - Refresh happens transparently during any CLI command
/// - User prompted to re-login if token expired
pub async fn refresh(
  State(state): State<AuthState>,
  TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
  // AuthenticatedUser( claims ): AuthenticatedUser

) -> impl IntoResponse {
  let claims = match state.jwt_secret.verify_refresh_token(&bearer.token()) {
    Ok(claims) => claims,
    Err(_) => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_TOKEN".to_string(),
            message: "Invalid or expired authentication token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  let blacklisted = match user_auth::get_blacklisted_token(&state.db_pool, &claims.jti).await {
    Ok(blacklisted) => blacklisted,
    Err(err) => {
      tracing::error!("Failed to check token blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to check token blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };
  if blacklisted.is_some() {
    return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "AUTH_INVALID_TOKEN".to_string(),
            message: "Invalid or expired authentication token".to_string(),
            details: None,
          },
        }),
      )
        .into_response();     
  }

  // Fetch user to get current role
  let user = match user_auth::get_user_by_id(&state.db_pool, &claims.sub).await {
    Ok(user) => user,
    Err(err) => {
      tracing::error!("Failed to get user by ID: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "USER_NOT_FOUND".to_string(),
            message: "User not found".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  };

  let user = match user {
    Some(user) => user,
    None => {
      return (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
          error: ErrorDetail {
          code: "USER_NOT_FOUND".to_string(),
          message: "User not found".to_string(),
          details: None,
        },
      }),
    )
      .into_response();
    }
  };

  // Generate new User Token (30 days)
  let new_token_id = format!("refresh_{}_{}", user.id, chrono::Utc::now().timestamp());
  let new_user_token = state.jwt_secret.generate_access_token(user.id, &user.role, &new_token_id).unwrap();

  // Blacklist old User Token (atomic operation)
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(claims.exp as i64);
  match user_auth::add_token_to_blacklist(&state.db_pool, &claims.jti, user.id, expires_at).await {
    Ok(()) => {},
    Err(err) => {
      tracing::error!("Failed to add token to blacklist: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
          error: ErrorDetail {
            code: "TOKEN_BLACKLIST_ERROR".to_string(),
            message: "Failed to add token to blacklist".to_string(),
            details: None,
          },
        }),
      )
        .into_response();
    }
  }

  // TODO: Calculate expiration (30 days from now)
  let expires_in = 2592000u64; // 30 days in seconds
  let expires_at = chrono::Utc::now() + chrono::Duration::seconds(expires_in as i64);

  // Placeholder response
  (
    StatusCode::OK,
    Json(RefreshResponse {
      user_token: new_user_token,
      token_type: "Bearer".to_string(),
      expires_in: expires_in,
      expires_at: expires_at.to_rfc3339(),
      user: UserInfo::from_user(&user),
    }),
  )
    .into_response()
}

// ============================================================================
// Validate Endpoint - POST /api/v1/auth/validate
// ============================================================================

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

/// POST /api/v1/auth/validate
///
/// Validate User Token (check if valid)
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `user_token` - User Token from Authorization header
///
/// # Returns
///
/// - 200 OK with validation result (always returns 200, even for invalid tokens)
///
/// # Note
///
/// Validate returns 200 OK even for invalid tokens (result in response body)
///
/// # Use Cases
///
/// - CLI checks token validity before operations
/// - Dashboard validates token on page load
/// - Pre-flight check before batch operations
pub async fn validate(
  State(state): State<AuthState>,
  TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
) -> impl IntoResponse {
  // Verify User Token
  let claims = match state.jwt_secret.verify_access_token(&bearer.token()) {
    Ok(claims) => claims,
    Err(_err) => {
      // Token expired or invalid
      return (StatusCode::OK, Json(ValidateResponse::Invalid {
        valid: false,
        reason: "TOKEN_EXPIRED".to_string(),
        expired_at: Some(chrono::Utc::now().to_rfc3339()),
        revoked_at: None,
      })).into_response();
    }
  };

  // Check if token is blacklisted
  let blacklisted = user_auth::get_blacklisted_token(&state.db_pool, &claims.jti).await;

  let blacklisted = match blacklisted {
    Ok(blacklisted) => blacklisted,
    Err(err) => {
      tracing::error!("Failed to check if token is blacklisted: {}", err);
      return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
        error: ErrorDetail {
          code: "TOKEN_BLACKLIST_ERROR".to_string(),
          message: "Failed to check if token is blacklisted".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  match blacklisted {
    Some(blacklisted) => {
      let blacklisted_at = chrono::DateTime::from_timestamp(blacklisted.blacklisted_at, 0);

      match blacklisted_at {
        Some(timestamp) => {
          return (StatusCode::OK, Json(ValidateResponse::Invalid {
            valid: false,
            reason: "TOKEN_REVOKED".to_string(),
            expired_at: None,
            revoked_at: Some(timestamp.to_rfc3339()),
          })).into_response();
        },
        None => {
          return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
            error: ErrorDetail {
              code: "TOKEN_BLACKLIST_ERROR".to_string(),
              message: "Failed to check if token is blacklisted".to_string(),
              details: None,
            },
          })).into_response();
        },
      }
    },
    None => {},
  }
  
  // Fetch user to get current info
  let user = user_auth::get_user_by_id(&state.db_pool, &claims.sub).await;

  let user_option = match user {
    Ok(user) => user,
    Err(err) => {
      tracing::error!("Failed to fetch user: {}", err);
      return (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse {
        error: ErrorDetail {
          code: "USER_FETCH_ERROR".to_string(),
          message: "Failed to fetch user".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  let user = match user_option {
    Some(user) => user,
    None => {
      return (StatusCode::NOT_FOUND, Json(ErrorResponse {
        error: ErrorDetail {
          code: "USER_NOT_FOUND".to_string(),
          message: "User not found".to_string(),
          details: None,
        },
      })).into_response();
    }
  };

  let expires_at = chrono::DateTime::from_timestamp(claims.exp as i64, 0).unwrap();
  let expires_in = (expires_at - chrono::Utc::now()).num_seconds() as u64;

  // Placeholder response
  (
    StatusCode::OK,
    Json(ValidateResponse::Valid {
      valid: true,
      user: UserInfo::from_user(&user),
      expires_at: expires_at.to_rfc3339(),
      expires_in: expires_in,
    }),
  )
    .into_response()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_login_request_validation_valid() {
    let request = LoginRequest {
      email: "developer@example.com".to_string(),
      password: "secure_password_123".to_string(),
    };
    assert!(request.validate().is_ok());
  }

  #[test]
  fn test_login_request_validation_empty_email() {
    let request = LoginRequest {
      email: "".to_string(),
      password: "secure_password_123".to_string(),
    };
    assert!(request.validate().is_err());
  }

  #[test]
  fn test_login_request_validation_empty_password() {
    let request = LoginRequest {
      email: "developer@example.com".to_string(),
      password: "".to_string(),
    };
    assert!(request.validate().is_err());
  }

  #[test]
  fn test_login_request_validation_email_too_long() {
    let request = LoginRequest {
      email: "a".repeat(256),
      password: "secure_password_123".to_string(),
    };
    assert!(request.validate().is_err());
  }

  #[test]
  fn test_login_request_validation_password_too_long() {
    let request = LoginRequest {
      email: "developer@example.com".to_string(),
      password: "a".repeat(1001),
    };
    assert!(request.validate().is_err());
  }

  // ============================================================================
  // Logout Endpoint Tests
  // ============================================================================

  /// Helper function to create test database with migrations
  async fn create_test_db() -> Result<SqlitePool, sqlx::Error> {
    let pool = SqlitePool::connect(":memory:").await?;

    // Run migration 003 (users table)
    let migration_003 =
      include_str!("../../../iron_token_manager/migrations/003_create_users_table.sql");
    sqlx::raw_sql(migration_003).execute(&pool).await?;

    // Run migration 006 (user audit log)
    let migration_006 =
      include_str!("../../../iron_token_manager/migrations/006_create_user_audit_log.sql");
    sqlx::raw_sql(migration_006).execute(&pool).await?;

    // Run migration 007 (blacklist table)
    let migration_007 =
      include_str!("../../../iron_token_manager/migrations/007_create_blacklist_table.sql");
    sqlx::raw_sql(migration_007).execute(&pool).await?;

    Ok(pool)
  }

  /// Helper function to create test user
  async fn create_test_user(pool: &SqlitePool, username: &str, email: &str, password: &str, role: &str) -> Result<i64, sqlx::Error> {
    let password_hash = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();
    let created_at = chrono::Utc::now().timestamp();
    
    let result = sqlx::query(
      "INSERT INTO users (username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, 1, ?)"
    )
    .bind(username)
    .bind(email)
    .bind(password_hash)
    .bind(role)
    .bind(created_at)
    .execute(pool)
    .await?;

    Ok(result.last_insert_rowid())
  }

  #[tokio::test]
  async fn test_logout_success() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");
    
    // Create test user
    let id = create_test_user(&pool, "user1", "test@example.com", "testpass", "developer")
      .await
      .expect("Failed to create test user");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool.clone(),
    };

    // Generate valid token
    let token_id = "test_token_123";
    let token = jwt_secret
      .generate_access_token(id, "developer", token_id)
      .expect("Failed to generate token");

    // Create logout request
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/logout", axum::routing::post(logout))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/logout")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert successful logout (204 No Content)
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    // Verify token was added to blacklist
    let blacklisted: Option<(String,)> = sqlx::query_as(
      "SELECT jti FROM blacklist WHERE jti = ?"
    )
    .bind(token_id)
    .fetch_optional(&pool)
    .await
    .expect("Failed to query blacklist");

    assert!(blacklisted.is_some(), "Token should be in blacklist");
    assert_eq!(blacklisted.unwrap().0, token_id);
  }

  #[tokio::test]
  async fn test_logout_invalid_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret,
      db_pool: pool,
    };

    // Create logout request with invalid token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/logout", axum::routing::post(logout))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/logout")
      .header("Authorization", "Bearer invalid_token_here")
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401)
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify error response contains correct error code
    assert!(body_str.contains("AUTH_INVALID_TOKEN"));
  }

  #[tokio::test]
  async fn test_logout_expired_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool,
    };

    // Create an expired token manually
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = crate::jwt_auth::AccessTokenClaims {
      sub: "test@example.com".to_string(),
      role: "developer".to_string(),
      jti: "expired_token_123".to_string(),
      token_type: "access".to_string(),
      exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(), // Expired 1 hour ago
      iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
    };

    let expired_token = encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret("test_secret_key_for_testing".as_bytes()),
    )
    .unwrap();

    // Create logout request with expired token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/logout", axum::routing::post(logout))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/logout")
      .header("Authorization", format!("Bearer {}", expired_token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401) for expired token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn test_logout_missing_authorization_header() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret,
      db_pool: pool,
    };

    // Create logout request without Authorization header
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/logout", axum::routing::post(logout))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/logout")
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401) for missing header
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  // ============================================================================
  // Refresh Endpoint Tests
  // ============================================================================

  #[tokio::test]
  async fn test_refresh_success() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");
    
    // Create test user
    let user_id = create_test_user(&pool, "user1", "test@example.com", "testpass", "developer")
      .await
      .expect("Failed to create test user");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool.clone(),
    };

    // Generate valid token
    let token_id = "refresh_test_token_123";
    let token = jwt_secret
      .generate_access_token(user_id, "developer", token_id)
      .expect("Failed to generate token");

    // Create refresh request
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/refresh", axum::routing::post(refresh))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/refresh")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert successful refresh (200 OK)
    assert_eq!(response.status(), StatusCode::OK);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify response contains new token
    assert!(body_str.contains("user_token"));
    assert!(body_str.contains("Bearer"));
    assert!(body_str.contains("expires_in"));

    // Verify old token is now blacklisted
    let blacklisted = user_auth::get_blacklisted_token(&pool, token_id).await
      .expect("Failed to query blacklist");
    assert!(blacklisted.is_some(), "Old token should be blacklisted");
  }

  #[tokio::test]
  async fn test_refresh_invalid_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret,
      db_pool: pool,
    };

    // Create refresh request with invalid token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/refresh", axum::routing::post(refresh))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/refresh")
      .header("Authorization", "Bearer invalid_token_here")
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401)
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify error response
    assert!(body_str.contains("AUTH_INVALID_TOKEN"));
  }

  #[tokio::test]
  async fn test_refresh_expired_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool,
    };

    // Create an expired token manually
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = crate::jwt_auth::AccessTokenClaims {
      sub: "1".to_string(),
      role: "developer".to_string(),
      jti: "expired_refresh_token_123".to_string(),
      token_type: "access".to_string(),
      exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(),
      iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
    };

    let expired_token = encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret("test_secret_key_for_testing".as_bytes()),
    )
    .unwrap();

    // Create refresh request with expired token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/refresh", axum::routing::post(refresh))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/refresh")
      .header("Authorization", format!("Bearer {}", expired_token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401) for expired token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  #[tokio::test]
  async fn test_refresh_blacklisted_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");
    
    // Create test user
    let user_id = create_test_user(&pool, "user1", "test@example.com", "testpass", "developer")
      .await
      .expect("Failed to create test user");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool.clone(),
    };

    // Generate valid token
    let token_id = "blacklisted_refresh_token";
    let token = jwt_secret
      .generate_access_token(user_id, "developer", token_id)
      .expect("Failed to generate token");

    // Blacklist the token
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    user_auth::add_token_to_blacklist(&pool, token_id, user_id, expires_at)
      .await
      .expect("Failed to blacklist token");

    // Create refresh request with blacklisted token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/refresh", axum::routing::post(refresh))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/refresh")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert unauthorized (401) for blacklisted token
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
  }

  // ============================================================================
  // Validate Endpoint Tests
  // ============================================================================

  #[tokio::test]
  async fn test_validate_valid_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");
    
    // Create test user
    let user_id = create_test_user(&pool, "user1", "test@example.com", "testpass", "developer")
      .await
      .expect("Failed to create test user");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool.clone(),
    };

    // Generate valid token
    let token_id = "validate_test_token_123";
    let token = jwt_secret
      .generate_access_token(user_id, "developer", token_id)
      .expect("Failed to generate token");

    // Create validate request
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/validate", axum::routing::post(validate))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/validate")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert successful validation (200 OK)
    assert_eq!(response.status(), StatusCode::OK);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify response indicates token is valid
    assert!(body_str.contains("\"valid\":true"));
    assert!(body_str.contains("test@example.com"));
    assert!(body_str.contains("developer"));
  }

  #[tokio::test]
  async fn test_validate_invalid_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret,
      db_pool: pool,
    };

    // Create validate request with invalid token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/validate", axum::routing::post(validate))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/validate")
      .header("Authorization", "Bearer invalid_token_here")
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert OK (200) - validate always returns 200, result in body
    assert_eq!(response.status(), StatusCode::OK);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify response indicates token is invalid
    assert!(body_str.contains("\"valid\":false"));
    assert!(body_str.contains("TOKEN_EXPIRED"));
  }

  #[tokio::test]
  async fn test_validate_expired_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool,
    };

    // Create an expired token manually
    use jsonwebtoken::{encode, EncodingKey, Header};
    let claims = crate::jwt_auth::AccessTokenClaims {
      sub: "1".to_string(),
      role: "developer".to_string(),
      jti: "expired_validate_token_123".to_string(),
      token_type: "access".to_string(),
      exp: (chrono::Utc::now() - chrono::Duration::hours(1)).timestamp(),
      iat: (chrono::Utc::now() - chrono::Duration::hours(2)).timestamp(),
    };

    let expired_token = encode(
      &Header::default(),
      &claims,
      &EncodingKey::from_secret("test_secret_key_for_testing".as_bytes()),
    )
    .unwrap();

    // Create validate request with expired token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/validate", axum::routing::post(validate))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/validate")
      .header("Authorization", format!("Bearer {}", expired_token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert OK (200) - validate always returns 200
    assert_eq!(response.status(), StatusCode::OK);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify response indicates token is expired
    assert!(body_str.contains("\"valid\":false"));
    assert!(body_str.contains("TOKEN_EXPIRED"));
  }

  #[tokio::test]
  async fn test_validate_blacklisted_token() {
    // Setup test database
    let pool = create_test_db().await.expect("Failed to create test database");
    
    // Create test user
    let user_id = create_test_user(&pool, "user1", "test@example.com", "testpass", "developer")
      .await
      .expect("Failed to create test user");

    // Create auth state
    let jwt_secret = Arc::new(JwtSecret::new("test_secret_key_for_testing".to_string()));
    let state = AuthState {
      jwt_secret: jwt_secret.clone(),
      db_pool: pool.clone(),
    };

    // Generate valid token
    let token_id = "blacklisted_validate_token";
    let token = jwt_secret
      .generate_access_token(user_id, "developer", token_id)
      .expect("Failed to generate token");

    // Blacklist the token
    let expires_at = chrono::Utc::now() + chrono::Duration::days(30);
    user_auth::add_token_to_blacklist(&pool, token_id, user_id, expires_at)
      .await
      .expect("Failed to blacklist token");

    // Create validate request with blacklisted token
    use axum::http::{Request, StatusCode};
    use axum::body::Body;
    use tower::ServiceExt;

    let app = axum::Router::new()
      .route("/validate", axum::routing::post(validate))
      .with_state(state);

    let request = Request::builder()
      .method("POST")
      .uri("/validate")
      .header("Authorization", format!("Bearer {}", token))
      .body(Body::empty())
      .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert OK (200) - validate always returns 200
    assert_eq!(response.status(), StatusCode::OK);

    // Read response body
    use http_body_util::BodyExt;
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
    
    // Verify response indicates token is revoked
    assert!(body_str.contains("\"valid\":false"));
    assert!(body_str.contains("TOKEN_REVOKED"));
    assert!(body_str.contains("revoked_at"));
  }
}