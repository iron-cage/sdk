//! User management API endpoints
//!
//! Provides admin operations for managing users:
//! - Create users
//! - List users with filters
//! - Get user details
//! - Suspend/activate accounts
//! - Delete users
//! - Change user roles
//! - Reset passwords
//!
//! All endpoints require Admin role (ManageUsers permission).

use axum::
{
  extract::{ Path, Query, State },
  http::StatusCode,
  response::IntoResponse,
  Json,
};
use serde::{ Deserialize, Serialize };
use sqlx::{ Pool, Sqlite };
use std::sync::Arc;

use crate::rbac::{ Permission, PermissionChecker, Role };
use iron_token_manager::user_service::
{
  CreateUserParams, ListUsersFilters, User, UserService,
};
use crate::jwt_auth::AuthenticatedUser;
use std::str::FromStr;

/// State for user management endpoints
#[ derive( Clone ) ]
pub struct UserManagementState
{
  pub db_pool: Pool< Sqlite >,
  pub permission_checker: Arc< PermissionChecker >,
}

impl UserManagementState
{
  pub fn new( db_pool: Pool< Sqlite >, permission_checker: Arc< PermissionChecker > ) -> Self
  {
    Self {
      db_pool,
      permission_checker,
    }
  }
}

/// Helper to get admin ID from username
async fn get_admin_id( pool: &Pool< Sqlite >, username: &str ) -> Result< i64, sqlx::Error >
{
  let row: ( i64, ) = sqlx::query_as( "SELECT id FROM users WHERE username = ?" )
    .bind( username )
    .fetch_one( pool )
    .await?;
  Ok( row.0 )
}

//
// Request/Response types
//

/// Request to create a new user
#[ derive( Debug, Deserialize, Serialize ) ]
pub struct CreateUserRequest
{
  pub username: String,
  pub password: String,
  pub email: String,
  pub role: String,
}

impl CreateUserRequest
{
  const MAX_USERNAME_LENGTH: usize = 255;
  const MAX_PASSWORD_LENGTH: usize = 1000;
  const MAX_EMAIL_LENGTH: usize = 255;
  const MIN_PASSWORD_LENGTH: usize = 8;

  pub fn validate( &self ) -> Result< (), String >
  {
    // Username validation
    if self.username.is_empty()
    {
      return Err( "username cannot be empty".to_string() );
    }
    if self.username.len() > Self::MAX_USERNAME_LENGTH
    {
      return Err( format!( "username exceeds maximum length of {}", Self::MAX_USERNAME_LENGTH ) );
    }

    // Password validation
    if self.password.len() < Self::MIN_PASSWORD_LENGTH
    {
      return Err( format!( "password must be at least {} characters", Self::MIN_PASSWORD_LENGTH ) );
    }
    if self.password.len() > Self::MAX_PASSWORD_LENGTH
    {
      return Err( format!( "password exceeds maximum length of {}", Self::MAX_PASSWORD_LENGTH ) );
    }

    // Email validation
    if self.email.is_empty()
    {
      return Err( "email cannot be empty".to_string() );
    }
    if self.email.len() > Self::MAX_EMAIL_LENGTH
    {
      return Err( format!( "email exceeds maximum length of {}", Self::MAX_EMAIL_LENGTH ) );
    }
    if !self.email.contains( '@' )
    {
      return Err( "email must contain @ symbol".to_string() );
    }

    // Role validation
    let valid_roles = [ "viewer", "user", "admin" ];
    if !valid_roles.contains( &self.role.as_str() )
    {
      return Err( format!( "role must be one of: {}", valid_roles.join( ", " ) ) );
    }

    Ok( () )
  }
}

/// Response from creating a user
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct CreateUserResponse
{
  pub id: i64,
  pub username: String,
  pub email: Option< String >,
  pub role: String,
  pub is_active: bool,
  pub created_at: i64,
}

impl From< User > for CreateUserResponse
{
  fn from( user: User ) -> Self
  {
    Self {
      id: user.id,
      username: user.username,
      email: user.email,
      role: user.role,
      is_active: user.is_active,
      created_at: user.created_at,
    }
  }
}

