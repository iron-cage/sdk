//! Traces adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 3 traces management adapters: list, get, export
//! Total: 15 tests (5 per adapter)

use iron_cli::adapters::TracesService;
use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ Formatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

async fn create_adapter_with_traces() -> Arc<InMemoryAdapter>
{
  let adapter = Arc::new( InMemoryAdapter::new() );
  let _ = adapter.record_trace( "trace-1", "GET /api/users", 150 ).await;
  let _ = adapter.record_trace( "trace-2", "POST /api/auth", 200 ).await;
  let _ = adapter.record_trace( "trace-3", "GET /api/data", 100 ).await;
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
// .traces.list adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_list_traces_adapter_success()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".traces.list", &[] );

  let result = iron_cli::adapters::traces::list_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed listing traces" );

  let output = result.unwrap();
  assert!(
    output.contains( "trace" ) || output.contains( "Trace" ),
    "Output should contain trace information"
  );
}

#[ tokio::test ]
async fn test_list_traces_adapter_empty()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".traces.list", &[] );

  let result = iron_cli::adapters::traces::list_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed even with no traces" );
}

#[ tokio::test ]
async fn test_list_traces_adapter_with_filter()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.list",
    &[("filter", "GET")],
  );

  let result = iron_cli::adapters::traces::list_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with filter" );
}

#[ tokio::test ]
async fn test_list_traces_adapter_with_limit()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.list",
    &[("limit", "10")],
  );

  let result = iron_cli::adapters::traces::list_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with limit" );
}

#[ tokio::test ]
async fn test_list_traces_adapter_json_format()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".traces.list",
    &[("format", "json")],
  );

  let result = iron_cli::adapters::traces::list_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );
}

// ============================================================================
// .traces.get adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_get_trace_adapter_success()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.get",
    &[("trace_id", "trace-1")],
  );

  let result = iron_cli::adapters::traces::get_trace_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed getting trace" );

  let output = result.unwrap();
  assert!(
    output.contains( "trace" ) || output.contains( "Trace" ),
    "Output should contain trace details"
  );
}

#[ tokio::test ]
async fn test_get_trace_adapter_missing_trace_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".traces.get", &[] );

  let result = iron_cli::adapters::traces::get_trace_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without trace_id" );
}

#[ tokio::test ]
async fn test_get_trace_adapter_empty_trace_id()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.get",
    &[("trace_id", "")],
  );

  let result = iron_cli::adapters::traces::get_trace_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty trace_id" );
}

#[ tokio::test ]
async fn test_get_trace_adapter_not_found()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.get",
    &[("trace_id", "nonexistent")],
  );

  let result = iron_cli::adapters::traces::get_trace_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with not found error" );
}

#[ tokio::test ]
async fn test_get_trace_adapter_all_formats()
{
  let adapter = create_adapter_with_traces().await;

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
      ".traces.get",
      &[
        ("trace_id", "trace-1"),
        ("format", format_str),
      ],
    );

    let result = iron_cli::adapters::traces::get_trace_adapter(
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
// .traces.export adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_export_traces_adapter_success()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.export",
    &[("output", "/tmp/traces_export.json")],
  );

  let result = iron_cli::adapters::traces::export_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid output path" );

  let output = result.unwrap();
  assert!(
    output.contains( "export" ) || output.contains( "Export" ),
    "Output should confirm export"
  );
}

#[ tokio::test ]
async fn test_export_traces_adapter_missing_output()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".traces.export", &[] );

  let result = iron_cli::adapters::traces::export_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without output parameter" );
}

#[ tokio::test ]
async fn test_export_traces_adapter_empty_output()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.export",
    &[("output", "")],
  );

  let result = iron_cli::adapters::traces::export_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty output path" );
}

#[ tokio::test ]
async fn test_export_traces_adapter_format_json()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".traces.export",
    &[
      ("output", "/tmp/traces_export.json"),
      ("format", "json"),
    ],
  );

  let result = iron_cli::adapters::traces::export_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );

  let output = result.unwrap();
  assert!(
    output.contains( "json" ) || output.contains( "JSON" ),
    "Output should mention JSON format"
  );
}

#[ tokio::test ]
async fn test_export_traces_adapter_multiple_traces()
{
  let adapter = create_adapter_with_traces().await;
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".traces.export",
    &[("output", "/tmp/all_traces.json")],
  );

  let result = iron_cli::adapters::traces::export_traces_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed exporting multiple traces" );
}
