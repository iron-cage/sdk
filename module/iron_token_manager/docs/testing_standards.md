# Database Testing Standards

## Overview

This document establishes comprehensive testing standards for the `iron_token_manager` module's database operations. These standards emerged from a multi-phase implementation effort to standardize paths, validation, and test data management.

**Purpose:**
- Ensure consistent database testing practices across all tests
- Prevent common pitfalls and anti-patterns
- Enable reliable, isolated, and maintainable tests
- Provide clear guidance for new test development

**Scope:** All database-related testing in `iron_token_manager`, including unit tests, integration tests, and validation scripts.

---

## Database Path Standards

### Path Conventions

All database paths must follow these standards to ensure portability, validation, and proper cleanup:

#### 1. Test Databases - In-Memory (Preferred)

**Standard:** `:memory:`

**Usage:**
```rust
let db = TestDatabase::new().await; // Uses :memory: by default
```

**Benefits:**
- Complete isolation between tests
- No filesystem cleanup required
- Fastest execution
- No path validation needed

**When to Use:**
- Unit tests
- Integration tests
- Any test not requiring persistence between runs

#### 2. Test Databases - Temporary Files

**Standard:** `/tmp/iron_token_manager_test_*.db`

**Pattern:** `/tmp/iron_token_manager_test_{random}.db`

**Usage:**
```rust
let temp_dir = tempfile::tempdir()?;
let db_path = temp_dir.path().join("test_db.db");
// Cleanup handled by tempfile::TempDir drop
```

**Benefits:**
- Test isolation via unique paths
- Automatic cleanup via RAII
- Suitable for debugging (inspect database after failure)

**When to Use:**
- Tests requiring database inspection
- Multi-process tests
- Performance benchmarks

#### 3. Development Database

**Standard:** `./iron.db` (relative to crate root)

**Usage:**
```bash
# Reset and seed dev database
./scripts/reset_and_seed.sh

# Seed existing dev database
./scripts/seed_dev_data.sh
```

**Benefits:**
- Consistent location for manual testing
- Easy to reset and seed
- Git-ignored by default

**When to Use:**
- Manual API testing
- Development iteration
- Script validation

#### 4. CI/CD Databases

**Standard:** `./target/test_db_{unique}.db`

**Pattern:** `./target/test_db_{timestamp}_{pid}.db`

**Benefits:**
- Isolated from source tree
- Cleaned by `cargo clean`
- Parallel test execution safe

**When to Use:**
- CI/CD pipelines
- Parallel test suites
- Build artifact isolation

### Path Validation

All database paths must pass validation before use:

**Validation Script:** `scripts/validate_db_paths.sh`

**Rules:**
1. Test paths must be in `/tmp/` or `:memory:`
2. Dev paths must be `./iron.db`
3. CI paths must be in `./target/`
4. No absolute paths outside `/tmp/`
5. No hardcoded user home paths

**Enforcement:**
- Pre-commit hook (Git)
- CI/CD validation step
- Manual validation via script

**Example:**
```bash
# Validate all database paths in codebase
./scripts/validate_db_paths.sh

# Expected output:
# ✓ Rule 1: No absolute paths outside /tmp (0 violations)
# ✓ Rule 2: Test files use :memory: or /tmp (0 violations)
# ✓ Rule 3: Dev scripts use ./iron.db (0 violations)
```

---

## Test Database Lifecycle

### Creation and Initialization

#### Standard Pattern (v2 Helpers)

**Recommended:** Use `iron_test_db` v2 helpers for automatic lifecycle management.

**Basic Setup:**
```rust
mod common;
use common::create_test_db_v2;

#[tokio::test]
async fn my_test() {
  let db = create_test_db_v2().await;
  // Database automatically initialized with migrations
  // Cleanup handled by Drop trait
}
```

**With Seed Data:**
```rust
use common::create_test_db_with_seed;

#[tokio::test]
async fn my_test_with_seed() {
  let db = create_test_db_with_seed().await;
  // Database has 5 users, 8 tokens, usage records, limits
}
```

