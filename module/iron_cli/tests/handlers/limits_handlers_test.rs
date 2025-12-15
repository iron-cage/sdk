//! Limits handler tests
//!
//! ## Test Coverage
//!
//! Covers list, get, create, update, delete limits handlers.
//! Total: 15 test cases

use std::collections::HashMap;
use iron_cli::handlers::{ limits_handlers::*, CliError };

// ============================================================================
// .limits.list tests (3 tests)
// ============================================================================

#[test]
fn test_list_limits_handler_success()
{
  let params = HashMap::new();

  let result = list_limits_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_list_limits_handler_empty()
{
  let params = HashMap::new();

  let result = list_limits_handler(&params);

  assert!(result.is_ok(), "Should succeed with empty list");
}

#[test]
fn test_list_limits_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("format".into(), format.into());

    let result = list_limits_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .limits.get tests (3 tests)
// ============================================================================

#[test]
fn test_get_limit_handler_success()
{
  let mut params = HashMap::new();
  params.insert("limit_id".into(), "lim_abc123".into());

  let result = get_limit_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_get_limit_handler_missing_limit_id()
{
  let params = HashMap::new();

  let result = get_limit_handler(&params);

  assert!(result.is_err(), "Should fail without limit_id");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "limit_id"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_get_limit_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("limit_id".into(), "lim_abc123".into());
    params.insert("format".into(), format.into());

    let result = get_limit_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .limits.create tests (3 tests)
// ============================================================================

#[test]
fn test_create_limit_handler_success()
{
  let mut params = HashMap::new();
  params.insert("resource_type".into(), "requests".into());
  params.insert("limit_value".into(), "1000".into());

  let result = create_limit_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_create_limit_handler_missing_required_fields()
{
  let mut params = HashMap::new();
  params.insert("resource_type".into(), "requests".into());

  let result = create_limit_handler(&params);

  assert!(result.is_err(), "Should fail without limit_value");
}

#[test]
fn test_create_limit_handler_invalid_limit_value()
{
  let mut params = HashMap::new();
  params.insert("resource_type".into(), "requests".into());
  params.insert("limit_value".into(), "-100".into());

  let result = create_limit_handler(&params);

  assert!(result.is_err(), "Should fail with negative limit");
}

// ============================================================================
// .limits.update tests (3 tests)
// ============================================================================

#[test]
fn test_update_limit_handler_success()
{
  let mut params = HashMap::new();
  params.insert("limit_id".into(), "lim_abc123".into());
  params.insert("limit_value".into(), "2000".into());

  let result = update_limit_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_update_limit_handler_missing_limit_id()
{
  let mut params = HashMap::new();
  params.insert("limit_value".into(), "2000".into());

  let result = update_limit_handler(&params);

  assert!(result.is_err(), "Should fail without limit_id");
}

#[test]
fn test_update_limit_handler_invalid_update_fields()
{
  let mut params = HashMap::new();
  params.insert("limit_id".into(), "lim_abc123".into());
  params.insert("limit_value".into(), "-100".into());

  let result = update_limit_handler(&params);

  assert!(result.is_err(), "Should fail with invalid value");
}

// ============================================================================
// .limits.delete tests (3 tests)
// ============================================================================

#[test]
fn test_delete_limit_handler_success()
{
  let mut params = HashMap::new();
  params.insert("limit_id".into(), "lim_abc123".into());

  let result = delete_limit_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_delete_limit_handler_missing_limit_id()
{
  let params = HashMap::new();

  let result = delete_limit_handler(&params);

  assert!(result.is_err(), "Should fail without limit_id");
}

#[test]
fn test_delete_limit_handler_confirmation()
{
  let mut params = HashMap::new();
  params.insert("limit_id".into(), "lim_abc123".into());

  let result = delete_limit_handler(&params);

  assert!(result.is_ok(), "Should succeed");
  let output = result.unwrap();
  assert!(output.contains("delet"), "Should mention deletion");
}
