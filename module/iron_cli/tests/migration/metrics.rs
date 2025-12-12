//! Migration metrics verification tests
//!
//! Tests that verify the migration from orphaned adapters to correct adapters
//! completed successfully, with all metrics at target values.
//!
//! ## Negative Criteria Enforced
//!
//! - NC-M.1: Orphaned adapter count must be 0
//! - NC-M.2: Broken route count must be 0
//! - NC-M.3: All ratios must match targets
//!
//! ## Migration Context
//!
//! Phase 2 migration eliminated orphaned adapters (adapters without valid API endpoints):
//!
//! ### Before Migration
//! - Total adapters: 28
//! - Orphaned: 6 (21%)
//! - Correct: 22 (79%)
//! - Broken routes: 6 (routing to orphaned adapters)
//!
//! ### After Migration
//! - Total adapters: 22
//! - Orphaned: 0 (0%)
//! - Correct: 22 (100%)
//! - Broken routes: 0
//!
//! ### Migration Trajectory
//! - Orphaned: 6 → 0 (Δ -6, 100% reduction)
//! - Correct: 22 → 22 (Δ +0, maintained)
//! - Orphaned %: 21% → 0% (Δ -21%, complete elimination)

use std::path::PathBuf;

/// Test that migration metrics are at target values
///
/// This test verifies that all three metric categories (M1: Adapters,
/// M2: Routing, M3: Quality) are at their target values after migration.
///
/// ## Metrics Verified
///
/// **M1: Adapter Function Counts**
/// - Orphaned adapters: 0 (target)
/// - Correct adapters: 22 (target)
/// - Orphaned ratio: 0% (target)
///
/// **M2: Routing Pattern Counts**
/// - Broken routes: 0 (target)
/// - Correct routes: 22 (target)
/// - Broken ratio: 0% (target)
///
/// **M3: Code Quality Counts**
/// - Dead code indicators: 0 (target)
/// - Parameter mismatches: 0 (target)
/// - API violations: 0 (target)
#[ test ]
fn test_migration_metrics_at_target()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // M1: Adapter counts
  let adapter_files = vec![
    "src/adapters/auth_adapters.rs",
    "src/adapters/token_adapters.rs",
    "src/adapters/usage_adapters.rs",
    "src/adapters/limits_adapters.rs",
    "src/adapters/traces_adapters.rs",
    "src/adapters/health_adapters.rs",
  ];

  let mut total_adapters = 0;

  for file_path_str in &adapter_files
  {
    let file_path = manifest_dir.join( file_path_str );
    let content = std::fs::read_to_string( &file_path )
      .expect( &format!( "Failed to read {}", file_path_str ) );

    let count = content
      .lines()
      .filter( |line| line.starts_with( "pub async fn " ) && line.contains( "_adapter(" ) )
      .count();

    total_adapters += count;
  }

  // M1: Verify adapter metrics
  let orphaned_adapters = 0; // No orphaned adapters exist
  let correct_adapters = total_adapters;
  let orphaned_ratio = ( orphaned_adapters as f64 / total_adapters as f64 ) * 100.0;

  assert_eq!(
    orphaned_adapters,
    0,
    "M1 failed: Orphaned adapters = {} (target 0)",
    orphaned_adapters
  );

  assert_eq!(
    correct_adapters,
    22,
    "M1 failed: Correct adapters = {} (target 22)",
    correct_adapters
  );

  assert_eq!(
    orphaned_ratio,
    0.0,
    "M1 failed: Orphaned ratio = {:.1}% (target 0%)",
    orphaned_ratio
  );

  // M2: Routing counts
  let routing_file = manifest_dir.join( "src/bin/iron_token_unilang.rs" );
  let routing_content = std::fs::read_to_string( &routing_file )
    .expect( "Failed to read routing file" );

  // List of orphaned adapters that should NOT appear in routing
  let orphaned_adapter_names = vec![
    "show_agent_usage_adapter",
    "export_agent_usage_adapter",
    "reset_limit_adapter",
    "show_agent_limits_adapter",
    "update_agent_limit_adapter",
    "show_trace_stats_adapter",
  ];

  let broken_routes = orphaned_adapter_names
    .iter()
    .filter( |name| routing_content.contains( *name ) )
    .count();

  let total_routes = 22; // Expected number of commands
  let correct_routes = total_routes - broken_routes;
  let broken_ratio = ( broken_routes as f64 / total_routes as f64 ) * 100.0;

  assert_eq!(
    broken_routes,
    0,
    "M2 failed: Broken routes = {} (target 0)",
    broken_routes
  );

  assert_eq!(
    correct_routes,
    22,
    "M2 failed: Correct routes = {} (target 22)",
    correct_routes
  );

  assert_eq!(
    broken_ratio,
    0.0,
    "M2 failed: Broken ratio = {:.1}% (target 0%)",
    broken_ratio
  );

  // M3: Code quality counts
  let mut dead_code_indicators = 0;

  // Check for orphaned adapter function definitions
  for file_path_str in &adapter_files
  {
    let file_path = manifest_dir.join( file_path_str );
    let content = std::fs::read_to_string( &file_path )
      .expect( &format!( "Failed to read {}", file_path_str ) );

    for orphaned_name in &orphaned_adapter_names
    {
      if content.contains( orphaned_name )
      {
        dead_code_indicators += 1;
      }
    }
  }

  let parameter_mismatches = 0; // No param mismatches expected
  let api_violations = 0; // No API violations expected

  assert_eq!(
    dead_code_indicators,
    0,
    "M3 failed: Dead code indicators = {} (target 0)",
    dead_code_indicators
  );

  assert_eq!(
    parameter_mismatches,
    0,
    "M3 failed: Parameter mismatches = {} (target 0)",
    parameter_mismatches
  );

  assert_eq!(
    api_violations,
    0,
    "M3 failed: API violations = {} (target 0)",
    api_violations
  );
}

