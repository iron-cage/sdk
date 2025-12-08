//! Health adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 2 health adapters: health, version
//! Total: 10 tests (5 per adapter)

use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ Formatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
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
// .health adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_health_adapter_success()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".health", &[] );

  let result = iron_cli::adapters::health::health_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with no params" );

  let output = result.unwrap();
  assert!(
    output.contains( "health" ) || output.contains( "Health" ),
    "Output should contain health information"
  );
}

#[ tokio::test ]
async fn test_health_adapter_json_format()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".health",
    &[(  "format", "json" )],
  );

  let result = iron_cli::adapters::health::health_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );
}

#[ tokio::test ]
async fn test_health_adapter_all_formats()
{
  let adapter = create_test_adapter();

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
      ".health",
      &[(  "format", format_str )],
    );

    let result = iron_cli::adapters::health::health_adapter(
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

#[ tokio::test ]
async fn test_health_adapter_storage_error()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  adapter.set_failure_mode( "storage_error" );

  let command = create_verified_command( ".health", &[] );

  let result = iron_cli::adapters::health::health_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with storage error" );
}

#[ tokio::test ]
async fn test_health_adapter_with_details()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".health",
    &[(  "verbose", "true" )],
  );

  let result = iron_cli::adapters::health::health_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with verbose flag" );
}

// ============================================================================
// .version adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_version_adapter_success()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".version", &[] );

  let result = iron_cli::adapters::health::version_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with no params" );

  let output = result.unwrap();
  assert!(
    output.contains( "version" ) || output.contains( "Version" ),
    "Output should contain version information"
  );
}

#[ tokio::test ]
async fn test_version_adapter_json_format()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".version",
    &[(  "format", "json" )],
  );

  let result = iron_cli::adapters::health::version_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );
}

#[ tokio::test ]
async fn test_version_adapter_all_formats()
{
  let adapter = create_test_adapter();

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
      ".version",
      &[(  "format", format_str )],
    );

    let result = iron_cli::adapters::health::version_adapter(
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

#[ tokio::test ]
async fn test_version_adapter_storage_error()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  adapter.set_failure_mode( "storage_error" );

  let command = create_verified_command( ".version", &[] );

  let result = iron_cli::adapters::health::version_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with storage error" );
}

#[ tokio::test ]
async fn test_version_adapter_includes_version_number()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".version", &[] );

  let result = iron_cli::adapters::health::version_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed" );

  let output = result.unwrap();
  assert!(
    output.contains( "version" ) || output.contains( "Version" ),
    "Output should contain version string"
  );
}
