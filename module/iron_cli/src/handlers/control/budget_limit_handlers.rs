//! Budget limit command handlers for control API (admin only)
//!
//! Pure functions for budget limit management operations.
//! No I/O - all external operations handled by adapter layer.

use crate::handlers::validation::validate_non_negative_integer;
use crate::handlers::CliError;
use std::collections::HashMap;

/// Handle `.budget_limit.get` command
///
/// Gets current budget limit (admin only).
///
/// ## Parameters
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if validation fails.
pub fn get_budget_limit_handler<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, CliError> {
  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Get budget limit parameters valid\nFormat: {format}"
  ))
}

/// Handle `.budget_limit.set` command
///
/// Sets budget limit (admin only).
///
/// ## Parameters
///
/// Required:
/// - limit: String (non-negative integer)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
///
/// # Errors
///
/// Returns `Err(CliError)` if required parameters are missing or validation fails.
pub fn set_budget_limit_handler<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, CliError> {
  // Validate required parameter
  let limit_str = params
    .get("limit")
    .ok_or(CliError::MissingParameter("limit"))?;

  validate_non_negative_integer(limit_str, "limit")?;

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry") {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1 {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map_or("table", String::as_str);

  Ok(format!(
    "Set budget limit parameters valid\nLimit: {limit_str}\nFormat: {format}"
  ))
}