/// Query parameters for listing users
#[ derive( Debug, Deserialize ) ]
pub struct ListUsersQuery
{
  pub role: Option< String >,
  pub is_active: Option< bool >,
  pub search: Option< String >,
  pub page: Option< u32 >,
  pub page_size: Option< u32 >,
}

/// Response from listing users
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ListUsersResponse
{
  pub users: Vec< UserResponse >,
  pub total: i64,
  pub page: u32,
  pub page_size: u32,
}

/// User information in list response
#[ derive( Debug, Serialize, Deserialize ) ]
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

impl From< User > for UserResponse
{
  fn from( user: User ) -> Self
  {
    Self {
      id: user.id,
      username: user.username,
      email: user.email,
      role: user.role,
      is_active: user.is_active,
      created_at: user.created_at,
      last_login: user.last_login,
      suspended_at: user.suspended_at,
      deleted_at: user.deleted_at,
    }
  }
}

/// Request to suspend a user
#[ derive( Debug, Deserialize ) ]
pub struct SuspendUserRequest
{
  pub reason: Option< String >,
}

impl SuspendUserRequest
{
  const MAX_REASON_LENGTH: usize = 1000;

  pub fn validate( &self ) -> Result< (), String >
  {
    if let Some( ref reason ) = self.reason
    {
      if reason.len() > Self::MAX_REASON_LENGTH
      {
        return Err( format!( "reason exceeds maximum length of {}", Self::MAX_REASON_LENGTH ) );
      }
    }
    Ok( () )
  }
}

/// Request to change user role
#[ derive( Debug, Deserialize ) ]
pub struct ChangeRoleRequest
{
  pub role: String,
}

impl ChangeRoleRequest
{
  pub fn validate( &self ) -> Result< (), String >
  {
    let valid_roles = [ "viewer", "user", "admin" ];
    if !valid_roles.contains( &self.role.as_str() )
    {
      return Err( format!( "role must be one of: {}", valid_roles.join( ", " ) ) );
    }
    Ok( () )
  }
}

/// Request to reset password
#[ derive( Debug, Deserialize ) ]
pub struct ResetPasswordRequest
{
  pub new_password: String,
  pub force_change: Option< bool >,
}

impl ResetPasswordRequest
{
  const MAX_PASSWORD_LENGTH: usize = 1000;
  const MIN_PASSWORD_LENGTH: usize = 8;

  pub fn validate( &self ) -> Result< (), String >
  {
    if self.new_password.len() < Self::MIN_PASSWORD_LENGTH
    {
      return Err( format!( "password must be at least {} characters", Self::MIN_PASSWORD_LENGTH ) );
    }
    if self.new_password.len() > Self::MAX_PASSWORD_LENGTH
    {
      return Err( format!( "password exceeds maximum length of {}", Self::MAX_PASSWORD_LENGTH ) );
    }
    Ok( () )
  }
}

//
// Handler functions
//

/// Create a new user
///
/// POST /api/v1/users
/// Requires: Admin role
pub async fn create_user(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Json( request ): Json< CreateUserRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!
    ({
      "error": validation_error
    }) ) ).into_response();
  }

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Create user parameters
  let params = CreateUserParams
  {
    username: request.username,
    password: request.password,
    email: request.email,
    role: request.role,
  };

  let admin_id = claims.sub.parse::< i64 >().unwrap_or( 0 );

  // Create user
  match user_service.create_user( params, admin_id ).await
  {
    Ok( user ) =>
    {
      let response = CreateUserResponse::from( user );
      ( StatusCode::CREATED, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to create user: {}", e )
      }) ) ).into_response()
    }
  }
}

