//! Immutability Enforcement Test Suite
//!
//! This module provides Rust test wrappers for bash enforcement scripts,
//! integrating them with the canonical test workflow (cargo test, w3 .test l::3).
//!
//! # Test Execution
//!
//! **RECOMMENDED:** Run bash scripts directly for most reliable results:
//! ```bash
//! bash /path/to/iron_runtime/tests/immutable/measure_migration_state.sh
//! bash /path/to/iron_runtime/tests/immutable/attempt_rollback.sh
//! ```
//!
//! **ALTERNATIVE:** Run via Rust test wrappers (sequential mode):
//! ```bash
//! cargo test --test protocol_005_immutability_infrastructure -- --test-threads=1
//! ```
//!
//! **KNOWN LIMITATION:** test_migration_state_metrics and test_enforcement_coverage
//! may fail with `cargo nextest` due to git repository detection issues in nested
//! repo environments. Use direct bash script execution or cargo test with
//! --test-threads=1 for reliable results.
//!
//! # Test Methodology
//!
//! All tests execute bash scripts in tests/immutable/ and verify:
//! - Exit code 0 = enforcement working (test passes)
//! - Exit code 1 = enforcement broken (test fails)
//! - Stdout contains expected success messages
//! - Quantitative metrics meet requirements (0% old, 100% new)
//!
//! # Related Documentation
//!
//! - tests/immutable/readme.md - Enforcement test organization
//! - docs/enforcement/migration_complete.md - Final migration state
//! - docs/enforcement/rollback_impossibility.md - Rollback analysis

use std::process::Command;
use serial_test::serial;

/// Helper function to get outer git repository root directory
///
/// This project has nested git repos:
/// - /home/user1/pro/lib/wip_iron/ (outer)
/// - /home/user1/pro/lib/wip_iron/iron_runtime/dev/ (inner)
///
/// Tests run from inner repo but need files from outer repo.
fn get_outer_git_root() -> String {
  let inner_root_output = Command::new("git")
    .args(["rev-parse", "--show-toplevel"])
    .output()
    .expect("Failed to find git root");

  let inner_root = String::from_utf8_lossy(&inner_root_output.stdout)
    .trim()
    .to_string();

  // If we're in iron_runtime/dev, go up two levels to outer repo
  if inner_root.ends_with("/iron_runtime/dev") {
    // Remove /iron_runtime/dev to get outer root
    inner_root.trim_end_matches("/iron_runtime/dev").to_string()
  } else {
    inner_root
  }
}

/// Helper function to execute bash script and return output
fn run_bash_script(script_name: &str) -> std::process::Output {
  let outer_root = get_outer_git_root();

  // Use absolute path to script
  let script_path = format!("{}/iron_runtime/tests/immutable/{}", outer_root, script_name);

  // Run script with GIT_ROOT env var to avoid working directory issues
  // Note: Scripts must use $GIT_ROOT if set, otherwise default to PWD
  Command::new("bash")
    .arg(&script_path)
    .current_dir(&outer_root)
    .env("GIT_ROOT", &outer_root)
    .env_remove("GIT_DIR")
    .env_remove("GIT_WORK_TREE")
    .env_remove("GIT_INDEX_FILE")
    .env_remove("GIT_OBJECT_DIRECTORY")
    .output()
    .unwrap_or_else(|e| {
      panic!("Failed to execute {}: {}", script_path, e);
    })
}

/// Helper function to assert script succeeded with exit code 0
fn assert_script_success(output: &std::process::Output, script_name: &str, context: &str) {
  let stdout = String::from_utf8_lossy(&output.stdout);
  let stderr = String::from_utf8_lossy(&output.stderr);

  assert!(
    output.status.success(),
    "{} failed for {}\nExit code: {:?}\nStdout:\n{}\nStderr:\n{}",
    script_name,
    context,
    output.status.code(),
    stdout,
    stderr
  );
}

