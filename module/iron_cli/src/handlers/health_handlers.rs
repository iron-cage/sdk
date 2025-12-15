//! Health and version command handlers
//!
//! Pure functions for health and version check operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;

/// Handle .health command
///
/// Returns health status of the service.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn health_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Health status: OK\nFormat: {}",
    format
  ))
}

/// Handle .version command
///
/// Returns version information.
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn version_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let version = env!("CARGO_PKG_VERSION");

  Ok(format!(
    "iron-cli version: {}\nFormat: {}",
    version, format
  ))
}
