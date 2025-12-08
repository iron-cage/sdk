//! Usage command handlers
//!
//! Pure functions for usage show, by-project, by-provider, export operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use super::validation::{ validate_date_format, validate_non_empty };

/// Handle .usage.show command
///
/// Shows usage statistics with optional date range filtering.
///
/// ## Parameters
///
/// Optional:
/// - start_date: String (format: YYYY-MM-DD)
/// - end_date: String (format: YYYY-MM-DD, must be after start_date)
/// - format: String (table|expanded|json|yaml, default: table)
pub fn show_usage_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate date range if provided
  if let (Some(start), Some(end)) = (params.get("start_date"), params.get("end_date"))
  {
    // Validate date format
    validate_date_format(start, "start_date")?;
    validate_date_format(end, "end_date")?;

    // Check date ordering
    if start > end
    {
      return Err(CliError::InvalidParameter {
        param: "end_date",
        reason: "must be after start_date",
      });
    }
  }
  else if let Some(start) = params.get("start_date")
  {
    validate_date_format(start, "start_date")?;
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let date_range = match (params.get("start_date"), params.get("end_date"))
  {
    (Some(start), Some(end)) => format!("{} to {}", start, end),
    (Some(start), None) => format!("from {}", start),
    _ => "all time".to_string(),
  };

  Ok(format!(
    "Show usage\nDate range: {}\nFormat: {}",
    date_range, format
  ))
}

/// Handle .usage.by-project command
///
/// Shows usage statistics for a specific project.
///
/// ## Parameters
///
/// Required:
/// - project_id: String (non-empty)
///
/// Optional:
/// - start_date: String (format: YYYY-MM-DD)
/// - end_date: String (format: YYYY-MM-DD)
/// - format: String (table|json|yaml, default: table)
pub fn usage_by_project_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let project_id = params
    .get("project_id")
    .ok_or(CliError::MissingParameter("project_id"))?;

  // Validate project_id
  validate_non_empty(project_id, "project_id")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let date_range = match params.get("start_date")
  {
    Some(start) => format!("from {}", start),
    None => "all time".to_string(),
  };

  Ok(format!(
    "Usage by project\nProject ID: {}\nDate range: {}\nFormat: {}",
    project_id, date_range, format
  ))
}

/// Handle .usage.by-provider command
///
/// Shows usage statistics for a specific provider.
///
/// ## Parameters
///
/// Required:
/// - provider: String (must be valid provider: openai, anthropic, etc.)
///
/// Optional:
/// - aggregation: String (daily, weekly, monthly)
/// - format: String (table|json|yaml, default: table)
pub fn usage_by_provider_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let provider = params
    .get("provider")
    .ok_or(CliError::MissingParameter("provider"))?;

  // Validate provider
  let valid_providers = ["openai", "anthropic", "cohere", "together"];
  if !valid_providers.contains(&provider.as_str())
  {
    return Err(CliError::InvalidParameter {
      param: "provider",
      reason: "must be one of: openai, anthropic, cohere, together",
    });
  }

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");
  let aggregation = params.get("aggregation").map(|s| s.as_str()).unwrap_or("default");

  Ok(format!(
    "Usage by provider\nProvider: {}\nAggregation: {}\nFormat: {}",
    provider, aggregation, format
  ))
}

/// Handle .usage.export command
///
/// Exports usage data to a file.
///
/// ## Parameters
///
/// Required:
/// - output: String (file path, non-empty)
///
/// Optional:
/// - format: String (json|csv, default: json)
pub fn export_usage_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required parameters
  let output = params
    .get("output")
    .ok_or(CliError::MissingParameter("output"))?;

  // Validate output path
  validate_non_empty(output, "output")?;

  // Format output
  let format = params.get("format").map(|s| s.as_str()).unwrap_or("json");

  Ok(format!(
    "Export usage\nOutput: {}\nFormat: {}",
    output, format
  ))
}
