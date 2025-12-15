# Test Organization Rulebook

**Version:** 1.0
**Status:** Active
**Authority:** Project-Level Testing Standards
**Scope:** All test code in iron_runtime workspace

## Purpose

This rulebook defines comprehensive testing standards for database cleanup, test data population, and test organization across all modules in the iron_runtime project. These standards ensure consistency, reliability, and maintainability of the test suite.

## Core Principles

### Principle 1: Automatic Database Cleanup

**Every test must automatically clean up its database state.** Manual cleanup is forbidden.

**Implementation:**
- Use Rust Drop trait for automatic cleanup
- TempDir pattern for file-based databases
- Connection pool drop for in-memory databases
- Never rely on manual cleanup commands

**Rationale:** Manual cleanup is error-prone and leaves test artifacts after failures.

### Principle 2: Test Isolation

**Every test gets an independent database instance.** Shared database state is forbidden.

**Implementation:**
- Each test creates its own database (memory or tempfile)
- No shared database connections between tests
- Tests can run in parallel without conflicts
- Each test applies migrations independently

**Rationale:** Shared state causes flaky tests and makes debugging difficult.

### Principle 3: Real Implementations

**All tests use real database implementations.** Mocking is forbidden.

**Implementation:**
- SQLite for development and testing (in-memory or tempfile)
- Real migrations applied to test databases
- Real SQL queries, no mocked database layers
- Real crypto, real validation, real business logic

**Rationale:** Mocks test mock behavior, not real behavior. See ADR-007.

### Principle 4: Idempotent Seed Data

**All seed data operations must be idempotent.** Re-running seed should be safe.

**Implementation:**
- Use `INSERT OR IGNORE` pattern
- Check existence before creation
- Document seed data contracts in tests/readme.md
- Seed functions return Result, not panic

**Rationale:** Idempotency enables safe test setup and debug workflows.

### Principle 5: Loud Test Failures

**Every test failure must include descriptive context.** Silent failures are forbidden.

**Implementation:**
- Every assertion includes error message
- Error messages describe expected vs actual
- Include relevant state in failure output
- Use `expect()` with context, never `unwrap()`

**Rationale:** Context-free failures waste debugging time.

## Database Selection Matrix

### When to Use In-Memory SQLite

**Pattern:** `sqlite::memory:?cache=shared`

**Use When:**
- Fast test execution is priority
- No need to inspect database after test
- Testing stateless operations
- Running in CI/CD pipelines

**Example (iron_control_api):**
```rust
pub async fn create_test_database() -> SqlitePool {
  let pool = SqlitePoolOptions::new()
    .max_connections(5)
    .connect("sqlite::memory:?cache=shared")
    .await
    .expect("Failed to create test database");

  apply_migrations(&pool).await
    .expect("Failed to apply migrations");

  pool
}

#[tokio::test]
async fn test_user_creation() {
  let pool = create_test_database().await;
  // pool dropped → automatic cleanup
}
```

### When to Use Tempfile SQLite

**Pattern:** TempDir with `sqlite://{path}?mode=rwc`

**Use When:**
- Need to inspect database after test failure
- Testing migration rollback scenarios
- Debugging complex database states
- Testing file-based database features

**Example (iron_token_manager):**
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
    .expect("Failed to connect to test database");

  apply_migrations(&pool).await
    .expect("Failed to apply migrations");

  (pool, temp_dir)
}

#[tokio::test]
async fn test_token_generation() {
  let (pool, _temp) = create_test_db().await;
  // _temp dropped → TempDir Drop → automatic cleanup
}
```

### When to Use Config-Based Database

**Pattern:** Read from `config.dev.toml` or environment variable

**Use When:**
- Manual testing (debug runs)
- Interactive development workflow
- Need to inspect state between runs
- Testing with persistent data

**Example (iron_runtime):**
```rust
// config.dev.toml
[database]
url = "sqlite:///./dev_tokens.db?mode=rwc"
wipe_and_seed = true

