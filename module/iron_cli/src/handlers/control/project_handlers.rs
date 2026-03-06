//! Project command handlers for control API
//!
//! Pure functions for project access operations (read-only in Pilot).
//! No I/O - all external operations handled by adapter layer.

use crate::handlers::validation::validate_non_empty;
use crate::handlers::CliError;
use std::collections::HashMap;

/// Handle .project.list command
///
/// Lists all projects.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn list_projects_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!("Project list parameters valid\nFormat: {format}"))
}

/// Handle .project.get command
///
/// Gets project details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn get_project_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  // Validate required parameter
  let id = params.get("id").ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Get project parameters valid\nID: {id}\nFormat: {format}"
  ))
}
