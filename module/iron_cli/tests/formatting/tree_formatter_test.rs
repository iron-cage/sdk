//! TreeFmtFormatter tests
//!
//! ## Test Coverage
//!
//! Tests tree_fmt-based formatter implementation.
//! Verifies all 4 output formats: table, expanded, json, yaml.
//!
//! ## Test Strategy
//!
//! Pure function testing: data → formatted string
//! Verifies same API compatibility as legacy Formatter.

use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use std::collections::HashMap;

// ============================================================================
// Category 1: Single Item Formatting (4 tests)
// ============================================================================

#[ test ]
fn test_format_single_item_table()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut data = HashMap::new();
  data.insert( "id".to_string(), "tok_123".to_string() );
  data.insert( "name".to_string(), "test".to_string() );

  let result = formatter.format_single( &data );

  assert!( result.contains( "id" ) );
  assert!( result.contains( "tok_123" ) );
  assert!( result.contains( "name" ) );
  assert!( result.contains( "test" ) );
}

#[ test ]
fn test_format_single_item_expanded()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Expanded );

  let mut data = HashMap::new();
  data.insert( "id".to_string(), "tok_123".to_string() );
  data.insert( "name".to_string(), "test".to_string() );

  let result = formatter.format_single( &data );

  assert!( result.contains( "id:" ) );
  assert!( result.contains( "tok_123" ) );
  assert!( result.contains( "name:" ) );
  assert!( result.contains( "test" ) );
}

#[ test ]
fn test_format_single_item_json()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  let mut data = HashMap::new();
  data.insert( "id".to_string(), "tok_123".to_string() );
  data.insert( "name".to_string(), "test".to_string() );

  let result = formatter.format_single( &data );

  // JSON format - verify structure
  assert!( result.contains( "\"id\"" ) );
  assert!( result.contains( "\"tok_123\"" ) );
  assert!( result.contains( "\"name\"" ) );
  assert!( result.contains( "\"test\"" ) );
}

#[ test ]
fn test_format_single_item_yaml()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Yaml );

  let mut data = HashMap::new();
  data.insert( "id".to_string(), "tok_123".to_string() );
  data.insert( "name".to_string(), "test".to_string() );

  let result = formatter.format_single( &data );

  // YAML format - verify structure
  assert!( result.contains( "id:" ) );
  assert!( result.contains( "tok_123" ) );
  assert!( result.contains( "name:" ) );
  assert!( result.contains( "test" ) );
}

// ============================================================================
// Category 2: List Formatting (4 tests)
// ============================================================================

#[ test ]
fn test_format_list_table()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let mut item1 = HashMap::new();
  item1.insert( "id".to_string(), "tok_1".to_string() );
  item1.insert( "name".to_string(), "first".to_string() );

  let mut item2 = HashMap::new();
  item2.insert( "id".to_string(), "tok_2".to_string() );
  item2.insert( "name".to_string(), "second".to_string() );

  let items = vec![ item1, item2 ];
  let result = formatter.format_list( &items );

  assert!( result.contains( "tok_1" ) );
  assert!( result.contains( "tok_2" ) );
  assert!( result.contains( "first" ) );
  assert!( result.contains( "second" ) );
}

#[ test ]
fn test_format_list_expanded()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Expanded );

  let mut item1 = HashMap::new();
  item1.insert( "id".to_string(), "tok_1".to_string() );

  let mut item2 = HashMap::new();
  item2.insert( "id".to_string(), "tok_2".to_string() );

  let items = vec![ item1, item2 ];
  let result = formatter.format_list( &items );

  assert!( result.contains( "Item 1:" ) );
  assert!( result.contains( "Item 2:" ) );
  assert!( result.contains( "tok_1" ) );
  assert!( result.contains( "tok_2" ) );
}

#[ test ]
fn test_format_list_json()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  let mut item1 = HashMap::new();
  item1.insert( "id".to_string(), "tok_1".to_string() );

  let items = vec![ item1 ];
  let result = formatter.format_list( &items );

  // JSON array format
  assert!( result.starts_with( '[' ) );
  assert!( result.ends_with( ']' ) );
  assert!( result.contains( "\"tok_1\"" ) );
}

#[ test ]
fn test_format_list_yaml()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Yaml );

  let mut item1 = HashMap::new();
  item1.insert( "id".to_string(), "tok_1".to_string() );

  let items = vec![ item1 ];
  let result = formatter.format_list( &items );

  // YAML format
  assert!( result.contains( "tok_1" ) );
}

// ============================================================================
// Category 3: Empty Data Handling (3 tests)
// ============================================================================

#[ test ]
fn test_format_single_empty()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );
  let data = HashMap::new();
  let result = formatter.format_single( &data );

  assert!( result.is_empty() );
}

#[ test ]
fn test_format_list_empty()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );
  let items: Vec< HashMap< String, String > > = vec![];
  let result = formatter.format_list( &items );

  assert_eq!( result, "No items found" );
}

#[ test ]
fn test_format_single_expanded_empty()
{
  let formatter = TreeFmtFormatter::new( OutputFormat::Expanded );
  let data = HashMap::new();
  let result = formatter.format_single( &data );

  assert!( result.is_empty() );
}
