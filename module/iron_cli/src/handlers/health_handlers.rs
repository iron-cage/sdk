//! Health and version command handlers
//!
//! Pure functions for health and version check operations.
//! No I/O - all external operations handled by adapter layer.

use crate::handlers::CliError;
use std::collections::HashMap;

/// Handle .health command
///
/// Returns health status of the service.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn health_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!("Health status: OK\nFormat: {format}"))
}

/// Handle .version command
///
/// Returns version information.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn version_handler(params: &HashMap<String, String>) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);
  let version = env!("CARGO_PKG_VERSION");

  Ok(format!("iron-cli version: {version}\nFormat: {format}"))
}