**With Components:**
```rust
use common::{create_test_storage_v2, create_test_tracker_v2};

#[tokio::test]
async fn my_storage_test() {
  let (storage, _db) = create_test_storage_v2().await;
  // Storage uses shared pool with initialized database
}

#[tokio::test]
async fn my_tracker_test() {
  let (tracker, storage, _db) = create_test_tracker_v2().await;
  // All components share the same database pool
}
```

#### Benefits of v2 Helpers

1. **No Manual TempDir Management:** Automatic cleanup via RAII
2. **Shared Pool:** Multiple components use same database connection
3. **Consistent Initialization:** Migrations always applied in correct order
4. **Ergonomic API:** Single function call for full setup
5. **Test Isolation:** Each test gets independent database

### Migration Management

**Standard:** All test databases must apply migrations before use.

**Implementation:**
```rust
// Handled automatically by iron_test_db::TestDatabase
let db = TestDatabase::new().await;
// Migrations already applied
```

**Migration Order:**
1. Create tables (schema)
2. Create indexes
3. Create foreign keys
4. Seed data (if requested)

**Validation:**
```bash
# Validate schema matches migrations
./scripts/validate_db_schema.sh ./iron.db
```

### Cleanup and Isolation

#### Automatic Cleanup (Preferred)

**v2 Helpers:** Cleanup handled automatically via `Drop` trait.

```rust
#[tokio::test]
async fn test_with_auto_cleanup() {
  let db = create_test_db_v2().await;
  // Use database
  // Cleanup happens automatically when `db` goes out of scope
}
```

#### Manual Cleanup (Legacy)

**Only use when v2 helpers unavailable:**

```rust
#[tokio::test]
async fn test_with_manual_cleanup() {
  let temp_dir = tempfile::tempdir()?;
  let db_path = temp_dir.path().join("test.db");

  // Use database

  // Cleanup
  drop(temp_dir); // Removes directory and all contents
}
```

#### Test Isolation Guarantees

Each test must have complete isolation:

1. **Separate Database:** Never share databases between tests
2. **Independent Data:** No test should see another test's data
3. **Clean State:** Each test starts with known initial state
4. **No Side Effects:** Test execution order must not matter

**Example of Isolation:**
```rust
#[tokio::test]
async fn test_isolation_example() {
  let db = create_test_db_with_seed().await;

  // Delete all users (destructive operation)
  sqlx::query("DELETE FROM users")
    .execute(db.pool())
    .await?;

  // This deletion does NOT affect other tests!
  // Each test gets its own isolated database
}
```

---

## Seed Data Management

### Two-Tier Approach

The module uses a two-tier seed data strategy:

#### Tier 1: Bash Seed (Simple)

**Purpose:** Quick manual testing with minimal data

**Script:** `scripts/seed_dev_data.sh`

**Contents:**
- 3 users (admin, developer, viewer)
- 3 tokens (one per user)
- 7 usage records
- 3 usage limits

**Use Cases:**
- Manual API testing with curl
- Quick development iteration
- Basic scenario validation

**Command:**
```bash
./scripts/reset_and_seed.sh
```

#### Tier 2: Rust Seed (Comprehensive)

**Purpose:** Automated testing with edge cases

**Implementation:** `src/seed.rs`

**Contents:**
- 5 users (admin, developer, viewer, tester, guest)
- 8 tokens (various expiration states)
- 10+ usage records (diverse patterns)
- 3 usage limits

**Edge Cases Covered:**
- Expired tokens
- Users without tokens (guest)
- Users without limits (tester)
- Multiple tokens per user
- Various expiration states
- Mixed provider usage

**Use Cases:**
- Automated test suites
- Edge case validation
- Integration testing

**Command:**
```rust
// Called automatically in tests
seed::seed_all(&pool).await?;
```

### Seed Data Validation

**Script:** `scripts/validate_seed_data.sh`

**Acceptance Criteria:**
- Users: 3-5 (bash: 3, Rust: 5)
- Tokens: 3-8 (bash: 3, Rust: 8)
- Usage records: ≥7
- Usage limits: exactly 3
- Core users: admin, developer, viewer present
- Optional users: tester, guest (Rust only)