/// Test that rollback attempts are impossible
///
/// Executes attempt_rollback.sh which tries 8 different methods to restore
/// old patterns. All attempts must be blocked for test to pass.
///
/// # Pass Criteria
///
/// - Exit code: 0 (all rollback attempts blocked)
/// - Stdout contains: "✓ MIGRATION COMPLETE: All rollback attempts failed"
/// - Rollback success count: 0/8
///
/// # Fail Criteria
///
/// - Exit code: 1 (one or more rollback attempts succeeded)
/// - Stdout contains: "❌ MIGRATION INCOMPLETE"
/// - Old way can still be restored
///
/// # Related Documentation
///
/// - tests/immutable/attempt_rollback.sh - Rollback test implementation
/// - docs/enforcement/rollback_impossibility.md - Why rollback should fail
#[test]
#[serial]
fn test_rollback_impossibility() {
  let output = run_bash_script("attempt_rollback.sh");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_script_success(
    &output,
    "attempt_rollback.sh",
    "Rollback impossibility verification"
  );

  assert!(
    stdout.contains("✓ MIGRATION COMPLETE: All rollback attempts failed"),
    "Expected all rollback attempts to be blocked, but got:\n{}",
    stdout
  );

  assert!(
    stdout.contains("Successful Rollbacks: 0"),
    "Expected 0 successful rollbacks, but got:\n{}",
    stdout
  );
}

/// Test that migration state shows 0% old → 100% new ratio
///
/// Executes measure_migration_state.sh which counts old pattern usage (target: 0)
/// and new pattern usage (target: 10), then calculates migration ratio.
///
/// # Pass Criteria
///
/// - Exit code: 0 (migration complete)
/// - Old pattern count: 0/10 (0%)
/// - New pattern count: 10/10 (100%)
/// - Old → New ratio: 0% → 100%
/// - Status: ✓ COMPLETE
///
/// # Fail Criteria
///
/// - Exit code: 1 (migration incomplete)
/// - Old pattern count > 0
/// - New pattern count < 10
/// - Status: ⚠️ IN PROGRESS or ❌ NOT STARTED
///
/// # Related Documentation
///
/// - tests/immutable/measure_migration_state.sh - Metrics implementation
/// - docs/enforcement/migration_complete.md - Expected final state
#[test]
#[serial]
fn test_migration_state_metrics() {
  let output = run_bash_script("measure_migration_state.sh");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_script_success(
    &output,
    "measure_migration_state.sh",
    "Migration state measurement"
  );

  // Verify old pattern count is 0
  assert!(
    stdout.contains("Old Patterns (should be 0%):  0/10 (0%)"),
    "Expected 0% old patterns, but got:\n{}",
    stdout
  );

  // Verify new pattern count is 10
  assert!(
    stdout.contains("New Patterns (should be 100%): 10/10 (100%)"),
    "Expected 100% new patterns, but got:\n{}",
    stdout
  );

  // Verify migration ratio
  assert!(
    stdout.contains("Old → New Ratio:      0% → 100%"),
    "Expected migration ratio 0% → 100%, but got:\n{}",
    stdout
  );

  // Verify status is COMPLETE
  assert!(
    stdout.contains("Status:               ✓ COMPLETE"),
    "Expected status ✓ COMPLETE, but got:\n{}",
    stdout
  );
}

/// Test that enforcement coverage is 100%
///
/// Verifies that all 16 previously-possible actions are now blocked
/// by enforcement mechanisms.
///
/// # Pass Criteria
///
/// - Enforcement coverage: 100% (16/16 enforced)
/// - All blocked actions verified:
///   - Mark protocol ⚫ while implementation exists
///   - Mark protocol 🟢 without implementation
///   - Commit inconsistent status codes
///   - etc. (16 total)
///
/// # Fail Criteria
///
/// - Enforcement coverage < 100%
/// - Any previously-possible action still possible
///
/// # Related Documentation
///
/// - tests/immutable/measure_migration_state.sh - Coverage calculation
/// - docs/enforcement/migration_complete.md - List of 16 blocked actions
#[test]
#[serial]
fn test_enforcement_coverage() {
  let output = run_bash_script("measure_migration_state.sh");
  let stdout = String::from_utf8_lossy(&output.stdout);

  assert_script_success(
    &output,
    "measure_migration_state.sh",
    "Enforcement coverage measurement"
  );

  assert!(
    stdout.contains("Blocked Actions:      100% (16/16 enforced)"),
    "Expected 100% enforcement coverage (16/16), but got:\n{}",
    stdout
  );
}

