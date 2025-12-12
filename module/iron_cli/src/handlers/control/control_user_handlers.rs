//! User management command handlers for control API
//!
//! Pure functions for user administration operations.
//! No I/O - all external operations handled by adapter layer.
//!
//! Note: Named `control_user_handlers` to distinguish from token CLI's user_handlers.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::{ validate_non_empty, validate_non_negative_integer };

/// Validates email format
fn validate_email(email: &str, param_name: &'static str) -> Result<(), CliError>
{
  if !email.contains('@') || !email.contains('.')
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "must be valid email format",
    });
  }

  Ok(())
}

/// Validates username pattern (alphanumeric, underscore, hyphen)
fn validate_username_pattern(username: &str) -> Result<(), CliError>
{
  let username_pattern = regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
    .map_err(|_| CliError::ValidationError("regex compilation failed".into()))?;

  if !username_pattern.is_match(username)
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "must contain only alphanumeric characters, underscores, and hyphens",
    });
  }

  Ok(())
}

/// Handle .user.list command
///
/// Lists all users.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_users_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "User list parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .user.create command
///
/// Creates new user.
///
/// ## Parameters
///
/// Required:
/// - username: String (3-50 chars, pattern: ^[a-zA-Z0-9_-]+$)
/// - email: String (valid email format)
/// - role: String (user|admin)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn create_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let username = params
    .get("username")
    .ok_or(CliError::MissingParameter("username"))?;

  let email = params
    .get("email")
    .ok_or(CliError::MissingParameter("email"))?;

  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  // Validate username
  validate_non_empty(username, "username")?;

  if username.len() < 3
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "must be at least 3 characters",
    });
  }

  if username.len() > 50
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "cannot exceed 50 characters",
    });
  }

  validate_username_pattern(username)?;

  // Validate email
  validate_non_empty(email, "email")?;
  validate_email(email, "email")?;

  // Validate role
  match role.as_str()
  {
    "user" | "admin" => {},
    _ => return Err(CliError::InvalidParameter {
      param: "role",
      reason: "must be either 'user' or 'admin'",
    }),
  }

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Create user parameters valid\nUsername: {}\nEmail: {}\nRole: {}\nFormat: {}",
    username, email, role, format
  ))
}

/// Handle .user.get command
///
/// Gets user details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Get user parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .user.update command
///
/// Updates user profile.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - email: String (valid email format)
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn update_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional email
  if let Some(email) = params.get("email")
  {
    validate_non_empty(email, "email")?;
    validate_email(email, "email")?;
  }

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Update user parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .user.delete command
///
/// Deletes user by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn delete_user_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Delete user parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .user.set_role command
///
/// Sets user role (admin/user).
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
/// - role: String (user|admin)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn set_user_role_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  let role = params
    .get("role")
    .ok_or(CliError::MissingParameter("role"))?;

  validate_non_empty(id, "id")?;

  // Validate role
  match role.as_str()
  {
    "user" | "admin" => {},
    _ => return Err(CliError::InvalidParameter {
      param: "role",
      reason: "must be either 'user' or 'admin'",
    }),
  }

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Set user role parameters valid\nID: {}\nRole: {}\nFormat: {}",
    id, role, format
  ))
}

/// Handle .user.reset_password command
///
/// Triggers password reset for user.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn reset_password_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Reset password parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .user.get_permissions command
///
/// Gets user permissions.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_user_permissions_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Get user permissions parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}
