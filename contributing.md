# Contributing to Iron Runtime

Thank you for contributing to Iron Runtime! This guide covers development setup, documentation standards, testing procedures, and contribution workflows.

## Table of Contents

- [Development Setup](#development-setup)
- [Documentation Standards](#documentation-standards)
- [Database Testing Guide](#database-testing-guide)
- [Manual Testing Procedures](#manual-testing-procedures)
- [Debug Run Procedures](#debug-run-procedures)
- [Python Development Workflow](#python-development-workflow)
- [Testing Standards](#testing-standards)
- [Bug Fix Workflow](#bug-fix-workflow)
- [Code Review Checklist](#code-review-checklist)
- [Pull Request Process](#pull-request-process)

---

## Development Setup

### Prerequisites

- **Rust:** 1.75+ (`rustup update`)
- **Python:** 3.9+ (`python --version`)
- **uv:** Package manager (`curl -LsSf https://astral.sh/uv/install.sh | sh`)
- **Node.js:** 18+ for dashboard (`node --version`)
- **SQLite:** 3.35+ (usually pre-installed)
- **cargo-nextest:** Test runner (`cargo install cargo-nextest`)

### Initial Setup

```bash
# Clone repository
git clone https://github.com/iron-cage/iron_runtime.git
cd iron_runtime/dev

# Build workspace
cargo build --workspace

# Run test suite
cargo nextest run --all-features
```

### Daily Commands

```bash
make dev          # Run full stack (API + Dashboard)
make test         # Run all tests
make lint-docs    # Check documentation compliance
make validate     # Full validation before PR
```

---

## Documentation Standards

### ID Format Standards

**Critical Rule:** All entity IDs in documentation MUST use underscore format (`prefix_identifier`).

#### Format Requirements

| Entity Type | Correct Format | Incorrect Format |
|-------------|----------------|------------------|
| Provider ID | `ip_openai_001` | ~~`ip-openai-001`~~ |
| User ID | `user_xyz789` | ~~`user-xyz789`~~ |
| Project ID | `proj_master` | ~~`proj-master`~~ |
| Agent ID | `agent_abc123` | ~~`agent-abc123`~~ |
| IC Token ID | `ic_def456` | ~~`ic-def456`~~ |
| IP Token ID | `ip_ghi789` | ~~`ip-ghi789`~~ |

#### Why This Matters

1. **Consistency**: Uniform format across all documentation
2. **Searchability**: Easy to grep and find all instances
3. **Code Generation**: Documentation examples directly translate to code
4. **API Standards**: Matches actual API implementation

#### Edge Cases (NOT Entity IDs)

These terms use hyphens because they're descriptive, not entity IDs:
- `user-token` (token type descriptor)
- `user-facing` (adjective)
- `user-level` (scope descriptor)

### Checking Compliance

Before submitting documentation changes:

```bash
# Run the lint check
make lint-docs

# Should output:
# ✓ No ID format violations found
```

If violations are found, the script will show:
- File paths and line numbers
- The specific violations
- Expected format for each entity type

### Canonical Examples

All documentation examples should use canonical values from `docs/standards/canonical_examples.md`:

```markdown
**Primary User:** `user_xyz789`
**Primary Providers:** `["ip_openai_001", "ip_anthropic_001"]`
**Primary Project:** `proj_master`
**Primary Agent:** `agent_abc123`
```

Using canonical examples ensures:
- Consistency across all documentation
- Easy cross-referencing between documents
- Recognizable patterns for users

### Documentation Files

- **Protocol Specs** (`docs/protocol/*.md`) - API endpoint specifications
- **Standards** (`docs/standards/*.md`) - Format and design standards
- **Architecture** (`docs/architecture/*.md`) - System design documents
- **Features** (`docs/features/*.md`) - Feature documentation

---

## Database Testing Guide

### Overview

Iron Runtime uses SQLite for development and testing. All database tests must follow these principles:

1. **Automatic Cleanup:** Use TempDir or in-memory databases, never manual cleanup
2. **Test Isolation:** Each test gets independent database instance
3. **Real Implementations:** No mocking, use real SQLite databases
4. **Idempotent Seed Data:** Seed operations safe to run multiple times

For complete standards, see [test_organization.rulebook.md](test_organization.rulebook.md).

### Database Types

#### In-Memory (Fastest)

**Use for:** Fast test execution, CI/CD pipelines, stateless operations

```rust
pub async fn create_test_database() -> SqlitePool {
  SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite::memory:?cache=shared")
    .await
    .expect("Failed to create test database")
}

#[tokio::test]
async fn test_something() {
  let pool = create_test_database().await;
  // pool dropped → automatic cleanup
}
```

#### Tempfile (Inspectable)

**Use for:** Debugging, migration testing, inspecting state after failure

```rust
pub async fn create_test_db() -> (SqlitePool, TempDir) {
  let temp_dir = TempDir::new()
    .expect("Failed to create temp dir");
  let db_path = temp_dir.path().join("test.db");
  let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect(&db_url)
    .await
    .expect("Failed to connect");

  apply_migrations(&pool).await
    .expect("Failed to apply migrations");

  (pool, temp_dir)
}

#[tokio::test]
async fn test_something() {
  let (pool, _temp) = create_test_db().await;
  // _temp dropped → TempDir Drop → cleanup
}
```

#### Config-Based (Manual Testing)

**Use for:** Interactive development, manual testing, debug runs only

```toml
# config.dev.toml
[database]
url = "sqlite:///./dev_tokens.db?mode=rwc"
wipe_and_seed = true  # Reset on startup
```

**CRITICAL:** Never use config-based databases in automated tests.

---

## Manual Testing Procedures

### Before Manual Testing

Manual testing requires a clean database with realistic seed data. Follow these procedures to ensure consistent test environment.

### Procedure 1: Fresh Database Setup

**When:** Starting new debug session, testing migrations, clean slate needed

**Commands:**
```bash
# Unified command (recommended)
make db-reset-seed

# Individual steps (if make unavailable)
rm -f dev_*.db          # Delete all development databases
cargo run --config config.dev.toml  # Restart with fresh database
```

**What Happens:**
1. All `dev_*.db` files deleted
2. Application starts and detects missing database
3. Migrations applied automatically (if `auto_migrate = true`)
4. Seed data populated (if `wipe_and_seed = true`)
5. Database ready with standard test data

**Verify Success:**
```bash
# Check database exists
ls -lh dev_*.db

# Inspect seed data
make db-inspect
# Then: SELECT COUNT(*) FROM users;
```

### Procedure 2: Database Inspection

**When:** Debugging, verifying state, understanding data structure

**Commands:**
```bash
# Quick inspection (recommended)
make db-inspect

# Direct access
sqlite3 ./dev_tokens.db

# Read-only access (safe, can't corrupt)
sqlite3 -readonly ./dev_tokens.db
```

**Useful SQL Queries:**
```sql
-- List all tables
.tables

-- Show table structure
.schema users
.schema api_tokens

-- Inspect seed data
SELECT username, role FROM users;
SELECT token_prefix, status FROM api_tokens;

-- Check row counts
SELECT
  (SELECT COUNT(*) FROM users) as users,
  (SELECT COUNT(*) FROM api_tokens) as tokens,
  (SELECT COUNT(*) FROM usage_limits) as limits;

-- Find specific records
SELECT * FROM api_tokens WHERE status = 'active';
SELECT * FROM users WHERE role = 'admin';
```

**Exit:** Type `.exit` or press Ctrl+D

### Procedure 3: Partial Reset (Data Only)

**When:** Schema correct but data corrupted, testing seed changes

**Commands:**
```bash
# Using CLI (if available)
iron-cli db wipe    # Deletes all rows, keeps schema
iron-cli db seed    # Repopulates seed data

# Using config flag
# Edit config.dev.toml: set wipe_and_seed = true
# Restart application
cargo run --config config.dev.toml
```

**Advantages:**
- Faster than full db-reset (no migrations)
- Preserves schema modifications
- Safe for testing seed data changes

### Procedure 4: Debug Run with Automatic Reset

**When:** Frequent iteration, want fresh state every run

**Configuration:**
```toml
# config.dev.toml
[database]
url = "sqlite:///./dev_tokens.db?mode=rwc"
auto_migrate = true
foreign_keys = true

[development]
debug = true
auto_seed = false        # Manual seed only
wipe_and_seed = true     # Automatic wipe+seed on startup
```

**Start Debug Run:**
```bash
# Application wipes and seeds automatically on startup
cargo run --config config.dev.toml

# Or with custom config
cargo run -- --config /path/to/custom.toml
```

**Automatic Actions:**
1. Database opened (or created if missing)
2. Migrations applied (if `auto_migrate = true`)
3. All rows deleted respecting foreign keys (if `wipe_and_seed = true`)
4. Seed data populated
5. Application ready with fresh state

---

## Debug Run Procedures

### Overview

Debug runs allow interactive testing with real database, logging, and step-through debugging. All debug runs should start with clean database state to avoid confusion.

### Debug Run Types

#### Type 1: Standard Debug Run

**Purpose:** Interactive development with automatic database reset

**Setup:**
```bash
# Edit config.dev.toml
[development]
wipe_and_seed = true
debug = true
log_level = "debug"

# Run
cargo run --config config.dev.toml
```

**Features:**
- Fresh database on every start
- Debug logging enabled
- Seed data available immediately
- Database persists for inspection after exit

#### Type 2: Breakpoint Debugging

**Purpose:** Step-through debugging with IDE or rust-gdb

**VS Code (.vscode/launch.json):**
```json
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug iron_runtime",
      "cargo": {
        "args": [
          "build",
          "--bin=iron-runtime",
          "--package=iron_runtime"
        ],
        "filter": {
          "name": "iron-runtime",
          "kind": "bin"
        }
      },
      "args": ["--config", "config.dev.toml"],
      "cwd": "${workspaceFolder}",
      "preLaunchTask": "db-reset-seed"
    }
  ]
}
```

**CLI Debugging:**
```bash
# Fresh database first
make db-reset-seed

# Start debugger
rust-gdb --args target/debug/iron-runtime --config config.dev.toml

# Or with lldb
rust-lldb -- target/debug/iron-runtime --config config.dev.toml
```

#### Type 3: Test-Specific Debug Run

**Purpose:** Debug specific feature with custom test data

**Steps:**
```bash
# 1. Create custom seed for feature
cat > /tmp/custom_seed.sql <<EOF
INSERT INTO users (username, password_hash, role)
VALUES ('testuser', 'hash', 'normal');

INSERT INTO api_tokens (user_id, token_hash, status)
VALUES (1, 'test_hash', 'active');
EOF

# 2. Reset and apply custom seed
make db-reset
sqlite3 dev_tokens.db < /tmp/custom_seed.sql

# 3. Run with preserved data (wipe_and_seed = false)
cargo run --config config.dev.toml
```

### Debug Run Checklist

Before starting debug run, verify:

- [ ] `config.dev.toml` has correct database settings
- [ ] `wipe_and_seed` flag matches intent (true for fresh, false for preserved)
- [ ] Previous `dev_*.db` files cleaned if starting fresh
- [ ] Seed data appropriate for test scenario
- [ ] Log level set appropriately (`debug` for detailed, `info` for normal)

After debug run, inspect:

- [ ] Database state matches expectations: `make db-inspect`
- [ ] Logs contain expected operations: `tail -f logs/debug.log`
- [ ] No leaked resources: `ls -lh dev_*.db` (should only see expected files)

### Common Debug Scenarios

#### Scenario 1: Testing Token Generation

```bash
# Fresh database with seed users
make db-reset-seed

# Start runtime
cargo run --config config.dev.toml

# In another terminal, verify seed data
sqlite3 dev_tokens.db "SELECT username FROM users;"

# Generate tokens via API or CLI
# Inspect results
sqlite3 dev_tokens.db "SELECT token_prefix, status FROM api_tokens;"
```

#### Scenario 2: Testing Migration

```bash
# Start with database at previous migration
make db-reset
cargo run --config config.old.toml  # Old schema

# Apply new migration
cargo run --config config.dev.toml

# Verify schema
sqlite3 dev_tokens.db ".schema new_table"
```

#### Scenario 3: Testing Budget Enforcement

```bash
# Fresh database with low-limit token
make db-reset-seed

# Modify seed to have low limit
sqlite3 dev_tokens.db "UPDATE usage_limits SET max_usd = 1.0 WHERE id = 1;"

# Run runtime
cargo run --config config.dev.toml

# Trigger budget exceeded scenario via API
# Inspect usage records
sqlite3 dev_tokens.db "SELECT * FROM usage_records ORDER BY timestamp DESC LIMIT 5;"
```

---

## Python Development Workflow

### Prerequisites

- Python 3.9+ (auto-managed by uv)
- uv (`curl -LsSf https://astral.sh/uv/install.sh | sh`)

### Setup

```bash
# Navigate to module
cd module/iron_sdk  # or iron_runtime, iron_cli_py

# Install dependencies (one command!)
uv sync

# This automatically:
# - Downloads correct Python version
# - Creates .venv directory
# - Installs all dependencies
# - Creates uv.lock file
```

### Adding Dependencies

```bash
# Runtime dependency
uv add requests

# Development dependency
uv add --dev pytest

# This automatically:
# - Adds to pyproject.toml
# - Updates uv.lock
# - Installs the package
```

### Running Code

```bash
# Run script
uv run python main.py

# Run tests
uv run pytest

# No need to activate virtualenv!
```

### Python Pre-Commit Checklist

Before submitting PRs with Python changes:

- [ ] Tests pass (`uv run pytest` or `make py-test`)
- [ ] Python tooling compliant (`make lint-python`)
- [ ] No `.python-version` or `requirements.txt` files
- [ ] `uv.lock` committed if dependencies changed

### Python DON'Ts

❌ Never create `.python-version` files
❌ Never create `requirements.txt` files
❌ Never run `pip install` (use `uv add`)
❌ Never manually edit dependencies in `pyproject.toml`
❌ Never commit without running `make lint-python`

For complete Python workflow documentation, see [Python Tooling Standards](docs/standards/python_tooling_standards.md).

---

## Testing Standards

### Test Execution

```bash
# Run all tests (recommended)
cargo nextest run --all-features

# Run specific module tests
cargo nextest run -p iron_token_manager

# Run with output
cargo nextest run --nocapture

# Run single test
cargo nextest run test_token_generation
```

### Test Levels

```bash
# Level 1: Fast unit tests
cargo nextest run --all-features

# Level 3: Full validation (unit + doc + clippy)
make test  # Runs w3 .test l::3

# Documentation lint
make lint-docs
```

### Test Quality Requirements

Every test must:

1. **Use automatic cleanup** (TempDir or in-memory database)
2. **Include descriptive assertions** with error messages
3. **Be isolated** (no shared state between tests)
4. **Use real implementations** (no mocking)
5. **Be reproducible** (no flaky tests)

### Example: Good Test

```rust
#[tokio::test]
async fn test_token_validation_with_explicit_values() {
  let (pool, _temp) = create_test_db().await;

  // Explicit test data (no default parameters)
  let token = "test_token_abc123";
  let user_id = 1;

  create_token(&pool, token, user_id).await
    .expect("Failed to create token");

  let result = validate_token(&pool, token).await;

  assert!(
    result.is_ok(),
    "Expected token validation to succeed for token '{}' but got error: {:?}",
    token,
    result.err()
  );

  // _temp dropped here → automatic cleanup
}
```

### Example: Bad Test (Don't Do This)

```rust
#[tokio::test]
async fn test_token() {  // ❌ Vague name
  let pool = create_shared_db();  // ❌ Shared state

  create_token(&pool, None, None);  // ❌ Default parameters (fragile)

  let result = validate_token(&pool, "token");
  assert!(result.is_ok());  // ❌ No error message

  // ❌ No cleanup mechanism
}
```

---

## Bug Fix Workflow

### Complete Bug Fix Process

Every bug fix must follow this 7-step workflow:

#### Step 1: Create Bug Reproducer Test

```rust
/// Reproduces token lookup failure due to BCrypt non-determinism (issue-123).
///
/// ## Root Cause
/// BCrypt hashing includes random salt, causing same token to produce
/// different hashes. Database lookups by hash_eq() never match.
///
/// ## Why Not Caught Initially
/// Original test only generated hash once. Didn't test hash consistency
/// across multiple generations of same token.
///
/// ## Fix Applied
/// Switched from BCrypt (non-deterministic) to SHA-256 (deterministic).
///
/// ## Prevention
/// For security-critical hashing, understand algorithm properties:
/// - BCrypt: random salt (good for passwords)
/// - SHA-256: deterministic (good for content-addressable lookups)
///
/// ## Pitfall to Avoid
/// BCrypt's non-determinism is a feature for password storage but breaks
/// lookup-by-hash patterns. Choose hash function based on use case.
// test_kind: bug_reproducer(issue-123)
#[test]
fn test_bcrypt_breaks_token_lookup() {
  let token = "my_secret_token";

  // Generate hash twice
  let hash1 = hash_token_bcrypt(token);
  let hash2 = hash_token_bcrypt(token);

  // BCrypt produces different hashes
  assert_ne!(
    hash1, hash2,
    "BCrypt should produce different hashes (includes random salt)"
  );

  // This breaks database lookup by hash
  // SELECT * FROM tokens WHERE hash = ? will never match
}
```

**Requirements:**
- Test marked with `// test_kind: bug_reproducer(issue-NNN)`
- 5-section documentation (Root Cause, Why Not Caught, Fix Applied, Prevention, Pitfall)
- Specific, technical, actionable content (not generic)
- Must FAIL before fix applied

#### Step 2: Implement Fix

```rust
// Fix(issue-123): Switch token hashing from BCrypt to SHA-256
// Root cause: BCrypt non-determinism (random salt) broke hash equality lookups
// Pitfall: BCrypt randomness is feature for passwords, breaks lookup-by-hash
pub fn hash_token(token: &str) -> String {
  use sha2::{Sha256, Digest};
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  format!("{:x}", hasher.finalize())
}
```

**Requirements:**
- 3-field comment (Fix(issue-NNN), Root cause, Pitfall)
- Specific technical explanation
- References issue number

#### Step 3: Verify Reproducer Passes

```bash
cargo nextest run test_bcrypt_breaks_token_lookup
# Should pass after fix
```

#### Step 4: Run Full Test Suite

```bash
cargo nextest run --all-features
# All tests must pass
```

#### Step 5: Update Documentation

If bug revealed systemic issue, update module docs:

```rust
//! # Known Pitfalls
//!
//! ## Token Hashing Algorithm Selection
//!
//! Use SHA-256 for token hashing (content-addressable lookup), not BCrypt.
//! BCrypt includes random salt (good for passwords) but breaks lookup-by-hash.
//! See issue-123 for details.
```

#### Step 6: Code Review

Reviewer checks:
- [ ] Bug reproducer test has 5-section documentation
- [ ] Documentation is specific, technical, actionable
- [ ] Source code has 3-field fix comment
- [ ] Reproducer test passes
- [ ] Full test suite passes
- [ ] No similar bugs exist elsewhere in codebase

#### Step 7: Merge and Close

Close issue referencing commit with reproducer test.

---

## Code Review Checklist

### Before Requesting Review

- [ ] All tests pass: `cargo nextest run --all-features`
- [ ] Clippy clean: `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Code formatted per rulebooks (NOT cargo fmt)
- [ ] Documentation updated
- [ ] No forbidden patterns (backup files, non-hyphenated temp files)

### Reviewer Checks

#### Round 0: Organizational Governance

- [ ] Specification alignment (`spec/requirements.md`)
- [ ] File organization follows `organizational_principles.rulebook.md`
- [ ] No duplicate knowledge across files
- [ ] Each file has unique, single responsibility

#### Round 1: Critical Rules

- [ ] Error handling uses `error_tools` crate (NOT anyhow/thiserror)
- [ ] No mocking in tests (uses real implementations)
- [ ] All temp files prefixed with hyphen (`-*.md`)
- [ ] No backup files (`*_old`, `*_backup`, `*_v1`)
- [ ] Database tests use automatic cleanup (TempDir or in-memory)

#### Round 2: Testing Quality

- [ ] All assertions include descriptive error messages
- [ ] Tests use explicit fixtures (no default parameters)
- [ ] Bug fixes include 5-section test documentation
- [ ] Bug fixes include 3-field source code comment
- [ ] Test names describe specific behavior tested

#### Round 3: Code Quality

- [ ] Functions under 50 lines (ideally)
- [ ] Descriptive variable names
- [ ] Comments explain "why" not "what"
- [ ] Security reviewed (no SQL injection, XSS, command injection)
- [ ] Error handling appropriate for context

---

## Pull Request Process

### Pre-Submission Checklist

Before submitting a PR, ensure:

- [ ] All tests pass (`make test`)
- [ ] Documentation lint passes (`make lint-docs`)
- [ ] Python tooling compliant (`make lint-python`)
- [ ] New features have tests
- [ ] API changes are documented
- [ ] Commit messages are clear and descriptive

### PR Title Format

```
<type>(<scope>): <description>

Examples:
feat(token-manager): Add token expiration enforcement
fix(control-api): Resolve BCrypt non-determinism in token lookup
docs(contributing): Add manual testing procedures
refactor(runtime): Consolidate error handling to error_tools
```

### Commit Message Format

Use conventional commits format:

- `feat:` - New feature
- `fix:` - Bug fix
- `docs:` - Documentation only
- `test:` - Adding or updating tests
- `refactor:` - Code refactoring
- `chore:` - Maintenance tasks

Examples:
```
feat: add budget request approval workflow
fix: handle null provider response correctly
docs: update authentication API examples
test: add integration tests for token rotation
```

### PR Description Template

```markdown
### Summary
[1-3 sentences describing the change]

### Changes
- Bullet list of specific changes
- Include file paths for major modifications

### Testing
- [ ] Unit tests pass
- [ ] Integration tests pass
- [ ] Manual testing completed (if applicable)
- [ ] Bug reproducer test included (if bug fix)

### Documentation
- [ ] Specification updated (if requirements changed)
- [ ] Module readme updated (if public API changed)
- [ ] Bug documentation includes 5 sections (if bug fix)

### Checklist
- [ ] No forbidden crates (anyhow, thiserror, clap)
- [ ] No backup files or non-hyphenated temp files
- [ ] Tests use automatic cleanup
- [ ] Code follows all applicable rulebooks
```

### Review Stages

1. **Automated Checks:** CI runs tests, clippy, builds
2. **Reviewer Round 0:** Organizational governance checks
3. **Reviewer Round 1:** Critical rules enforcement
4. **Reviewer Round 2:** Testing quality review
5. **Reviewer Round 3:** Code quality review
6. **Approval:** Merge when all checks pass

### Merge Requirements

- [ ] All CI checks pass
- [ ] At least one approving review
- [ ] All review comments resolved
- [ ] Branch up-to-date with master
- [ ] No merge conflicts

---

## Additional Resources

- **Getting Started:** [docs/getting_started.md](docs/getting_started.md)
- **Architecture:** [docs/architecture/readme.md](docs/architecture/readme.md)
- **API Protocol Specs:** [docs/protocol/](docs/protocol/)
- **ID Format Standards:** [docs/standards/id_format_standards.md](docs/standards/id_format_standards.md)
- **Python Tooling Standards:** [docs/standards/python_tooling_standards.md](docs/standards/python_tooling_standards.md)
- **Canonical Examples:** [docs/standards/canonical_examples.md](docs/standards/canonical_examples.md)
- **Testing Philosophy:** [docs/decisions/adr_007_testing_philosophy.md](docs/decisions/adr_007_testing_philosophy.md)
- **Testing Standards:** [test_organization.rulebook.md](test_organization.rulebook.md)
- **Project Specification:** [spec/requirements.md](spec/requirements.md)

---

## License

By contributing to Iron Runtime, you agree that your contributions will be licensed under the Apache-2.0 License.

---

Thank you for contributing to Iron Runtime! 🦀
