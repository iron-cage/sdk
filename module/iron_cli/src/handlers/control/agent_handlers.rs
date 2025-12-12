//! Agent command handlers for control API
//!
//! Pure functions for agent management operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .agent.list command
///
/// Lists all agents with optional filtering and pagination.
///
/// ## Parameters
///
/// Optional:
/// - page: String (positive integer)
/// - limit: String (positive integer)
/// - v: String (verbosity level 0-5)
/// - format: String (table|json|yaml, default: table)
pub fn list_agents_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional pagination
  if let Some(page_str) = params.get("page")
  {
    validate_non_negative_integer(page_str, "page")?;
  }

  if let Some(limit_str) = params.get("limit")
  {
    validate_non_negative_integer(limit_str, "limit")?;
  }

  // Validate verbosity
  if let Some(v_str) = params.get("v")
  {
    let v = validate_non_negative_integer(v_str, "v")?;
    if v > 5
    {
      return Err(CliError::InvalidParameter {
        param: "v",
        reason: "verbosity must be 0-5",
      });
    }
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Agent list parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .agent.create command
///
/// Creates new agent with name and budget.
///
/// ## Parameters
///
/// Required:
/// - name: String (non-empty, max 100 chars, pattern: ^[a-zA-Z0-9_-]+$)
/// - budget: String (non-negative integer)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn create_agent_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let name = params
    .get("name")
    .ok_or(CliError::MissingParameter("name"))?;

  let budget_str = params
    .get("budget")
    .ok_or(CliError::MissingParameter("budget"))?;

  // Validate name
  validate_non_empty(name, "name")?;

  if name.len() > 100
  {
    return Err(CliError::InvalidParameter {
      param: "name",
      reason: "cannot exceed 100 characters",
    });
  }

  // Validate name pattern
  let name_pattern = regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
    .map_err(|_| CliError::ValidationError("regex compilation failed".into()))?;

  if !name_pattern.is_match(name)
  {
    return Err(CliError::InvalidParameter {
      param: "name",
      reason: "must match pattern ^[a-zA-Z0-9_-]+$",
    });
  }

  // Validate budget
  validate_non_negative_integer(budget_str, "budget")?;

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
    "Agent creation parameters valid\nName: {}\nBudget: {}\nFormat: {}",
    name, budget_str, format
  ))
}

/// Handle .agent.get command
///
/// Gets agent details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_agent_handler(
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
    "Agent get parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .agent.update command
///
/// Updates agent name and/or budget.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - name: String (non-empty, max 100 chars, pattern: ^[a-zA-Z0-9_-]+$)
/// - budget: String (non-negative integer)
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn update_agent_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameter
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  validate_non_empty(id, "id")?;

  // Validate optional name
  if let Some(name) = params.get("name")
  {
    validate_non_empty(name, "name")?;

    if name.len() > 100
    {
      return Err(CliError::InvalidParameter {
        param: "name",
        reason: "cannot exceed 100 characters",
      });
    }

    let name_pattern = regex::Regex::new(r"^[a-zA-Z0-9_-]+$")
      .map_err(|_| CliError::ValidationError("regex compilation failed".into()))?;

    if !name_pattern.is_match(name)
    {
      return Err(CliError::InvalidParameter {
        param: "name",
        reason: "must match pattern ^[a-zA-Z0-9_-]+$",
      });
    }
  }

  // Validate optional budget
  if let Some(budget_str) = params.get("budget")
  {
    validate_non_negative_integer(budget_str, "budget")?;
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
    "Agent update parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .agent.delete command
///
/// Deletes agent by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn delete_agent_handler(
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
    "Agent delete parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .agent.assign_providers command
///
/// Assigns providers to agent.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty agent ID)
/// - provider_ids: String (comma-separated list of provider IDs)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn assign_providers_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  let provider_ids = params
    .get("provider_ids")
    .ok_or(CliError::MissingParameter("provider_ids"))?;

  validate_non_empty(id, "id")?;
  validate_non_empty(provider_ids, "provider_ids")?;

  // Validate provider_ids format (should be comma-separated)
  if provider_ids.split(',').any(|id| id.trim().is_empty())
  {
    return Err(CliError::InvalidParameter {
      param: "provider_ids",
      reason: "cannot contain empty provider IDs",
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
    "Assign providers parameters valid\nAgent ID: {}\nProvider IDs: {}\nFormat: {}",
    id, provider_ids, format
  ))
}

/// Handle .agent.list_providers command
///
/// Lists providers assigned to agent.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty agent ID)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_agent_providers_handler(
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
    "List agent providers parameters valid\nAgent ID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .agent.remove_provider command
///
/// Removes provider from agent.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty agent ID)
/// - provider_id: String (non-empty provider ID)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn remove_provider_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  let provider_id = params
    .get("provider_id")
    .ok_or(CliError::MissingParameter("provider_id"))?;

  validate_non_empty(id, "id")?;
  validate_non_empty(provider_id, "provider_id")?;

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
    "Remove provider parameters valid\nAgent ID: {}\nProvider ID: {}\nFormat: {}",
    id, provider_id, format
  ))
}