**Validation Command:**
```bash
./scripts/validate_seed_data.sh ./iron.db
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Seed Data Completeness Validator
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[INFO] Validating seed data in: ./iron.db

[INFO] Rule 1: Checking test user count (expect 3-5)...
[✓] User count correct: 5 (bash seed: 3, Rust seed: 5)

[INFO] Rule 2: Checking specific test users...
[✓] Test user exists: admin (role=admin, active=1)
[✓] Test user exists: developer (role=user, active=1)
[✓] Test user exists: viewer (role=user, active=0)
[✓] Test user exists: tester (role=user, active=1) [Rust seed]
[✓] Test user exists: guest (role=user, active=1) [Rust seed]

...

[✓] All seed data validations passed!
```

### Seed Data Reference

**Documentation:** [docs/seed_data_reference.md](./seed_data_reference.md)

Complete reference including:
- User profiles with purposes
- Token catalog with metadata
- Usage patterns
- Manual testing guide
- Test scenarios covered

### Idempotent Operations

Seed operations should fail on duplicate insertion to prevent accidental data duplication:

**Pattern:**
```sql
-- ❌ BAD: Silently creates duplicates
INSERT INTO users (username, role) VALUES ('admin', 'admin');

-- ✅ GOOD: Fails on duplicate (prevents accidental re-seeding)
INSERT INTO users (username, role) VALUES ('admin', 'admin');
-- Unique constraint violation on second call

-- ✅ ACCEPTABLE: Explicit idempotency when needed
INSERT OR IGNORE INTO users (username, role) VALUES ('admin', 'admin');
```

**Validation Test:**
```rust
#[tokio::test]
async fn validate_seed_idempotency() {
  let db = create_test_db_with_seed().await;

  // Attempt to seed again
  let result = seed::seed_all(db.pool()).await;

  assert!(
    result.is_err(),
    "Seeding twice should fail (prevents accidental data duplication)"
  );
}
```

### Wipe and Re-seed

**Use Case:** Testing database reset workflows

**Implementation:**
```rust
// Wipe database
iron_token_manager::seed::wipe_database(db.pool()).await?;

// Verify empty
let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
  .fetch_one(db.pool())
  .await?;
assert_eq!(count, 0);

// Re-seed
iron_token_manager::seed::seed_all(db.pool()).await?;

// Verify restored
let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
  .fetch_one(db.pool())
  .await?;
assert_eq!(count, 5);
```

**Why Manual Wipe Function:**

The module uses a custom `wipe_database()` function because SQLite doesn't support `TRUNCATE` and the foreign key relationships require careful deletion order:

1. Delete from tables with no dependencies first
2. Work up the dependency chain
3. Ensure referential integrity maintained

---

## Test Organization

### Test Location

**Standard:** All tests in `tests/` directory at crate root.

**Structure:**
```
tests/
├── common/
│   └── mod.rs           # Shared test helpers
├── database_initialization.rs
├── seed_data_validation.rs
├── storage.rs
├── tracker.rs
└── -example_v2_helpers.rs   # Example tests (temporary)
```

**Rules:**
1. No `#[cfg(test)]` modules in `src/` - use `tests/` directory
2. Shared helpers in `tests/common/mod.rs`
3. One test file per major component/feature
4. Temporary examples prefixed with `-` (git-ignored)

### Test Naming

**Standard:** Descriptive names following `validate_` or `test_` prefix.

**Examples:**
```rust
// ✅ GOOD: Descriptive, clear intent
#[tokio::test]
async fn validate_seeded_users_count() { }

#[tokio::test]
async fn validate_admin_user_properties() { }

#[tokio::test]
async fn test_create_token_with_project() { }

// ❌ BAD: Vague, unclear intent
#[tokio::test]
async fn test_users() { }

#[tokio::test]
async fn check_data() { }
```

### Test Documentation

**Standard:** All validation tests must have comprehensive doc comments.

