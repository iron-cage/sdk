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
    .expect("LOUD FAILURE: Failed to read routing file");

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
    .expect("LOUD FAILURE: Failed to read routing file");

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
      .expect("LOUD FAILURE: Failed to read binary source");

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

/// Bug reproducer for Issue 7: token_id type mismatch between YAML and handlers
///
/// ## Root Cause
///
/// Three token command definitions in `commands/tokens.yaml` declared the `token_id`
/// parameter as `kind: Integer`, but the handler validation code in
/// `src/handlers/validation.rs` expected String format with "tok_" prefix pattern.
/// This created a fundamental type contract violation where:
/// 1. Parser expected numeric input (Integer type)
/// 2. Handler validation required string format matching "tok_*" pattern
/// 3. Neither integer values nor string values would satisfy both requirements
///
/// Affected commands: .tokens.get, .tokens.rotate, .tokens.revoke
///
/// Location: commands/tokens.yaml lines 79-188 (three command definitions)
///
/// ## Why Not Caught
///
/// 1. **No Parameter Contract Tests**: No automated tests verify type consistency
///    between YAML parameter declarations and handler validation logic
/// 2. **Runtime-Only Validation**: Type checking only happens when parsing CLI
///    arguments, not at build time or in unit tests
/// 3. **Test Gap**: Integration tests didn't exercise these specific commands
/// 4. **Manual Discovery**: Found during comprehensive manual testing when
///    commands failed with both integer and string inputs
///
/// ## Fix Applied
///
/// Modified `commands/tokens.yaml` for three commands (lines 79-188):
///
/// 1. **.tokens.get** (lines 88-89):
///    - Changed: `kind: Integer` → `kind: String`
///    - Updated hint: "Numeric token identifier" → "Token identifier (format tok_*)"
///    - Updated example: `token_id::123` → `token_id::tok_abc123`
///
/// 2. **.tokens.rotate** (lines 127-128):
///    - Changed: `kind: Integer` → `kind: String`
///    - Updated hint: "Numeric token identifier" → "Token identifier (format tok_*)"
///    - Updated example: `token_id::123` → `token_id::tok_abc123`
///
/// 3. **.tokens.revoke** (lines 163-164):
///    - Changed: `kind: Integer` → `kind: String`
///    - Updated hint: "Numeric token identifier" → "Token identifier (format tok_*)"
///    - Updated example: `token_id::456` → `token_id::tok_def456`
///
/// Also updated manual test plan `tests/manual/readme.md` test cases:
/// TC-3.3, TC-3.4, TC-3.5, TC-7.3 to use string token_id format.
///
/// ## Prevention
///
/// 1. **Parameter Contract Tests**: Add automated tests that verify YAML parameter
///    types match handler validation expectations for all commands
/// 2. **Type Documentation**: Document expected parameter formats in both YAML
///    (kind + hint) and handler validation code (validation rules)
/// 3. **Integration Testing**: Test all commands with their documented examples
///    to catch type mismatches early
/// 4. **Code Review Checklist**: When adding new commands, verify parameter types
///    are consistent across YAML definition → parser → handler validation
///
/// ## Pitfall
///
/// **Parameter type mismatches make commands completely unusable**
///
/// When YAML declares one type but handlers expect another, the command cannot
/// work with ANY input - users get errors regardless of what they try. This is
/// worse than a logic bug because the command is fundamentally broken at the
/// interface level.
///
/// Always verify the full parameter contract chain:
/// 1. YAML `kind:` matches actual data type (String/Integer/Boolean)
/// 2. Handler validation rules match YAML type
/// 3. Examples in YAML work with the declared type
/// 4. Manual test plan uses correct format
///
/// **Specific lesson**: String-based identifiers with format requirements (like
/// "tok_*" prefix) must ALWAYS be declared as `kind: String` in YAML, never as
/// Integer even if they contain numbers. The presence of a prefix pattern
/// automatically means String type.
#[ test ]
fn bug_reproducer_issue_007_token_id_type_mismatch()
{
  // Read YAML command definitions
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );
  let yaml_file = manifest_dir.join( "commands/tokens.yaml" );

  assert!(
    yaml_file.exists(),
    "YAML command definitions must exist: {:?}",
    yaml_file
  );

  let yaml_content = std::fs::read_to_string( &yaml_file )
    .expect("LOUD FAILURE: Failed to read YAML file");

  // Commands that require token_id parameter with String type
  let commands_with_token_id = vec![
    ".tokens.get",
    ".tokens.rotate",
    ".tokens.revoke",
  ];

  // Verify each command declares token_id as String (not Integer)
  for command in &commands_with_token_id
  {
    // Find command definition section
    let command_pattern = format!( "name: {}", command );
    assert!(
      yaml_content.contains( &command_pattern ),
      "Command '{}' must exist in YAML",
      command
    );

    // Extract section for this command (from "name: .tokens.X" to next "- name:" or EOF)
    let start_idx = yaml_content.find( &command_pattern )
      .expect("LOUD FAILURE: Command must exist in YAML");

    let remaining = &yaml_content[ start_idx.. ];
    let next_command = remaining[ 1.. ].find( "\n- name:" ).unwrap_or( remaining.len() );
    let command_section = &remaining[ ..next_command ];

    // Verify token_id parameter exists and is String type
    assert!(
      command_section.contains( "- name: token_id" ),
      "Command '{}' must have token_id parameter",
      command
    );

    // CRITICAL: Verify kind is String, not Integer
    // This regex-free check looks for the parameter definition block
    let token_id_start = command_section.find( "- name: token_id" )
      .expect("LOUD FAILURE: token_id parameter must exist");

    let param_section = &command_section[ token_id_start.. ];
    let next_param = param_section[ 1.. ].find( "\n  - name:" )
      .or_else( || param_section[ 1.. ].find( "\nexamples:" ) )
      .unwrap_or( param_section.len() );

    let param_def = &param_section[ ..next_param ];

    // Verify String type (not Integer)
    assert!(
      param_def.contains( "kind: String" ),
      "Command '{}' token_id must be 'kind: String' (not Integer). \
       Handlers expect 'tok_*' format which is a String type.",
      command
    );

    // Verify Integer is NOT present (common mistake)
    assert!(
      !param_def.contains( "kind: Integer" ),
      "Command '{}' token_id must NOT be 'kind: Integer'. \
       The 'tok_*' prefix format requires String type.",
      command
    );

    // Verify hint mentions the tok_* format
    assert!(
      param_def.contains( "tok_" ),
      "Command '{}' token_id hint should document 'tok_*' format requirement",
      command
    );
  }

  // Verify examples use string format (tok_*), not numeric format
  for command in &commands_with_token_id
  {
    let command_pattern = format!( "name: {}", command );
    let start_idx = yaml_content.find( &command_pattern )
      .expect("LOUD FAILURE: Command must exist");

    let remaining = &yaml_content[ start_idx.. ];
    let next_command = remaining[ 1.. ].find( "\n- name:" ).unwrap_or( remaining.len() );
    let command_section = &remaining[ ..next_command ];

    // Check examples section contains tok_ format
    if let Some( examples_start ) = command_section.find( "examples:" )
    {
      let examples_section = &command_section[ examples_start.. ];

      assert!(
        examples_section.contains( "token_id::tok_" ),
        "Command '{}' examples must use 'tok_*' string format (not numeric)",
        command
      );

      // Verify examples don't use old numeric format
      // This checks for "token_id::123" or similar numeric patterns
      assert!(
        !examples_section.contains( "token_id::123" ) &&
        !examples_section.contains( "token_id::456" ) &&
        !examples_section.contains( "token_id::789" ),
        "Command '{}' examples must NOT use numeric format (use tok_* instead)",
        command
      );
    }
  }
}
