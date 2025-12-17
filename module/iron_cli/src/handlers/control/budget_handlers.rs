//! Budget command handlers for control API
//!
//! Pure functions for budget analytics operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::validate_non_negative_integer;

/// Handle .budget.status command
///
/// Gets budget status across agents (analytics).
///
/// ## Parameters
///
/// Optional:
/// - agent_id: String (positive integer)
/// - threshold: String (0-100)
/// - status: String (active|exhausted)
/// - page: String (positive integer)
/// - per_page: String (positive integer)
/// - format: String (table|json|yaml, default: table)
pub fn budget_status_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional agent_id
  if let Some(agent_id_str) = params.get("agent_id")
  {
    let agent_id = validate_non_negative_integer(agent_id_str, "agent_id")?;
    if agent_id < 1
    {
      return Err(CliError::InvalidParameter {
        param: "agent_id",
        reason: "must be a positive integer",
      });
    }
  }

  // Validate optional threshold
  if let Some(threshold_str) = params.get("threshold")
  {
    let threshold = validate_non_negative_integer(threshold_str, "threshold")?;
    if threshold > 100
    {
      return Err(CliError::InvalidParameter {
        param: "threshold",
        reason: "must be between 0 and 100",
      });
    }
  }

  // Validate optional status
  if let Some(status) = params.get("status")
  {
    match status.as_str()
    {
      "active" | "exhausted" => {},
      _ => return Err(CliError::InvalidParameter {
        param: "status",
        reason: "must be 'active' or 'exhausted'",
      }),
    }
  }

  // Validate optional pagination
  if let Some(page_str) = params.get("page")
  {
    validate_non_negative_integer(page_str, "page")?;
  }

  if let Some(per_page_str) = params.get("per_page")
  {
    validate_non_negative_integer(per_page_str, "per_page")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Budget status parameters valid\nFormat: {}",
    format
  ))
}