**Format:**
```rust
#[tokio::test]
async fn validate_admin_unlimited_limits()
{
  // Doc comment explaining:
  // - What is being validated
  // - Why this matters
  // - Expected behavior

  let db = create_test_db_with_seed().await;

  let admin_limit: (String, Option<i64>, Option<i64>, Option<i64>) =
    sqlx::query_as(
      "SELECT user_id, max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month
       FROM usage_limits
       WHERE user_id = 'admin'"
    )
    .fetch_one(db.pool())
    .await
    .expect("LOUD FAILURE: Admin usage limit should exist");

  assert_eq!(admin_limit.0, "admin", "User ID should match");
  assert_eq!(admin_limit.1, None, "Max tokens should be UNLIMITED (NULL)");
  assert_eq!(admin_limit.2, None, "Max requests should be UNLIMITED (NULL)");
  assert_eq!(admin_limit.3, None, "Max cost should be UNLIMITED (NULL)");
}
```

### Loud Failures

**Standard:** All test failures must be loud and descriptive.

**Pattern:**
```rust
// ✅ GOOD: Loud failure with context
.expect("LOUD FAILURE: Admin user should exist");

assert_eq!(
  user_count, 5,
  "LOUD FAILURE: Seed should create exactly 5 users (admin, developer, viewer, tester, guest)"
);

// ❌ BAD: Silent or vague failure
.expect("Failed");

assert_eq!(user_count, 5);
```

**Benefits:**
- Immediate identification of what broke
- No need to read test code to understand failure
- Clear expected vs actual values
- Context about why assertion matters

---

## Foreign Key Integrity

### Validation Requirements

All test data must maintain foreign key integrity:

**Validation Test Pattern:**
```rust
#[tokio::test]
async fn validate_foreign_key_integrity()
{
  let db = create_test_db_with_seed().await;

  // Check provider keys reference valid users
  let orphaned_keys: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM ai_provider_keys pk
     WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.username = pk.user_id)"
  )
  .fetch_one(db.pool())
  .await
  .expect("LOUD FAILURE: Failed to check orphaned provider keys");

  assert_eq!(orphaned_keys, 0, "No provider keys should be orphaned");

  // Check tokens reference valid users
  let orphaned_tokens: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM api_tokens t
     WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.username = t.user_id)"
  )
  .fetch_one(db.pool())
  .await
  .expect("LOUD FAILURE: Failed to check orphaned tokens");

  assert_eq!(orphaned_tokens, 0, "No tokens should be orphaned");
}
```

### Cascade Behavior

**Standard:** Define cascade behavior explicitly in schema.

**Example:**
```sql
CREATE TABLE api_tokens (
  id INTEGER PRIMARY KEY,
  user_id TEXT NOT NULL,
  token_hash TEXT NOT NULL,

  -- Explicit cascade: deleting user deletes their tokens
  FOREIGN KEY (user_id) REFERENCES users(username) ON DELETE CASCADE
);
```

**Alternatives:**
- `ON DELETE CASCADE` - Delete child records
- `ON DELETE RESTRICT` - Prevent deletion if children exist
- `ON DELETE SET NULL` - Set foreign key to NULL

**Selection Criteria:**
- Use `CASCADE` for owned relationships (user → tokens)
- Use `RESTRICT` for referenced relationships (token → usage_records)
- Use `SET NULL` for optional relationships

---

## Validation and Enforcement

### Validation Layers

The module implements three validation layers:

#### Layer 1: Pre-commit Hooks

**Purpose:** Catch violations before commit

**Script:** `scripts/pre-commit.hook`

**Checks:**
- Database path compliance
- Seed data completeness (if dev db exists)
- Schema validation

**Installation:**
```bash
# Symlink to .git/hooks/pre-commit
ln -s ../../scripts/pre-commit.hook .git/hooks/pre-commit
```

**Behavior:**
- Violations block commit
- Provides clear error messages
- Suggests fixes

#### Layer 2: CI/CD Pipeline

**Purpose:** Enforce standards in automation

**Checks:**
- All Layer 1 checks
- Full test suite (120 tests)
- Clippy warnings as errors
- Documentation tests

**Implementation:**
```yaml
# .github/workflows/test.yml (example)
- name: Validate Database Paths
  run: ./scripts/validate_db_paths.sh

- name: Validate Database Schema
  run: ./scripts/validate_db_schema.sh ./target/test_db.db

- name: Run Test Suite
  run: cargo test --all-features
```

#### Layer 3: Manual Validation

**Purpose:** On-demand verification

**Scripts:**
- `validate_db_paths.sh` - Path compliance
- `validate_db_schema.sh` - Schema correctness
- `validate_seed_data.sh` - Seed data completeness

