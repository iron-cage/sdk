//! Project adapter functions

use super::{ControlApiClient, ControlApiConfig};
use crate::formatting::{OutputFormat, TreeFmtFormatter};
use crate::handlers::control::project_handlers;
use core::str::FromStr;
use std::collections::HashMap;

/// List all projects
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn list_projects_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  project_handlers::list_projects_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let response = client
    .get("/api/v1/projects", None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Get project by ID
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn get_project_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  project_handlers::get_project_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let id = params.get("id").unwrap(); // Already validated
  let path = format!("/api/v1/projects/{id}");

  let response = client
    .get(&path, None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}
