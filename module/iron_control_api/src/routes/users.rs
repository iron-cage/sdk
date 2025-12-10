//! User management endpoints
//!
//! Provides endpoints for:
//! - Creating new users (Admin only)
//! - Listing all users (Admin only)
//! - Getting user details (Admin only)
//! - Suspending/Activating users (Admin only)
//! - Soft deleting users (Admin only)
//! - Changing user roles (Admin only)
//! - Resetting user passwords (Admin only)

use crate::rbac::Role;
use crate::jwt_auth::AuthenticatedUser;
use crate::routes::auth::AuthState;
use axum::{
  extract::{ State, Path, Query },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };
use sqlx::FromRow;
use std::str::FromStr;
use chrono::Utc;

/// Request body for creating a new user
#[ derive( Debug, Deserialize, Serialize ) ]
pub struct CreateUserRequest
{
  pub username: String,
  pub password: String,
  pub email: String,
  pub role: Option< String >,
}

/// Request body for suspending a user
#[ derive( Debug, Deserialize ) ]
pub struct SuspendUserRequest
{
  pub reason: Option< String >,
}

/// Request body for changing user role
#[ derive( Debug, Deserialize ) ]
pub struct ChangeRoleRequest
{
  pub role: String,
}

/// Request body for resetting password
#[ derive( Debug, Deserialize ) ]
pub struct ResetPasswordRequest
{
  pub new_password: String,
  pub force_change: bool,
}

/// Query parameters for listing users
#[ derive( Debug, Deserialize ) ]
pub struct ListUsersQuery
{
  pub role: Option< String >,
  pub is_active: Option< bool >,
  pub search: Option< String >,
  pub page: Option< i64 >,
  pub page_size: Option< i64 >,
}

/// User response object
#[ derive( Debug, Serialize, Deserialize, FromRow ) ]
pub struct UserResponse
{
  pub id: i64,
  pub username: String,
  pub email: Option< String >,
  pub role: String,
  pub is_active: bool,
  pub created_at: i64,
  pub last_login: Option< i64 >,
  pub suspended_at: Option< i64 >,
  pub deleted_at: Option< i64 >,
}

/// List users response
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ListUsersResponse
{
  pub users: Vec< UserResponse >,
  pub total: i64,
  pub page: i64,
  pub page_size: i64,
}

/// Helper to get admin ID from username
async fn get_admin_id( pool: &sqlx::SqlitePool, username: &str ) -> Result< i64, sqlx::Error >
{
  let row: ( i64, ) = sqlx::query_as( "SELECT id FROM users WHERE username = ?" )
    .bind( username )
    .fetch_one( pool )
    .await?;
  Ok( row.0 )
}