**Usage:**
```bash
# Validate all aspects
./scripts/validate_db_paths.sh
./scripts/validate_db_schema.sh ./iron.db
./scripts/validate_seed_data.sh ./iron.db
```

### Enforcement Philosophy

**Principle:** Prevent violations rather than fix them.

**Strategy:**
1. **Make correct usage easy:** v2 helpers, clear patterns
2. **Make incorrect usage hard:** Validation scripts, pre-commit hooks
3. **Make violations obvious:** Loud failures, clear error messages
4. **Provide guidance:** Documentation, examples, suggestions

**Example:**
```bash
# Violation detected
[✗] Absolute path outside /tmp: src/config.rs:127

# Guidance provided
To fix:
  - Move database to /tmp/ for tests
  - Use :memory: for unit tests
  - Use ./iron.db for development

See: docs/database_path_standards.md
```

---

## Best Practices

### 1. Prefer In-Memory Databases

**Rationale:** Fastest, most isolated, no cleanup needed

```rust
// ✅ PREFERRED
let db = create_test_db_v2().await; // Uses :memory:

// ⚠️ ONLY WHEN NEEDED
let temp_dir = tempfile::tempdir()?;
let db_path = temp_dir.path().join("test.db");
```

### 2. Use v2 Helpers Consistently

**Rationale:** Eliminates boilerplate, ensures consistency

```rust
// ✅ GOOD: v2 helper
let db = create_test_db_with_seed().await;

// ❌ BAD: Manual setup
let temp_dir = tempfile::tempdir()?;
let db_path = temp_dir.path().join("test.db");
let pool = SqlitePoolOptions::new()
  .connect(&format!("sqlite://{}", db_path.display()))
  .await?;
sqlx::migrate!().run(&pool).await?;
seed::seed_all(&pool).await?;
```

### 3. Test One Thing Per Test

**Rationale:** Clear failures, easy debugging

```rust
// ✅ GOOD: Single responsibility
#[tokio::test]
async fn validate_admin_user_properties() {
  // Only tests admin user properties
}

#[tokio::test]
async fn validate_developer_user_properties() {
  // Only tests developer user properties
}

// ❌ BAD: Multiple responsibilities
#[tokio::test]
async fn validate_all_users() {
  // Tests admin, developer, viewer, tester, guest
  // Failures are ambiguous
}
```

### 4. Use Descriptive Assertions

**Rationale:** Self-documenting tests, clear failures

```rust
// ✅ GOOD: Descriptive message
assert_eq!(
  user_count, 5,
  "Should have 5 seeded users (admin, developer, viewer, tester, guest)"
);

// ❌ BAD: No context
assert_eq!(user_count, 5);
```

### 5. Validate Foreign Keys

**Rationale:** Catch referential integrity issues early

```rust
#[tokio::test]
async fn validate_no_orphaned_records() {
  let db = create_test_db_with_seed().await;

  // Check each foreign key relationship
  let orphaned_tokens = check_orphaned_tokens(db.pool()).await;
  assert_eq!(orphaned_tokens, 0, "No orphaned tokens");

  let orphaned_usage = check_orphaned_usage(db.pool()).await;
  assert_eq!(orphaned_usage, 0, "No orphaned usage records");
}
```

### 6. Document Edge Cases

**Rationale:** Preserve knowledge, prevent regression

```rust
#[tokio::test]
async fn validate_expired_token_is_expired()
{
  // Edge case: Token marked active but expired by timestamp
  // This tests that expiration is based on timestamp, not is_active flag

  let db = create_test_db_with_seed().await;
  let now_ms = current_timestamp_ms();

  let expired_token: (String, i64, Option<i64>) = sqlx::query_as(
    "SELECT name, is_active, expires_at FROM api_tokens WHERE name = 'Expired Token'"
  )
  .fetch_one(db.pool())
  .await
  .expect("LOUD FAILURE: Expired Token should exist");

  assert_eq!(expired_token.1, 1, "Token should be marked active (but expired)");

  let expires_at = expired_token.2.expect("Should have expires_at timestamp");
  assert!(
    expires_at < now_ms,
    "LOUD FAILURE: Token should have expired 30 days ago"
  );
}
```

