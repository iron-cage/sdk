//! Traces command handlers
//!
//! Pure functions for traces list, get, export operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;

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
pub fn list_traces_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let filter = params.get("filter").map(|s| s.as_str()).unwrap_or("none");
  let limit = params.get("limit").map(|s| s.as_str()).unwrap_or("default");

  Ok(format!(
    "List traces\nFilter: {}\nLimit: {}\nFormat: {}",
    filter, limit, format
  ))
}

/// Handle .traces.get command
///
/// Gets details for a specific trace.
///
/// ## Parameters
///
/// Required:
/// - trace_id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_trace_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let trace_id = params
    .get("trace_id")
    .ok_or(CliError::MissingParameter("trace_id"))?;

  // Validate trace_id
  if trace_id.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "trace_id",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Get trace\nTrace ID: {}\nFormat: {}",
    trace_id, format
  ))
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
pub fn export_traces_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let output = params
    .get("output")
    .ok_or(CliError::MissingParameter("output"))?;

  // Validate output path
  if output.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: "output",
      reason: "cannot be empty",
    });
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("json");

  Ok(format!(
    "Export traces\nOutput: {}\nFormat: {}",
    output, format
  ))
}