/// Helper to create audit log entry
async fn create_audit_log(
  pool: &sqlx::SqlitePool,
  operation: &str,
  target_user_id: i64,
  performed_by: i64,
  previous_state: Option< String >,
  new_state: Option< String >,
  reason: Option< String >,
) -> Result< (), sqlx::Error >
{
  sqlx::query(
    "INSERT INTO user_audit_log (operation, target_user_id, performed_by, timestamp, previous_state, new_state, reason) VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( operation )
  .bind( target_user_id )
  .bind( performed_by )
  .bind( Utc::now().timestamp_millis() )
  .bind( previous_state )
  .bind( new_state )
  .bind( reason )
  .execute( pool )
  .await?;

  Ok( () )
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
  if request.username.trim().is_empty() || request.password.trim().is_empty() || request.email.trim().is_empty()
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Username, password and email cannot be empty" }) ),
    )
      .into_response();
  }

  if !request.email.contains( '@' )
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Invalid email format" }) ),
    )
      .into_response();
  }

  if request.password.len() < 8
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Password must be at least 8 characters" }) ),
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

  // Validate role
  let role_str = request.role.unwrap_or_else( || "user".to_string() );
  if !["viewer", "user", "admin"].contains( &role_str.as_str() )
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Role must be one of: viewer, user, admin" }) ),
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

  // Insert user
  let created_at = Utc::now().timestamp_millis();
  let result = sqlx::query(
    "INSERT INTO users (username, password_hash, email, role, is_active, created_at) VALUES (?, ?, ?, ?, 1, ?)"
  )
  .bind( &request.username )
  .bind( &password_hash )
  .bind( &request.email )
  .bind( &role_str )
  .bind( created_at )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( result ) => 
    {
      let user_id = result.last_insert_rowid();
      
      // Audit log
      let new_state = serde_json::json!({
        "username": request.username,
        "email": request.email,
        "role": role_str
      }).to_string();

      // Get admin ID from claims
      let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

      let _ = create_audit_log(
        &state.db_pool,
        "create",
        user_id,
        admin_id,
        None,
        Some( new_state ),
        None
      ).await;

      (
        StatusCode::CREATED,
        Json( serde_json::json!({
          "id": user_id,
          "username": request.username,
          "email": request.email,
          "role": role_str,
          "is_active": true,
          "created_at": created_at
        }) ),
      )
        .into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to create user: {}", e );
      if e.to_string().contains( "UNIQUE constraint failed" )
      {
        return (
          StatusCode::CONFLICT,
          Json( serde_json::json!({ "error": "Username or email already exists" }) ),
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
  Query( query ): Query< ListUsersQuery >,
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

  let page = query.page.unwrap_or( 1 ).max( 1 );
  let page_size = query.page_size.unwrap_or( 20 ).max( 1 ).min( 100 );
  let offset = ( page - 1 ) * page_size;

  let mut sql = "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE deleted_at IS NULL".to_string();
  let mut count_sql = "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL".to_string();
  let mut params = Vec::new();

  if let Some( role_filter ) = &query.role
  {
    sql.push_str( " AND role = ?" );
    count_sql.push_str( " AND role = ?" );
    params.push( role_filter.clone() );
  }

  if let Some( is_active ) = query.is_active
  {
    sql.push_str( " AND is_active = ?" );
    count_sql.push_str( " AND is_active = ?" );
    // SQLite uses 0/1 for boolean
    params.push( if is_active { "1".to_string() } else { "0".to_string() } );
  }

  if let Some( search ) = &query.search
  {
    sql.push_str( " AND (username LIKE ? OR email LIKE ?)" );
    count_sql.push_str( " AND (username LIKE ? OR email LIKE ?)" );
    let search_pattern = format!( "%{}%", search );
    params.push( search_pattern.clone() );
    params.push( search_pattern ); // Push twice for OR
  }

  sql.push_str( " ORDER BY created_at DESC LIMIT ? OFFSET ?" );

  // Execute count query
  // Note: This manual query building is a bit hacky, normally we'd use QueryBuilder but for simplicity in this context:
  // We'll use sqlx::query_as with manual binding which is tricky with dynamic params.
  // Let's use QueryBuilder if possible, but I don't want to change dependencies too much.
  // Actually, let's just use a simpler approach since we have limited filters.
  
  // Re-implement with sqlx::QueryBuilder if I could, but let's stick to simple string concat and manual binding logic or just fetch all and filter in memory if dataset is small?
  // No, pagination implies large dataset.
  // Let's try to bind properly.
  
  // Actually, let's just use the QueryBuilder which is standard in sqlx.
  // But I need to check if sqlx features include it. It usually does.
  
  let mut query_builder = sqlx::QueryBuilder::new( "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE deleted_at IS NULL" );
  let mut count_builder = sqlx::QueryBuilder::new( "SELECT COUNT(*) FROM users WHERE deleted_at IS NULL" );

  if let Some( role_filter ) = &query.role
  {
    query_builder.push( " AND role = " );
    query_builder.push_bind( role_filter );
    count_builder.push( " AND role = " );
    count_builder.push_bind( role_filter );
  }

  if let Some( is_active ) = query.is_active
  {
    query_builder.push( " AND is_active = " );
    query_builder.push_bind( is_active );
    count_builder.push( " AND is_active = " );
    count_builder.push_bind( is_active );
  }

  if let Some( search ) = &query.search
  {
    let pattern = format!( "%{}%", search );
    query_builder.push( " AND (username LIKE " );
    query_builder.push_bind( pattern.clone() );
    query_builder.push( " OR email LIKE " );
    query_builder.push_bind( pattern );
    query_builder.push( ")" );

    count_builder.push( " AND (username LIKE " );
    let pattern = format!( "%{}%", search );
    count_builder.push_bind( pattern.clone() );
    count_builder.push( " OR email LIKE " );
    count_builder.push_bind( pattern );
    count_builder.push( ")" );
  }

  query_builder.push( " ORDER BY created_at DESC LIMIT " );
  query_builder.push_bind( page_size );
  query_builder.push( " OFFSET " );
  query_builder.push_bind( offset );

  let users: Vec< UserResponse > = match query_builder.build_query_as().fetch_all( &state.db_pool ).await
  {
    Ok( u ) => u,
    Err( e ) =>
    {
      tracing::error!( "Failed to list users: {}", e );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to list users" }) ),
      )
        .into_response();
    }
  };

  let total: i64 = match count_builder.build_query_scalar().fetch_one( &state.db_pool ).await
  {
    Ok( c ) => c,
    Err( e ) =>
    {
      tracing::error!( "Failed to count users: {}", e );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to count users" }) ),
      )
        .into_response();
    }
  };

  (
    StatusCode::OK,
    Json( ListUsersResponse {
      users,
      total,
      page,
      page_size,
    } ),
  )
    .into_response()
}

