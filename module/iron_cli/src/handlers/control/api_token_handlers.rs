//! API token command handlers for control API
//!
//! Pure functions for API token management operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .api_token.list command
///
/// Lists all API tokens.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_api_tokens_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "API token list parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .api_token.create command
///
/// Creates new API token.
///
/// ## Parameters
///
/// Required:
/// - name: String (non-empty, max 100 chars)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn create_api_token_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let name = params
    .get("name")
    .ok_or(CliError::MissingParameter("name"))?;

  validate_non_empty(name, "name")?;

  if name.len() > 100
  {
    return Err(CliError::InvalidParameter {
      param: "name",
      reason: "cannot exceed 100 characters",
    });
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
    "API token creation parameters valid\nName: {}\nFormat: {}",
    name, format
  ))
}

/// Handle .api_token.get command
///
/// Gets API token details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_api_token_handler(
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
    "Get API token parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .api_token.revoke command
///
/// Revokes API token by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn revoke_api_token_handler(
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
    "Revoke API token parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}