### 7. Avoid Test Interdependencies

**Rationale:** Tests must run in any order

```rust
// ✅ GOOD: Independent tests
#[tokio::test]
async fn test_a() {
  let db = create_test_db_with_seed().await;
  // Completely independent
}

#[tokio::test]
async fn test_b() {
  let db = create_test_db_with_seed().await;
  // Completely independent
}

// ❌ BAD: Shared state
static mut SHARED_DB: Option<Pool<Sqlite>> = None;

#[tokio::test]
async fn test_a() {
  unsafe { SHARED_DB = Some(setup_db().await); }
}

#[tokio::test]
async fn test_b() {
  let db = unsafe { SHARED_DB.as_ref().unwrap() };
  // Order-dependent!
}
```

---

## Anti-Patterns to Avoid

### 1. Hardcoded Paths

**Problem:** Not portable, validation failures

```rust
// ❌ BAD: Hardcoded absolute path
let db_path = "/home/user/test.db";

// ❌ BAD: Hardcoded relative path
let db_path = "../../../test.db";

// ✅ GOOD: Use helpers
let db = create_test_db_v2().await;

// ✅ ACCEPTABLE: Temporary directory
let temp_dir = tempfile::tempdir()?;
let db_path = temp_dir.path().join("test.db");
```

### 2. Shared Test Databases

**Problem:** Tests affect each other, order-dependent failures

```rust
// ❌ BAD: Shared database
static DB: OnceCell<TestDatabase> = OnceCell::new();

#[tokio::test]
async fn test_a() {
  let db = DB.get_or_init(|| create_test_db_v2().await);
  // Mutations visible to test_b!
}

// ✅ GOOD: Independent databases
#[tokio::test]
async fn test_a() {
  let db = create_test_db_v2().await;
  // Isolated
}
```

### 3. Silent Failures

**Problem:** Tests pass when they should fail

```rust
// ❌ BAD: Silent failure
#[tokio::test]
async fn test_integration() {
  if env::var("API_KEY").is_err() {
    return; // Test passes without running!
  }
  // actual test...
}

// ✅ GOOD: Loud failure
#[tokio::test]
async fn test_integration() {
  let api_key = env::var("API_KEY")
    .expect("LOUD FAILURE: API_KEY required for integration test");
  // actual test...
}
```

### 4. No Cleanup

**Problem:** Disk space leaks, test artifacts

```rust
// ❌ BAD: No cleanup
#[tokio::test]
async fn test_with_file_db() {
  let db_path = "/tmp/test.db";
  let pool = create_pool(db_path).await;
  // File leaks!
}

// ✅ GOOD: Automatic cleanup
#[tokio::test]
async fn test_with_file_db() {
  let temp_dir = tempfile::tempdir()?;
  let db_path = temp_dir.path().join("test.db");
  let pool = create_pool(&db_path).await;
  // Cleanup when temp_dir drops
}

// ✅ BEST: Use :memory:
#[tokio::test]
async fn test_with_memory_db() {
  let db = create_test_db_v2().await;
  // No files, no cleanup needed
}
```

### 5. Skipped Tests

**Problem:** Tests disabled don't catch regressions

```rust
// ❌ BAD: Skipped test
#[tokio::test]
#[ignore] // "Temporarily" disabled 6 months ago
async fn test_important_feature() {
  // Never runs!
}

// ✅ GOOD: Fix or remove
#[tokio::test]
async fn test_important_feature() {
  // Runs on every test execution
}
```

### 6. Insufficient Assertions

**Problem:** Tests pass but don't validate behavior

```rust
// ❌ BAD: No assertions
#[tokio::test]
async fn test_create_user() {
  let db = create_test_db_v2().await;
  create_user(&db, "test").await; // Could fail silently!
}

// ✅ GOOD: Verify behavior
#[tokio::test]
async fn test_create_user() {
  let db = create_test_db_v2().await;
  let user_id = create_user(&db, "test").await?;

  // Verify user exists
  let exists = user_exists(&db, user_id).await?;
  assert!(exists, "User should exist after creation");

  // Verify properties
  let user = get_user(&db, user_id).await?;
  assert_eq!(user.username, "test");
}
```

