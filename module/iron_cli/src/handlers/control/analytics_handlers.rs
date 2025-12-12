//! Analytics command handlers for control API
//!
//! Pure functions for analytics and reporting operations.
//! No I/O - all external operations handled by adapter layer.

use std::collections::HashMap;
use crate::handlers::CliError;
use crate::handlers::validation::validate_non_negative_integer;

/// Validates date format (YYYY-MM-DD)
fn validate_date(date_str: &str, param_name: &'static str) -> Result<(), CliError>
{
  let parts: Vec<&str> = date_str.split('-').collect();

  if parts.len() != 3
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "must be in YYYY-MM-DD format",
    });
  }

  // Validate year
  if parts[0].len() != 4 || parts[0].parse::<u32>().is_err()
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "year must be 4 digits",
    });
  }

  // Validate month
  match parts[1].parse::<u32>()
  {
    Ok(month) if (1..=12).contains(&month) => {},
    _ => return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "month must be 01-12",
    }),
  }

  // Validate day
  match parts[2].parse::<u32>()
  {
    Ok(day) if (1..=31).contains(&day) => {},
    _ => return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "day must be 01-31",
    }),
  }

  Ok(())
}

/// Handle .analytics.usage command
pub fn usage_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Usage statistics parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.spending command
pub fn spending_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Spending statistics parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.metrics command
pub fn metrics_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Performance metrics parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.usage_by_agent command
pub fn usage_by_agent_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  // Validate optional limit
  if let Some(limit_str) = params.get("limit")
  {
    validate_non_negative_integer(limit_str, "limit")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Usage by agent parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.usage_by_provider command
pub fn usage_by_provider_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  // Validate optional limit
  if let Some(limit_str) = params.get("limit")
  {
    validate_non_negative_integer(limit_str, "limit")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Usage by provider parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.spending_by_period command
pub fn spending_by_period_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate optional period
  if let Some(period) = params.get("period")
  {
    match period.as_str()
    {
      "day" | "week" | "month" | "year" => {},
      _ => return Err(CliError::InvalidParameter {
        param: "period",
        reason: "must be one of: day, week, month, year",
      }),
    }
  }

  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Spending by period parameters valid\nFormat: {}",
    format
  ))
}

/// Handle .analytics.export_usage command
pub fn export_usage_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required output_format
  let output_format = params
    .get("output_format")
    .ok_or(CliError::MissingParameter("output_format"))?;

  match output_format.as_str()
  {
    "csv" | "json" | "xlsx" => {},
    _ => return Err(CliError::InvalidParameter {
      param: "output_format",
      reason: "must be one of: csv, json, xlsx",
    }),
  }

  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Export usage parameters valid\nOutput format: {}\nFormat: {}",
    output_format, format
  ))
}

/// Handle .analytics.export_spending command
pub fn export_spending_handler(
  params: &HashMap<String, String>,
) -> Result<String, CliError>
{
  // Validate required output_format
  let output_format = params
    .get("output_format")
    .ok_or(CliError::MissingParameter("output_format"))?;

  match output_format.as_str()
  {
    "csv" | "json" | "xlsx" => {},
    _ => return Err(CliError::InvalidParameter {
      param: "output_format",
      reason: "must be one of: csv, json, xlsx",
    }),
  }

  // Validate optional date range
  if let Some(start_date) = params.get("start_date")
  {
    validate_date(start_date, "start_date")?;
  }

  if let Some(end_date) = params.get("end_date")
  {
    validate_date(end_date, "end_date")?;
  }

  let format = params.get("format").map(|s| s.as_str()).unwrap_or("table");

  Ok(format!(
    "Export spending parameters valid\nOutput format: {}\nFormat: {}",
    output_format, format
  ))
}
