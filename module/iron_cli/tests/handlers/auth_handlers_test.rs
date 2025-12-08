//! Authentication handler tests
//!
//! ## Test Coverage
//!
//! Covers login, refresh, logout handlers.
//! Total: 17 test cases
//!
//! ## Test Strategy
//!
//! Pure function testing: HashMap → Result<String>
//! No mocking - handlers have no I/O to mock.
//!
//! ## Test Matrix
//!
//! See tests/handlers/-test_matrix.md for complete coverage plan.
//!
//! ## Handler Signatures
//!
//! All handlers accept `&HashMap<String, String>` and return `Result<String, CliError>`.
//! Tests verify both success paths and all error conditions.

use std::collections::HashMap;
use iron_cli::handlers::{ auth_handlers::*, CliError };

// ============================================================================
// .auth.login tests (9 tests)
// ============================================================================

#[test]
fn test_login_handler_success()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "alice@example.com".into());
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_ok(), "Should succeed with valid params");

  let output = result.unwrap();
  assert!(
    output.contains("alice@example.com"),
    "Output should contain username: {}",
    output
  );
}

#[test]
fn test_login_handler_missing_username()
{
  let mut params = HashMap::new();
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail without username");

  match result.unwrap_err()
  {
    CliError::MissingParameter(name) =>
    {
      assert_eq!(name, "username", "Error should mention 'username'");
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_missing_password()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "alice@example.com".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail without password");

  match result.unwrap_err()
  {
    CliError::MissingParameter(name) =>
    {
      assert_eq!(name, "password", "Error should mention 'password'");
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_empty_username()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "".into());
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail with empty username");

  match result.unwrap_err()
  {
    CliError::InvalidParameter { param, reason } =>
    {
      assert_eq!(param, "username");
      assert!(
        reason.contains("empty"),
        "Reason should mention 'empty': {}",
        reason
      );
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_username_too_short()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "ab".into()); // Only 2 chars
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail with username < 3 chars");

  match result.unwrap_err()
  {
    CliError::InvalidParameter { param, reason } =>
    {
      assert_eq!(param, "username");
      assert!(
        reason.contains("3") || reason.contains("least"),
        "Reason should mention minimum length: {}",
        reason
      );
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_username_too_long()
{
  let mut params = HashMap::new();
  let long_username = "a".repeat(101); // 101 chars (max is 100)
  params.insert("username".into(), long_username);
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail with username > 100 chars");

  match result.unwrap_err()
  {
    CliError::InvalidParameter { param, reason } =>
    {
      assert_eq!(param, "username");
      assert!(
        reason.contains("100") || reason.contains("most"),
        "Reason should mention maximum length: {}",
        reason
      );
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_invalid_username_pattern()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "alice!@#$%^&*()".into()); // Invalid chars
  params.insert("password".into(), "secret123".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail with invalid username pattern");

  match result.unwrap_err()
  {
    CliError::InvalidParameter { param, reason } =>
    {
      assert_eq!(param, "username");
      assert!(
        reason.contains("pattern") || reason.contains("format"),
        "Reason should mention pattern: {}",
        reason
      );
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_empty_password()
{
  let mut params = HashMap::new();
  params.insert("username".into(), "alice@example.com".into());
  params.insert("password".into(), "".into());

  let result = login_handler(&params);

  assert!(result.is_err(), "Should fail with empty password");

  match result.unwrap_err()
  {
    CliError::InvalidParameter { param, reason } =>
    {
      assert_eq!(param, "password");
      assert!(
        reason.contains("empty"),
        "Reason should mention 'empty': {}",
        reason
      );
    }
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_login_handler_all_formats()
{
  // Test that handler respects format parameter
  let formats = vec!["table", "expanded", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("username".into(), "alice@example.com".into());
    params.insert("password".into(), "secret123".into());
    params.insert("format".into(), format.into());

    let result = login_handler(&params);

    assert!(
      result.is_ok(),
      "Should succeed with format '{}': {:?}",
      format,
      result
    );

    // Note: Actual format validation will happen when formatter is implemented (Phase 3)
    // For now, we just verify the handler doesn't reject the format parameter
  }
}

// ============================================================================
// .auth.refresh tests (5 tests)
// ============================================================================

#[test]
fn test_refresh_handler_success_default_format()
{
  let params = HashMap::new(); // No params required for refresh

  let result = refresh_handler(&params);

  assert!(result.is_ok(), "Should succeed with no params");
}

#[test]
fn test_refresh_handler_format_table()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "table".into());

  let result = refresh_handler(&params);

  assert!(result.is_ok(), "Should succeed with table format");
}

#[test]
fn test_refresh_handler_format_expanded()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "expanded".into());

  let result = refresh_handler(&params);

  assert!(result.is_ok(), "Should succeed with expanded format");
}

#[test]
fn test_refresh_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "json".into());

  let result = refresh_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}

#[test]
fn test_refresh_handler_format_yaml()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "yaml".into());

  let result = refresh_handler(&params);

  assert!(result.is_ok(), "Should succeed with yaml format");
}

// ============================================================================
// .auth.logout tests (3 tests)
// ============================================================================

#[test]
fn test_logout_handler_success()
{
  let params = HashMap::new();

  let result = logout_handler(&params);

  assert!(result.is_ok(), "Should succeed with no params");
}

#[test]
fn test_logout_handler_all_formats()
{
  let formats = vec!["table", "expanded", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("format".into(), format.into());

    let result = logout_handler(&params);

    assert!(
      result.is_ok(),
      "Should succeed with format '{}': {:?}",
      format,
      result
    );
  }
}

#[test]
fn test_logout_handler_confirmation_message()
{
  let params = HashMap::new();

  let result = logout_handler(&params);

  assert!(result.is_ok(), "Should succeed");

  let output = result.unwrap();
  assert!(
    output.contains("logout") || output.contains("Logout"),
    "Output should mention logout: {}",
    output
  );
}
