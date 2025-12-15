//! Usage handler tests
//!
//! ## Test Coverage
//!
//! Covers show, by-project, by-provider, export usage handlers.
//! Total: 20 test cases

use std::collections::HashMap;
use iron_cli::handlers::{ usage_handlers::*, CliError };

// ============================================================================
// .usage.show tests (5 tests)
// ============================================================================

#[test]
fn test_show_usage_handler_success_default()
{
  let params = HashMap::new();

  let result = show_usage_handler(&params);

  assert!(result.is_ok(), "Should succeed with default params");
}

#[test]
fn test_show_usage_handler_with_date_range()
{
  let mut params = HashMap::new();
  params.insert("start_date".into(), "2025-01-01".into());
  params.insert("end_date".into(), "2025-01-31".into());

  let result = show_usage_handler(&params);

  assert!(result.is_ok(), "Should succeed with date range");
}

#[test]
fn test_show_usage_handler_invalid_date_format()
{
  let mut params = HashMap::new();
  params.insert("start_date".into(), "invalid-date".into());

  let result = show_usage_handler(&params);

  assert!(result.is_err(), "Should fail with invalid date");
}

#[test]
fn test_show_usage_handler_date_range_backwards()
{
  let mut params = HashMap::new();
  params.insert("start_date".into(), "2025-02-01".into());
  params.insert("end_date".into(), "2025-01-01".into());

  let result = show_usage_handler(&params);

  assert!(result.is_err(), "Should fail with backwards date range");
}

#[test]
fn test_show_usage_handler_all_formats()
{
  let formats = vec!["table", "expanded", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("format".into(), format.into());

    let result = show_usage_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .usage.by-project tests (5 tests)
// ============================================================================

#[test]
fn test_usage_by_project_handler_success()
{
  let mut params = HashMap::new();
  params.insert("project_id".into(), "proj_123".into());

  let result = usage_by_project_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_usage_by_project_handler_missing_project_id()
{
  let params = HashMap::new();

  let result = usage_by_project_handler(&params);

  assert!(result.is_err(), "Should fail without project_id");
  match result.unwrap_err()
  {
    CliError::MissingParameter(name) => assert_eq!(name, "project_id"),
    other => panic!("Wrong error type: {:?}", other),
  }
}

#[test]
fn test_usage_by_project_handler_empty_project_id()
{
  let mut params = HashMap::new();
  params.insert("project_id".into(), "".into());

  let result = usage_by_project_handler(&params);

  assert!(result.is_err(), "Should fail with empty project_id");
}

#[test]
fn test_usage_by_project_handler_with_date_range()
{
  let mut params = HashMap::new();
  params.insert("project_id".into(), "proj_123".into());
  params.insert("start_date".into(), "2025-01-01".into());

  let result = usage_by_project_handler(&params);

  assert!(result.is_ok(), "Should succeed with date range");
}

#[test]
fn test_usage_by_project_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("project_id".into(), "proj_123".into());
    params.insert("format".into(), format.into());

    let result = usage_by_project_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .usage.by-provider tests (5 tests)
// ============================================================================

#[test]
fn test_usage_by_provider_handler_success()
{
  let mut params = HashMap::new();
  params.insert("provider".into(), "openai".into());

  let result = usage_by_provider_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_usage_by_provider_handler_missing_provider()
{
  let params = HashMap::new();

  let result = usage_by_provider_handler(&params);

  assert!(result.is_err(), "Should fail without provider");
}

#[test]
fn test_usage_by_provider_handler_invalid_provider()
{
  let mut params = HashMap::new();
  params.insert("provider".into(), "invalid-provider".into());

  let result = usage_by_provider_handler(&params);

  assert!(result.is_err(), "Should fail with invalid provider");
}

#[test]
fn test_usage_by_provider_handler_with_aggregation()
{
  let mut params = HashMap::new();
  params.insert("provider".into(), "openai".into());
  params.insert("aggregation".into(), "daily".into());

  let result = usage_by_provider_handler(&params);

  assert!(result.is_ok(), "Should succeed with aggregation");
}

#[test]
fn test_usage_by_provider_handler_all_formats()
{
  let formats = vec!["table", "json", "yaml"];

  for format in formats
  {
    let mut params = HashMap::new();
    params.insert("provider".into(), "openai".into());
    params.insert("format".into(), format.into());

    let result = usage_by_provider_handler(&params);

    assert!(result.is_ok(), "Should succeed with format '{}'", format);
  }
}

// ============================================================================
// .usage.export tests (5 tests)
// ============================================================================

#[test]
fn test_export_usage_handler_success()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "/tmp/usage.json".into());

  let result = export_usage_handler(&params);

  assert!(result.is_ok(), "Should succeed");
}

#[test]
fn test_export_usage_handler_missing_output()
{
  let params = HashMap::new();

  let result = export_usage_handler(&params);

  assert!(result.is_err(), "Should fail without output");
}

#[test]
fn test_export_usage_handler_empty_output_path()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "".into());

  let result = export_usage_handler(&params);

  assert!(result.is_err(), "Should fail with empty output path");
}

#[test]
fn test_export_usage_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "/tmp/usage.json".into());
  params.insert("format".into(), "json".into());

  let result = export_usage_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}

#[test]
fn test_export_usage_handler_format_csv()
{
  let mut params = HashMap::new();
  params.insert("output".into(), "/tmp/usage.csv".into());
  params.insert("format".into(), "csv".into());

  let result = export_usage_handler(&params);

  assert!(result.is_ok(), "Should succeed with csv format");
}
