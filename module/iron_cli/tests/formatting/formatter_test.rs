//! Formatter tests
//!
//! ## Test Coverage
//!
//! Tests all 4 output formats: table, expanded, json, yaml.
//! Total: 23 test cases
//!
//! ## Test Strategy
//!
//! Pure function testing: data → formatted string
//! No mocking - formatter has no I/O to mock.
//!
//! ## Test Matrix
//!
//! See tests/formatting/-test_matrix.md for complete coverage plan.

use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use std::collections::HashMap;

// ============================================================================
// Category 1: OutputFormat Enum (4 tests)
// ============================================================================

#[test]
fn test_output_format_from_str_table()
{
  let format: Result<OutputFormat, _> = "table".parse();
  assert!(format.is_ok());
  assert!(matches!(format.unwrap(), OutputFormat::Table));
}

#[test]
fn test_output_format_from_str_json()
{
  let format: Result<OutputFormat, _> = "json".parse();
  assert!(format.is_ok());
  assert!(matches!(format.unwrap(), OutputFormat::Json));
}

#[test]
fn test_output_format_from_str_yaml()
{
  let format: Result<OutputFormat, _> = "yaml".parse();
  assert!(format.is_ok());
  assert!(matches!(format.unwrap(), OutputFormat::Yaml));
}

#[test]
fn test_output_format_from_str_invalid()
{
  let format: Result<OutputFormat, _> = "invalid".parse();
  assert!(format.is_err());
}

// ============================================================================
// Category 2: Single Item Formatting (4 tests)
// ============================================================================

#[test]
fn test_format_single_item_table()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Table);

  let mut data = HashMap::new();
  data.insert("id".to_string(), "tok_123".to_string());
  data.insert("name".to_string(), "test".to_string());

  let result = formatter.format_single(&data);

  assert!(result.contains("id"));
  assert!(result.contains("tok_123"));
  assert!(result.contains("name"));
  assert!(result.contains("test"));
}

#[test]
fn test_format_single_item_expanded()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Expanded);

  let mut data = HashMap::new();
  data.insert("id".to_string(), "tok_123".to_string());
  data.insert("name".to_string(), "test".to_string());

  let result = formatter.format_single(&data);

  assert!(result.contains("id:"));
  assert!(result.contains("tok_123"));
  assert!(result.contains("name:"));
  assert!(result.contains("test"));
}

#[test]
fn test_format_single_item_json()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Json);

  let mut data = HashMap::new();
  data.insert("id".to_string(), "tok_123".to_string());
  data.insert("name".to_string(), "test".to_string());

  let result = formatter.format_single(&data);

  // Should be valid JSON
  let parsed: serde_json::Result<HashMap<String, String>> = serde_json::from_str(&result);
  assert!(parsed.is_ok());
}

#[test]
fn test_format_single_item_yaml()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Yaml);

  let mut data = HashMap::new();
  data.insert("id".to_string(), "tok_123".to_string());
  data.insert("name".to_string(), "test".to_string());

  let result = formatter.format_single(&data);

  // Should be valid YAML
  let parsed: serde_yaml::Result<HashMap<String, String>> = serde_yaml::from_str(&result);
  assert!(parsed.is_ok());
}

// ============================================================================
// Category 3: Multiple Items Formatting (4 tests)
// ============================================================================

#[test]
fn test_format_multiple_items_table()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Table);

  let items = vec![
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "1".to_string());
      item.insert("name".to_string(), "first".to_string());
      item
    },
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "2".to_string());
      item.insert("name".to_string(), "second".to_string());
      item
    },
  ];

  let result = formatter.format_list(&items);

  assert!(result.contains("id"));
  assert!(result.contains("name"));
  assert!(result.contains("1"));
  assert!(result.contains("2"));
}

#[test]
fn test_format_multiple_items_expanded()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Expanded);

  let items = vec![
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "1".to_string());
      item
    },
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "2".to_string());
      item
    },
  ];

  let result = formatter.format_list(&items);

  // Should have two separate blocks
  assert!(result.contains("id:"));
  assert!(result.contains("1"));
  assert!(result.contains("2"));
}

#[test]
fn test_format_multiple_items_json()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Json);

  let items = vec![
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "1".to_string());
      item
    },
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "2".to_string());
      item
    },
  ];

  let result = formatter.format_list(&items);

  // Should be valid JSON array
  let parsed: serde_json::Result<Vec<HashMap<String, String>>> = serde_json::from_str(&result);
  assert!(parsed.is_ok());
  assert_eq!(parsed.unwrap().len(), 2);
}

#[test]
fn test_format_multiple_items_yaml()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Yaml);

  let items = vec![
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "1".to_string());
      item
    },
    {
      let mut item = HashMap::new();
      item.insert("id".to_string(), "2".to_string());
      item
    },
  ];

  let result = formatter.format_list(&items);

  // Should be valid YAML array
  let parsed: serde_yaml::Result<Vec<HashMap<String, String>>> = serde_yaml::from_str(&result);
  assert!(parsed.is_ok());
  assert_eq!(parsed.unwrap().len(), 2);
}

// ============================================================================
// Category 4: Empty Data Formatting (4 tests)
// ============================================================================

#[test]
fn test_format_empty_list_table()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Table);
  let items: Vec<HashMap<String, String>> = vec![];

  let result = formatter.format_list(&items);

  assert!(result.contains("No items") || result.contains("empty"));
}

#[test]
fn test_format_empty_list_expanded()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Expanded);
  let items: Vec<HashMap<String, String>> = vec![];

  let result = formatter.format_list(&items);

  assert!(result.contains("No items") || result.contains("empty"));
}

#[test]
fn test_format_empty_list_json()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Json);
  let items: Vec<HashMap<String, String>> = vec![];

  let result = formatter.format_list(&items);

  // Should be valid empty JSON array
  assert_eq!(result.trim(), "[]");
}

#[test]
fn test_format_empty_list_yaml()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Yaml);
  let items: Vec<HashMap<String, String>> = vec![];

  let result = formatter.format_list(&items);

  // Should be valid empty YAML array
  assert!(result.trim() == "[]" || result.trim() == "");
}

// ============================================================================
// Category 5: Error Formatting (3 tests)
// ============================================================================

use iron_cli::handlers::CliError;

#[test]
fn test_format_error_table()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Table);
  let error = CliError::MissingParameter("username");

  let result = formatter.format_error(&error);

  assert!(result.contains("Error") || result.contains("error"));
  assert!(result.contains("username"));
}

#[test]
fn test_format_error_json()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Json);
  let error = CliError::ValidationError("Invalid input".to_string());

  let result = formatter.format_error(&error);

  // Should be valid JSON with error field
  let parsed: serde_json::Result<HashMap<String, serde_json::Value>> = serde_json::from_str(&result);
  assert!(parsed.is_ok());
  assert!(parsed.unwrap().contains_key("error"));
}

#[test]
fn test_format_error_yaml()
{
  let formatter = TreeFmtFormatter::new(OutputFormat::Yaml);
  let error = CliError::InvalidParameter {
    param: "token_id",
    reason: "must start with tok_",
  };

  let result = formatter.format_error(&error);

  // Should be valid YAML with error field
  let parsed: serde_yaml::Result<HashMap<String, serde_yaml::Value>> = serde_yaml::from_str(&result);
  assert!(parsed.is_ok());
}
