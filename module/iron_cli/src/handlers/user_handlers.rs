//! User management command handlers
//!
//! Pure functions for user create, list, get, suspend, activate, delete,
//! role change, and password reset operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use super::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .users.create command
///
/// Validates user creation parameters.
///
/// ## Parameters
///
/// Required:
/// - username: String (non-empty, max 255 chars)
/// - password: String (min 8 chars, max 1000 chars)
/// - email: String (non-empty, contains @, max 255 chars)
/// - role: String (viewer|user|admin)
pub fn create_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let username = params
    .get("username")
    .ok_or(CliError::MissingParameter("username"))?;

  let password = params
    .get("password")
    .ok_or(CliError::MissingParameter("password"))?;

  let email = params
    .get("email")
    .ok_or(CliError::MissingParameter("email"))?;

  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  // Validate username
  validate_non_empty(username, "username")?;
  if username.len() > 255
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "cannot exceed 255 characters",
    });
  }

  // Validate password
  if password.len() < 8
  {
    return Err(CliError::InvalidParameter {
      param: "password",
      reason: "must be at least 8 characters",
    });
  }
  if password.len() > 1000
  {
    return Err(CliError::InvalidParameter {
      param: "password",
      reason: "cannot exceed 1000 characters",
    });
  }

  // Validate email
  validate_non_empty(email, "email")?;
  if !email.contains('@')
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "must contain @ symbol",
    });
  }
  if email.len() > 255
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "cannot exceed 255 characters",
    });
  }

  // Validate role
  validate_user_role(role)?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!(
    "User '{}' created successfully with role '{}'",
    username, role
  ))
}

/// Handle .users.list command
///
/// Validates user listing parameters.
///
/// ## Parameters
///
/// All optional:
/// - role: String (viewer|user|admin)
/// - is_active: String (true|false)
/// - search: String (username or email search)
/// - page: String (integer, default 1)
/// - page_size: String (integer, 1-100, default 20)
pub fn list_users_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional role filter
  if let Some(role) = params.get("role")
  {
    validate_user_role(role)?;
  }

  // Validate optional is_active filter
  if let Some(is_active) = params.get("is_active")
  {
    validate_boolean(is_active, "is_active")?;
  }

  // Validate optional page
  if let Some(page_str) = params.get("page")
  {
    let page = validate_non_negative_integer(page_str, "page")?;
    if page < 1
    {
      return Err(CliError::InvalidParameter {
        param: "page",
        reason: "must be at least 1",
      });
    }
  }

  // Validate optional page_size
  if let Some(page_size_str) = params.get("page_size")
  {
    let page_size = validate_non_negative_integer(page_size_str, "page_size")?;
    if !(1..=100).contains(&page_size)
    {
      return Err(CliError::InvalidParameter {
        param: "page_size",
        reason: "must be between 1 and 100",
      });
    }
  }

  // Placeholder response - actual API call handled by adapter
  Ok("User list retrieved successfully".to_string())
}

/// Handle .users.get command
///
/// Validates user retrieval parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn get_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} retrieved successfully", user_id))
}

/// Handle .users.suspend command
///
/// Validates user suspension parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
///
/// Optional:
/// - reason: String (suspension reason)
pub fn suspend_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} suspended successfully", user_id))
}

/// Handle .users.activate command
///
/// Validates user activation parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn activate_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} activated successfully", user_id))
}

/// Handle .users.delete command
///
/// Validates user deletion parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
pub fn delete_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} deleted successfully", user_id))
}

/// Handle .users.change_role command
///
/// Validates user role change parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
/// - role: String (viewer|user|admin)
pub fn change_user_role_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Validate required role
  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  validate_user_role(role)?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("User {} role changed to '{}' successfully", user_id, role))
}

/// Handle .users.reset_password command
///
/// Validates password reset parameters.
///
/// ## Parameters
///
/// Required:
/// - user_id: String (integer)
/// - new_password: String (min 8 chars, max 1000 chars)
/// - force_change: String (true|false)
pub fn reset_password_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required user_id
  let user_id_str = params
    .get("user_id")
    .ok_or(CliError::MissingParameter("user_id"))?;

  let user_id = validate_non_negative_integer(user_id_str, "user_id")?;

  // Validate required new_password
  let new_password = params
    .get("new_password")
    .ok_or(CliError::MissingParameter("new_password"))?;

  if new_password.len() < 8
  {
    return Err(CliError::InvalidParameter {
      param: "new_password",
      reason: "must be at least 8 characters",
    });
  }
  if new_password.len() > 1000
  {
    return Err(CliError::InvalidParameter {
      param: "new_password",
      reason: "cannot exceed 1000 characters",
    });
  }

  // Validate required force_change
  let force_change_str = params
    .get("force_change")
    .ok_or(CliError::MissingParameter("force_change"))?;

  validate_boolean(force_change_str, "force_change")?;

  // Placeholder response - actual API call handled by adapter
  Ok(format!("Password reset for user {} successfully", user_id))
}

// Helper validation functions

/// Validates that role is one of: viewer, user, admin
fn validate_user_role(role: &str) -> Result<(), CliError>
{
  match role
  {
    "viewer" | "user" | "admin" => Ok(()),
    _ =>
    {
      Err(CliError::InvalidParameter {
        param: "role",
        reason: "must be one of: viewer, user, admin",
      })
    }
  }
}

/// Validates that a string is a boolean (true|false)
fn validate_boolean(value: &str, param_name: &'static str) -> Result<(), CliError>
{
  match value
  {
    "true" | "false" => Ok(()),
    _ =>
    {
      Err(CliError::InvalidParameter {
        param: param_name,
        reason: "must be 'true' or 'false'",
      })
    }
  }
}
