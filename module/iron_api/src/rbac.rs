//! RBAC (Role-Based Access Control) module
//!
//! Phase 4 Day 27: RBAC Authorization implementation
//!
//! # Architecture
//!
//! Three-tier permission model:
//! - **Admin**: Full system access (manage users, tokens, all agent operations)
//! - **User**: Standard access (read/write/stop own agents, read metrics)
//! - **Viewer**: Read-only access (read agents and metrics only)
//!
//! # Usage
//!
//! ```rust
//! use iron_api::rbac::{ Role, Permission, PermissionChecker };
//!
//! let checker = PermissionChecker::new();
//!
//! // Check if user role has permission to stop agents
//! if checker.has_permission( Role::User, Permission::StopAgents )
//! {
//!   // Allow operation
//! }
//! ```

use serde::{ Serialize, Deserialize };
use core::str::FromStr;

/// User role in RBAC system
///
/// Roles are ordered by privilege level: Admin > User > Viewer
#[ derive( Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize ) ]
pub enum Role
{
  /// Read-only access
  Viewer = 0,
  /// Standard user access
  User = 1,
  /// Full administrative access
  Admin = 2,
}

impl Role
{
  /// Convert role to string
  #[ must_use ]
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      Self::Admin => "admin",
      Self::User => "user",
      Self::Viewer => "viewer",
    }
  }
}

impl FromStr for Role
{
  type Err = String;

  /// Parse role from string (case-insensitive)
  ///
  /// # Arguments
  ///
  /// * `s` - Role name ("admin", "user", "viewer")
  ///
  /// # Errors
  ///
  /// Returns error if role name is invalid
  fn from_str( s: &str ) -> Result< Self, Self::Err >
  {
    match s.to_lowercase().as_str()
    {
      "admin" => Ok( Self::Admin ),
      "user" => Ok( Self::User ),
      "viewer" => Ok( Self::Viewer ),
      _ => Err( format!( "Invalid role: {}", s ) ),
    }
  }
}

/// System permissions
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize ) ]
pub enum Permission
{
  /// Read agent status and information
  ReadAgents,
  /// Create and modify agents
  WriteAgents,
  /// Stop running agents
  StopAgents,
  /// Read usage metrics and statistics
  ReadMetrics,
  /// Manage user accounts and roles
  ManageUsers,
  /// Manage API tokens
  ManageTokens,
}

impl FromStr for Permission
{
  type Err = String;

  /// Parse permission from string
  ///
  /// # Arguments
  ///
  /// * `s` - Permission name (snake_case: "read_agents", "manage_users", etc.)
  ///
  /// # Errors
  ///
  /// Returns error if permission name is invalid
  fn from_str( s: &str ) -> Result< Self, Self::Err >
  {
    match s
    {
      "read_agents" => Ok( Self::ReadAgents ),
      "write_agents" => Ok( Self::WriteAgents ),
      "stop_agents" => Ok( Self::StopAgents ),
      "read_metrics" => Ok( Self::ReadMetrics ),
      "manage_users" => Ok( Self::ManageUsers ),
      "manage_tokens" => Ok( Self::ManageTokens ),
      _ => Err( format!( "Invalid permission: {}", s ) ),
    }
  }
}

/// Permission checker for RBAC
///
/// Determines which permissions each role has access to.
#[ derive( Debug, Clone ) ]
pub struct PermissionChecker
{
  // Future: Could add dynamic permission overrides here
}

impl PermissionChecker
{
  /// Create new permission checker
  #[ must_use ]
  pub fn new() -> Self
  {
    Self {}
  }

  /// Check if role has permission
  ///
  /// # Arguments
  ///
  /// * `role` - User role
  /// * `permission` - Required permission
  ///
  /// # Returns
  ///
  /// `true` if role has permission, `false` otherwise
  ///
  /// # Permission Matrix
  ///
  /// | Permission      | Admin | User | Viewer |
  /// |-----------------|-------|------|--------|
  /// | ReadAgents      | ✓     | ✓    | ✓      |
  /// | WriteAgents     | ✓     | ✓    | ✗      |
  /// | StopAgents      | ✓     | ✓    | ✗      |
  /// | ReadMetrics     | ✓     | ✓    | ✓      |
  /// | ManageUsers     | ✓     | ✗    | ✗      |
  /// | ManageTokens    | ✓     | ✗    | ✗      |
  #[ must_use ]
  pub fn has_permission( &self, role: Role, permission: Permission ) -> bool
  {
    match role
    {
      Role::Admin => true, // Admin has all permissions

      Role::User =>
      {
        matches!(
          permission,
          Permission::ReadAgents
            | Permission::WriteAgents
            | Permission::StopAgents
            | Permission::ReadMetrics
        )
      }

      Role::Viewer =>
      {
        matches!( permission, Permission::ReadAgents | Permission::ReadMetrics )
      }
    }
  }
}

impl Default for PermissionChecker
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Axum middleware for role-based route protection
///
/// # Usage
///
/// ```rust,no_run
/// use iron_api::rbac::middleware::{ check_permission, UserContext };
/// use iron_api::rbac::{ Permission, Role };
/// use axum::{ extract::Request, middleware::Next, response::Response };
///
/// async fn permission_middleware( request: Request, next: Next ) -> Response
/// {
///   let user_context = UserContext
///   {
///     user_id: "user_123".to_string(),
///     role: Role::User,
///   };
///   check_permission( user_context, Permission::ReadAgents, request, next ).await
/// }
/// ```
pub mod middleware
{
  use super::{ Permission, PermissionChecker, Role };
  use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{ IntoResponse, Response },
  };
  use serde_json::json;

  /// User context extracted from request
  ///
  /// This should be populated by JWT authentication middleware
  #[ derive( Debug, Clone ) ]
  pub struct UserContext
  {
    pub user_id: String,
    pub role: Role,
  }

  /// Require specific permission for route access
  ///
  /// # Arguments
  ///
  /// * `request` - HTTP request
  /// * `next` - Next middleware in chain
  /// * `required_permission` - Permission required to access route
  ///
  /// # Returns
  ///
  /// 403 Forbidden if permission denied, otherwise continues to next middleware
  pub async fn check_permission(
    user_context: UserContext,
    required_permission: Permission,
    request: Request,
    next: Next,
  ) -> Response
  {
    let checker = PermissionChecker::new();

    if checker.has_permission( user_context.role, required_permission )
    {
      // Permission granted - continue to handler
      next.run( request ).await
    }
    else
    {
      // Permission denied
      (
        StatusCode::FORBIDDEN,
        axum::Json( json!({
          "error": "Insufficient permissions",
          "required_permission": format!( "{:?}", required_permission ),
          "user_role": user_context.role.as_str(),
        }) ),
      )
        .into_response()
    }
  }

  /// Extract user role from JWT claims (helper for testing)
  ///
  /// In production, this would be replaced with actual JWT extraction middleware
  #[ must_use ]
  pub fn extract_role_from_claims( claims: &crate::jwt_auth::AccessTokenClaims ) -> Role
  {
    use std::str::FromStr;
    Role::from_str( &claims.role ).unwrap_or( Role::User )
  }
}
