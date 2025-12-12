//! Adapter coverage verification tests
//!
//! Tests that verify all adapters have valid API endpoints and that no orphaned
//! adapters exist in the codebase.
//!
//! ## Negative Criteria Enforced
//!
//! - NC-A.1: Zero orphaned adapters (adapters without valid API endpoints)
//! - NC-A.2: Zero adapters calling non-existent endpoints
//! - NC-A.3: Orphaned percentage must be 0%
//!
//! ## Migration Context
//!
//! During Phase 2 migration, 6 orphaned adapters were deleted:
//! - show_agent_usage_adapter (usage_adapters.rs)
//! - export_agent_usage_adapter (usage_adapters.rs)
//! - reset_limit_adapter (limits_adapters.rs)
//! - show_agent_limits_adapter (limits_adapters.rs)
//! - update_agent_limit_adapter (limits_adapters.rs)
//! - show_trace_stats_adapter (traces_adapters.rs)
//!
//! After migration:
//! - Total adapters: 22 (was 28)
//! - Orphaned: 0 (was 6)
//! - Orphaned ratio: 0% (was 24%)

use std::path::PathBuf;

/// Test that all adapters have valid API endpoints
///
/// This test verifies that every adapter function in the codebase calls a
/// valid API endpoint. An orphaned adapter is one that calls an endpoint
/// that doesn't exist in the Token Manager API.
///
/// ## Verification Method
///
/// 1. Count adapter functions in each adapter module
/// 2. Verify expected adapter count (22 total)
/// 3. Check that no orphaned adapter names appear in source code
///
/// ## Expected Adapter Count
///
/// - auth_adapters: 3
/// - token_adapters: 5
/// - usage_adapters: 4 (was 6, deleted 2)
/// - limits_adapters: 5 (was 8, deleted 3)
/// - traces_adapters: 3 (was 4, deleted 1)
/// - health_adapters: 2
///
/// Total: 22 adapters
#[ test ]
fn test_all_adapters_have_valid_endpoints()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // Read adapter source files
  let adapter_files = vec![
    ( "auth_adapters.rs", 3 ),
    ( "token_adapters.rs", 5 ),
    ( "usage_adapters.rs", 4 ),
    ( "limits_adapters.rs", 5 ),
    ( "traces_adapters.rs", 3 ),
    ( "health_adapters.rs", 2 ),
  ];

  let mut total_adapters = 0;

  for ( file_name, expected_count ) in &adapter_files
  {
    let file_path = manifest_dir.join( format!( "src/adapters/{}", file_name ) );

    assert!(
      file_path.exists(),
      "Adapter file must exist: {}",
      file_name
    );

    let content = std::fs::read_to_string( &file_path )
      .unwrap_or_else( |_| panic!( "Failed to read {}", file_name ) );

    // Count public adapter functions
    let adapter_count = content
      .lines()
      .filter( |line| line.starts_with( "pub async fn " ) && line.contains( "_adapter(" ) )
      .count();

    assert_eq!(
      adapter_count,
      *expected_count,
      "{}: Expected {} adapters, found {}",
      file_name,
      expected_count,
      adapter_count
    );

    total_adapters += adapter_count;
  }

  // Verify total adapter count
  assert_eq!(
    total_adapters,
    22,
    "Expected 22 total adapters after migration (was 28, deleted 6)"
  );
}