/// List users with optional filters
///
/// GET /api/v1/users?role=admin&is_active=true&search=john&page=1&page_size=20
/// Requires: Admin role
pub async fn list_users(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Query( query ): Query< ListUsersQuery >,
) -> impl IntoResponse
{
  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Build filters
  let page = query.page.unwrap_or( 1 );
  let page_size = query.page_size.unwrap_or( 20 ).min( 100 ); // Cap at 100
  let offset = ( page - 1 ) * page_size;

  let filters = ListUsersFilters
  {
    role: query.role,
    is_active: query.is_active,
    search: query.search,
    limit: Some( page_size as i64 ),
    offset: Some( offset as i64 ),
  };

  // List users
  match user_service.list_users( filters ).await
  {
    Ok( ( users, total ) ) =>
    {
      let response = ListUsersResponse
      {
        users: users.into_iter().map( UserResponse::from ).collect(),
        total,
        page,
        page_size,
      };
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to list users: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Get user by ID
///
/// GET /api/v1/users/{id}
/// Requires: Admin role
pub async fn get_user(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Get user
  match user_service.get_user_by_id( user_id ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::NOT_FOUND, Json( serde_json::json!
      ({
        "error": format!( "user not found: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Suspend a user account
///
/// PUT /api/v1/users/{id}/suspend
/// Requires: Admin role
pub async fn suspend_user(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< SuspendUserRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!
    ({
      "error": validation_error
    }) ) ).into_response();
  }

  // Get admin ID from claims
  let admin_id = claims.sub.parse::< i64 >().unwrap_or( 0 );

  tracing::info!( "admin_id: {}", admin_id );

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Suspend user
  match user_service.suspend_user( user_id, admin_id, request.reason ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to suspend user: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Activate a user account
///
/// PUT /api/v1/users/{id}/activate
/// Requires: Admin role
pub async fn activate_user(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Get admin ID from claims
  let admin_id = claims.sub.parse::< i64 >().unwrap_or( 0 );

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Activate user
  match user_service.activate_user( user_id, admin_id ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to activate user: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Delete a user account (soft delete)
///
/// DELETE /api/v1/users/{id}
/// Requires: Admin role
pub async fn delete_user(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Get admin ID from claims
  let admin_id = get_admin_id( &state.db_pool, &claims.sub ).await.unwrap_or( 0 );

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Delete user
  match user_service.delete_user( user_id, admin_id ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to delete user: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Change a user's role
///
/// PUT /api/v1/users/{id}/role
/// Requires: Admin role
pub async fn change_user_role(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< ChangeRoleRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!
    ({
      "error": validation_error
    }) ) ).into_response();
  }

  // Get admin ID from claims
  let admin_id = claims.sub.parse::< i64 >().unwrap_or( 0 );

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Change role
  match user_service.change_user_role( user_id, admin_id, request.role ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to change user role: {}", e )
      }) ) ).into_response()
    }
  }
}

/// Reset a user's password
///
/// POST /api/v1/users/{id}/reset-password
/// Requires: Admin role
pub async fn reset_password(
  State( state ): State< UserManagementState >,
  AuthenticatedUser( claims ): AuthenticatedUser,
  Path( user_id ): Path< i64 >,
  Json( request ): Json< ResetPasswordRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!
    ({
      "error": validation_error
    }) ) ).into_response();
  }

  // Get admin ID from claims
  let admin_id = claims.sub.parse::< i64 >().unwrap_or( 0 );

  // Check RBAC permission
  let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
  if !state.permission_checker.has_permission( role, Permission::ManageUsers )
  {
    return ( StatusCode::FORBIDDEN, Json( serde_json::json!
    ({
      "error": "insufficient permissions"
    }) ) ).into_response();
  }

  // Create user service
  let user_service = UserService::new( state.db_pool.clone() );

  // Reset password
  let force_change = request.force_change.unwrap_or( true );
  match user_service.reset_password( user_id, admin_id, request.new_password, force_change ).await
  {
    Ok( user ) =>
    {
      let response = UserResponse::from( user );
      ( StatusCode::OK, Json( response ) ).into_response()
    }
    Err( e ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!
      ({
        "error": format!( "failed to reset password: {}", e )
      }) ) ).into_response()
    }
  }
}