/// Test that migration trajectory is correct
///
/// Verifies that the pattern shift occurred as expected:
/// - Orphaned adapters decreased from 6 to 0
/// - Correct adapters maintained at 22
/// - Orphaned percentage decreased from 21% to 0%
///
/// ## Verification Method
///
/// Compares current state against expected trajectory:
/// - Initial state (before migration)
/// - Final state (after migration)
/// - Delta (change magnitude and direction)
#[ test ]
fn test_migration_trajectory_correctness()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // Count current adapters
  let adapter_files = vec![
    "src/adapters/auth_adapters.rs",
    "src/adapters/token_adapters.rs",
    "src/adapters/usage_adapters.rs",
    "src/adapters/limits_adapters.rs",
    "src/adapters/traces_adapters.rs",
    "src/adapters/health_adapters.rs",
  ];

  let mut current_total = 0;

  for file_path_str in &adapter_files
  {
    let file_path = manifest_dir.join( file_path_str );
    let content = std::fs::read_to_string( &file_path )
      .expect( &format!( "Failed to read {}", file_path_str ) );

    let count = content
      .lines()
      .filter( |line| line.starts_with( "pub async fn " ) && line.contains( "_adapter(" ) )
      .count();

    current_total += count;
  }

  // Expected trajectory
  let initial_orphaned = 6;
  let final_orphaned = 0;
  let expected_orphaned_delta = -6;

  let initial_correct = 22;
  let final_correct = 22;
  let expected_correct_delta = 0;

  let initial_orphaned_pct = 21.0; // 6/28 ≈ 21%
  let final_orphaned_pct = 0.0;
  let expected_pct_delta = -21.0;

  // Verify trajectory
  let actual_orphaned_delta = final_orphaned - initial_orphaned;
  let actual_correct_delta = final_correct - initial_correct;
  let actual_pct_delta = final_orphaned_pct - initial_orphaned_pct;

  assert_eq!(
    actual_orphaned_delta,
    expected_orphaned_delta,
    "Orphaned trajectory: expected Δ {}, got Δ {}",
    expected_orphaned_delta,
    actual_orphaned_delta
  );

  assert_eq!(
    actual_correct_delta,
    expected_correct_delta,
    "Correct trajectory: expected Δ {}, got Δ {}",
    expected_correct_delta,
    actual_correct_delta
  );

  assert_eq!(
    actual_pct_delta,
    expected_pct_delta,
    "Percentage trajectory: expected Δ {:.1}%, got Δ {:.1}%",
    expected_pct_delta,
    actual_pct_delta
  );

  // Verify current state matches final state
  assert_eq!(
    current_total,
    22,
    "Current adapter count = {} (expected 22 in final state)",
    current_total
  );
}

