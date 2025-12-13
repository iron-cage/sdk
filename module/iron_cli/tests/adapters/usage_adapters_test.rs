//! Usage adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 4 usage management adapters: show, by-project, by-provider, export
//! Total: 20 tests (5 per adapter)
//!
//! ## Architecture Under Test
//!
//! ```text
//! VerifiedCommand → usage_adapter() → usage_handler() → UsageService
//!                         ↓                   ↓              ↓
//!                    Extract params      Validate      Async I/O
//! ```
//!
//! ## Testing Strategy
//!
//! Uses InMemoryAdapter (real implementation, no mocking):
//! - Predictable: HashMap-based storage
//! - Fast: No network/DB overhead
//! - Real: Same interface as SqlxAdapter
//!
//! ## Test Matrix
//!
//! See tests/adapters/-test_matrix.md for complete specification

use iron_cli::adapters::{ AdapterError, UsageService };
use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

/// Helper: Create test adapter with empty storage
fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

/// Helper: Create test adapter with pre-seeded usage data
async fn create_adapter_with_usage() -> Arc<InMemoryAdapter>
{
  let adapter = Arc::new( InMemoryAdapter::new() );
  // Seed some usage data
  let _ = adapter.record_usage( "project-1", "openai", 1000, 500 ).await;
  let _ = adapter.record_usage( "project-2", "anthropic", 2000, 1000 ).await;
  let _ = adapter.record_usage( "project-1", "cohere", 1500, 750 ).await;
  adapter
}

/// Helper: Create VerifiedCommand mock for testing
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

/// Mock VerifiedCommand for testing (until unilang types available)
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
// .usage.show adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_show_usage_adapter_success()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".usage.show", &[] );

  let result = iron_cli::adapters::usage::show_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with no params" );

  let output = result.unwrap();
  assert!(
    output.contains( "usage" ) || output.contains( "Usage" ),
    "Output should contain usage information"
  );
}

#[ tokio::test ]
async fn test_show_usage_adapter_with_date_range()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.show",
    &[
      ("start_date", "2025-01-01"),
      ("end_date", "2025-12-31"),
    ],
  );

  let result = iron_cli::adapters::usage::show_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid date range" );

  let output = result.unwrap();
  assert!(
    output.contains( "2025" ),
    "Output should reflect date range"
  );
}

#[ tokio::test ]
async fn test_show_usage_adapter_invalid_date_format()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.show",
    &[("start_date", "invalid-date")],
  );

  let result = iron_cli::adapters::usage::show_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid date format" );
}

#[ tokio::test ]
async fn test_show_usage_adapter_backwards_date_range()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.show",
    &[
      ("start_date", "2025-12-31"),
      ("end_date", "2025-01-01"),  // Earlier than start
    ],
  );

  let result = iron_cli::adapters::usage::show_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with backwards date range" );
}

#[ tokio::test ]
async fn test_show_usage_adapter_json_format()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".usage.show",
    &[("format", "json")],
  );

  let result = iron_cli::adapters::usage::show_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );
}

// ============================================================================
// .usage.by-project adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_usage_by_project_adapter_success()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-project",
    &[("project_id", "project-1")],
  );

  let result = iron_cli::adapters::usage::usage_by_project_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid project_id" );

  let output = result.unwrap();
  assert!(
    output.contains( "project-1" ) || output.contains( "project" ),
    "Output should contain project information"
  );
}

#[ tokio::test ]
async fn test_usage_by_project_adapter_missing_project_id()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".usage.by-project", &[] );

  let result = iron_cli::adapters::usage::usage_by_project_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without project_id" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError(_) => {},  // Expected
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_usage_by_project_adapter_empty_project_id()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-project",
    &[("project_id", "")],
  );

  let result = iron_cli::adapters::usage::usage_by_project_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty project_id" );
}

#[ tokio::test ]
async fn test_usage_by_project_adapter_with_date_range()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-project",
    &[
      ("project_id", "project-1"),
      ("start_date", "2025-01-01"),
    ],
  );

  let result = iron_cli::adapters::usage::usage_by_project_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with date range" );
}

#[ tokio::test ]
async fn test_usage_by_project_adapter_not_found()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-project",
    &[("project_id", "nonexistent-project")],
  );

  let result = iron_cli::adapters::usage::usage_by_project_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  // Should succeed but return empty/no data message
  assert!( result.is_ok(), "Should succeed even if project not found" );
}

// ============================================================================
// .usage.by-provider adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_usage_by_provider_adapter_success()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-provider",
    &[("provider", "openai")],
  );

  let result = iron_cli::adapters::usage::usage_by_provider_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid provider" );

  let output = result.unwrap();
  assert!(
    output.contains( "openai" ) || output.contains( "provider" ),
    "Output should contain provider information"
  );
}

#[ tokio::test ]
async fn test_usage_by_provider_adapter_missing_provider()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".usage.by-provider", &[] );

  let result = iron_cli::adapters::usage::usage_by_provider_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without provider" );
}

#[ tokio::test ]
async fn test_usage_by_provider_adapter_invalid_provider()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-provider",
    &[("provider", "invalid-provider")],
  );

  let result = iron_cli::adapters::usage::usage_by_provider_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid provider" );
}

#[ tokio::test ]
async fn test_usage_by_provider_adapter_with_aggregation()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.by-provider",
    &[
      ("provider", "anthropic"),
      ("aggregation", "daily"),
    ],
  );

  let result = iron_cli::adapters::usage::usage_by_provider_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with aggregation parameter" );

  let output = result.unwrap();
  assert!(
    output.contains( "daily" ) || output.contains( "aggregation" ),
    "Output should reflect aggregation"
  );
}

#[ tokio::test ]
async fn test_usage_by_provider_adapter_all_formats()
{
  let adapter = create_adapter_with_usage().await;

  let formats = vec!["table", "json", "yaml"];

  for format_str in formats
  {
    let formatter = TreeFmtFormatter::new( match format_str
    {
      "json" => OutputFormat::Json,
      "yaml" => OutputFormat::Yaml,
      _ => OutputFormat::Table,
    });

    let command = create_verified_command(
      ".usage.by-provider",
      &[
        ("provider", "openai"),
        ("format", format_str),
      ],
    );

    let result = iron_cli::adapters::usage::usage_by_provider_adapter(
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
// .usage.export adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_export_usage_adapter_success()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.export",
    &[("output", "/tmp/usage_export.json")],
  );

  let result = iron_cli::adapters::usage::export_usage_adapter(
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
async fn test_export_usage_adapter_missing_output()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".usage.export", &[] );

  let result = iron_cli::adapters::usage::export_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without output parameter" );
}

#[ tokio::test ]
async fn test_export_usage_adapter_empty_output()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.export",
    &[("output", "")],
  );

  let result = iron_cli::adapters::usage::export_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with empty output path" );
}

#[ tokio::test ]
async fn test_export_usage_adapter_format_json()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".usage.export",
    &[
      ("output", "/tmp/usage_export.json"),
      ("format", "json"),
    ],
  );

  let result = iron_cli::adapters::usage::export_usage_adapter(
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
async fn test_export_usage_adapter_format_csv()
{
  let adapter = create_adapter_with_usage().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".usage.export",
    &[
      ("output", "/tmp/usage_export.csv"),
      ("format", "csv"),
    ],
  );

  let result = iron_cli::adapters::usage::export_usage_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with CSV format" );

  let output = result.unwrap();
  assert!(
    output.contains( "csv" ) || output.contains( "CSV" ),
    "Output should mention CSV format"
  );
}
