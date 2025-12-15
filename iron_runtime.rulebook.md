# iron_runtime

Repository-specific rulebook for Iron Runtime project capturing lessons learned from systematic compliance audits and preventing repeat mistakes.

### Vocabulary

- **Handler-Validated Parameter Pattern**: Architecture where CLI handlers validate parameter existence and format before adapters extract values with `.unwrap()`. Creates architectural contract: if handler succeeds, parameters guaranteed valid.
- **Test Infrastructure Protection**: Use of `#[cfg(test)]` with `compile_error!` guards to ensure test-only code never compiles in production.
- **Responsibility Table**: Two-column markdown table (`| File | Responsibility |`) documenting every file in a directory with single-sentence (3-10 words) responsibility statement.
- **One-Second Test**: Quick scan of directory readme.md to detect overlapping file responsibilities before creating new files.
- **File Creation Protocol**: Five-step workflow before creating any file: (1) verify directory, (2) open readme.md, (3) one-second test, (4) add row, (5) create file.

### Governing Principles

This rulebook establishes Iron Runtime-specific standards based on organizational governance principles and lessons learned from systematic compliance audits:

1. **Anti-Duplication Principle**: Every piece of knowledge exists in exactly ONE location. Test docs for bug understanding, code comments for implementation decisions, module docs for cross-file concerns, specification for requirements (see organizational_principles.rulebook.md).

2. **Unique Responsibility Principle**: Every file answers exactly ONE question. Files with same responsibility in same directory must be consolidated (see organizational_principles.rulebook.md).

3. **Hierarchical Separation**: Two-level enforcement (directory + file) guarantees non-overlap. Different directories = guaranteed no responsibility overlap (see organizational_principles.rulebook.md).

4. **File Creation Protocol Compliance**: ALL subdirectories containing .rs files must have readme.md with Responsibility Table documenting every file. No exceptions (learned from Phase 5 audit).

5. **No Mocking Policy**: Use real implementations in tests. Real databases via iron_test_db, real CLI execution via process spawn, real HTTP servers on random ports (established pattern across test infrastructure).

### Scope

**Responsibilities:**
Defines Iron Runtime project-specific development standards, common mistake patterns, and enforcement methods based on systematic compliance audits (Phases 1-5). Covers file organization enforcement, test quality standards, CLI architecture patterns, unwrap safety guidelines, and verification commands. Complements global rulebooks with project-specific lessons learned.

**In Scope:**
- File Creation Protocol enforcement and verification methods
- Test quality standards (assert messages, documentation, no mocking)
- CLI architecture patterns (handler-validated parameters, adapter design)
- Unwrap safety guidelines and classification criteria
- Project-specific verification commands and compliance checking
- Common mistake patterns identified in audits
- Repository structure and organization standards

**Out of Scope:**
- Universal organizational governance (see organizational_principles.rulebook.md)
- Rulebook creation meta-standards (see knowledge.rulebook.md)
- Language-agnostic code style rules (see code_style.rulebook.md)
- Generic test organization principles (see test_organization.rulebook.md)
- General file structure conventions (see files_structure.rulebook.md)

**Applicability:**
Rust-specific with project-specific architectural patterns. Applies to all Iron Runtime crates (iron_cli, iron_control_api, iron_token_manager, iron_types, iron_secrets, iron_runtime). Standards derived from actual audit findings, not theoretical requirements. Mandatory for all new development and recommended for existing code improvements.

### Quick Reference Summary

