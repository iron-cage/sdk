//! Common validation helpers for handlers
//!
//! Reusable validation functions to reduce code duplication
//! and maintain consistent error messages.

use crate::handlers::CliError;

/// Validates that a token ID matches the expected format (tok_*)
// Fix(corner-case-token-id): Added check for content after "tok_" prefix
// Root cause: Only validated prefix existence, not that there's actual ID content after it
// Pitfall: Format validation should verify both structure AND meaningful content
pub fn validate_token_id(token_id: &str, param_name: &'static str) -> Result<(), CliError>
{
  if token_id.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "cannot be empty",
    });
  }

  if !token_id.starts_with("tok_")
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "must start with 'tok_'",
    });
  }

  // Verify there's content after the "tok_" prefix (minimum length 5: "tok_X")
  if token_id.len() <= 4
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "must have content after 'tok_' prefix",
    });
  }

  // Check that suffix is not just whitespace
  let suffix = &token_id[4..];
  if suffix.trim().is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "suffix cannot be whitespace-only",
    });
  }

  Ok(())
}

/// Validates that a string is non-empty and not whitespace-only
// Fix(corner-case-whitespace): Added trim() check to reject whitespace-only strings
// Root cause: is_empty() returns false for strings containing only whitespace
// Pitfall: Always validate that strings contain meaningful content, not just non-zero length
pub fn validate_non_empty(value: &str, param_name: &'static str) -> Result<(), CliError>
{
  if value.trim().is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "cannot be empty or whitespace-only",
    });
  }

  Ok(())
}

/// Validates that a string can be parsed as a non-negative integer
pub fn validate_non_negative_integer(
  value_str: &str,
  param_name: &'static str,
) -> Result<i64, CliError>
{
  match value_str.parse::<i64>()
  {
    Ok(value) =>
    {
      if value < 0
      {
        return Err(CliError::InvalidParameter {
          param: param_name,
          reason: "must be non-negative",
        });
      }
      Ok(value)
    }
    Err(_) =>
    {
      Err(CliError::InvalidParameter {
        param: param_name,
        reason: "must be a valid integer",
      })
    }
  }
}

/// Validates that a date string matches YYYY-MM-DD format
pub fn validate_date_format(date: &str, param_name: &'static str) -> Result<(), CliError>
{
  if !is_valid_date_format(date)
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "must match format YYYY-MM-DD",
    });
  }

  Ok(())
}

/// Helper to check if date format is valid YYYY-MM-DD
fn is_valid_date_format(date: &str) -> bool
{
  if date.len() != 10
  {
    return false;
  }

  let parts: Vec<&str> = date.split('-').collect();
  if parts.len() != 3
  {
    return false;
  }

  // Year: 4 digits
  if parts[0].len() != 4 || parts[0].parse::<u32>().is_err()
  {
    return false;
  }

  // Month: 01-12
  if parts[1].len() != 2
  {
    return false;
  }
  if let Ok(month) = parts[1].parse::<u32>()
  {
    if !(1..=12).contains(&month)
    {
      return false;
    }
  }
  else
  {
    return false;
  }

  // Day: 01-31
  if parts[2].len() != 2
  {
    return false;
  }
  if let Ok(day) = parts[2].parse::<u32>()
  {
    if !(1..=31).contains(&day)
    {
      return false;
    }
  }
  else
  {
    return false;
  }

  true
}