/// GET /api/users/:id
///
/// Get user details (Admin only)
pub async fn get_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
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

  let user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
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
      tracing::error!( "Failed to get user: {}", e );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  ( StatusCode::OK, Json( user ) ).into_response()
}

/// PUT /api/users/:id/suspend
///
/// Suspend a user (Admin only)
pub async fn suspend_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< SuspendUserRequest >,
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

  // Fetch target user
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) => return ( StatusCode::NOT_FOUND, Json( serde_json::json!({ "error": "User not found" }) ) ).into_response(),
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Database error" }) ) ).into_response(),
  };

  // Prevent suspending self
  if target_user.username == claims.sub
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "Cannot suspend yourself" }) ) ).into_response();
  }

  // Prevent suspending admin
  if target_user.username == "admin"
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "Cannot suspend the admin user" }) ) ).into_response();
  }

  // Already suspended?
  if !target_user.is_active
  {
    return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "User is already suspended" }) ) ).into_response();
  }

  let suspended_at = Utc::now().timestamp_millis();
  let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

  // Update user
  let result = sqlx::query(
    "UPDATE users SET is_active = 0, suspended_at = ?, suspended_by = ? WHERE id = ?"
  )
  .bind( suspended_at )
  .bind( admin_id )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) =>
    {
      // Audit log
      let _ = create_audit_log(
        &state.db_pool,
        "suspend",
        user_id,
        admin_id,
        Some( serde_json::json!({ "is_active": true }).to_string() ),
        Some( serde_json::json!({ "is_active": false }).to_string() ),
        request.reason
      ).await;

      // Return updated user
      let updated_user = UserResponse {
        is_active: false,
        suspended_at: Some( suspended_at ),
        ..target_user
      };
      ( StatusCode::OK, Json( updated_user ) ).into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to suspend user: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to suspend user" }) ) ).into_response()
    }
  }
}

/// PUT /api/users/:id/activate
///
/// Activate a user (Admin only)
pub async fn activate_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
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

  // Fetch target user
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) => return ( StatusCode::NOT_FOUND, Json( serde_json::json!({ "error": "User not found" }) ) ).into_response(),
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Database error" }) ) ).into_response(),
  };

  // Already active?
  if target_user.is_active
  {
    return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "User is already active" }) ) ).into_response();
  }

  let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

  // Update user
  let result = sqlx::query(
    "UPDATE users SET is_active = 1, suspended_at = NULL, suspended_by = NULL WHERE id = ?"
  )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) =>
    {
      // Audit log
      let _ = create_audit_log(
        &state.db_pool,
        "activate",
        user_id,
        admin_id,
        Some( serde_json::json!({ "is_active": false }).to_string() ),
        Some( serde_json::json!({ "is_active": true }).to_string() ),
        None
      ).await;

      // Return updated user
      let updated_user = UserResponse {
        is_active: true,
        suspended_at: None,
        ..target_user
      };
      ( StatusCode::OK, Json( updated_user ) ).into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to activate user: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to activate user" }) ) ).into_response()
    }
  }
}

/// DELETE /api/users/:id
///
/// Soft delete a user (Admin only)
pub async fn delete_user(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
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

  // Fetch target user
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) => return ( StatusCode::NOT_FOUND, Json( serde_json::json!({ "error": "User not found" }) ) ).into_response(),
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Database error" }) ) ).into_response(),
  };

  // Prevent deleting self
  if target_user.username == claims.sub
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "Cannot delete yourself" }) ) ).into_response();
  }

  // Prevent deleting admin
  if target_user.username == "admin"
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "Cannot delete the admin user" }) ) ).into_response();
  }

  let deleted_at = Utc::now().timestamp_millis();
  let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

  // Soft delete user
  let result = sqlx::query(
    "UPDATE users SET is_active = 0, deleted_at = ?, deleted_by = ? WHERE id = ?"
  )
  .bind( deleted_at )
  .bind( admin_id )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) =>
    {
      // Audit log
      let _ = create_audit_log(
        &state.db_pool,
        "delete",
        user_id,
        admin_id,
        None,
        None,
        None
      ).await;

      // Return updated user
      let updated_user = UserResponse {
        is_active: false,
        deleted_at: Some( deleted_at ),
        ..target_user
      };
      ( StatusCode::OK, Json( updated_user ) ).into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to delete user: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to delete user" }) ) ).into_response()
    }
  }
}

