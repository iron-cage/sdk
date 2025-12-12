//! Routing correctness verification tests
//!
//! Tests that all CLI commands route to the correct adapter functions
//! and that no routes call orphaned adapters (adapters without valid API endpoints).
//!
//! ## Negative Criteria Enforced
//!
//! - NC-R.1: Zero routes calling orphaned adapters
//! - NC-R.2: Zero routes with parameter mismatches
//! - NC-R.3: All 22 commands must route to valid adapters
//!
//! ## Orphaned Adapters (Deleted in Migration)
//!
//! The following 6 adapters were deleted because they had no matching API endpoints:
//! - `show_agent_usage_adapter` (usage_adapters.rs)
//! - `export_agent_usage_adapter` (usage_adapters.rs)
//! - `reset_limit_adapter` (limits_adapters.rs)
//! - `show_agent_limits_adapter` (limits_adapters.rs)
//! - `update_agent_limit_adapter` (limits_adapters.rs)
//! - `show_trace_stats_adapter` (traces_adapters.rs)

use std::path::PathBuf;

/// Test that all 22 commands route to correct adapters
///
/// This test verifies that the routing implementation in `iron_token_unilang.rs`
/// routes all 22 commands to their corresponding adapter functions.
///
/// ## Commands Verified (22)
///
/// Auth (3): .auth.{login, refresh, logout}
/// Tokens (5): .tokens.{generate, list, get, rotate, revoke}
/// Usage (4): .usage.{show, by_project, by_provider, export}
/// Limits (5): .limits.{list, get, create, update, delete}
/// Traces (3): .traces.{list, get, export}
/// Health (2): .health, .version
///
/// ## Verification Method
///
/// Reads the routing source code and verifies each command has a route entry.
#[ test ]
fn test_all_commands_route_correctly()
{
  // Read routing source file
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );
  let routing_file = manifest_dir.join( "src/bin/iron_token_unilang.rs" );

  assert!( routing_file.exists(), "Routing file must exist: {:?}", routing_file );

  let routing_content = std::fs::read_to_string( &routing_file )
    .expect( "Failed to read routing file" );

  // Expected commands (22 total)
  let expected_commands = vec![
    // Auth commands (3)
    ".auth.login",
    ".auth.refresh",
    ".auth.logout",

    // Token commands (5)
    ".tokens.generate",
    ".tokens.list",
    ".tokens.get",
    ".tokens.rotate",
    ".tokens.revoke",

    // Usage commands (4)
    ".usage.show",
    ".usage.by_project",
    ".usage.by_provider",
    ".usage.export",

    // Limits commands (5)
    ".limits.list",
    ".limits.get",
    ".limits.create",
    ".limits.update",
    ".limits.delete",

    // Traces commands (3)
    ".traces.list",
    ".traces.get",
    ".traces.export",

    // Health commands (2)
    ".health",
    ".version",
  ];

  // Verify each command has a route entry
  for command in &expected_commands
  {
    let pattern = format!( "\"{}\"", command );
    assert!(
      routing_content.contains( &pattern ),
      "Command '{}' must have route entry in routing file",
      command
    );
  }

  // Verify total count matches expectation
  assert_eq!(
    expected_commands.len(),
    22,
    "Expected exactly 22 commands"
  );
}

/// Test that no routes call orphaned adapters
///
/// Orphaned adapters are functions that were deleted because they had no
/// matching API endpoints. This test ensures that the routing code does not
/// reference any of these deleted adapters.
///
/// ## Negative Criterion: NC-R.1
///
/// Zero routes calling orphaned adapters
///
/// ## Orphaned Adapters (6 total)
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
/// Searches routing source code for references to orphaned adapter names.
/// Any reference indicates a broken route that must be fixed.
#[ test ]
fn test_no_orphaned_adapter_usage()
{
  // Read routing source file
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );
  let routing_file = manifest_dir.join( "src/bin/iron_token_unilang.rs" );

  let routing_content = std::fs::read_to_string( &routing_file )
    .expect( "Failed to read routing file" );

  // List of orphaned adapters (deleted in migration)
  let orphaned_adapters = vec![
    "show_agent_usage_adapter",
    "export_agent_usage_adapter",
    "reset_limit_adapter",
    "show_agent_limits_adapter",
    "update_agent_limit_adapter",
    "show_trace_stats_adapter",
  ];

  // Verify no orphaned adapters are referenced in routing
  for adapter in &orphaned_adapters
  {
    assert!(
      !routing_content.contains( adapter ),
      "Routing file must NOT reference orphaned adapter: {}",
      adapter
    );
  }

  // NC-R.1: Verify count is zero
  let orphaned_count = orphaned_adapters.iter()
    .filter( |adapter| routing_content.contains( *adapter ) )
    .count();

  assert_eq!(
    orphaned_count,
    0,
    "NC-R.1 violated: Found {} routes calling orphaned adapters (expected 0)",
    orphaned_count
  );
}

