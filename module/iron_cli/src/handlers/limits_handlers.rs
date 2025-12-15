//! Limits command handlers
//!
//! Pure functions for limits list, get, create, update, delete operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use super::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .limits.list command
///
/// Lists all limits.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_limits_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!("List limits\nFormat: {}", format))
}

/// Handle .limits.get command
///
/// Gets details for a specific limit.
///
/// ## Parameters
///
/// Required:
/// - limit_id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_limit_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let limit_id = params
    .get("limit_id")
    .ok_or(CliError::MissingParameter("limit_id"))?;

  // Validate limit_id
  validate_non_empty(limit_id, "limit_id")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Get limit\nLimit ID: {}\nFormat: {}",
    limit_id, format
  ))
}

/// Handle .limits.create command
///
/// Creates a new limit.
///
/// ## Parameters
///
/// Required:
/// - resource_type: String (non-empty)
/// - limit_value: String (positive integer)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn create_limit_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let resource_type = params
    .get("resource_type")
    .ok_or(CliError::MissingParameter("resource_type"))?;

  let limit_value_str = params
    .get("limit_value")
    .ok_or(CliError::MissingParameter("limit_value"))?;

  // Validate resource_type
  validate_non_empty(resource_type, "resource_type")?;

  // Validate limit_value
  validate_non_negative_integer(limit_value_str, "limit_value")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Create limit\nResource type: {}\nLimit value: {}\nFormat: {}",
    resource_type, limit_value_str, format
  ))
}

/// Handle .limits.update command
///
/// Updates an existing limit.
///
/// ## Parameters
///
/// Required:
/// - limit_id: String (non-empty)
/// - limit_value: String (positive integer)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn update_limit_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let limit_id = params
    .get("limit_id")
    .ok_or(CliError::MissingParameter("limit_id"))?;

  let limit_value_str = params
    .get("limit_value")
    .ok_or(CliError::MissingParameter("limit_value"))?;

  // Validate limit_id
  validate_non_empty(limit_id, "limit_id")?;

  // Validate limit_value
  validate_non_negative_integer(limit_value_str, "limit_value")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Update limit\nLimit ID: {}\nNew value: {}\nFormat: {}",
    limit_id, limit_value_str, format
  ))
}

/// Handle .limits.delete command
///
/// Deletes a limit.
///
/// ## Parameters
///
/// Required:
/// - limit_id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn delete_limit_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let limit_id = params
    .get("limit_id")
    .ok_or(CliError::MissingParameter("limit_id"))?;

  // Validate limit_id
  validate_non_empty(limit_id, "limit_id")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Limit deleted successfully\nLimit ID: {}\nFormat: {}",
    limit_id, format
  ))
}