// Startup code
if config.database.wipe_and_seed {
  wipe_database(&pool).await?;
  seed_all(&pool).await?;
}
```

**CRITICAL:** Config-based databases are for manual testing only, never for automated tests.

## Database Cleanup Mechanisms

### Mechanism 1: TempDir Drop (File-Based)

**How It Works:**
1. Test creates TempDir instance
2. Database file created inside TempDir
3. Test completes (success or failure)
4. TempDir Drop trait executes
5. Entire directory deleted recursively

**Code Pattern:**
```rust
pub async fn create_test_db() -> (SqlitePool, TempDir) {
  let temp_dir = TempDir::new().expect("Failed to create temp dir");
  // Store temp_dir, return it
  (pool, temp_dir)
}

#[tokio::test]
async fn test_something() {
  let (pool, _temp) = create_test_db().await;
  // Do test work
  // _temp dropped here → cleanup happens automatically
}
```

**Key Properties:**
- Works even if test panics
- Cleans up partial writes
- No manual cleanup needed
- File system guarantees

### Mechanism 2: Connection Pool Drop (In-Memory)

**How It Works:**
1. Test creates SqlitePool to :memory:
2. Database exists in RAM only
3. Test completes (success or failure)
4. SqlitePool Drop closes connections
5. Memory database destroyed

**Code Pattern:**
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
  // Do test work
  // pool dropped here → memory freed automatically
}
```

**Key Properties:**
- Fastest cleanup (just free memory)
- No file system operations
- Works even if test panics
- No disk I/O

### Mechanism 3: Explicit Wipe (Manual Testing Only)

**How It Works:**
1. Debug run starts
2. Explicitly call wipe_database()
3. Explicitly call seed_all()
4. Work with fresh database
5. Leave database for inspection

**Code Pattern:**
```rust
// ONLY for manual testing, NEVER in automated tests
pub async fn wipe_database(pool: &SqlitePool) -> Result<()> {
  // Delete in reverse dependency order
  sqlx::query("DELETE FROM usage_records").execute(pool).await?;
  sqlx::query("DELETE FROM project_assignments").execute(pool).await?;
  sqlx::query("DELETE FROM usage_limits").execute(pool).await?;
  sqlx::query("DELETE FROM api_tokens").execute(pool).await?;
  sqlx::query("DELETE FROM provider_keys").execute(pool).await?;
  sqlx::query("DELETE FROM users").execute(pool).await?;
  Ok(())
}

// Startup for debug runs
if config.development.wipe_and_seed {
  wipe_database(&pool).await?;
  seed_all(&pool).await?;
}
```

**Key Properties:**
- Only for manual testing
- Database persists for inspection
- Idempotent (safe to run multiple times)
- Respects foreign key constraints

## Seed Data Standards

### Standard 1: Idempotent Operations

**Requirement:** Seed functions must be safe to run multiple times.

**Implementation:**
```rust
// GOOD: Idempotent
pub async fn seed_users(pool: &SqlitePool) -> Result<()> {
  sqlx::query(
    "INSERT OR IGNORE INTO users (username, password_hash, role)
     VALUES (?, ?, ?)"
  )
  .bind("admin")
  .bind("hash_value")
  .bind("admin")
  .execute(pool)
  .await?;
  Ok(())
}

// BAD: Not idempotent
pub async fn seed_users(pool: &SqlitePool) -> Result<()> {
  sqlx::query("INSERT INTO users (username, password_hash, role) VALUES (?, ?, ?)")
    // Second run will fail with UNIQUE constraint violation
}
```

### Standard 2: Seed Data Contract

**Requirement:** Document seed data in tests/readme.md Responsibility Table.

**Format:**
```markdown
### Seed Data Contract

| Entity | Count | Purpose |
|--------|-------|---------|
| users | 3 | Admin, normal user, read-only user for permission testing |
| api_tokens | 5 | Valid, expired, revoked, low-limit, high-limit tokens |
| provider_keys | 2 | OpenAI and Anthropic keys for provider testing |
| usage_limits | 3 | Default, custom, zero-limit for quota testing |
```

**Rationale:** Tests depend on seed data structure. Document it as test contract.

### Standard 3: Seed Data Organization

**Requirement:** Seed functions organized by entity, called in dependency order.