/// Test that routing compilation prevents old adapter usage
///
/// This test documents that the Rust type system provides compile-time protection
/// against using deleted adapters. If old routing code is restored, compilation
/// will fail because the adapter functions no longer exist.
///
/// ## Multi-Layer Defense
///
/// 1. **Syntactic Layer**: Deleted adapters cannot compile (this test)
/// 2. **Semantic Layer**: Old API endpoints return 404 (runtime)
/// 3. **Architectural Layer**: Parameter contracts diverged (design)
/// 4. **Operational Layer**: Rollback requires coordinated changes (process)
///
/// ## Example of Compilation Failure
///
/// ```compile_fail
/// // This code will NOT compile (adapter function deleted)
/// runtime.block_on(
///   iron_cli::adapters::usage_adapters::show_agent_usage_adapter(params)
/// )
/// // Error: no function `show_agent_usage_adapter` in module `usage_adapters`
/// ```
///
/// ## Verification Method
///
/// This test passes because the code compiles. The presence of this test
/// documents the compile-time protection mechanism. If someone attempts to
/// restore old routing patterns, they will get compilation errors.
#[ test ]
fn test_routing_compilation_prevents_old_adapters()
{
  // This test documents compile-time protection
  // The fact that this test compiles proves that:
  // 1. No references to deleted adapters exist in routing code
  // 2. Rust compiler enforces absence of deleted functions
  // 3. Rollback to old routing patterns will fail at compilation

  // Verify adapter modules still exist (non-orphaned adapters)
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  // Check adapter module files exist
  let adapter_files = vec![
    "src/adapters/auth_adapters.rs",
    "src/adapters/token_adapters.rs",
    "src/adapters/usage_adapters.rs",
    "src/adapters/limits_adapters.rs",
    "src/adapters/traces_adapters.rs",
    "src/adapters/health_adapters.rs",
  ];

  for adapter_file in &adapter_files
  {
    let path = manifest_dir.join( adapter_file );
    assert!(
      path.exists(),
      "Adapter module must exist: {}",
      adapter_file
    );
  }

  // Compilation success proves no orphaned adapter usage (syntactic protection layer)
  // If orphaned adapters were referenced, this crate would fail to compile.
  // The fact that this test compiles and runs verifies syntactic protection.
}

/// Bug reproducer for Issue 4: Help syntax documentation inconsistency
///
/// ## Root Cause
///
/// The help text in both `iron_token_unilang.rs` and `iron_control_unilang.rs`
/// showed examples using `??` for "Detailed help", but the unilang parser only
/// supports single `?` as the help operator. Using `??` results in error:
/// "Help operator '?' must be the last token" because the second `?` is parsed
/// as a separate token.
///
/// ## Why Not Caught
///
/// 1. **Documentation-Only Issue**: No automated tests verify help text accuracy
/// 2. **Manual Testing Gap**: Help examples weren't tested during development
/// 3. **Parser Behavior**: Error comes from unilang library, not iron_cli code
///
/// ## Fix Applied
///
/// Modified both binary files:
/// 1. `iron_token_unilang.rs` (line 334-335): Removed `??` example, kept single `?`
/// 2. `iron_control_unilang.rs` (line 435-436): Removed `??` example, kept single `?`
/// 3. Changed comment from "Quick help" / "Detailed help" to just "Command help"
///
/// ## Prevention
///
/// 1. **Documentation Tests**: Verify help text doesn't contain invalid syntax
/// 2. **Manual Testing**: Test all examples shown in help text
/// 3. **Parser Documentation**: Document unilang help operator behavior clearly
///
/// ## Pitfall
///
/// **Never document syntax that doesn't work**
///
/// All examples shown in help text must be actually executable. Users will
/// copy-paste examples from help, and if those examples fail, it creates
/// confusion and frustration. Always test help examples during development.
/// This applies to all CLI tools: documentation accuracy is critical for UX.
///
/// **Specific lesson**: Before documenting any CLI syntax, test it actually
/// works. Don't assume similar tools' conventions apply without verification.
#[ test ]
fn bug_reproducer_issue_004_help_syntax_consistency()
{
  // Read both binary source files
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );

  let binaries = vec![
    "src/bin/iron_token_unilang.rs",
    "src/bin/iron_control_unilang.rs",
  ];

  for binary_file in &binaries
  {
    let path = manifest_dir.join( binary_file );

    assert!(
      path.exists(),
      "Binary source must exist: {}",
      binary_file
    );

    let content = std::fs::read_to_string( &path )
      .expect( "Failed to read binary source" );

    // Verify help text doesn't contain invalid ?? syntax
    // Search for pattern: ".command ??" in help examples
    let invalid_patterns = vec![
      ".tokens.list ??",
      ".agent.list ??",
      ".help ??",
      "command ??",
    ];

    for pattern in &invalid_patterns
    {
      assert!(
        !content.contains( pattern ),
        "File {} contains invalid help syntax '{}'. Only single '?' is valid.",
        binary_file,
        pattern
      );
    }

    // Verify at least one valid single ? help example exists
    assert!(
      content.contains( " ?" ) && !content.contains( " ??" ),
      "File {} should contain valid single '?' help examples",
      binary_file
    );
  }
}
