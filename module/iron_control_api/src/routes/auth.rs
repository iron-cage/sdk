//! Authentication REST API endpoints
//!
//! Phase 4 Day 28: REST API Endpoints - Authentication
//!
//! Endpoints:
//! - POST /api/auth/login - User login (returns access + refresh tokens)
//! - POST /api/auth/refresh - Refresh access token
//! - POST /api/auth/logout - Logout (blacklist refresh token)

use crate::jwt_auth::JwtSecret;
use crate::user_auth;
use axum::{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };
use sqlx::{ Pool, Sqlite, SqlitePool };
use std::sync::Arc;

/// Shared authentication state
#[ derive( Clone ) ]
pub struct AuthState
{
  pub jwt_secret: Arc< JwtSecret >,
  pub db_pool: Pool< Sqlite >,
}

impl AuthState
{
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
  pub async fn new( jwt_secret_key: String, database_url: &str ) -> Result< Self, sqlx::Error >
  {
    let db_pool = SqlitePool::connect( database_url ).await?;

    // Run migration 003 (users table) if not already applied
    let migration_003_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_003_completed'"
    )
    .fetch_one( &db_pool )
    .await?;

    if migration_003_completed == 0
    {
      let migration_003 = include_str!( "../../../iron_token_manager/migrations/003_create_users_table.sql" );
      sqlx::raw_sql( migration_003 )
        .execute( &db_pool )
        .await?;
    }

    // Migration 006: Enhance users table
    let migration_006_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_006_completed'"
    )
    .fetch_one( &db_pool )
    .await?;

    if migration_006_completed == 0
    {
      let migration_006 = include_str!( "../../../iron_token_manager/migrations/006_enhance_users_table.sql" );
      sqlx::raw_sql( migration_006 )
        .execute( &db_pool )
        .await?;
    }

    // Migration 007: Create user audit log
    let migration_007_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_007_completed'"
    )
    .fetch_one( &db_pool )
    .await?;

    if migration_007_completed == 0
    {
      let migration_007 = include_str!( "../../../iron_token_manager/migrations/007_create_user_audit_log.sql" );
      sqlx::raw_sql( migration_007 )
        .execute( &db_pool )
        .await?;
    }

    Ok( Self
    {
      jwt_secret: Arc::new( JwtSecret::new( jwt_secret_key ) ),
      db_pool,
    } )
  }
}

/// Login request body
#[ derive( Debug, Deserialize ) ]
pub struct LoginRequest
{
  pub username: String,
  pub password: String,
}

impl LoginRequest
{
  /// Maximum username length for DoS prevention
  const MAX_USERNAME_LENGTH: usize = 255;

  /// Maximum password length for DoS prevention
  const MAX_PASSWORD_LENGTH: usize = 1000;

  /// Validate login request parameters
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - username is empty or whitespace-only
  /// - password is empty or whitespace-only
  /// - username exceeds 255 characters
  /// - password exceeds 1000 characters
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate username is not empty
    if self.username.trim().is_empty()
    {
      return Err( "username cannot be empty".to_string() );
    }

    // Validate password is not empty
    if self.password.trim().is_empty()
    {
      return Err( "password cannot be empty".to_string() );
    }

    // Validate username length
    if self.username.len() > Self::MAX_USERNAME_LENGTH
    {
      return Err( format!(
        "username too long (max {} characters)",
        Self::MAX_USERNAME_LENGTH
      ) );
    }

    // Validate password length
    if self.password.len() > Self::MAX_PASSWORD_LENGTH
    {
      return Err( format!(
        "password too long (max {} characters)",
        Self::MAX_PASSWORD_LENGTH
      ) );
    }

    Ok( () )
  }
}

/// Login response body
#[ derive( Debug, Serialize ) ]
pub struct LoginResponse
{
  pub access_token: String,
  pub refresh_token: String,
  pub token_type: String,
  pub expires_in: u64,
  pub role: String,
}

/// Refresh request body
#[ derive( Debug, Deserialize ) ]
pub struct RefreshRequest
{
  pub refresh_token: String,
}

impl RefreshRequest
{
  /// Maximum refresh token length for DoS prevention (JWT tokens can be long)
  const MAX_TOKEN_LENGTH: usize = 2000;

  /// Validate refresh request parameters
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - refresh_token is empty or whitespace-only
  /// - refresh_token exceeds 2000 characters
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate refresh_token is not empty
    if self.refresh_token.trim().is_empty()
    {
      return Err( "refresh_token cannot be empty".to_string() );
    }

    // Validate refresh_token length (DoS prevention)
    if self.refresh_token.len() > Self::MAX_TOKEN_LENGTH
    {
      return Err( format!(
        "refresh_token too long (max {} characters)",
        Self::MAX_TOKEN_LENGTH
      ) );
    }

    Ok( () )
  }
}

/// Refresh response body
#[ derive( Debug, Serialize ) ]
pub struct RefreshResponse
{
  pub access_token: String,
  pub refresh_token: String,
  pub token_type: String,
  pub expires_in: u64,
  pub role: String,
}

/// Logout request body
#[ derive( Debug, Deserialize ) ]
pub struct LogoutRequest
{
  pub refresh_token: String,
}

impl LogoutRequest
{
  /// Maximum refresh token length for DoS prevention (JWT tokens can be long)
  const MAX_TOKEN_LENGTH: usize = 2000;

