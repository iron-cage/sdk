//! RBAC (Role-Based Access Control) module
//!
//! # Architecture
//!
//! Three-tier permission model:
//! - **Admin**: Full system access (manage users, roles, keys, tokens, all agent operations)
//! - **Manager**: Key/token management (read/write agents, manage provider keys and IC tokens)
//! - **Developer**: Read-only access (read agents, metrics, inspect own token)
//!
//! # Usage
//!
//! ```rust
//! use iron_control_api::rbac::{ Role, Permission, PermissionChecker };
//!
//! let checker = PermissionChecker::new();
//!
//! // Check if a Manager can manage provider keys
//! if checker.has_permission( Role::Manager, Permission::ManageProviderKeys )
//! {
//!   // Allow operation
//! }
//! ```

use core::str::FromStr;

use serde::{Deserialize, Serialize};

// Role is defined in iron_types as the single source of truth
pub use iron_types::Role;

/// System permissions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Permission {
  /// Read agent status and information
  ReadAgents,
  /// Create and modify agents
  WriteAgents,
  /// Manage AI provider keys (CRUD, rotation)
  ManageProviderKeys,
  /// Manage IC tokens (create, revoke, assign)
  ManageIcTokens,
  /// Inspect own IC token details
  ReadOwnToken,
  /// Manage user accounts (CRUD, suspend, activate)
  ManageUsers,
  /// Assign or change user roles (Admin-only privilege escalation guard)
  AssignRoles,
  /// Read usage metrics and statistics
  ReadMetrics,
}

impl FromStr for Permission {
  type Err = String;

  /// Parse permission from string
  ///
  /// # Errors
  ///
  /// Returns error if permission name is invalid
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    match s {
      "read_agents" => Ok(Self::ReadAgents),
      "write_agents" => Ok(Self::WriteAgents),
      "manage_provider_keys" => Ok(Self::ManageProviderKeys),
      "manage_ic_tokens" => Ok(Self::ManageIcTokens),
      "read_own_token" => Ok(Self::ReadOwnToken),
      "manage_users" => Ok(Self::ManageUsers),
      "assign_roles" => Ok(Self::AssignRoles),
      "read_metrics" => Ok(Self::ReadMetrics),
      _ => Err(format!("Invalid permission: {s}")),
    }
  }
}

/// Permission checker for RBAC
///
/// Determines which permissions each role has access to.
#[derive(Debug, Clone)]
pub struct PermissionChecker {
  // Future: Could add dynamic permission overrides here
}

impl PermissionChecker {
  /// Create new permission checker
  #[must_use]
  pub fn new() -> Self {
    Self {}
  }

  /// Check if role has permission
  ///
  /// # Permission Matrix
  ///
  /// | Permission         | Admin | Manager | Developer |
  /// |--------------------|-------|---------|-----------|
  /// | ReadAgents         | ✓     | ✓       | ✓         |
  /// | WriteAgents        | ✓     | ✓       | ✗         |
  /// | ManageProviderKeys | ✓     | ✓       | ✗         |
  /// | ManageIcTokens     | ✓     | ✓       | ✗         |
  /// | ReadOwnToken       | ✓     | ✓       | ✓         |
  /// | ManageUsers        | ✓     | ✗       | ✗         |
  /// | AssignRoles        | ✓     | ✗       | ✗         |
  /// | ReadMetrics        | ✓     | ✓       | ✓         |
  #[must_use]
  pub fn has_permission(&self, role: Role, permission: Permission) -> bool {
    match role {
      Role::Admin => true,

      Role::Manager => {
        matches!(
          permission,
          Permission::ReadAgents
            | Permission::WriteAgents
            | Permission::ManageProviderKeys
            | Permission::ManageIcTokens
            | Permission::ReadOwnToken
            | Permission::ReadMetrics
        )
      }

      Role::Developer => {
        matches!(
          permission,
          Permission::ReadAgents | Permission::ReadOwnToken | Permission::ReadMetrics
        )
      }
    }
  }
}

impl Default for PermissionChecker {
  fn default() -> Self {
    Self::new()
  }
}

/// Axum middleware for role-based route protection
///
/// # Usage
///
/// ```rust,no_run
/// use iron_control_api::rbac::middleware::{ check_permission, UserContext };
/// use iron_control_api::rbac::{ Permission, Role };
/// use axum::{ extract::Request, middleware::Next, response::Response };
///
/// async fn permission_middleware( request: Request, next: Next ) -> Response
/// {
///   let user_context = UserContext
///   {
///     user_id: "user_123".to_string(),
///     role: Role::Manager,
///   };
///   check_permission( user_context, Permission::ReadAgents, request, next ).await
/// }
/// ```
pub mod middleware {
  use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
  };
  use serde_json::json;

  use super::{Permission, PermissionChecker, Role};

  /// User context extracted from request
  ///
  /// This should be populated by JWT authentication middleware
  #[derive(Debug, Clone)]
  pub struct UserContext {
    /// Authenticated user identifier
    pub user_id: String,
    /// User assigned RBAC role
    pub role: Role,
  }

  /// Require specific permission for route access
  ///
  /// Returns 403 Forbidden if permission denied, otherwise continues to next middleware
  pub async fn check_permission(
    user_context: UserContext,
    required_permission: Permission,
    request: Request,
    next: Next,
  ) -> Response {
    let checker = PermissionChecker::new();

    if checker.has_permission(user_context.role, required_permission) {
      next.run(request).await
    } else {
      (
        StatusCode::FORBIDDEN,
        axum::Json(json!({
          "error": "Insufficient permissions",
          "required_permission": format!( "{:?}", required_permission ),
          "user_role": user_context.role.as_str(),
        })),
      )
        .into_response()
    }
  }

  /// Extract user role from JWT claims
  #[must_use]
  pub fn extract_role_from_claims(claims: &crate::jwt_auth::AccessTokenClaims) -> Role {
    use core::str::FromStr;
    Role::from_str(&claims.role).unwrap_or(Role::Developer)
  }
}