### 7. Fragile Tests

**Problem:** Tests break from unrelated changes

```rust
// ❌ BAD: Depends on exact ordering
#[tokio::test]
async fn test_list_users() {
  let db = create_test_db_with_seed().await;
  let users = list_users(&db).await?;

  assert_eq!(users[0].username, "admin");
  assert_eq!(users[1].username, "developer");
  // Breaks if seed order changes!
}

// ✅ GOOD: Order-independent
#[tokio::test]
async fn test_list_users() {
  let db = create_test_db_with_seed().await;
  let users = list_users(&db).await?;

  let usernames: Vec<_> = users.iter().map(|u| &u.username).collect();
  assert!(usernames.contains(&"admin"));
  assert!(usernames.contains(&"developer"));
}
```

---

## Migration and Schema Management

### Migration Files

**Location:** `migrations/` directory at crate root

**Naming:** `{timestamp}_{description}.sql`

**Example:**
```
migrations/
├── 20231201_001_create_users.sql
├── 20231201_002_create_api_tokens.sql
├── 20231201_003_create_token_usage.sql
└── 20231201_004_create_usage_limits.sql
```

### Schema Validation

**Script:** `scripts/validate_db_schema.sh`

**Checks:**
1. All expected tables exist
2. All expected columns exist with correct types
3. All indexes exist
4. All foreign keys exist

**Usage:**
```bash
./scripts/validate_db_schema.sh ./iron.db
```

**Output:**
```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Database Schema Validator
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

[INFO] Validating schema in: ./iron.db

[INFO] Checking table: users
[✓] Table exists: users
[✓] Column: username (TEXT)
[✓] Column: password_hash (TEXT)
[✓] Column: role (TEXT)
[✓] Column: is_active (INTEGER)

[INFO] Checking table: api_tokens
[✓] Table exists: api_tokens
...

[✓] All schema validations passed!
```

### Migration Testing

**Pattern:** Test migrations in isolation

```rust
#[tokio::test]
async fn test_migrations_apply_cleanly() {
  let db = TestDatabase::new().await;
  // Migrations applied during construction

  // Verify schema
  let table_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table'"
  )
  .fetch_one(db.pool())
  .await?;

  assert!(table_count >= 4, "Should have at least 4 tables");
}

#[tokio::test]
async fn test_migrations_idempotent() {
  let db = TestDatabase::new().await;

  // Attempt to apply migrations again (should succeed or be no-op)
  let result = sqlx::migrate!().run(db.pool()).await;

  assert!(result.is_ok(), "Re-applying migrations should be safe");
}
```

---

## Performance Considerations

### Test Execution Speed

**Optimization Strategies:**

1. **Use :memory: databases** - 10-100x faster than file-based
2. **Parallel execution** - Run tests concurrently with `cargo nextest`
3. **Minimal seed data** - Only seed what test needs
4. **Connection pooling** - Reuse connections within test

**Benchmarks (approximate):**
- In-memory database creation: ~5ms
- File-based database creation: ~50ms
- Migration application: ~10ms
- Seed data (5 users, 8 tokens): ~5ms

### Database Connection Limits

SQLite has connection limits based on configuration:

**Recommended Pool Size:**
```rust
// For tests
let pool = SqlitePoolOptions::new()
  .max_connections(1) // SQLite works best with single connection per db
  .connect("sqlite::memory:")
  .await?;
```

**Why Single Connection:**
- SQLite locks entire database on writes
- Multiple connections don't improve concurrency
- Single connection simpler, faster

---

## Troubleshooting

### Common Issues

#### Issue 1: "Database is locked"

**Cause:** Multiple connections attempting writes

**Solution:** Use single connection per database
```rust
let pool = SqlitePoolOptions::new()
  .max_connections(1) // Fix: Single connection
  .connect(db_path)
  .await?;
```

#### Issue 2: "Foreign key constraint failed"

**Cause:** Inserting child record before parent

**Solution:** Insert in dependency order
```rust
// 1. Insert user first
create_user(&db, "test").await?;

// 2. Then insert token (references user)
create_token(&db, "test", "token_hash").await?;
```