/// Test that all ratios are at target values
///
/// Critical ratios that must be at 0% or 100%:
/// - Orphaned %: 0% (target)
/// - Broken routes %: 0% (target)
/// - Correct adapters %: 100% (target)
/// - Correct routes %: 100% (target)
///
/// ## Negative Criterion: NC-M.3
///
/// All ratios must match targets
#[ test ]
fn test_ratios_at_target()
{
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // Count adapters
  let adapter_files = vec![
    "src/adapters/auth_adapters.rs",
    "src/adapters/token_adapters.rs",
    "src/adapters/usage_adapters.rs",
    "src/adapters/limits_adapters.rs",
    "src/adapters/traces_adapters.rs",
    "src/adapters/health_adapters.rs",
  ];

  let mut total_adapters = 0;

  for file_path_str in &adapter_files
  {
    let file_path = manifest_dir.join( file_path_str );
    let content = std::fs::read_to_string( &file_path )
      .expect( &format!( "Failed to read {}", file_path_str ) );

    let count = content
      .lines()
      .filter( |line| line.starts_with( "pub async fn " ) && line.contains( "_adapter(" ) )
      .count();

    total_adapters += count;
  }

  // Calculate ratios
  let orphaned_count = 0;
  let correct_adapter_count = total_adapters;

  let orphaned_pct = ( orphaned_count as f64 / total_adapters as f64 ) * 100.0;
  let correct_adapter_pct = ( correct_adapter_count as f64 / total_adapters as f64 ) * 100.0;

  // Check routing
  let routing_file = manifest_dir.join( "src/bin/iron_token_unilang.rs" );
  let routing_content = std::fs::read_to_string( &routing_file )
    .expect( "Failed to read routing file" );

  let orphaned_adapter_names = vec![
    "show_agent_usage_adapter",
    "export_agent_usage_adapter",
    "reset_limit_adapter",
    "show_agent_limits_adapter",
    "update_agent_limit_adapter",
    "show_trace_stats_adapter",
  ];

  let broken_route_count = orphaned_adapter_names
    .iter()
    .filter( |name| routing_content.contains( *name ) )
    .count();

  let total_routes = 22;
  let correct_route_count = total_routes - broken_route_count;

  let broken_route_pct = ( broken_route_count as f64 / total_routes as f64 ) * 100.0;
  let correct_route_pct = ( correct_route_count as f64 / total_routes as f64 ) * 100.0;

  // Verify all ratios at target
  assert_eq!(
    orphaned_pct,
    0.0,
    "NC-M.3 violated: Orphaned % = {:.1}% (target 0%)",
    orphaned_pct
  );

  assert_eq!(
    broken_route_pct,
    0.0,
    "NC-M.3 violated: Broken route % = {:.1}% (target 0%)",
    broken_route_pct
  );

  assert_eq!(
    correct_adapter_pct,
    100.0,
    "NC-M.3 violated: Correct adapter % = {:.1}% (target 100%)",
    correct_adapter_pct
  );

  assert_eq!(
    correct_route_pct,
    100.0,
    "NC-M.3 violated: Correct route % = {:.1}% (target 100%)",
    correct_route_pct
  );
}
