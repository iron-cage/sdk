//! User management endpoints
//!
//! Provides endpoints for:
//! - Creating new users (Admin only)
//! - Listing all users (Admin only)

use crate::rbac::Role;
use crate::jwt_auth::AuthenticatedUser;
use crate::routes::auth::AuthState;
use axum::{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };
use sqlx::FromRow;
use std::str::FromStr;

/// Request body for creating a new user
#[ derive( Debug, Deserialize ) ]
pub struct CreateUserRequest
{
  pub username: String,
  pub password: String,
  pub role: Option< String >,
}

/// User response object
#[ derive( Debug, Serialize, FromRow ) ]
pub struct UserResponse
{
  pub id: i64,
  pub username: String,
  pub role: String,
  pub is_active: bool,
}

/// POST /api/users
///
/// Create a new user (Admin only)
pub async fn create_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Json( request ): Json< CreateUserRequest >,
) -> impl IntoResponse
{
  // Check permission (Admin only)
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if role != Role::Admin
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Insufficient permissions" }) ),
    )
      .into_response();
  }

  // Validate request
  if request.username.trim().is_empty() || request.password.trim().is_empty()
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Username and password cannot be empty" }) ),
    )
      .into_response();
  }

  // Hash password
  let password_hash = match bcrypt::hash( &request.password, bcrypt::DEFAULT_COST )
  {
    Ok( hash ) => hash,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to hash password" }) ),
      )
        .into_response();
    }
  };

  let role_str = request.role.unwrap_or_else( || "user".to_string() );

  // Insert user
  let result = sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active) VALUES (?, ?, ?, 1)"
  )
  .bind( &request.username )
  .bind( &password_hash )
  .bind( &role_str )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) => (
      StatusCode::CREATED,
      Json( serde_json::json!({ "success": true }) ),
    )
      .into_response(),
    Err( e ) =>
    {
      tracing::error!( "Failed to create user: {}", e );
      // Check if error is constraint violation (username already exists)
      // This is a simplification, ideally we check error code
      if e.to_string().contains( "UNIQUE constraint failed" )
      {
        return (
          StatusCode::CONFLICT,
          Json( serde_json::json!({ "error": "Username already exists" }) ),
        )
          .into_response();
      }
      
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to create user" }) ),
      )
        .into_response()
    }
  }
}

/// GET /api/users
///
/// List all users (Admin only)
pub async fn list_users(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
) -> impl IntoResponse
{
  // Check permission (Admin only)
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if role != Role::Admin
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Insufficient permissions" }) ),
    )
      .into_response();
  }

  let users = sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, role, is_active FROM users"
  )
  .fetch_all( &state.db_pool )
  .await;

  match users
  {
    Ok( users ) => ( StatusCode::OK, Json( users ) ).into_response(),
    Err( e ) =>
    {
      tracing::error!( "Failed to list users: {}", e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to list users" }) ),
      )
        .into_response()
    }
  }
}