**Pattern:**
```rust
pub async fn seed_all(pool: &SqlitePool) -> Result<()> {
  // Call in dependency order (base entities first)
  seed_users(pool).await?;           // No dependencies
  seed_provider_keys(pool).await?;   // No dependencies
  seed_api_tokens(pool).await?;      // Depends on users
  seed_usage_limits(pool).await?;    // Depends on api_tokens
  seed_project_assignments(pool).await?; // Depends on users + api_tokens
  Ok(())
}

async fn seed_users(pool: &SqlitePool) -> Result<()> { /* ... */ }
async fn seed_provider_keys(pool: &SqlitePool) -> Result<()> { /* ... */ }
async fn seed_api_tokens(pool: &SqlitePool) -> Result<()> { /* ... */ }
```

### Standard 4: Wipe in Reverse Order

**Requirement:** Deletion must respect foreign key constraints.

**Pattern:**
```rust
pub async fn wipe_database(pool: &SqlitePool) -> Result<()> {
  // Delete in REVERSE dependency order (dependents first)
  sqlx::query("DELETE FROM project_assignments").execute(pool).await?;
  sqlx::query("DELETE FROM usage_records").execute(pool).await?;
  sqlx::query("DELETE FROM usage_limits").execute(pool).await?;
  sqlx::query("DELETE FROM api_tokens").execute(pool).await?;
  sqlx::query("DELETE FROM provider_keys").execute(pool).await?;
  sqlx::query("DELETE FROM users").execute(pool).await?;
  Ok(())
}
```

## Database Path Conventions

### Dual-Path Architecture

Iron Runtime uses a dual-path approach for database files:

1. **Canonical Path** (`iron.db`): Default for standalone module use
2. **Project Convention** (`dev_{module}.db`): Enforced by Makefile for workspace consistency

**Rationale:**
- Modules work standalone with sensible defaults (iron.db)
- Project-level Makefile enforces consistent naming (dev_tokens.db)
- Environment variables allow production overrides

### Convention 1: Canonical Path (Standalone Use)

**Pattern:** `./iron.db`

**When Used:**
- Running module binaries directly without Makefile
- Module config files (config.dev.toml) specify this path
- Standalone development and testing

**Examples:**
```bash
# iron_token_manager standalone
cd module/iron_token_manager
cargo run --config config.dev.toml  # Creates ./iron.db

# iron_control_api standalone
cd module/iron_control_api
cargo run  # Creates ./iron.db (or uses DATABASE_URL if set)
```

**Configuration:**
```toml
# module/iron_token_manager/config.dev.toml
[database]
url = "sqlite:///./iron.db?mode=rwc"  # Canonical path
```

### Convention 2: Project Convention (Makefile-Enforced)

**Pattern:** `./dev_{module}.db`

**When Used:**
- Running via project-level Makefile targets
- Provides consistent naming across workspace
- Makefile passes explicit database paths to scripts

**Examples:**
- `./dev_tokens.db` (iron_token_manager via Makefile)
- `./dev_runtime.db` (iron_runtime via Makefile)
- `./dev_control.db` (iron_control_api via Makefile)

**Implementation:**
```makefile
# Makefile explicitly passes project convention path
db-reset-seed:
	@module/iron_token_manager/scripts/reset_and_seed.sh dev_tokens.db

# Scripts accept path as parameter, use canonical as default
DB_PATH="${1:-./iron.db}"  # Canonical default, overridable
```

**Location:** Project root (same directory as Cargo.toml)

**Gitignore:** `*.db` pattern covers both canonical and convention paths

### Convention 2: Test Databases

**Pattern:** `{tempdir}/test.db` or `:memory:`

**Implementation:**
```rust
// Tempfile pattern
let temp_dir = TempDir::new()?;
let db_path = temp_dir.path().join("test.db");
let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

// In-memory pattern
let db_url = "sqlite::memory:?cache=shared";
```

**Key Requirement:** Never use fixed paths for test databases.

### Convention 3: Production Databases

**Pattern:** Environment variable `DATABASE_URL`

**Implementation:**
```rust
let db_url = std::env::var("DATABASE_URL")
  .expect("DATABASE_URL must be set");
```

**Fallback:** None. Production must set DATABASE_URL explicitly.

## Manual Testing Procedures