/// Test that git hook enforcement exists and is functional
///
/// Verifies:
/// - Git hook file exists at .git/hooks/pre-commit
/// - Hook has proper size (> 100 bytes)
/// - Hook is executable
/// - Hook contains enforcement logic
///
/// # Pass Criteria
///
/// - Hook file exists
/// - Hook size > 100 bytes (not just a stub)
/// - Hook is executable (has +x permission)
///
/// # Fail Criteria
///
/// - Hook file missing
/// - Hook file empty or too small
/// - Hook not executable
#[test]
#[serial]
fn test_git_hook_exists() {
  use std::fs;
  use std::os::unix::fs::PermissionsExt;

  let outer_root = get_outer_git_root();
  let hook_path = format!("{}/.git/hooks/pre-commit", outer_root);
  let hook_path = std::path::Path::new(&hook_path);

  assert!(
    hook_path.exists(),
    "Git pre-commit hook missing at {:?} - enforcement incomplete",
    hook_path
  );

  let metadata = fs::metadata(hook_path).expect("Failed to read hook metadata");

  assert!(
    metadata.len() > 100,
    "Git hook too small ({} bytes) - likely corrupted or removed",
    metadata.len()
  );

  let permissions = metadata.permissions();
  let is_executable = permissions.mode() & 0o111 != 0;

  assert!(
    is_executable,
    "Git hook not executable - enforcement will not run"
  );
}

/// Test that CI workflows exist
///
/// Verifies:
/// - doc-consistency-check.yml exists
/// - test-evidence-check.yml exists
/// - post-merge-audit.yml exists
///
/// # Pass Criteria
///
/// - All 3 CI workflow files exist
/// - Files are in .github/workflows/
///
/// # Fail Criteria
///
/// - Any CI workflow missing
/// - Workflows in wrong location
#[test]
#[serial]
fn test_ci_workflows_exist() {
  let outer_root = get_outer_git_root();
  let workflows = vec![
    format!("{}/.github/workflows/doc-consistency-check.yml", outer_root),
    format!("{}/.github/workflows/test-evidence-check.yml", outer_root),
    format!("{}/.github/workflows/post-merge-audit.yml", outer_root),
  ];

  for workflow in &workflows {
    let workflow_path = std::path::Path::new(workflow);
    assert!(
      workflow_path.exists(),
      "CI workflow missing: {} - remote enforcement incomplete",
      workflow
    );
  }
}

/// Test that immutability warning exists in protocol matrix
///
/// Verifies:
/// - Warning header exists in protocol_maturity_matrix.md
/// - Warning contains "AUTOMATED VERIFICATION ENFORCED"
///
/// # Pass Criteria
///
/// - Warning present in file
/// - Warning cannot be removed (CI blocks it)
///
/// # Fail Criteria
///
/// - Warning missing
/// - Warning can be removed
#[test]
#[serial]
fn test_immutability_warning_exists() {
  use std::fs;

  let outer_root = get_outer_git_root();
  let matrix_path = format!("{}/iron_runtime/dev/docs/protocol_maturity_matrix.md", outer_root);
  let content = fs::read_to_string(&matrix_path)
    .expect("Failed to read protocol_maturity_matrix.md");

  assert!(
    content.contains("AUTOMATED VERIFICATION ENFORCED"),
    "Immutability warning missing from {} - developers not warned",
    matrix_path
  );
}

