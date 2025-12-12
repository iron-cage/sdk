//! Budget request command handlers for control API
//!
//! Pure functions for budget change request workflow operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .budget_request.list command
///
/// Lists budget change requests with optional filtering.
///
/// ## Parameters
///
/// Optional:
/// - status: String (pending|approved|rejected)
/// - format: String (table|json|yaml, default: table)
pub fn list_budget_requests_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional status
  if let Some(status) = params.get("status")
  {
    match status.as_str()
    {
      "pending" | "approved" | "rejected" => {},
      _ => return Err(CliError::InvalidParameter {
        param: "status",
        reason: "must be one of: pending, approved, rejected",
      }),
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Budget request list parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .budget_request.create command
///
/// Creates budget increase request.
///
/// ## Parameters
///
/// Required:
/// - agent_id: String (non-empty)
/// - amount: String (positive integer)
/// - reason: String (10-500 chars)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn create_budget_request_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let agent_id = params
    .get("agent_id")
    .ok_or(CliError::MissingParameter("agent_id"))?;

  let amount_str = params
    .get("amount")
    .ok_or(CliError::MissingParameter("amount"))?;

  let reason = params
    .get("reason")
    .ok_or(CliError::MissingParameter("reason"))?;

  validate_non_empty(agent_id, "agent_id")?;

  // Validate amount (must be positive)
  let amount = validate_non_negative_integer(amount_str, "amount")?;
  if amount < 1
  {
    return Err(CliError::InvalidParameter {
      param: "amount",
      reason: "must be at least 1",
    });
  }

  // Validate reason length
  validate_non_empty(reason, "reason")?;
  if reason.len() < 10
  {
    return Err(CliError::InvalidParameter {
      param: "reason",
      reason: "must be at least 10 characters",
    });
  }
  if reason.len() > 500
  {
    return Err(CliError::InvalidParameter {
      param: "reason",
      reason: "cannot exceed 500 characters",
    });
  }

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Create budget request parameters valid\nAgent ID: {}\nAmount: {}\nFormat: {}",
    agent_id, amount_str, format
  ))
}

/// Handle .budget_request.get command
///
/// Gets budget request details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_budget_request_handler(
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
    "Get budget request parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .budget_request.approve command
///
/// Approves budget request (admin only).
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn approve_budget_request_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Approve budget request parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .budget_request.reject command
///
/// Rejects budget request (admin only).
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - reason: String (max 500 chars)
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn reject_budget_request_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional reason
  if let Some(reason) = params.get("reason")
  {
    if reason.len() > 500
    {
      return Err(CliError::InvalidParameter {
        param: "reason",
        reason: "cannot exceed 500 characters",
      });
    }
  }

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Reject budget request parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .budget_request.cancel command
///
/// Cancels pending budget request.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn cancel_budget_request_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional dry run
  if let Some(dry_str) = params.get("dry")
  {
    let dry = validate_non_negative_integer(dry_str, "dry")?;
    if dry > 1
    {
      return Err(CliError::InvalidParameter {
        param: "dry",
        reason: "must be 0 or 1",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Cancel budget request parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}