### Procedure 1: Fresh Database Setup

**Use Case:** Starting fresh debug session with clean seed data.

**Commands:**
```bash
# Unified command (recommended)
make db-reset-seed

# Individual steps (if make target unavailable)
make db-reset  # Deletes dev_*.db files
make db-seed   # Creates fresh database with seed data
```

**What Happens:**
1. All `dev_*.db` files deleted
2. Application starts
3. Migrations applied to fresh database
4. Seed data populated
5. Database ready for manual testing

### Procedure 2: Database Inspection

**Use Case:** Examining database state during debugging.

**Commands:**
```bash
# Interactive SQL shell
make db-inspect

# Or directly
sqlite3 ./dev_tokens.db

# Useful queries
.tables                    -- List all tables
.schema users             -- Show table schema
SELECT * FROM api_tokens; -- Inspect data
```

### Procedure 3: Partial Reset

**Use Case:** Keep database but reset data only.

**Implementation:**
```bash
# In application code or CLI
iron-cli db wipe    # Delete all rows
iron-cli db seed    # Repopulate seed data
```

**When to Use:**
- Schema is correct, data is corrupted
- Want to test seed data changes
- Faster than full db-reset-seed

### Procedure 4: Debug Run with Config

**Use Case:** Starting application with automatic wipe+seed.

**Configuration (config.dev.toml):**
```toml
[database]
url = "sqlite:///./dev_tokens.db?mode=rwc"
auto_migrate = true
foreign_keys = true

[development]
debug = true
auto_seed = false
wipe_and_seed = true  # Enable automatic wipe+seed on startup
```

**Start Application:**
```bash
cargo run -- --config config.dev.toml
# Database wiped and seeded automatically
```

## Test Organization Patterns

### Pattern 1: Common Test Infrastructure

**Location:** `tests/common/mod.rs`

**Responsibilities:**
- Database creation functions
- Shared fixtures (explicit, no defaults)
- Test helper functions
- Constants for test data

**Example Structure:**
```rust
// tests/common/mod.rs
pub mod fixtures;  // Test data builders
pub mod helpers;   // Database setup functions

pub async fn create_test_db() -> (SqlitePool, TempDir) {
  // Standard tempfile pattern
}

pub async fn create_test_database() -> SqlitePool {
  // Standard in-memory pattern
}
```

### Pattern 2: Test File Organization

**Structure:** Organize by domain, not by test methodology.

```
tests/
├── common/
│   ├── mod.rs           # Test infrastructure
│   ├── fixtures.rs      # Test data builders
│   └── helpers.rs       # Setup helpers
├── token_operations.rs  # All token-related tests
├── user_management.rs   # All user-related tests
├── budget_tracking.rs   # All budget-related tests
└── readme.md            # Test documentation
```

**Rationale:** Domain organization reflects how developers think about features.

### Pattern 3: Test Documentation

**Requirement:** Every tests/ directory must have readme.md with Responsibility Table.

**Minimum Content:**
```markdown
### Tests Responsibility Table

| File | Responsibility |
|------|----------------|
| token_operations.rs | Test token generation, validation, and lifecycle |
| user_management.rs | Test user CRUD operations and authentication |
| budget_tracking.rs | Test usage tracking and limit enforcement |

### Test Infrastructure

| Component | Responsibility |
|-----------|----------------|
| common/helpers.rs | Provide database setup functions |
| common/fixtures.rs | Build test data with explicit parameters |

### Seed Data Contract

[Document as per Seed Data Standards]
```

## Bug Fix Testing Standards

### Standard 1: Bug Reproducer Tests

**Requirement:** Every bug fix must include bug reproducer test.

**Test Marker:**
```rust
// test_kind: bug_reproducer(issue-NNN)
#[test]
fn test_describe_specific_bug() {
  // Test implementation
}
```

### Standard 2: Bug Documentation Format

**Requirement:** Bug reproducer must include 5-section documentation.

