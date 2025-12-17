//! Provider command handlers for control API
//!
//! Pure functions for provider management operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::{ validate_non_empty, validate_non_negative_integer };

/// Handle .provider.list command
///
/// Lists all providers with optional filtering and pagination.
///
/// ## Parameters
///
/// Optional:
/// - page: String (positive integer)
/// - limit: String (positive integer)
/// - v: String (verbosity level 0-5)
/// - format: String (table|json|yaml, default: table)
pub fn list_providers_handler(
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
    "Provider list parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .provider.create command
///
/// Creates new provider with type and API key.
///
/// ## Parameters
///
/// Required:
/// - provider: String ("openai" or "anthropic")
/// - api_key: String (non-empty)
///
/// Optional:
/// - base_url: String (custom API endpoint)
/// - description: String (provider key description)
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn create_provider_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let provider = params
    .get("provider")
    .ok_or(CliError::MissingParameter("provider"))?;

  let api_key = params
    .get("api_key")
    .ok_or(CliError::MissingParameter("api_key"))?;

  // Validate provider type
  validate_non_empty(provider, "provider")?;

  if provider != "openai" && provider != "anthropic"
  {
    return Err(CliError::InvalidParameter {
      param: "provider",
      reason: "must be 'openai' or 'anthropic'",
    });
  }

  // Validate API key
  validate_non_empty(api_key, "api_key")?;

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
    "Provider creation parameters valid\nProvider: {}\nFormat: {}",
    provider, format
  ))
}

/// Handle .provider.get command
///
/// Gets provider details by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn get_provider_handler(
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
    "Provider get parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .provider.update command
///
/// Updates provider configuration.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - name: String (non-empty, max 100 chars)
/// - api_key: String (non-empty)
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn update_provider_handler(
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
  }

  // Validate optional api_key
  if let Some(api_key) = params.get("api_key")
  {
    validate_non_empty(api_key, "api_key")?;
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
    "Provider update parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .provider.delete command
///
/// Deletes provider by ID.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn delete_provider_handler(
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
    "Provider delete parameters valid\nID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .provider.assign_agents command
///
/// Assigns agents to provider.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty provider ID)
/// - agent_ids: String (comma-separated list of agent IDs)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn assign_agents_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  let agent_ids = params
    .get("agent_ids")
    .ok_or(CliError::MissingParameter("agent_ids"))?;

  validate_non_empty(id, "id")?;
  validate_non_empty(agent_ids, "agent_ids")?;

  // Validate agent_ids format (should be comma-separated)
  if agent_ids.split(',').any(|id| id.trim().is_empty())
  {
    return Err(CliError::InvalidParameter {
      param: "agent_ids",
      reason: "cannot contain empty agent IDs",
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
    "Assign agents parameters valid\nProvider ID: {}\nAgent IDs: {}\nFormat: {}",
    id, agent_ids, format
  ))
}

/// Handle .provider.list_agents command
///
/// Lists agents assigned to provider.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty provider ID)
///
/// Optional:
/// - format: String (table|json|yaml, default: table)
pub fn list_provider_agents_handler(
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
    "List provider agents parameters valid\nProvider ID: {}\nFormat: {}",
    id, format
  ))
}

/// Handle .provider.remove_agent command
///
/// Removes agent from provider.
///
/// ## Parameters
///
/// Required:
/// - id: String (non-empty provider ID)
/// - agent_id: String (non-empty agent ID)
///
/// Optional:
/// - dry: String (0 or 1, default: 0)
/// - format: String (table|json|yaml, default: table)
pub fn remove_agent_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let id = params
    .get("id")
    .ok_or(CliError::MissingParameter("id"))?;

  let agent_id = params
    .get("agent_id")
    .ok_or(CliError::MissingParameter("agent_id"))?;

  validate_non_empty(id, "id")?;
  validate_non_empty(agent_id, "agent_id")?;

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
    "Remove agent parameters valid\nProvider ID: {}\nAgent ID: {}\nFormat: {}",
    id, agent_id, format
  ))
}