| Group | Rule | Description |
|-------|------|-------------|
| **File Creation Protocol** | [Mandatory Readme Files](#file-creation-protocol--mandatory-readme-files) | All subdirectories with .rs files must have readme.md |
| | [Five-Step Workflow](#file-creation-protocol--five-step-workflow) | Follow five steps before creating any file |
| | [Responsibility Table Format](#file-creation-protocol--responsibility-table-format) | Two-column table with single-sentence responsibilities |
| | [Automated Verification](#file-creation-protocol--automated-verification) | Use verification script to check compliance |
| **Test Quality Standards** | [Assert Message Coverage](#test-quality-standards--assert-message-coverage) | Target 95%+ assert message coverage for debuggability |
| | [No Mocking Policy](#test-quality-standards--no-mocking-policy) | Use real implementations, never mocks or fakes |
| | [Loud Failure Requirement](#test-quality-standards--loud-failure-requirement) | Tests must fail loudly and clearly, never silently |
| | [Test Documentation](#test-quality-standards--test-documentation) | Integration tests in tests/ directory with proper doc comments |
| **CLI Architecture** | [Handler-Validated Parameters](#cli-architecture--handler-validated-parameters) | Handlers validate before adapters extract |
| | [Adapter Unwrap Safety](#cli-architecture--adapter-unwrap-safety) | Unwraps safe when handler validates parameters |
| | [Infallible Operation Detection](#cli-architecture--infallible-operation-detection) | Recognize truly infallible operations |
| **Unwrap Guidelines** | [Classification Criteria](#unwrap-guidelines--classification-criteria) | Test Infrastructure, Infallible, Documentation, Handler-Validated |
| | [High-Risk Identification](#unwrap-guidelines--high-risk-identification) | User input without validation is high-risk |
| | [Expect Preference](#unwrap-guidelines--expect-preference) | Prefer .expect() with diagnostic messages over .unwrap() |
| **Common Mistakes** | [Missing Readme Files](#common-mistakes--missing-readme-files) | Subdirectories missing readme.md files |
| | [Incomplete Responsibility Tables](#common-mistakes--incomplete-responsibility-tables) | Readme exists but doesn't document all files |
| | [Generic Filenames](#common-mistakes--generic-filenames) | Never use utils.rs, helpers.rs, common.rs, misc.rs |
| | [Backup File Creation](#common-mistakes--backup-file-creation) | Never create _backup, _old, _v1, _legacy files |
| **Verification Methods** | [File Creation Compliance](#verification-methods--file-creation-compliance) | Run /tmp/check_subdir_readme.sh for directory audit |
| | [Test Level Commands](#verification-methods--test-level-commands) | Use ctest1-5 or w3 .test level::N for verification |
| | [Full Compliance Check](#verification-methods--full-compliance-check) | Level 3 minimum for pre-commit verification |

### File Creation Protocol : Mandatory Readme Files

All subdirectories containing one or more .rs files must have a `readme.md` file with a complete Responsibility Table documenting every .rs file in that directory.

**Rationale:** Prevents file responsibility overlap within directories. Enables quick "one-second test" before creating new files. Discovered during Phase 5 audit: 6 violations found, 37 files undocumented.

**Enforcement:**
```bash
# Verify compliance across all subdirectories
/tmp/check_subdir_readme.sh

# Expected output when compliant:
# ✅ 25 subdirectories verified complete
# ❌ 0 violations remaining
```

**Exceptions:** None. Main readme.md for repository onboarding is exempt (not a subdirectory readme).

### File Creation Protocol : Five-Step Workflow

Before creating ANY file, follow this mandatory workflow:

1. **Directory Check**: Verify file belongs in this directory (reference files_structure.rulebook.md § Directory Responsibility Table)
2. **Open** `[directory]/readme.md`
3. **One-Second Test**: Scan existing files - does any file in THIS directory have same responsibility?
   - Yes → Use existing file, do not create new
   - No → Proceed to step 4
4. **Add Row**: Add `| filename.rs | Single sentence responsibility (3-10 words) |` to Responsibility Table
5. **Create**: Create readme.md update + new file together (same session)

**Rationale:** Hierarchical Separation guarantees different directories = no overlap check needed. Only check files in SAME directory (Level 2 - within-directory uniqueness).

**Common Violation:** Creating file first, updating readme.md later (or never). This breaks the protocol - both operations must occur together.

> ✅ **Good**

```text
# Step 1: Check directory structure
# Step 2: Read module/iron_cli/src/formatting/readme.md
# Step 3: Scan table - no "tree formatting" responsibility exists
# Step 4: Add row: | tree_formatter.rs | Format data structures as ASCII trees |
# Step 5: Create both readme.md update + tree_formatter.rs in same session
```

> ❌ **Bad**

```text
# Created tree_formatter.rs
# (Never updated readme.md - violation)
# or
# Created tree_formatter.rs
# (Updated readme.md 2 days later - violation)
```

### File Creation Protocol : Responsibility Table Format

Responsibility Tables must follow strict two-column format with single-sentence responsibilities.

**Format:**
```markdown
| File | Responsibility |
|------|----------------|
| `mod.rs` | Export module public interface |
| `specific_task.rs` | Single sentence describing unique responsibility (3-10 words) |
```

**Quality Standards:**
- **Single Sentence**: 3-10 words, no compound statements
- **Specific**: Describes unique responsibility, not generic purpose
- **Action-Oriented**: Use verbs (Export, Test, Format, Handle, Validate)
- **No Duplication**: Each responsibility must be unique within directory

> ✅ **Good**

```text
| `tree_formatter.rs` | Format data structures as ASCII trees |
| `table_formatter.rs` | Format data structures as aligned tables |
```

> ❌ **Bad** (Too generic)

```text
| `utils.rs` | Utility functions |
| `helpers.rs` | Helper functions |
```

> ❌ **Bad** (Too verbose)

```text
| `formatter.rs` | Provides comprehensive formatting functionality including table, tree, expanded, JSON, and YAML output formats with automatic column width calculation and proper alignment |
```

**Discovered Pattern:** Parameter test files follow `{parameter}_parameter_test.rs` → "Test {Parameter} parameter validation" (automated during Phase 5).

### File Creation Protocol : Automated Verification

Use automated verification script to check File Creation Protocol compliance across entire repository.

**Verification Script:**
```bash
#!/bin/bash
# /tmp/check_subdir_readme.sh
# Finds all subdirectories with .rs files
# Checks for readme.md existence
# Verifies all .rs files documented in readme.md
# Reports violations

find module -type d | while read dir; do
  rs_files=$(find "$dir" -maxdepth 1 -name "*.rs" 2>/dev/null | wc -l)
  if [ "$rs_files" -gt 0 ]; then
    if [ ! -f "$dir/readme.md" ]; then
      echo "❌ $dir (missing readme.md, $rs_files files undocumented)"
    else
      # Check if all .rs files are documented
      for rs_file in $(find "$dir" -maxdepth 1 -name "*.rs"); do
        filename=$(basename "$rs_file")
        if ! grep -q "\`$filename\`" "$dir/readme.md"; then
          echo "❌ $dir (readme.md missing $filename)"
        fi
      done
    fi
  fi
done
```

**Compliance Target:** 100% (25/25 subdirectories verified complete as of Phase 5 completion)

**CI Integration:** Consider adding to pre-commit hook:
```bash
#!/bin/bash
# .git/hooks/pre-commit
/tmp/check_subdir_readme.sh | grep "❌"
if [ $? -eq 0 ]; then
  echo "ERROR: File Creation Protocol violations detected"
  exit 1
fi
```

### Test Quality Standards : Assert Message Coverage

Target 95%+ assert message coverage for debuggability. Assertions without messages create debugging friction.

**Current Status:** 93.9% coverage (Phase 3.2 completion: 196/205 asserts with messages, 9 remaining)

**Quality Standard:**
- **Specific**: Describe what failed, not just "assertion failed"
- **Actionable**: Include values or context for debugging
- **Concise**: Single sentence, no novels

> ✅ **Good**

```rust
assert_eq!( actual, expected, "Expected agent name '{}', got '{}'", expected, actual );
assert!( result.is_ok(), "Failed to create agent: {:?}", result.unwrap_err() );
```

> ❌ **Bad**

```rust
assert_eq!( actual, expected ); // No context
assert!( result.is_ok() ); // No error details
```

**Remaining Work:** 9 asserts in iron_cli/tests/adapters/ and iron_control_api/tests/tokens/ need messages (1-2 hours to complete).

**Enforcement:** Manual code review. Future: Consider clippy lint for assertions without messages.

### Test Quality Standards : No Mocking Policy

Use real implementations in tests. Mocks hide integration issues and create false confidence.

**Established Patterns:**
- **Real Databases**: `iron_test_db::TestDatabase` with production schema via migrations
- **Real CLI**: Process spawn with actual binary execution (see tests/fixtures/test_harness.rs)
- **Real HTTP**: Axum server on random port (see tests/fixtures/test_server.rs)

**Rationale:** Integration failures caught early. Tests verify actual behavior, not mock behavior.

> ✅ **Good**

```rust
let db = iron_test_db::TestDatabase::new().await;
let response = reqwest::get(&format!("http://localhost:{}/health", server.port())).await?;
let output = Command::new("iron_control").arg("agents").arg("list").output()?;
```

> ❌ **Bad**

```rust
let mock_db = MockDatabase::new();
mock_db.expect_query().returning(|_| Ok(vec![]));
```

**Migration in Progress:** iron_token_manager/tests/common/ migrating from legacy `(pool, TempDir)` pattern to `iron_test_db::TestDatabase` (functions with `_v2` suffix are new pattern).

### Test Quality Standards : Loud Failure Requirement

Tests must fail loudly and clearly with diagnostic information. Silent passes hide missing functionality.

**Anti-Patterns:**
- Silently passing when resources unavailable (tokens, API keys)
- Catching errors and continuing without reporting
- Disabled tests (ignore, skip attributes)

> ✅ **Good**

```rust
#[tokio::test]
async fn test_api_integration() {
  let api_key = env::var("API_KEY")
    .expect("API_KEY required for integration tests - set in environment or skip with --skip");
  let response = api_call(&api_key).await
    .expect("API call failed - check API key validity and network connectivity");
  assert_eq!(response.status, 200, "Expected success, got: {:?}", response);
}
```

> ❌ **Bad**

```rust
#[tokio::test]
async fn test_api_integration() {
  if let Ok(api_key) = env::var("API_KEY") {
    if let Ok(response) = api_call(&api_key).await {
      assert_eq!(response.status, 200); // Silently passes if token missing or API fails
    }
  }
}
```

**Enforcement:** Code review for conditional test execution. All test failures must be explicit.

### Test Quality Standards : Test Documentation

Integration tests must be in `tests/` directory with comprehensive documentation in doc comments.

**Documentation Requirements:**
- **Purpose**: What functionality being tested
- **Setup**: What infrastructure required (database, server, fixtures)
- **Approach**: Test strategy (real implementations, no mocking)
- **Coverage**: What scenarios covered

> ✅ **Good**

```rust
//! # Agent Lifecycle Tests
//!
//! Tests complete agent CRUD operations using real database and HTTP server.
//! Covers creation, retrieval, update, deletion with proper error handling.
//!
//! ## Infrastructure
//! - Real PostgreSQL via iron_test_db
//! - Real Axum HTTP server on random port
//! - Real CLI execution via process spawn
//!
//! ## No Mocking
//! All tests use real implementations to catch integration issues early.

#[tokio::test]
async fn test_create_agent() {
  // Test implementation
}
```

**Manual Testing Plans:** Located in `tests/manual/readme.md` with documented test procedures for non-automated testing.

### CLI Architecture : Handler-Validated Parameters

CLI handlers validate parameter existence and format before adapters extract values. This creates architectural contract enabling safe `.unwrap()` usage.

**Pattern:**
```rust
// Step 1: Handler validates ALL parameters
pub fn create_agent_handler( params: &HashMap<String, String> ) -> Result<(), Error> {
  // Validate existence
  params.get("name").ok_or_else(|| Error::MissingParameter("name"))?;
  params.get("budget").ok_or_else(|| Error::MissingParameter("budget"))?;

  // Validate format
  params.get("budget")
    .and_then(|s| s.parse::<i64>().ok())
    .ok_or_else(|| Error::InvalidFormat("budget must be integer"))?;

  Ok(())
}

// Step 2: Adapter extracts validated parameters
pub fn create_agent_adapter( params: &HashMap<String, String> ) -> Agent {
  // Safe: handler guarantees these exist and are valid
  let name = params.get("name").unwrap();
  let budget = params.get("budget").unwrap();
  let budget_value = budget.parse::<i64>().unwrap();

  Agent { name, budget: budget_value }
}
```

**Discovered During:** Phase 3.3 audit found ~95 unwraps following this pattern across CLI adapters

**Safety Guarantee:** If handler returns `Ok(())`, all parameters exist and are valid format. Adapter unwraps cannot fail.

**Fragility:** Breaking handler validation breaks adapter safety. Maintain handler-adapter contract carefully.

### CLI Architecture : Adapter Unwrap Safety

Unwraps in adapters are safe when handler validates parameters. This is borderline but architecturally sound pattern.

**Classification (from Phase 3.3):**
- **Test Infrastructure**: 32 unwraps - ACCEPTED (compile_error protected)
- **Infallible Operations**: 9 unwraps - ACCEPTED (midnight, day 1)
- **Documentation Examples**: 5 unwraps - ACCEPTED (not executable)
- **Handler-Validated**: ~95 unwraps - BORDERLINE (safe by architecture)
- **Total High-Risk**: 0 found

**Optional Improvement:** Replace `.unwrap()` with `.expect()` for better diagnostics:

> ✅ **Better**

```rust
let name = params.get("name")
  .expect("name parameter validated by handler");
let budget_value = budget.parse::<i64>()
  .expect("budget format validated by handler");
```

**Effort:** 2-3 hours to add expect messages to ~95 unwraps in CLI adapters (optional, not mandatory).

### CLI Architecture : Infallible Operation Detection

Recognize truly infallible operations that cannot fail. These unwraps are always safe.

**Infallible Examples (from Phase 3.3 audit):**
```rust
// Midnight (0:0:0) always valid
let start = date.and_hms_opt(0, 0, 0).unwrap();

// First day of month (1) always valid
let month_start = date.with_day(1).unwrap();

// Compile-time constant parsing
let uuid = Uuid::parse_str("550e8400-e29b-41d4-a716-446655440000").unwrap();
```

**Rationale:** Operations with valid hardcoded values cannot fail. No error handling needed.

**Verification:** Confirm values are compile-time constants or proven valid (0-23 hours, 1-31 days).

### Unwrap Guidelines : Classification Criteria

Classify unwraps into risk categories before determining if replacement needed.

**Categories (from Phase 3.3 audit):**

1. **Test Infrastructure** - ACCEPTED
   - Protected by `#[cfg(test)]` with `compile_error!` guards
   - Never compiles in production
   - Example: `unwrap()` in test setup code

2. **Infallible Operations** - ACCEPTED
   - Operations that mathematically cannot fail
   - Example: `.and_hms_opt(0, 0, 0).unwrap()` (midnight)

3. **Documentation Examples** - ACCEPTED
   - Code in doc comments not executed
   - Example: Module-level examples in `//!` comments

4. **Handler-Validated Parameters** - BORDERLINE
   - Safe by architectural contract
   - Fragile if validation changes
   - Example: Adapter extracting handler-validated parameters

5. **User Input Without Validation** - HIGH RISK
   - Must be replaced immediately
   - Example: `.unwrap()` on user-provided strings
   - **None found in Phase 3.3 audit**

**Enforcement:** Audit new unwraps using these categories. HIGH RISK requires immediate fix.

### Unwrap Guidelines : High-Risk Identification

User input without validation is always high-risk. These unwraps must be replaced immediately.

**High-Risk Pattern:**
```rust
// ❌ DANGEROUS
let value = user_input.parse::<i64>().unwrap(); // User input not validated

// ❌ DANGEROUS
let file = File::open(&user_path).unwrap(); // User path not validated

// ❌ DANGEROUS
let data = serde_json::from_str(&user_json).unwrap(); // User JSON not validated
```

**Phase 3.3 Finding:** Zero high-risk unwraps found across entire codebase.

**Replacement Pattern:**
```rust
// ✅ SAFE
let value = user_input.parse::<i64>()
  .map_err(|e| Error::InvalidFormat(format!("Invalid integer: {}", e)))?;

// ✅ SAFE
let file = File::open(&user_path)
  .map_err(|e| Error::FileAccess(format!("Cannot open {}: {}", user_path, e)))?;
```

**Enforcement:** Any unwrap on user-provided data is violation requiring immediate fix.

### Unwrap Guidelines : Expect Preference

Prefer `.expect()` with diagnostic messages over `.unwrap()` for better debuggability.

**Rationale:** When unwrap fails (e.g., due to architectural changes), expect message provides context.

> ✅ **Good**

```rust
let name = params.get("name")
  .expect("name parameter must exist - validated by create_agent_handler");

let budget_value = budget.parse::<i64>()
  .expect("budget must be valid integer - validated by create_agent_handler");
```

> ❌ **Acceptable but less helpful**

```rust
let name = params.get("name").unwrap();
let budget_value = budget.parse::<i64>().unwrap();
```

**Migration Effort:** 2-3 hours to add expect messages to ~95 unwraps in CLI adapters (optional improvement from Phase 3.3).

### Common Mistakes : Missing Readme Files

Subdirectories containing .rs files missing readme.md files entirely.

**Phase 5 Violations Found:**
- iron_cli/src/bin/ (2 files undocumented)
- iron_cli/tests/fixtures/ (4 files undocumented)
- iron_cli/tests/formatting/ (1 file undocumented)
- iron_cli/tests/parameters/ (27 files undocumented - largest violation)
- iron_token_manager/tests/common/ (1 file undocumented)

**Prevention:** Before creating first .rs file in new directory, create readme.md with Responsibility Table.

**Detection:** Run `/tmp/check_subdir_readme.sh` regularly to catch violations early.

### Common Mistakes : Incomplete Responsibility Tables

Readme exists but doesn't document all .rs files in directory.

**Phase 5 Example:**
- iron_control_api/tests/tokens/readme.md existed with 11 files documented
- Missing: audit_logging.rs, rate_limiting.rs
- **Fix:** Added 2 missing files to table (now 13/13 documented)

**Prevention:** When creating new file, ALWAYS update readme.md in same session (File Creation Protocol step 5).

**Detection:** Verification script checks every .rs file appears in readme.md table.

### Common Mistakes : Generic Filenames

Never use generic names like utils.rs, helpers.rs, common.rs, misc.rs. These violate Unique Responsibility Principle.

**Prohibited Names (from files_structure.rulebook.md):**
- `utils.rs` - Too generic, doesn't indicate specific responsibility
- `helpers.rs` - Same problem as utils.rs
- `common.rs` - Encourages dumping ground pattern
- `misc.rs` - Admission of unclear responsibility

**Correct Approach:** Name file after its specific responsibility

> ✅ **Good**

```text
| `tree_formatter.rs` | Format data structures as ASCII trees |
| `table_formatter.rs` | Format data structures as aligned tables |
| `json_formatter.rs` | Format data structures as JSON |
```

> ❌ **Bad**

```text
| `utils.rs` | Utility functions |
| `helpers.rs` | Helper functions |
```

**Enforcement:** Code review rejects generic filenames. Must provide specific responsibility-based name.

### Common Mistakes : Backup File Creation

Never create backup files (_backup, _old, _v1, _legacy) or preserve old implementations. Delete completely and trust git history.

**Prohibited Patterns:**
- `*_backup.rs`, `*.bak`, `*.orig`
- `*_old.rs`, `*_legacy.rs`
- `*_v1.rs`, `*_v2.rs` (version suffixes)
- Commented-out old code blocks

**Correct Approach:** Delete old implementation completely

> ✅ **Good**

```bash
# Replace implementation completely
rm old_implementation.rs
# Create new implementation
# Commit changes
# Old code preserved in git history
```

> ❌ **Bad**

```bash
mv old_implementation.rs old_implementation_backup.rs
# or
mv old_implementation.rs old_implementation_v1.rs
# or keeping both old and new files
```

**Rationale:** Git provides version history. Backup files clutter codebase and create maintenance burden.

### Verification Methods : File Creation Compliance

Run verification script to check File Creation Protocol compliance across all subdirectories.

**Command:**
```bash
/tmp/check_subdir_readme.sh
```

**Expected Output (Compliant):**
```
✅ 25 subdirectories verified complete
❌ 0 violations remaining
```

**Violation Output:**
```
❌ module/iron_cli/src/bin (missing readme.md, 2 files undocumented)
❌ module/iron_cli/tests/parameters (readme.md missing agent_id_parameter_test.rs)
```

**Frequency:** Run before committing new files or creating new subdirectories.

**CI Integration:** Add to pre-commit hook to prevent violations from being committed.

### Verification Methods : Test Level Commands

Use standardized test level commands for verification at different thoroughness levels.

**Level 1** (Basic - Unit Tests):
```bash
ctest1
# or
w3 .test level::1
# Runs: RUSTFLAGS="-D warnings" cargo nextest run --all-features
```

**Level 3** (Recommended - Full Verification):
```bash
ctest3
# or
w3 .test level::3
# Runs: Unit tests + Doc tests + Clippy
```

**Level 5** (Complete - CI/CD):
```bash
ctest5
# or
w3 .test level::5
# Runs: Level 3 + willbe tests + udeps + audit
```

**Default:** Use Level 3 (`ctest3` or `w3 .test level::3`) for standard verification before commits.

**Timeout:** Set explicit timeouts for multi-crate operations:
```bash
# Single crate: 3600000ms (1 hour)
# Workspace/subtree: 7200000ms (2 hours)
```

### Verification Methods : Full Compliance Check

Level 3 minimum for pre-commit verification ensures code quality standards met.

**Pre-Commit Checklist:**
1. ✅ All tests pass (Level 3)
2. ✅ No clippy warnings (Level 3)
3. ✅ Doc tests pass (Level 3)
4. ✅ File Creation Protocol compliant (`/tmp/check_subdir_readme.sh`)
5. ✅ No backup files created
6. ✅ Assert messages added to new assertions
7. ✅ No new high-risk unwraps introduced

**Command:**
```bash
# Full pre-commit verification
w3 .test level::3 && /tmp/check_subdir_readme.sh
```

**Expected Result:** All checks pass with zero warnings or violations.

**Enforcement:** Code review verifies compliance before merge. Consider git hooks for automation.
