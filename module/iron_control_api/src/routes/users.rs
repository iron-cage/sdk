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

  // Prevent creating user with username 'admin' (reserved)
  if request.username.to_lowercase() == "admin"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Username 'admin' is reserved" }) ),
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
    "INSERT INTO users (username, password_hash, role, is_active, created_at) VALUES (?, ?, ?, 1, strftime('%s', 'now'))"
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
    "SELECT id, username, role, is_active FROM users WHERE username != 'admin'"
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

/// Request body for updating user status
#[ derive( Debug, Deserialize ) ]
pub struct UpdateUserStatusRequest
{
  pub is_active: bool,
}

/// PATCH /api/users/:id/status
///
/// Enable or disable a user (Admin only)
pub async fn update_user_status(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  axum::extract::Path( user_id ): axum::extract::Path< i64 >,
  Json( request ): Json< UpdateUserStatusRequest >,
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

  // Prevent disabling self
  // We need to fetch the target user to check if it's the current user?
  // Or just check if user_id matches current user's ID?
  // But claims.sub is username, not ID.
  // Let's fetch the target user first to verify existence and get username.
  
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, role, is_active FROM users WHERE id = ?"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "User not found" }) ),
      )
        .into_response();
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to fetch user: {}", e );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  if target_user.username == claims.sub
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Cannot change your own status" }) ),
    )
      .into_response();
  }

  // Protect 'admin' user from being disabled
  if target_user.username == "admin"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Cannot disable the admin user" }) ),
    )
      .into_response();
  }

  // Update status
  let result = sqlx::query(
    "UPDATE users SET is_active = ? WHERE id = ?"
  )
  .bind( request.is_active )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) => (
      StatusCode::OK,
      Json( serde_json::json!({ "success": true }) ),
    )
      .into_response(),
    Err( e ) =>
    {
      tracing::error!( "Failed to update user status: {}", e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to update user status" }) ),
      )
        .into_response()
    }
  }
}

/// DELETE /api/users/:id
///
/// Delete a user (Admin only)
pub async fn delete_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  axum::extract::Path( user_id ): axum::extract::Path< i64 >,
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

  // Fetch target user to verify existence and check constraints
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, role, is_active FROM users WHERE id = ?"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "User not found" }) ),
      )
        .into_response();
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to fetch user: {}", e );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  // Prevent deleting self
  if target_user.username == claims.sub
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Cannot delete yourself" }) ),
    )
      .into_response();
  }

  // Protect 'admin' user from being deleted
  if target_user.username == "admin"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Cannot delete the admin user" }) ),
    )
      .into_response();
  }

  // Delete user
  let result = sqlx::query(
    "DELETE FROM users WHERE id = ?"
  )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) => (
      StatusCode::OK,
      Json( serde_json::json!({ "success": true }) ),
    )
      .into_response(),
    Err( e ) =>
    {
      tracing::error!( "Failed to delete user: {}", e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to delete user" }) ),
      )
        .into_response()
    }
  }
}