#### Issue 3: "Table already exists"

**Cause:** Migrations applied twice

**Solution:** Check migration table
```sql
SELECT * FROM _sqlx_migrations;
```

#### Issue 4: Test passes locally, fails in CI

**Cause:** Absolute paths, missing environment variables

**Solution:**
1. Use v2 helpers (no paths)
2. Validate paths with `validate_db_paths.sh`
3. Make env vars explicit with loud failures

#### Issue 5: Tests affect each other

**Cause:** Shared database state

**Solution:** Each test creates own database
```rust
// ✅ Each test isolated
#[tokio::test]
async fn test_a() {
  let db = create_test_db_v2().await; // New database
}

#[tokio::test]
async fn test_b() {
  let db = create_test_db_v2().await; // New database
}
```

### Debug Techniques

#### 1. Inspect Database Contents

```rust
#[tokio::test]
async fn debug_database_state() {
  let db = create_test_db_with_seed().await;

  // Dump all users
  let users: Vec<(String, String)> = sqlx::query_as(
    "SELECT username, role FROM users"
  )
  .fetch_all(db.pool())
  .await?;

  dbg!(users); // Print for inspection
}
```

#### 2. Use File-Based DB for Inspection

```rust
#[tokio::test]
async fn debug_with_file_db() {
  let temp_dir = tempfile::tempdir()?;
  let db_path = temp_dir.path().join("debug.db");

  // Create database
  let db = TestDatabase::new_with_path(&db_path).await;

  // Run test...

  // Inspect with: sqlite3 /tmp/.../debug.db
  println!("Database at: {}", db_path.display());
  std::thread::sleep(std::time::Duration::from_secs(60)); // Pause for inspection
}
```

#### 3. Enable SQLite Logging

```rust
// In test setup
std::env::set_var("RUST_LOG", "sqlx=debug");
env_logger::init();
```

---

## Related Documentation

- [Database Initialization Guide](./database_initialization.md) - Schema setup and migrations
- [Database Path Standards](./database_path_standards.md) - Path conventions and validation
- [Seed Data Reference](./seed_data_reference.md) - Complete seed data catalog
- [Test Database Troubleshooting](./test_database_troubleshooting.md) - Common issues and solutions
- [Quick Reference](./quick_reference_database.md) - Command cheat sheet

---

## Version History

**v1.0 (2024-12)** - Initial standards document
- Database path standards
- Test database lifecycle
- Seed data management (two-tier approach)
- Validation and enforcement
- Best practices and anti-patterns
- Comprehensive troubleshooting guide

---

## Appendix: Quick Reference

### Test Setup Patterns

```rust
// Basic test
#[tokio::test]
async fn basic_test() {
  let db = create_test_db_v2().await;
  // Use db.pool()
}

// Test with seed data
#[tokio::test]
async fn test_with_seed() {
  let db = create_test_db_with_seed().await;
  // 5 users, 8 tokens available
}

// Test with storage
#[tokio::test]
async fn test_storage() {
  let (storage, _db) = create_test_storage_v2().await;
  // Use storage
}

// Test with tracker
#[tokio::test]
async fn test_tracker() {
  let (tracker, storage, _db) = create_test_tracker_v2().await;
  // Use tracker and storage
}
```

### Validation Commands

```bash
# Validate database paths
./scripts/validate_db_paths.sh

# Validate database schema
./scripts/validate_db_schema.sh ./iron.db

# Validate seed data
./scripts/validate_seed_data.sh ./iron.db

# Run all validations
./scripts/validate_db_paths.sh && \
./scripts/validate_db_schema.sh ./iron.db && \
./scripts/validate_seed_data.sh ./iron.db
```

### Development Commands

```bash
# Reset and seed development database
./scripts/reset_and_seed.sh

# Seed existing database (bash seed)
./scripts/seed_dev_data.sh

# Fresh start (wipe + seed)
rm -f ./iron.db && ./scripts/reset_and_seed.sh
```

### Test Execution

```bash
# Run all tests
cargo test --all-features

# Run specific test file
cargo test --test seed_data_validation

# Run with output
cargo test -- --nocapture

# Run with nextest (parallel)
cargo nextest run --all-features
```
