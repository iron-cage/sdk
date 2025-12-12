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

  // ============================================================================
  // Bug Reproducer Tests
  // ============================================================================

  /// Bug reproducer: Whitespace-only parameters incorrectly accepted
  ///
  /// ## Root Cause
  ///
  /// validate_non_empty() only checked value.is_empty(), which returns false
  /// for strings containing whitespace characters (spaces, tabs, newlines).
  /// This allowed meaningless input like "   " to pass validation at
  /// src/handlers/validation.rs:36 (original implementation).
  ///
  /// ## Why Not Caught
  ///
  /// 1. **No Boundary Testing**: Original validation tests only checked truly
  ///    empty string ("") and valid values, skipping the whitespace boundary
  /// 2. **Implicit Assumption**: Developer assumed is_empty() would catch
  ///    "meaningless" input, not just zero-length strings
  /// 3. **Missing Corner Case Matrix**: Test suite lacked systematic edge
  ///    case enumeration (whitespace-only, control chars, unicode)
  ///
  /// ## Fix Applied
  ///
  /// Changed validation from value.is_empty() to value.trim().is_empty()
  /// in validate_non_empty() at src/handlers/validation.rs:36. Updated error
  /// message to "cannot be empty or whitespace-only" for clarity.
  ///
  /// ## Prevention
  ///
  /// 1. **Always Test Whitespace Boundaries**: For any is_empty() check,
  ///    test with spaces, tabs, newlines, and mixed whitespace
  /// 2. **Trim Before Validation**: Use trim() consistently for meaningful
  ///    content validation, reserve is_empty() for structural checks only
  /// 3. **Corner Case Checklist**: Systematically test empty, whitespace,
  ///    unicode, control characters, and boundary values
  ///
  /// ## Pitfall
  ///
  /// **is_empty() only checks string length, not semantic emptiness**
  ///
  /// The name "is_empty" suggests checking for "no meaningful content", but
  /// it only checks length == 0. For user input validation, always use
  /// value.trim().is_empty() to reject whitespace-only input. Reserve raw
  /// is_empty() for low-level structural checks where literal empty strings
  /// must be distinguished from whitespace.
  // test_kind: bug_reproducer(corner-case-whitespace)
  #[test]
  fn test_validate_non_empty_rejects_spaces()
  {
    assert!(
      validate_non_empty("   ", "param").is_err(),
      "Whitespace-only string should be rejected"
    );
  }

  /// Bug reproducer: Token ID "tok_" alone (no suffix) incorrectly accepted
  ///
  /// ## Root Cause
  ///
  /// validate_token_id() only verified token_id.starts_with("tok_") without
  /// checking that actual ID content exists after the prefix. This allowed
  /// meaningless token IDs like "tok_" or "tok_   " to pass validation at
  /// src/handlers/validation.rs:22 (original implementation).
  ///
  /// ## Why Not Caught
  ///
  /// 1. **Format-Only Testing**: Validation tests checked "tok_abc123" (valid)
  ///    and "invalid" (missing prefix), but skipped the boundary between them
  /// 2. **Implicit Content Assumption**: Developer assumed starts_with() check
  ///    implied meaningful content, didn't test edge case of prefix-only input
  /// 3. **Poor User Experience**: Even though API would reject it later, CLI
  ///    should validate upfront for better UX and clear error messages
  ///
  /// ## Fix Applied
  ///
  /// Added two checks to validate_token_id():
  /// 1. Minimum length check (token_id.len() <= 4) at line 31
  /// 2. Whitespace suffix check (suffix.trim().is_empty()) at line 41
  ///
  /// Provides clear error messages: "must have content after 'tok_' prefix"
  /// and "suffix cannot be whitespace-only".
  ///
  /// ## Prevention
  ///
  /// 1. **Test Prefix-Only Patterns**: When validating prefixed formats
  ///    (tok_*, usr_*, api_*), always test the prefix alone
  /// 2. **Length Validation**: For structured IDs, enforce minimum meaningful
  ///    length (prefix + minimum suffix length)
  /// 3. **Whitespace After Prefix**: Check that content after required prefix
  ///    is not just whitespace (suffix.trim().is_empty())
  ///
  /// ## Pitfall
  ///
  /// **Format validation should verify both structure AND meaningful content**
  ///
  /// starts_with() checks only prove prefix existence, not that the full
  /// identifier is meaningful. For user-facing IDs with required prefixes:
  /// 1. Check prefix exists (starts_with)
  /// 2. Check minimum length (len() > prefix.len())
  /// 3. Check suffix has content (suffix.trim().is_empty())
  ///
  /// Don't assume structural checks imply semantic validity.
  // test_kind: bug_reproducer(corner-case-token-id)
  #[test]
  fn test_validate_token_id_rejects_prefix_only()
  {
    assert!(
      validate_token_id("tok_", "token_id").is_err(),
      "Token ID 'tok_' without suffix should be rejected"
    );
  }

  /// Test zero (minimum valid non-negative integer)
  // test_kind: corner_case(integer)
  #[test]
  fn test_validate_non_negative_integer_zero()
  {
    assert_eq!(
      validate_non_negative_integer("0", "value").unwrap(),
      0,
      "Zero should be accepted as valid non-negative"
    );
  }

  /// Test i64::MAX boundary
  // test_kind: corner_case(integer)
  #[test]
  fn test_validate_non_negative_integer_i64_max()
  {
    assert_eq!(
      validate_non_negative_integer("9223372036854775807", "value").unwrap(),
      i64::MAX,
      "i64::MAX should be accepted"
    );
  }

  /// Test integer overflow (i64::MAX + 1)
  // test_kind: corner_case(integer)
  #[test]
  fn test_validate_non_negative_integer_overflow()
  {
    assert!(
      validate_non_negative_integer("9223372036854775808", "value").is_err(),
      "Integer overflow should be rejected"
    );
  }

  /// Test floating point number
  // test_kind: corner_case(integer)
  #[test]
  fn test_validate_non_negative_integer_float()
  {
    assert!(
      validate_non_negative_integer("3.14", "value").is_err(),
      "Floating point should be rejected"
    );
  }

  /// Test date with day 00
  // test_kind: corner_case(date)
  #[test]
  fn test_validate_date_format_day_zero()
  {
    assert!(
      validate_date_format("2025-01-00", "date").is_err(),
      "Day 00 should be rejected"
    );
  }

  /// Test date with day 32
  // test_kind: corner_case(date)
  #[test]
  fn test_validate_date_format_day_32()
  {
    assert!(
      validate_date_format("2025-01-32", "date").is_err(),
      "Day 32 should be rejected"
    );
  }
}
