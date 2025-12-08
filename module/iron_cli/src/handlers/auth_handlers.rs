//! Authentication command handlers
//!
//! Pure functions for login, refresh, logout operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;

/// Handle .auth.login command
///
/// Validates username and password parameters, returns formatted output.
/// Actual authentication happens in adapter layer (not here).
///
/// ## Parameters
///
/// Required:
/// - username: String (3-100 chars, pattern: ^[a-zA-Z0-9@._-]+$)
/// - password: String (non-empty)
///
/// Optional:
/// - format: String (table|expanded|json|yaml, default: table)
pub fn login_handler(
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

  // Validate username format
  if username.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "cannot be empty",
    });
  }

  if username.len() < 3
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "must be at least 3 characters",
    });
  }

  if username.len() > 100
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "must be at most 100 characters",
    });
  }

  // Validate username pattern: ^[a-zA-Z0-9@._-]+$
  if !username.chars().all(|c| {
    c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-')
  })
  {
    return Err(CliError::InvalidParameter {
      param: "username",
      reason: "must match pattern ^[a-zA-Z0-9@._-]+$",
    });
  }

  // Validate password
  if password.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "password",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Login validation successful\nUsername: {}\nFormat: {}",
    username, format
  ))
}

/// Handle .auth.refresh command
///
/// Refreshes authentication tokens. No parameters required.
pub fn refresh_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!("Refresh command\nFormat: {}", format))
}

/// Handle .auth.logout command
///
/// Logs out current user. No parameters required.
pub fn logout_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!("Logout successful\nFormat: {}", format))
}
