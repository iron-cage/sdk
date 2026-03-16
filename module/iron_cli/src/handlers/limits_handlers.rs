//! Limits command handlers
//!
//! Pure functions for limits list, get, create, update, delete operations.
//! No I/O - all external operations handled by adapter layer.

use super::validation::{validate_non_empty, validate_non_negative_integer};
use crate::handlers::CliError;
use std::collections::HashMap;

/// Handle .limits.list command
///
/// Lists all limits.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn list_limits_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!("List limits\nFormat: {format}"))
}

/// Handle .limits.get command
///
/// Gets details for a specific limit.
///
/// ## Parameters
///
/// Required:
/// - `limit_id`: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn get_limit_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  // Validate required parameters
  let limit_id = params
    .get("limit_id")
    .ok_or(CliError::MissingParameter("limit_id"))?;

  // Validate limit_id
  validate_non_empty(limit_id, "limit_id")?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!("Get limit\nLimit ID: {limit_id}\nFormat: {format}"))
}

/// Handle .limits.create command
///
/// Creates a new limit.
///
/// ## Parameters
///
/// Required:
/// - `resource_type`: String (non-empty)
/// - `limit_value`: String (positive integer)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn create_limit_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
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
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Create limit\nResource type: {resource_type}\nLimit value: {limit_value_str}\nFormat: {format}"
  ))
}

/// Handle .limits.update command
///
/// Updates an existing limit.
///
/// ## Parameters
///
/// Required:
/// - `limit_id`: String (non-empty)
/// - `limit_value`: String (positive integer)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn update_limit_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
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
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Update limit\nLimit ID: {limit_id}\nNew value: {limit_value_str}\nFormat: {format}"
  ))
}

/// Handle .limits.delete command
///
/// Deletes a limit.
///
/// ## Parameters
///
/// Required:
/// - `limit_id`: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn delete_limit_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  // Validate required parameters
  let limit_id = params
    .get("limit_id")
    .ok_or(CliError::MissingParameter("limit_id"))?;

  // Validate limit_id
  validate_non_empty(limit_id, "limit_id")?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Limit deleted successfully\nLimit ID: {limit_id}\nFormat: {format}"
  ))
}