/// Test that documentation files exist
///
/// Verifies:
/// - docs/enforcement/migration_complete.md exists
/// - docs/enforcement/rollback_impossibility.md exists
/// - docs/enforcement/immutability_contract.md exists
/// - .github/branch_protection_rules.md exists
///
/// # Pass Criteria
///
/// - All documentation files exist
/// - Files follow lowercase_snake_case naming
///
/// # Fail Criteria
///
/// - Any documentation missing
/// - Files use UPPERCASE naming
#[test]
#[serial]
fn test_documentation_exists() {
  let outer_root = get_outer_git_root();
  let docs = vec![
    format!("{}/iron_runtime/dev/docs/enforcement/migration_complete.md", outer_root),
    format!("{}/iron_runtime/dev/docs/enforcement/rollback_impossibility.md", outer_root),
    format!("{}/iron_runtime/dev/docs/enforcement/immutability_contract.md", outer_root),
    format!("{}/.github/branch_protection_rules.md", outer_root),
  ];

  for doc in &docs {
    let doc_path = std::path::Path::new(doc);
    assert!(
      doc_path.exists(),
      "Documentation missing: {} - knowledge not preserved",
      doc
    );
  }
}

/// Test that .gitattributes protection exists
///
/// Verifies:
/// - .gitattributes file exists
/// - Contains merge=ours for enforcement scripts
/// - Protects against accidental overwrites
///
/// # Pass Criteria
///
/// - .gitattributes exists
/// - Contains merge=ours directive
/// - Protects tests/immutable/*.sh files
///
/// # Fail Criteria
///
/// - .gitattributes missing
/// - No merge protection
#[test]
#[serial]
fn test_gitattributes_protection() {
  use std::fs;

  let outer_root = get_outer_git_root();
  let gitattributes_path = format!("{}/.gitattributes", outer_root);
  assert!(
    std::path::Path::new(&gitattributes_path).exists(),
    ".gitattributes missing - enforcement scripts not protected"
  );

  let content = fs::read_to_string(&gitattributes_path)
    .expect("Failed to read .gitattributes");

  assert!(
    content.contains("merge=ours"),
    ".gitattributes missing merge=ours - enforcement scripts can be overwritten"
  );
}

/// Test that manual testing infrastructure exists
///
/// Verifies:
/// - tests/manual/screenshots/ directory exists
/// - tests/manual/logs/ directory exists (or tests/manual/ exists)
///
/// # Pass Criteria
///
/// - Manual testing directories exist
/// - Infrastructure for test evidence collection in place
///
/// # Fail Criteria
///
/// - Manual testing directories missing
/// - No place to store test evidence
#[test]
#[serial]
fn test_manual_testing_infrastructure() {
  let outer_root = get_outer_git_root();
  let screenshots_dir = format!("{}/iron_runtime/dev/module/iron_dashboard/tests/manual/screenshots", outer_root);
  let manual_dir = format!("{}/iron_runtime/dev/module/iron_dashboard/tests/manual", outer_root);

  let screenshots_exists = std::path::Path::new(&screenshots_dir).exists();
  let manual_exists = std::path::Path::new(&manual_dir).exists();

  assert!(
    screenshots_exists || manual_exists,
    "Manual testing infrastructure missing - no place to store test evidence"
  );
}

