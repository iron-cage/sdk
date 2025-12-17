//! Authentication command handlers
//!
//! Pure functions for login, refresh, logout operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;

/// Handle .auth.login command
///
/// Validates email and password parameters, returns formatted output.
/// Actual authentication happens in adapter layer (not here).
///
/// ## Parameters
///
/// Required:
/// - email: String (3-100 chars, pattern: ^[a-zA-Z0-9@._-]+$)
/// - password: String (non-empty)
///
/// Optional:
/// - format: String (table|expanded|json|yaml, default: table)
pub fn login_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let email = params
    .get("email")
    .ok_or(CliError::MissingParameter("email"))?;

  let password = params
    .get("password")
    .ok_or(CliError::MissingParameter("password"))?;

  // Validate email format
  if email.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "cannot be empty",
    });
  }

  if email.len() < 3
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "must be at least 3 characters",
    });
  }

  if email.len() > 100
  {
    return Err(CliError::InvalidParameter {
      param: "email",
      reason: "must be at most 100 characters",
    });
  }

  // Validate email pattern: ^[a-zA-Z0-9@._-]+$
  if !email.chars().all(|c| {
    c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '_' | '-')
  })
  {
    return Err(CliError::InvalidParameter {
      param: "email",
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
    "Login validation successful\nEmail: {}\nFormat: {}",
    email, format
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
