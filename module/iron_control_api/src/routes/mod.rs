//! REST API route handlers
//!
//! Phase 4 Days 28-29: REST API Endpoints

use core::str::FromStr;

use crate::{
  error::ApiError,
  rbac::{Permission, PermissionChecker, Role},
};

pub mod agent_provider_key;
pub mod agents;
pub mod analytics;
pub mod auth;
pub mod budget;
pub mod freeform;
pub mod health;
pub mod ic_token;
pub mod invites;
pub mod keys;
pub mod limits;
pub mod providers;
pub mod tokens;
pub mod usage;
pub mod users;
pub mod version;
pub mod workspace;

/// Resolve `role_str` to a [`Role`] and verify it grants `permission`.
///
/// Shared RBAC gate for the route modules (`workspace`, `invites`, `freeform`)
/// so the role-string lookup and permission check live in one place.
///
/// # Errors
///
/// Returns [`ApiError::Forbidden`] if the role string is unknown or the role
/// does not hold `permission`.
pub(super) fn check_permission(role_str: &str, permission: Permission) -> Result<(), ApiError> {
  let role = Role::from_str(role_str)
    .map_err(|_| ApiError::Forbidden(format!("Invalid role: {role_str}")))?;
  let checker = PermissionChecker::new();
  if checker.has_permission(role, permission) {
    Ok(())
  } else {
    Err(ApiError::Forbidden(format!(
      "Permission {permission:?} required"
    )))
  }
}