/// PUT /api/users/:id/role
///
/// Change user role (Admin only)
pub async fn change_user_role(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< ChangeRoleRequest >,
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

  // Validate role
  if !["viewer", "user", "admin"].contains( &request.role.as_str() )
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "role must be one of: viewer, user, admin" }) ) ).into_response();
  }

  // Fetch target user
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) => return ( StatusCode::NOT_FOUND, Json( serde_json::json!({ "error": "User not found" }) ) ).into_response(),
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Database error" }) ) ).into_response(),
  };

  // Prevent changing self role
  if target_user.username == claims.sub
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "Cannot change your own role" }) ) ).into_response();
  }

  let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

  // Update role
  let result = sqlx::query(
    "UPDATE users SET role = ? WHERE id = ?"
  )
  .bind( &request.role )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) =>
    {
      // Audit log
      let _ = create_audit_log(
        &state.db_pool,
        "role_change",
        user_id,
        admin_id,
        Some( serde_json::json!({ "role": target_user.role }).to_string() ),
        Some( serde_json::json!({ "role": request.role }).to_string() ),
        None
      ).await;

      // Return updated user
      let updated_user = UserResponse {
        role: request.role,
        ..target_user
      };
      ( StatusCode::OK, Json( updated_user ) ).into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to change user role: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to change user role" }) ) ).into_response()
    }
  }
}

/// POST /api/users/:id/reset-password
///
/// Reset user password (Admin only)
pub async fn reset_user_password(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< ResetPasswordRequest >,
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

  // Validate password
  if request.new_password.len() < 8
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({ "error": "password must be at least 8 characters" }) ) ).into_response();
  }

  // Fetch target user
  let target_user = match sqlx::query_as::<_, UserResponse>(
    "SELECT id, username, email, role, is_active, created_at, last_login, suspended_at, deleted_at FROM users WHERE id = ? AND deleted_at IS NULL"
  )
  .bind( user_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( user ) ) => user,
    Ok( None ) => return ( StatusCode::NOT_FOUND, Json( serde_json::json!({ "error": "User not found" }) ) ).into_response(),
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Database error" }) ) ).into_response(),
  };

  // Hash password
  let password_hash = match bcrypt::hash( &request.new_password, bcrypt::DEFAULT_COST )
  {
    Ok( hash ) => hash,
    Err( _ ) => return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to hash password" }) ) ).into_response(),
  };

  let admin_id = 999; // Placeholder

  // Update password
  let result = sqlx::query(
    "UPDATE users SET password_hash = ?, force_password_change = ? WHERE id = ?"
  )
  .bind( password_hash )
  .bind( if request.force_change { 1 } else { 0 } )
  .bind( user_id )
  .execute( &state.db_pool )
  .await;

  match result
  {
    Ok( _ ) =>
    {
      // Audit log
      let _ = create_audit_log(
        &state.db_pool,
        "password_reset",
        user_id,
        admin_id,
        None,
        None,
        None
      ).await;

      ( StatusCode::OK, Json( target_user ) ).into_response()
    },
    Err( e ) =>
    {
      tracing::error!( "Failed to reset password: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({ "error": "Failed to reset password" }) ) ).into_response()
    }
  }
}

/// PATCH /api/users/:id/status
///
/// Deprecated: Use /api/users/:id/suspend or /api/users/:id/activate instead
pub async fn update_user_status(
  State( state ): State< AuthState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< crate::routes::users::UpdateUserStatusRequest >,
) -> impl IntoResponse
{
  // Reuse suspend/activate logic based on request
  // For backward compatibility or if frontend still uses it
  // But since we are updating frontend, we can remove this or keep it as wrapper
  
  // Just forward to suspend or activate
  if request.is_active
  {
    activate_user( State( state ), AuthenticatedUser( claims ), Path( user_id ) ).await.into_response()
  }
  else
  {
    let suspend_req = SuspendUserRequest { reason: None };
    suspend_user( State( state ), AuthenticatedUser( claims ), Path( user_id ), Json( suspend_req ) ).await.into_response()
  }
}

/// Request body for updating user status (Deprecated)
#[ derive( Debug, Deserialize ) ]
pub struct UpdateUserStatusRequest
{
  pub is_active: bool,
}