  /// Validate logout request parameters
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - refresh_token is empty or whitespace-only
  /// - refresh_token exceeds 2000 characters
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate refresh_token is not empty
    if self.refresh_token.trim().is_empty()
    {
      return Err( "refresh_token cannot be empty".to_string() );
    }

    // Validate refresh_token length (DoS prevention)
    if self.refresh_token.len() > Self::MAX_TOKEN_LENGTH
    {
      return Err( format!(
        "refresh_token too long (max {} characters)",
        Self::MAX_TOKEN_LENGTH
      ) );
    }

    Ok( () )
  }
}

/// Logout response body
#[ derive( Debug, Serialize ) ]
pub struct LogoutResponse
{
  pub success: bool,
  pub message: String,
}

/// POST /api/auth/login
///
/// Authenticate user and return JWT tokens
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret + database)
/// * `request` - Login credentials (username, password)
///
/// # Returns
///
/// - 200 OK with access + refresh tokens if authentication successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if credentials invalid
/// - 500 Internal Server Error if token generation or database query fails
pub async fn login(
  State( state ): State< AuthState >,
  Json( request ): Json< LoginRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Authenticate user against database
  let user = match user_auth::authenticate_user( &state.db_pool, &request.username, &request.password ).await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) =>
    {
      // Invalid credentials - return 401
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "Invalid username or password" }) ),
      )
        .into_response();
    }
    Err( err ) =>
    {
      // Database error - return 500
      tracing::error!( "Database error during authentication: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Authentication service unavailable" }) ),
      )
        .into_response();
    }
  };

  let user_id = &user.username;
  let user_role = &user.role;

  // Generate tokens
  let access_token = match state.jwt_secret.generate_access_token( user_id, user_role )
  {
    Ok( token ) => token,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to generate access token" }) ),
      )
        .into_response();
    }
  };

  // Generate unique refresh token ID
  let refresh_token_id = format!( "refresh_{}_{}", user_id, chrono::Utc::now().timestamp() );
  let refresh_token = match state.jwt_secret.generate_refresh_token( user_id, &refresh_token_id )
  {
    Ok( token ) => token,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to generate refresh token" }) ),
      )
        .into_response();
    }
  };

  ( StatusCode::OK, Json( LoginResponse
  {
    access_token,
    refresh_token,
    token_type: "Bearer".to_string(),
    expires_in: 3600, // 1 hour
    role: user_role.clone(),
  } ) )
    .into_response()
}

/// POST /api/auth/refresh
///
/// Refresh access token using valid refresh token
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret)
/// * `request` - Refresh token
///
/// # Returns
///
/// - 200 OK with new access + refresh tokens
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if refresh token invalid
pub async fn refresh(
  State( state ): State< AuthState >,
  Json( request ): Json< RefreshRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Verify refresh token
  let claims = match state.jwt_secret.verify_refresh_token( &request.refresh_token )
  {
    Ok( claims ) => claims,
    Err( _ ) =>
    {
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "Invalid or expired refresh token" }) ),
      )
        .into_response();
    }
  };

  // TODO: Check if refresh token is blacklisted
  // SELECT 1 FROM token_blacklist WHERE jti = ?

  let user_id = &claims.sub;

  // Fetch user to get role
  let user = match user_auth::get_user_by_username( &state.db_pool, user_id ).await
  {
    Ok( Some( user ) ) => user,
    _ =>
    {
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "User not found" }) ),
      )
        .into_response();
    }
  };

  // Generate new access token
  let access_token = match state.jwt_secret.generate_access_token( user_id, &user.role )
  {
    Ok( token ) => token,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to generate access token" }) ),
      )
        .into_response();
    }
  };

  // Generate new refresh token (token rotation)
  let new_refresh_token_id = format!( "refresh_{}_{}", user_id, chrono::Utc::now().timestamp() );
  let refresh_token = match state.jwt_secret.generate_refresh_token( user_id, &new_refresh_token_id )
  {
    Ok( token ) => token,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to generate refresh token" }) ),
      )
        .into_response();
    }
  };

  // TODO: Blacklist old refresh token
  // INSERT INTO token_blacklist (jti, blacklisted_at) VALUES (?, ?)

  ( StatusCode::OK, Json( RefreshResponse
  {
    access_token,
    refresh_token,
    token_type: "Bearer".to_string(),
    expires_in: 3600, // 1 hour
    role: user.role.clone(),
  } ) )
    .into_response()
}

/// POST /api/auth/logout
///
/// Logout user by blacklisting refresh token
///
/// # Arguments
///
/// * `state` - Authentication state (JWT secret)
/// * `request` - Refresh token to blacklist
///
/// # Returns
///
/// - 200 OK if logout successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if refresh token invalid
pub async fn logout(
  State( state ): State< AuthState >,
  Json( request ): Json< LogoutRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Verify refresh token
  let claims = match state.jwt_secret.verify_refresh_token( &request.refresh_token )
  {
    Ok( claims ) => claims,
    Err( _ ) =>
    {
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "Invalid refresh token" }) ),
      )
        .into_response();
    }
  };

  // TODO: Add refresh token to blacklist
  // INSERT INTO token_blacklist (jti, blacklisted_at) VALUES (?, ?)
  let _jti = claims.jti;

  ( StatusCode::OK, Json( LogoutResponse
  {
    success: true,
    message: "Logged out successfully".to_string(),
  } ) )
    .into_response()
}