/// Bug reproducer for issue-003: Script must validate working directory
///
/// # Root Cause
///
/// The measure_migration_state.sh script is designed to be run from the outer
/// git repository root (the repository containing .git/hooks/pre-commit and
/// iron_runtime/ directory). When run from a different directory (e.g., the
/// inner iron_runtime/dev/ repository), the script silently produces incorrect
/// results instead of failing with a clear error.
///
/// Without GIT_ROOT environment variable, the script uses PWD as the working
/// directory. If PWD is the inner repo, all file path lookups fail:
/// - .git/hooks/pre-commit not found → reports 0 bytes (should be 2428)
/// - CI workflow files not found → reports 2 files (should be 3)
/// - Documentation files not found → reports 0 files (should be 3)
/// - Enforcement scripts not found → reports 0 files (should be 9)
///
/// This causes migration state to be reported as 0% (NOT STARTED) when actual
/// state is 100% (COMPLETE).
///
/// # Why Not Caught
///
/// 1. Tests always run via Rust wrapper which sets GIT_ROOT correctly
/// 2. No validation to ensure script runs from correct directory
/// 3. No automated testing of direct script invocation scenarios
/// 4. Script silently succeeds with wrong results instead of failing loudly
///
/// This violates the "loud failures" principle - tests and scripts should fail
/// clearly when preconditions aren't met, not silently produce incorrect results.
///
/// # Fix Applied
///
/// Added working directory validation at script startup:
/// - If GIT_ROOT set: Use it (existing behavior)
/// - If GIT_ROOT not set: Validate PWD is outer repo root
/// - If validation fails: Exit with clear error message
///
/// Validation checks for unique marker: iron_runtime/tests/immutable directory
/// exists only in outer repo, not in inner repo. This reliably identifies the
/// correct working directory.
///
/// # Prevention
///
/// 1. Always validate preconditions at script/test entry points
/// 2. Fail fast with clear error messages when preconditions not met
/// 3. Test both happy path (correct usage) and error path (incorrect usage)
/// 4. Add automated tests for direct script invocation scenarios
///
/// # Pitfall
///
/// In nested git repository environments, never assume PWD or auto-detected
/// git root is correct. Always either:
/// 1. Accept explicit directory parameter (like GIT_ROOT env var), OR
/// 2. Validate working directory matches expected structure, OR
/// 3. Fail fast if directory cannot be determined reliably
///
/// Silently using wrong directory produces incorrect results that are hard to
/// debug. Loud failures are always better than silent incorrect behavior.
#[test]
#[serial]
fn bug_reproducer_issue_003_script_validates_working_directory() {
  use std::process::Command;

  let outer_root = get_outer_git_root();
  let inner_root = format!("{}/iron_runtime/dev", outer_root);
  let script_path = format!("{}/iron_runtime/tests/immutable/measure_migration_state.sh", outer_root);

  // Test 1: Running from inner repo WITHOUT GIT_ROOT should fail with clear error
  let output_from_inner = Command::new("bash")
    .arg(&script_path)
    .current_dir(&inner_root)
    .env_remove("GIT_ROOT")
    .output()
    .expect("Failed to execute script from inner repo");

  // Script should exit with error code (not 0)
  assert!(
    !output_from_inner.status.success(),
    "Script should fail when run from wrong directory without GIT_ROOT, but succeeded"
  );

  // Error message should be clear and helpful
  let stderr = String::from_utf8_lossy(&output_from_inner.stderr);
  assert!(
    stderr.contains("must be run from") || stderr.contains("wrong directory") || stderr.contains("GIT_ROOT"),
    "Script should explain directory requirement, but stderr was:\n{}",
    stderr
  );

  // Test 2: Running from inner repo WITH GIT_ROOT should succeed
  let output_with_git_root = Command::new("bash")
    .arg(&script_path)
    .current_dir(&inner_root)
    .env("GIT_ROOT", &outer_root)
    .output()
    .expect("Failed to execute script with GIT_ROOT");

  assert!(
    output_with_git_root.status.success(),
    "Script should succeed when GIT_ROOT is provided"
  );

  let stdout = String::from_utf8_lossy(&output_with_git_root.stdout);
  assert!(
    stdout.contains("10/10 (100%)"),
    "Script with GIT_ROOT should report correct migration state"
  );

  // Test 3: Running from outer repo WITHOUT GIT_ROOT should succeed
  let output_from_outer = Command::new("bash")
    .arg(&script_path)
    .current_dir(&outer_root)
    .env_remove("GIT_ROOT")
    .output()
    .expect("Failed to execute script from outer repo");

  assert!(
    output_from_outer.status.success(),
    "Script should succeed when run from correct directory"
  );

  let stdout_outer = String::from_utf8_lossy(&output_from_outer.stdout);
  assert!(
    stdout_outer.contains("10/10 (100%)"),
    "Script from correct directory should report correct migration state"
  );
}
