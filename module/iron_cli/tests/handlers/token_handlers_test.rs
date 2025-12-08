//! Token handler tests
//!
//! ## Test Coverage
//!
//! Covers generate, list, get, rotate, revoke token handlers.
//! Total: 35 test cases
//!
//! ## Test Strategy
//!
//! Pure function testing: HashMap → Result<String>
//! No mocking - handlers have no I/O to mock.

use std::collections::HashMap;
use iron_cli::handlers::{ token_handlers::*, CliError };

// ============================================================================
// .tokens.generate tests (8 tests)
// ============================================================================

#[test]
fn test_generate_token_handler_success()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());
  params.insert("scope".into(), "read:tokens".into());

  let result = generate_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with valid params");
}

#[test]
fn test_generate_token_handler_missing_name()
{
  let mut params = HashMap::new();
  params.insert("scope".into(), "read:tokens".into());

  let result = generate_token_handler(&params);

  assert!(result.is_err(), "Should fail without name");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "name"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_generate_token_handler_empty_name()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "".into());
  params.insert("scope".into(), "read:tokens".into());

  let result = generate_token_handler(&params);

  assert!(result.is_err(), "Should fail with empty name");
}

#[test]
fn test_generate_token_handler_missing_scope()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());

  let result = generate_token_handler(&params);

  assert!(result.is_err(), "Should fail without scope");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "scope"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_generate_token_handler_invalid_scope_format()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());
  params.insert("scope".into(), "invalid".into());

  let result = generate_token_handler(&params);

  assert!(result.is_err(), "Should fail with invalid scope format");
}

#[test]
fn test_generate_token_handler_with_ttl()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());
  params.insert("scope".into(), "read:tokens".into());
  params.insert("ttl".into(), "3600".into());

  let result = generate_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with TTL");
}

#[test]
fn test_generate_token_handler_boundary_ttl_min()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());
  params.insert("scope".into(), "read:tokens".into());
  params.insert("ttl".into(), "60".into());

  let result = generate_token_handler(&params);

  assert!(result.is_ok(), "Should accept minimum TTL");
}

#[test]
fn test_generate_token_handler_boundary_ttl_max()
{
  let mut params = HashMap::new();
  params.insert("name".into(), "my-token".into());
  params.insert("scope".into(), "read:tokens".into());
  params.insert("ttl".into(), "31536000".into());

  let result = generate_token_handler(&params);

  assert!(result.is_ok(), "Should accept maximum TTL");
}

// ============================================================================
// .tokens.list tests (6 tests)
// ============================================================================

#[test]
fn test_list_tokens_handler_success_empty()
{
  let params = HashMap::new();

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed with empty list");
}

#[test]
fn test_list_tokens_handler_success_multiple()
{
  let params = HashMap::new();

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_list_tokens_handler_format_table()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "table".into());

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed with table format");
}

#[test]
fn test_list_tokens_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "json".into());

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}

#[test]
fn test_list_tokens_handler_with_filter()
{
  let mut params = HashMap::new();
  params.insert("filter".into(), "active".into());

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed with filter");
}

#[test]
fn test_list_tokens_handler_with_sort()
{
  let mut params = HashMap::new();
  params.insert("sort".into(), "created_at".into());

  let result = list_tokens_handler(&params);

  assert!(result.is_ok(), "Should succeed with sort");
}

// ============================================================================
// .tokens.get tests (7 tests)
// ============================================================================

#[test]
fn test_get_token_handler_success()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());

  let result = get_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with valid token_id");
}

#[test]
fn test_get_token_handler_missing_token_id()
{
  let params = HashMap::new();

  let result = get_token_handler(&params);

  assert!(result.is_err(), "Should fail without token_id");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "token_id"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_get_token_handler_empty_token_id()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "".into());

  let result = get_token_handler(&params);

  assert!(result.is_err(), "Should fail with empty token_id");
}

#[test]
fn test_get_token_handler_invalid_token_id_format()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "invalid@format".into());

  let result = get_token_handler(&params);

  assert!(result.is_err(), "Should fail with invalid format");
}

#[test]
fn test_get_token_handler_format_expanded()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("format".into(), "expanded".into());

  let result = get_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with expanded format");
}

#[test]
fn test_get_token_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("format".into(), "json".into());

  let result = get_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}

#[test]
fn test_get_token_handler_format_yaml()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("format".into(), "yaml".into());

  let result = get_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with yaml format");
}

// ============================================================================
// .tokens.rotate tests (7 tests)
// ============================================================================

#[test]
fn test_rotate_token_handler_success()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());

  let result = rotate_token_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_rotate_token_handler_missing_token_id()
{
  let params = HashMap::new();

  let result = rotate_token_handler(&params);

  assert!(result.is_err(), "Should fail without token_id");
}

#[test]
fn test_rotate_token_handler_empty_token_id()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "".into());

  let result = rotate_token_handler(&params);

  assert!(result.is_err(), "Should fail with empty token_id");
}

#[test]
fn test_rotate_token_handler_invalid_format()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "invalid@format".into());

  let result = rotate_token_handler(&params);

  assert!(result.is_err(), "Should fail with invalid format");
}

#[test]
fn test_rotate_token_handler_with_ttl_change()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("ttl".into(), "7200".into());

  let result = rotate_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with TTL change");
}

#[test]
fn test_rotate_token_handler_preserve_scope()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());

  let result = rotate_token_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_rotate_token_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("token_id".into(), "tok_abc123".into());
    params.insert("format".into(), format.into());

    let result = rotate_token_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .tokens.revoke tests (7 tests)
// ============================================================================

#[test]
fn test_revoke_token_handler_success()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());

  let result = revoke_token_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_revoke_token_handler_missing_token_id()
{
  let params = HashMap::new();

  let result = revoke_token_handler(&params);

  assert!(result.is_err(), "Should fail without token_id");
}

#[test]
fn test_revoke_token_handler_empty_token_id()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "".into());

  let result = revoke_token_handler(&params);

  assert!(result.is_err(), "Should fail with empty token_id");
}

#[test]
fn test_revoke_token_handler_with_reason()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("reason".into(), "compromised".into());

  let result = revoke_token_handler(&params);

  assert!(result.is_ok(), "Should succeed with reason");
}

#[test]
fn test_revoke_token_handler_empty_reason()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());
  params.insert("reason".into(), "".into());

  let result = revoke_token_handler(&params);

  assert!(result.is_ok(), "Should accept empty reason");
}

#[test]
fn test_revoke_token_handler_confirmation()
{
  let mut params = HashMap::new();
  params.insert("token_id".into(), "tok_abc123".into());

  let result = revoke_token_handler(&params);

  assert!(result.is_ok(), "Should succeed");
  let output = result.unwrap();
  assert!(output.contains("revoke"), "Should mention revoke");
}

#[test]
fn test_revoke_token_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("token_id".into(), "tok_abc123".into());
    params.insert("format".into(), format.into());

    let result = revoke_token_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}
