//! Traces handler tests
//!
//! ## Test Coverage
//!
//! Covers list, get, export traces handlers.
//! Total: 10 test cases

use std::collections::HashMap;
use iron_cli::handlers::{ traces_handlers::*, CliError };

// ============================================================================
// .traces.list tests (4 tests)
// ============================================================================

#[test]
fn test_list_traces_handler_success()
{
  let params = HashMap::new();

  let result = list_traces_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_list_traces_handler_with_filters()
{
  let mut params = HashMap::new();
  params.insert("filter".into(), "status=success".into());

  let result = list_traces_handler(&params);

  assert!(result.is_ok(), "Should succeed with filters");
}

#[test]
fn test_list_traces_handler_with_pagination()
{
  let mut params = HashMap::new();
  params.insert("limit".into(), "10".into());

  let result = list_traces_handler(&params);

  assert!(result.is_ok(), "Should succeed with pagination");
}

#[test]
fn test_list_traces_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("format".into(), format.into());

    let result = list_traces_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .traces.get tests (3 tests)
// ============================================================================

#[test]
fn test_get_trace_handler_success()
{
  let mut params = HashMap::new();
  params.insert("trace_id".into(), "trace_abc123".into());

  let result = get_trace_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_get_trace_handler_missing_trace_id()
{
  let params = HashMap::new();

  let result = get_trace_handler(&params);

  assert!(result.is_err(), "Should fail without trace_id");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "trace_id"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_get_trace_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("trace_id".into(), "trace_abc123".into());
    params.insert("format".into(), format.into());

    let result = get_trace_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .traces.export tests (3 tests)
// ============================================================================

#[test]
fn test_export_traces_handler_success()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "/tmp/traces.json".into());

  let result = export_traces_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_export_traces_handler_missing_output()
{
  let params = HashMap::new();

  let result = export_traces_handler(&params);

  assert!(result.is_err(), "Should fail without output");
}

#[test]
fn test_export_traces_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "/tmp/traces.json".into());
  params.insert("format".into(), "json".into());

  let result = export_traces_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}
