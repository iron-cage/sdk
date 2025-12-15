//! Project command handlers for control API
//!
//! Pure functions for project access operations (read-only in Pilot).
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::validate_non_empty;

/// Handle .project.list command
///
/// Lists all projects.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_projects_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Project list parameters valid\nFormat: {}",
    format
  ))
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
pub fn get_project_handler(
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
    "Get project parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}