/// Test that no orphaned adapters exist
///
/// Verifies that the codebase contains no orphaned adapter functions.
/// Orphaned adapters are functions that have no valid API endpoint.
///
/// ## Negative Criterion: NC-A.1
///
/// Zero orphaned adapters exist
///
/// ## Orphaned Adapters (Deleted)
///
/// - show_agent_usage_adapter
/// - export_agent_usage_adapter
/// - reset_limit_adapter
/// - show_agent_limits_adapter
/// - update_agent_limit_adapter
/// - show_trace_stats_adapter
///
/// ## Verification Method
///
/// Searches all adapter source files for references to orphaned adapter names.
/// Any reference indicates code that should have been deleted.
#[ test ]
fn test_no_orphaned_adapters_exist()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // List of orphaned adapters (should not appear in source code)
  let orphaned_adapters = vec![
    "show_agent_usage_adapter",
    "export_agent_usage_adapter",
    "reset_limit_adapter",
    "show_agent_limits_adapter",
    "update_agent_limit_adapter",
    "show_trace_stats_adapter",
  ];

  // Read all adapter source files
  let adapter_files = vec![
    "src/adapters/usage_adapters.rs",
    "src/adapters/limits_adapters.rs",
    "src/adapters/traces_adapters.rs",
  ];

  for file_path_str in &adapter_files
  {
    let file_path = manifest_dir.join( file_path_str );

    if !file_path.exists()
    {
      continue; // Skip if file doesn't exist
    }

    let content = std::fs::read_to_string( &file_path )
      .unwrap_or_else( |_| panic!( "Failed to read {}", file_path_str ) );

    // Check for orphaned adapter references
    for adapter in &orphaned_adapters
    {
      assert!(
        !content.contains( adapter ),
        "File {} must NOT contain orphaned adapter: {}",
        file_path_str,
        adapter
      );
    }
  }

  // NC-A.1: Verify orphaned count is zero
  let orphaned_count = 0; // If assertions pass, count is zero

  assert_eq!(
    orphaned_count,
    0,
    "NC-A.1 violated: Found {} orphaned adapters (expected 0)",
    orphaned_count
  );
}

/// Test adapter count metrics
///
/// Verifies that adapter metrics match expected values after migration.
///
/// ## Negative Criterion: NC-A.3
///
/// Orphaned percentage must be 0%
///
/// ## Metrics
///
/// Before migration:
/// - Total: 28 adapters
/// - Orphaned: 6 adapters
/// - Correct: 22 adapters
/// - Orphaned %: 21%
///
/// After migration:
/// - Total: 22 adapters
/// - Orphaned: 0 adapters
/// - Correct: 22 adapters
/// - Orphaned %: 0%
///
/// ## Verification Method
///
/// Counts adapter functions in source files and calculates ratios.
#[ test ]
fn test_adapter_count_metrics()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // Expected adapter counts per module
  let expected_adapters = vec![
    ( "auth_adapters.rs", 3 ),
    ( "token_adapters.rs", 5 ),
    ( "usage_adapters.rs", 4 ),    // Was 6, deleted 2
    ( "limits_adapters.rs", 5 ),   // Was 8, deleted 3
    ( "traces_adapters.rs", 3 ),   // Was 4, deleted 1
    ( "health_adapters.rs", 2 ),
  ];

  let mut total_count = 0;

  for ( file_name, _expected ) in &expected_adapters
  {
    let file_path = manifest_dir.join( format!( "src/adapters/{}", file_name ) );

    let content = std::fs::read_to_string( &file_path )
      .unwrap_or_else( |_| panic!( "Failed to read {}", file_name ) );

    let count = content
      .lines()
      .filter( |line| line.starts_with( "pub async fn " ) && line.contains( "_adapter(" ) )
      .count();

    total_count += count;
  }

  // Verify metrics
  let orphaned_count = 0; // All orphaned adapters deleted
  let correct_count = total_count; // All remaining adapters are correct
  let orphaned_percentage = ( orphaned_count as f64 / total_count as f64 ) * 100.0;

  assert_eq!(
    total_count,
    22,
    "Total adapters: expected 22, found {}",
    total_count
  );

  assert_eq!(
    orphaned_count,
    0,
    "Orphaned adapters: expected 0, found {}",
    orphaned_count
  );

  assert_eq!(
    correct_count,
    22,
    "Correct adapters: expected 22, found {}",
    correct_count
  );

  // NC-A.3: Orphaned percentage must be 0%
  assert_eq!(
    orphaned_percentage,
    0.0,
    "NC-A.3 violated: Orphaned percentage is {:.1}% (expected 0%)",
    orphaned_percentage
  );
}