**Format:**
```rust
/// Reproduces [brief description] (issue-NNN).
///
/// ## Root Cause
/// [Specific technical explanation of why bug occurred]
///
/// ## Why Not Caught Initially
/// [Explanation of what was missing in original test coverage]
///
/// ## Fix Applied
/// [Specific technical description of fix]
///
/// ## Prevention
/// [What practices would have prevented this bug]
///
/// ## Pitfall to Avoid
/// [Specific technical lesson learned]
// test_kind: bug_reproducer(issue-NNN)
#[test]
fn test_specific_bug_description() { /* ... */ }
```

**Quality Requirements:**
- **Specific:** Name exact functions, types, conditions
- **Technical:** Include code, SQL, or algorithms
- **Actionable:** Clear guidance for preventing recurrence
- **Traceable:** Reference issue-NNN for full context
- **Concise:** Focus on critical insights only

### Standard 3: Source Code Fix Comment

**Requirement:** Fixed code must include 3-field comment.

**Format:**
```rust
// Fix(issue-NNN): [One sentence summary]
// Root cause: [Specific technical explanation]
// Pitfall: [Specific lesson learned]
pub fn fixed_function() {
  // Implementation
}
```

**Example:**
```rust
// Fix(issue-bcrypt-revert): Switch token hashing from BCrypt to SHA-256
// Root cause: BCrypt non-determinism (random salt) broke hash equality lookups
// Pitfall: BCrypt randomness is feature for passwords, breaks lookup-by-hash
pub fn hash_token(token: &str) -> String {
  use sha2::{Sha256, Digest};
  let mut hasher = Sha256::new();
  hasher.update(token.as_bytes());
  format!("{:x}", hasher.finalize())
}
```

## Integration with Existing Standards

This rulebook integrates with:

- **ADR-007 (Testing Philosophy):** No Mocking principle
- **docs/principles/004_testing_strategy.md:** TDD cycle and loud failures
- **codebase_hygiene.rulebook.md:** Disabled test format and fragile test detection
- **organizational_principles.rulebook.md:** Anti-duplication and unique responsibility

## Enforcement Checklist

Before merging code, verify:

- [ ] All tests use automatic cleanup (TempDir or pool drop)
- [ ] No shared database state between tests
- [ ] All seed functions are idempotent (INSERT OR IGNORE)
- [ ] tests/readme.md documents seed data contract
- [ ] Database paths follow conventions (dev_*.db, tempdir, :memory:)
- [ ] Bug fixes include 5-section test documentation + 3-field source comment
- [ ] All assertions include descriptive error messages
- [ ] No manual cleanup required
- [ ] Tests can run in parallel without conflicts

## Known Pitfalls

### Pitfall 1: Forgetting to Return TempDir

**Problem:** Returning only SqlitePool, TempDir dropped immediately.

```rust
// WRONG: TempDir dropped, database deleted before test uses it
pub async fn create_test_db() -> SqlitePool {
  let temp_dir = TempDir::new()?;
  let pool = /* connect to temp_dir */;
  pool  // temp_dir dropped HERE
}

// RIGHT: Return TempDir, kept alive during test
pub async fn create_test_db() -> (SqlitePool, TempDir) {
  let temp_dir = TempDir::new()?;
  let pool = /* connect to temp_dir */;
  (pool, temp_dir)  // temp_dir kept alive
}
```

### Pitfall 2: Shared In-Memory Database Without cache=shared

**Problem:** Multiple connections see different databases.

```rust
// WRONG: Each connection gets separate database
"sqlite::memory:"

// RIGHT: Connections share same in-memory database
"sqlite::memory:?cache=shared"
```

### Pitfall 3: Non-Idempotent Seed

**Problem:** Running seed twice causes unique constraint violations.

```rust
// WRONG: Second run fails
INSERT INTO users (username) VALUES ('admin');

// RIGHT: Second run is no-op
INSERT OR IGNORE INTO users (username) VALUES ('admin');
```

### Pitfall 4: Deleting in Wrong Order

**Problem:** Foreign key constraints violated during wipe.

```rust
// WRONG: Can't delete users with dependent api_tokens
DELETE FROM users;
DELETE FROM api_tokens;

// RIGHT: Delete dependents first
DELETE FROM api_tokens;
DELETE FROM users;
```

## Revision History

- **v1.0 (2025-12-11):** Initial version based on iron_token_manager and iron_control_api patterns
