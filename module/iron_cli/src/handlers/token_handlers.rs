//! Token command handlers
//!
//! Pure functions for token generate, list, get, rotate, revoke operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use super::validation::{ validate_token_id, validate_non_empty };

/// Handle .tokens.generate command
///
/// Validates token generation parameters.
///
/// ## Parameters
///
/// Required:
/// - name: String (non-empty)
/// - scope: String (format: action:resource, e.g., "read:tokens")
///
/// Optional:
/// - ttl: String (integer seconds, 60-31536000)
/// - format: String (table|expanded|json|yaml, default: table)
pub fn generate_token_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let name = params
    .get("name")
    .ok_or(CliError::MissingParameter("name"))?;

  let scope = params
    .get("scope")
    .ok_or(CliError::MissingParameter("scope"))?;

  // Validate name
  validate_non_empty(name, "name")?;

  // Validate scope format (must contain ':')
  if !scope.contains(':')
  {
    return Err(CliError::InvalidParameter {
      param: "scope",
      reason: "must match format 'action:resource' (e.g., 'read:tokens')",
    });
  }

  // Validate optional TTL
  if let Some(ttl_str) = params.get("ttl")
  {
    match ttl_str.parse::<i64>()
    {
      Ok(ttl) =>
      {
        if ttl < 60
        {
          return Err(CliError::InvalidParameter {
            param: "ttl",
            reason: "must be at least 60 seconds",
          });
        }
        if ttl > 31536000
        {
          return Err(CliError::InvalidParameter {
            param: "ttl",
            reason: "must be at most 31536000 seconds (1 year)",
          });
        }
      }
      Err(_) =>
      {
        return Err(CliError::InvalidParameter {
          param: "ttl",
          reason: "must be a valid integer",
        });
      }
    }
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Token generation parameters valid\nName: {}\nScope: {}\nFormat: {}",
    name, scope, format
  ))
}

/// Handle .tokens.list command
///
/// Lists tokens with optional filtering and sorting.
///
/// ## Parameters
///
/// Optional:
/// - filter: String (filter criteria)
/// - sort: String (sort field)
/// - format: String (table|json|yaml, default: table)
pub fn list_tokens_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("none");
  let sort = params.get("sort").map(|s| s.as_str()).unwrap_or("default");

  Ok(format!(
    "List tokens\nFormat: {}\nFilter: {}\nSort: {}",
    format, filter, sort
  ))
}

/// Handle .tokens.get command
///
/// Gets details for a specific token.
///
/// ## Parameters
///
/// Required:
/// - token_id: String (format: tok_*, non-empty)
///
/// Optional:
/// - format: String (table|expanded|json|yaml, default: table)
pub fn get_token_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let token_id = params
    .get("token_id")
    .ok_or(CliError::MissingParameter("token_id"))?;

  // Validate token_id format
  validate_token_id(token_id, "token_id")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Get token\nToken ID: {}\nFormat: {}",
    token_id, format
  ))
}

/// Handle .tokens.rotate command
///
/// Rotates a token (generates new value, preserves metadata).
///
/// ## Parameters
///
/// Required:
/// - token_id: String (format: tok_*, non-empty)
///
/// Optional:
/// - ttl: String (integer seconds)
/// - format: String (table|json|yaml, default: table)
pub fn rotate_token_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let token_id = params
    .get("token_id")
    .ok_or(CliError::MissingParameter("token_id"))?;

  // Validate token_id format
  validate_token_id(token_id, "token_id")?;

  // Validate optional TTL
  if let Some(ttl_str) = params.get("ttl")
  {
    if ttl_str.parse::<i64>().is_err()
    {
      return Err(CliError::InvalidParameter {
        param: "ttl",
        reason: "must be a valid integer",
      });
    }
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Rotate token\nToken ID: {}\nFormat: {}",
    token_id, format
  ))
}

/// Handle .tokens.revoke command
///
/// Revokes a token (makes it unusable).
///
/// ## Parameters
///
/// Required:
/// - token_id: String (format: tok_*, non-empty)
///
/// Optional:
/// - reason: String (revocation reason, can be empty)
/// - format: String (table|json|yaml, default: table)
pub fn revoke_token_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let token_id = params
    .get("token_id")
    .ok_or(CliError::MissingParameter("token_id"))?;

  // Validate token_id format
  if token_id.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "token_id",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let reason = params.get("reason").map(|s| s.as_str()).unwrap_or("none");

  Ok(format!(
    "Token revoke successful\nToken ID: {}\nReason: {}\nFormat: {}",
    token_id, reason, format
  ))
}
