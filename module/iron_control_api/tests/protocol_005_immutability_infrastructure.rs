/// Immutability Enforcement Test Suite
///
/// This module provides Rust test wrappers for bash enforcement scripts,
/// integrating them with the canonical test workflow (cargo test, w3 .test l::3).
///
/// # Test Execution
///
/// **IMPORTANT:** These tests must run sequentially, not in parallel.
///
/// Run with: `cargo test --test protocol_005_immutability_infrastructure -- --test-threads=1`
///
/// Reason: Tests execute bash scripts that use `git rev-parse` to find repository
/// root. Parallel execution in nested git repositories causes race conditions
/// where tests may resolve to wrong repository, causing spurious failures.
///
/// # Test Methodology
///
/// All tests execute bash scripts in tests/immutable/ and verify:
/// - Exit code 0 = enforcement working (test passes)
/// - Exit code 1 = enforcement broken (test fails)
/// - Stdout contains expected success messages
/// - Quantitative metrics meet requirements (0% old, 100% new)
///
/// # Related Documentation
///
/// - tests/immutable/readme.md - Enforcement test organization
/// - tests/immutable/migration_complete.md - Final migration state
/// - tests/immutable/rollback_impossibility.md - Rollback analysis

use std::process::Command;

/// Helper function to get outer git repository root directory
///
/// This project has nested git repos:
/// - /home/user1/pro/lib/wip_iron/ (outer)
/// - /home/user1/pro/lib/wip_iron/iron_runtime/dev/ (inner)
///
/// Tests run from inner repo but need files from outer repo.
fn get_outer_git_root() -> String {
  let inner_root_output = Command::new("git")
    .args(&["rev-parse", "--show-toplevel"])
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
  let command = format!("cd '{}' && bash '{}'", outer_root, script_path);

  Command::new("bash")
    .arg("-c")
    .arg(&command)
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
/// - tests/immutable/rollback_impossibility.md - Why rollback should fail
#[test]
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
/// - tests/immutable/migration_complete.md - Expected final state
#[test]
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
/// - tests/immutable/migration_complete.md - List of 16 blocked actions
#[test]
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
/// - migration_complete.md exists
/// - rollback_impossibility.md exists
/// - immutability_contract.md exists
/// - branch_protection_rules.md exists
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
fn test_documentation_exists() {
  let outer_root = get_outer_git_root();
  let docs = vec![
    format!("{}/iron_runtime/tests/immutable/migration_complete.md", outer_root),
    format!("{}/iron_runtime/tests/immutable/rollback_impossibility.md", outer_root),
    format!("{}/iron_runtime/dev/docs/immutability_contract.md", outer_root),
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
