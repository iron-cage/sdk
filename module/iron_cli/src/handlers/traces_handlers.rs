//! Traces command handlers
//!
//! Pure functions for traces list, get, export operations.
//! No I/O - all external operations handled by adapter layer.

use crate::handlers::CliError;
use std::collections::HashMap;

/// Handle .traces.list command
///
/// Lists traces with optional filtering and pagination.
///
/// ## Parameters
///
/// Optional:
/// - filter: String (filter criteria)
/// - limit: String (pagination limit)
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn list_traces_handler<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);
  let filter = params.get("filter").map_or("none", String::as_str);
  let limit = params.get("limit").map_or("default", String::as_str);

  Ok(format!(
    "List traces\nFilter: {filter}\nLimit: {limit}\nFormat: {format}"
  ))
}

/// Handle .traces.get command
///
/// Gets details for a specific trace.
///
/// ## Parameters
///
/// Required:
/// - `trace_id`: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn get_trace_handler<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, CliError> {
  // Validate required parameters
  let trace_id = params
    .get("trace_id")
    .ok_or(CliError::MissingParameter("trace_id"))?;

  // Validate trace_id
  if trace_id.is_empty() {
    return Err(CliError::InvalidParameter {
      param: "trace_id",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!("Get trace\nTrace ID: {trace_id}\nFormat: {format}"))
}

/// Handle .traces.export command
///
/// Exports traces to a file.
///
/// ## Parameters
///
/// Required:
/// - output: String (file path, non-empty)
///
/// Optional:
/// - format: String (json, default: json)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn export_traces_handler<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, CliError> {
  // Validate required parameters
  let output = params
    .get("output")
    .ok_or(CliError::MissingParameter("output"))?;

  // Validate output path
  if output.is_empty() {
    return Err(CliError::InvalidParameter {
      param: "output",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map_or("json", String::as_str);

  Ok(format!("Export traces\nOutput: {output}\nFormat: {format}"))
}
