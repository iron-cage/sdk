//! Common validation helpers for handlers
//!
//! Reusable validation functions to reduce code duplication
//! and maintain consistent error messages.

use crate::handlers::CliError;

/// Validates that a token ID matches the expected format (tok_*)
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

  Ok(())
}

/// Validates that a string is non-empty
pub fn validate_non_empty(value: &str, param_name: &'static str) -> Result<(), CliError>
{
  if value.is_empty()
  {
    return Err(CliError::InvalidParameter {
      param: param_name,
      reason: "cannot be empty",
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

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_validate_token_id_success()
  {
    assert!(validate_token_id("tok_abc123", "token_id").is_ok());
  }

  #[test]
  fn test_validate_token_id_empty()
  {
    assert!(validate_token_id("", "token_id").is_err());
  }

  #[test]
  fn test_validate_token_id_invalid_prefix()
  {
    assert!(validate_token_id("invalid", "token_id").is_err());
  }

  #[test]
  fn test_validate_non_empty_success()
  {
    assert!(validate_non_empty("value", "param").is_ok());
  }

  #[test]
  fn test_validate_non_empty_fails()
  {
    assert!(validate_non_empty("", "param").is_err());
  }

  #[test]
  fn test_validate_non_negative_integer_success()
  {
    assert_eq!(validate_non_negative_integer("100", "value").unwrap(), 100);
  }

  #[test]
  fn test_validate_non_negative_integer_negative()
  {
    assert!(validate_non_negative_integer("-100", "value").is_err());
  }

  #[test]
  fn test_validate_non_negative_integer_invalid()
  {
    assert!(validate_non_negative_integer("abc", "value").is_err());
  }

  #[test]
  fn test_validate_date_format_success()
  {
    assert!(validate_date_format("2025-01-15", "date").is_ok());
  }

  #[test]
  fn test_validate_date_format_invalid()
  {
    assert!(validate_date_format("invalid", "date").is_err());
    assert!(validate_date_format("2025-13-01", "date").is_err());
    assert!(validate_date_format("2025-00-01", "date").is_err());
  }
}
