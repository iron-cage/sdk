//! Limits adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 5 limits management adapters: list, get, create, update, delete
//! Total: 25 tests (5 per adapter)
//!
//! ## Testing Strategy
//!
//! Uses InMemoryAdapter (real implementation, no mocking)

use iron_cli::adapters::LimitsService;
use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ Formatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

async fn create_adapter_with_limits() -> Arc<InMemoryAdapter>
{
  let adapter = Arc::new( InMemoryAdapter::new() );
  let _ = adapter.create_limit( "api_requests", 1000 ).await;
  let _ = adapter.create_limit( "storage_gb", 100 ).await;
  let _ = adapter.create_limit( "bandwidth_mbps", 500 ).await;
  adapter
}

fn create_verified_command(command: &str, args: &[(&str, &str)]) -> MockVerifiedCommand
{
  let mut params = HashMap::new();
  for (key, value) in args
  {
    params.insert( key.to_string(), value.to_string() );
  }

  MockVerifiedCommand {
    #[ allow( dead_code ) ]
    command: command.to_string(),
    params,
  }
}

struct MockVerifiedCommand
{
  #[ allow( dead_code ) ]
  command: String,
  params: HashMap<String, String>,
}

impl HasParams for MockVerifiedCommand
{
  fn get_params(&self) -> HashMap<String, String>
  {
    self.params.clone()
  }
}

// ============================================================================
// .limits.list adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_list_limits_adapter_success()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.list", &[] );

  let result = iron_cli::adapters::limits::list_limits_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed listing limits" );

  let output = result.unwrap();
  assert!(
    output.contains( "limit" ) || output.contains( "Limit" ),
    "Output should contain limit information"
  );
}

#[ tokio::test ]
async fn test_list_limits_adapter_empty()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.list", &[] );

  let result = iron_cli::adapters::limits::list_limits_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed even with no limits" );
}

#[ tokio::test ]
async fn test_list_limits_adapter_json_format()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".limits.list",
    &[("format", "json")],
  );

  let result = iron_cli::adapters::limits::list_limits_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );
}

#[ tokio::test ]
async fn test_list_limits_adapter_with_count()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.list", &[] );

  let result = iron_cli::adapters::limits::list_limits_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok() );

  let output = result.unwrap();
  assert!(
    output.contains( "3" ) || output.contains( "count" ),
    "Output should show count of limits"
  );
}

#[ tokio::test ]
async fn test_list_limits_adapter_storage_error()
{
  let adapter = create_test_adapter();
  adapter.set_failure_mode( "database_error" );
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.list", &[] );

  let result = iron_cli::adapters::limits::list_limits_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with storage error" );
}

// ============================================================================
// .limits.get adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_get_limit_adapter_success()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.get",
    &[("limit_id", "lim_api_requests")],
  );

  let result = iron_cli::adapters::limits::get_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed getting limit" );

  let output = result.unwrap();
  assert!(
    output.contains( "api_requests" ) || output.contains( "limit" ),
    "Output should contain limit details"
  );
}

#[ tokio::test ]
async fn test_get_limit_adapter_missing_limit_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.get", &[] );

  let result = iron_cli::adapters::limits::get_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without limit_id" );
}

#[ tokio::test ]
async fn test_get_limit_adapter_not_found()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.get",
    &[("limit_id", "nonexistent")],
  );

  let result = iron_cli::adapters::limits::get_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with not found error" );
}

#[ tokio::test ]
async fn test_get_limit_adapter_empty_limit_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.get",
    &[("limit_id", "")],
  );

  let result = iron_cli::adapters::limits::get_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty limit_id" );
}

#[ tokio::test ]
async fn test_get_limit_adapter_all_formats()
{
  let adapter = create_adapter_with_limits().await;

  let formats = vec!["table", "json", "yaml"];

  for format_str in formats
  {
    let formatter = Formatter::new( match format_str
    {
      "json" => OutputFormat::Json,
      "yaml" => OutputFormat::Yaml,
      _ => OutputFormat::Table,
    });

    let command = create_verified_command(
      ".limits.get",
      &[
        ("limit_id", "lim_api_requests"),
        ("format", format_str),
      ],
    );

    let result = iron_cli::adapters::limits::get_limit_adapter(
      &command,
      adapter.clone(),
      &formatter,
    ).await;

    assert!(
      result.is_ok(),
      "Should succeed with format '{}'",
      format_str
    );
  }
}

// ============================================================================
// .limits.create adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_create_limit_adapter_success()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.create",
    &[
      ("resource_type", "api_calls"),
      ("limit_value", "5000"),
    ],
  );

  let result = iron_cli::adapters::limits::create_limit_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed creating limit" );

  // Verify limit was created
  let limits = adapter.list_limits().await.unwrap();
  assert_eq!( limits.len(), 1, "Should have 1 limit" );
}

#[ tokio::test ]
async fn test_create_limit_adapter_missing_resource_type()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.create",
    &[("limit_value", "1000")],
  );

  let result = iron_cli::adapters::limits::create_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without resource_type" );
}

#[ tokio::test ]
async fn test_create_limit_adapter_missing_limit_value()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.create",
    &[("resource_type", "api_calls")],
  );

  let result = iron_cli::adapters::limits::create_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without limit_value" );
}

#[ tokio::test ]
async fn test_create_limit_adapter_invalid_limit_value()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.create",
    &[
      ("resource_type", "api_calls"),
      ("limit_value", "invalid"),
    ],
  );

  let result = iron_cli::adapters::limits::create_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid limit_value" );
}

#[ tokio::test ]
async fn test_create_limit_adapter_negative_value()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.create",
    &[
      ("resource_type", "api_calls"),
      ("limit_value", "-100"),
    ],
  );

  let result = iron_cli::adapters::limits::create_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with negative limit_value" );
}

// ============================================================================
// .limits.update adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_update_limit_adapter_success()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.update",
    &[
      ("limit_id", "lim_api_requests"),
      ("limit_value", "2000"),
    ],
  );

  let result = iron_cli::adapters::limits::update_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed updating limit" );
}

#[ tokio::test ]
async fn test_update_limit_adapter_missing_limit_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.update",
    &[("limit_value", "2000")],
  );

  let result = iron_cli::adapters::limits::update_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without limit_id" );
}

#[ tokio::test ]
async fn test_update_limit_adapter_missing_limit_value()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.update",
    &[("limit_id", "lim_api_requests")],
  );

  let result = iron_cli::adapters::limits::update_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without limit_value" );
}

#[ tokio::test ]
async fn test_update_limit_adapter_not_found()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.update",
    &[
      ("limit_id", "nonexistent"),
      ("limit_value", "2000"),
    ],
  );

  let result = iron_cli::adapters::limits::update_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with not found error" );
}

#[ tokio::test ]
async fn test_update_limit_adapter_invalid_value()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.update",
    &[
      ("limit_id", "lim_api_requests"),
      ("limit_value", "invalid"),
    ],
  );

  let result = iron_cli::adapters::limits::update_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid limit_value" );
}

// ============================================================================
// .limits.delete adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_delete_limit_adapter_success()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.delete",
    &[("limit_id", "lim_api_requests")],
  );

  let result = iron_cli::adapters::limits::delete_limit_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed deleting limit" );

  // Verify limit was deleted
  let limits = adapter.list_limits().await.unwrap();
  assert_eq!( limits.len(), 2, "Should have 2 limits remaining" );
}

#[ tokio::test ]
async fn test_delete_limit_adapter_missing_limit_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".limits.delete", &[] );

  let result = iron_cli::adapters::limits::delete_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without limit_id" );
}

#[ tokio::test ]
async fn test_delete_limit_adapter_empty_limit_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.delete",
    &[("limit_id", "")],
  );

  let result = iron_cli::adapters::limits::delete_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty limit_id" );
}

#[ tokio::test ]
async fn test_delete_limit_adapter_not_found()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.delete",
    &[("limit_id", "nonexistent")],
  );

  let result = iron_cli::adapters::limits::delete_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with not found error" );
}

#[ tokio::test ]
async fn test_delete_limit_adapter_confirmation()
{
  let adapter = create_adapter_with_limits().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".limits.delete",
    &[("limit_id", "lim_storage_gb")],
  );

  let result = iron_cli::adapters::limits::delete_limit_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok() );

  let output = result.unwrap();
  assert!(
    output.contains( "delete" ) || output.contains( "Delete" ),
    "Output should confirm deletion"
  );
}
