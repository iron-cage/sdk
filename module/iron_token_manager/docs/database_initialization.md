# Database Initialization Practices

Comprehensive guide for database initialization, testing, and development workflows in `iron_token_manager`.

## Table of Contents

1. [Overview](#overview)
2. [Development Database Management](#development-database-management)
3. [Test Database Practices](#test-database-practices)
4. [Production Database Setup](#production-database-setup)
5. [Common Patterns](#common-patterns)
6. [Troubleshooting](#troubleshooting)

---

## Overview

The `iron_token_manager` crate uses **SQLite** for token storage, usage tracking, and limit enforcement. To avoid confusion and ensure consistent practices across the development team, this document establishes canonical patterns for database initialization in all contexts.

### Core Principles

1. **Isolation**: Every test gets an isolated temporary database
2. **Consistency**: Test databases must match production schema exactly
3. **Idempotency**: Scripts can be run multiple times safely
4. **Safety**: Destructive operations create backups first
5. **Simplicity**: One-command workflows for common tasks

### Quick Reference

```bash
# Development Database
make db-reset-seed          # Wipe + populate test data (one command)
make db-reset               # Wipe only (creates backup)
make db-seed                # Populate test data only

# Testing
make test                   # Run all tests (Level 3)
make test-quick             # Unit tests only (Level 1)
cargo nextest run           # Alternative test runner

# Verification
make clippy                 # Lint check
make check                  # Fast compilation check
```

---

## Development Database Management

### Default Database Location

Development database: `./dev_tokens.db`

Override with environment variable:
```bash
DB_PATH=/path/to/custom.db make db-reset-seed
```

### Reset + Seed Workflow (Recommended)

**Command:**
```bash
make db-reset-seed
```

**What it does:**
1. Creates backup: `./backups/dev_tokens_backup_YYYYMMDD_HHMMSS.db`
2. Deletes current database
3. Applies all migrations (001-008, skipping 007)
4. Populates test data:
   - 3 users (admin, project_manager, viewer)
   - 3 API tokens (one per user)
   - 7 usage records (for admin token)
   - 3 usage limits (one per user)

**Test Credentials:**
```
Users:
  - admin (role: admin, is_active: 1)
  - project_manager (role: user, is_active: 1)
  - viewer (role: viewer, is_active: 1)

API Tokens (save these for manual testing):
  - Admin:   iron_dev_admin_token_001
  - PM:      iron_dev_pm_token_002
  - Viewer:  iron_dev_viewer_token_003

Projects:
  - project_alpha (assigned to admin)
  - project_beta (assigned to pm + viewer)
```

### Reset Only (No Seed Data)

**Command:**
```bash
make db-reset
```

**Use case:** When you need a clean schema without test data.

**Safety:** Creates timestamped backup before deletion.

### Seed Data Only

**Command:**
```bash
make db-seed
```

**Use case:** Populate existing database with test data.

**Idempotency:** Uses `INSERT OR IGNORE` - safe to run multiple times.

### Manual Database Inspection

```bash
# Open SQLite shell
sqlite3 dev_tokens.db

# Count tables
sqlite> SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND substr(name,1,1) != '_';
# Expected: 11

# Count indexes
sqlite> SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%';
# Expected: 32

# List users
sqlite> SELECT username, role, is_active FROM users;

# List tokens
sqlite> SELECT name, user_id, project_id, is_active FROM api_tokens;

# Exit
sqlite> .quit
```

### Backup Location

All backups saved to: `./backups/`

**Format:** `dev_tokens_backup_YYYYMMDD_HHMMSS.db`

**Restore backup:**
```bash
cp ./backups/dev_tokens_backup_20241211_143000.db ./dev_tokens.db
```

---

## Test Database Practices

### Isolated Temporary Databases

**Every test gets its own isolated SQLite database.**

**Example:**
```rust
mod common;
use common::create_test_db;

#[tokio::test]
async fn test_example()
{
  let ( pool, _temp ) = create_test_db().await;

  // Database is:
  // - Created in temporary directory
  // - Schema applied (all 11 tables, 32 indexes)
  // - Foreign keys enabled
  // - Isolated from all other tests

  // ... your test code ...

} // _temp drops here → database automatically deleted
```

### Test Helper Functions

Located in: `tests/common/mod.rs`

**Available helpers:**

1. **`create_test_db()`** - Basic database pool
   ```rust
   let ( pool, _temp ) = create_test_db().await;
   // Returns: (SqlitePool, TempDir)
   ```

2. **`create_test_storage()`** - TokenStorage instance
   ```rust
   let ( storage, _temp ) = create_test_storage().await;
   // Returns: (TokenStorage, TempDir)
   ```

3. **`create_test_enforcer()`** - LimitEnforcer instance
   ```rust
   let ( enforcer, storage, _temp ) = create_test_enforcer().await;
   // Returns: (LimitEnforcer, TokenStorage, TempDir)
   ```

4. **`create_test_tracker()`** - UsageTracker instance
   ```rust
   let ( tracker, storage, _temp ) = create_test_tracker().await;
   // Returns: (UsageTracker, TokenStorage, TempDir)
   ```

### Schema Consistency Guarantee

**Test databases use the EXACT same migration files as production.**

Migration helper: `src/migrations.rs`

```rust
use iron_token_manager::migrations;

// Apply all migrations (idempotent)
migrations::apply_all_migrations( &pool ).await?;
```

**Migrations applied:**
- 001: Initial schema (5 tables, 15 indexes)
- 002: Length constraints (guarded - prevents CASCADE DELETE)
- 003: Users table (2 tables, 2 indexes)
- 004: AI provider keys (2 tables, 4 indexes)
- 005: User enhancements (4 indexes)
- 006: User audit log (1 table, 4 indexes)
- 007: RESERVED (intentionally skipped)
- 008: Agents support (1 table, 3 indexes)

**Total:** 11 tables, 32 indexes

### NO MOCKING Philosophy (ADR-007)

**We use REAL databases in tests, NOT mocks.**

❌ **Don't:**
```rust
// Mock database
let mock_db = MockDatabase::new();
```

✅ **Do:**
```rust
// Real SQLite in-memory database
let ( pool, _temp ) = create_test_db().await;
```

**Why?**
- Tests catch real schema issues
- Tests verify SQL syntax
- Tests validate foreign key constraints
- Tests match production behavior

### Test Isolation Verification

Test file: `tests/database_initialization.rs`

```rust
#[tokio::test]
async fn test_isolated_test_databases()
{
  let ( pool1, _temp1 ) = create_test_db().await;
  let ( pool2, _temp2 ) = create_test_db().await;

  // Insert into DB1
  sqlx::query( "INSERT INTO api_tokens ..." ).execute( &pool1 ).await?;

  // Verify DB2 is empty (isolated)
  let count: i64 = query_scalar( "SELECT COUNT(*) FROM api_tokens" )
    .fetch_one( &pool2 ).await?;
  assert_eq!( count, 0 );
}
```

### Automatic Cleanup

Databases are **automatically deleted** when `TempDir` goes out of scope:

```rust
{
  let ( pool, _temp ) = create_test_db().await;
  // Database file exists here
} // _temp dropped → database deleted
```

**No manual cleanup needed!**

---

## Production Database Setup

### Configuration File (Recommended)

**Production deployments should use config files for database setup.**

#### Step 1: Create Production Config

```bash
# Copy template
cp config.prod.toml.example config.prod.toml

# Edit with your settings
vim config.prod.toml
```

#### Step 2: Configure Database

```toml
# config.prod.toml
[database]
url = "sqlite:///./data/tokens.db?mode=rwc"
max_connections = 10
auto_migrate = true
foreign_keys = true

[production]
debug = false
auto_seed = false
```

#### Step 3: Initialize from Config

```rust
use iron_token_manager::storage::TokenStorage;

// Method 1: Load from IRON_ENV (recommended)
std::env::set_var("IRON_ENV", "production");
let storage = TokenStorage::from_config().await?;

// Method 2: Load specific config object
use iron_token_manager::config::Config;
let config = Config::from_file("config.prod.toml")?;
let storage = TokenStorage::from_config_object(&config).await?;
```

**What happens:**
1. Loads config from file (`config.{IRON_ENV}.toml`)
2. Applies environment variable overrides (if any)
3. Connects with configured max_connections
4. Enables foreign keys (`PRAGMA foreign_keys = ON`)
5. Applies migrations if `auto_migrate = true`
6. Returns ready-to-use `TokenStorage` instance

### Database URL Format (Legacy)

```
sqlite:///path/to/database.db?mode=rwc
```

**Parameters:**
- `mode=rwc` - Read/Write/Create (required)
- Max connections: 5 (hardcoded for legacy `new()` method)

### Legacy Direct URL Initialization

```rust
use iron_token_manager::storage::TokenStorage;

// Create storage with hardcoded URL (not recommended for production)
let storage = TokenStorage::new( "sqlite:///data/tokens.db?mode=rwc" ).await?;

// Schema is ready!
```

**Note:** This method is maintained for backward compatibility. Use `from_config()` for production deployments.

### Environment Variable Overrides

All config values can be overridden via environment variables:

```bash
# Override database URL
export DATABASE_URL="sqlite:///custom/path.db?mode=rwc"

# Override max connections
export DATABASE_MAX_CONNECTIONS=20

# Disable auto-migration
export DATABASE_AUTO_MIGRATE=false

# Then load config (will use overrides)
let storage = TokenStorage::from_config().await?;
```

### Migration Guard Pattern

**Destructive migrations are protected by guard tables.**

**Example:** Migration 002 (adds length constraints)

```sql
-- Check if migration already applied
SELECT COUNT(*) FROM sqlite_master
WHERE type='table' AND name='_migration_002_completed';

-- If count = 0, run migration and create guard table
CREATE TABLE _migration_002_completed (applied_at INTEGER NOT NULL);
INSERT INTO _migration_002_completed (applied_at) VALUES (strftime('%s', 'now') * 1000);
```

**Why?**
- Prevents CASCADE DELETE data loss (issue-003)
- Allows idempotent migration runs
- Safe to call `apply_all_migrations()` multiple times

### Foreign Keys Enforcement

**Foreign keys are ENABLED by default.**

```sql
PRAGMA foreign_keys = ON;
```

**What this means:**
- Deleting a token → cascades to `token_usage` (automatic cleanup)
- Invalid `token_id` in `token_usage` → INSERT fails
- Data integrity enforced at schema level

**Verification:**
```rust
let fk_on: i64 = query_scalar( "PRAGMA foreign_keys" ).fetch_one( &pool ).await?;
assert_eq!( fk_on, 1 );
```

---

## Common Patterns

### Pattern 1: Manual Testing Setup

```bash
# Reset database + populate test data
make db-reset-seed

# Test API with curl
curl -H "Authorization: Bearer iron_dev_admin_token_001" \
     http://localhost:3000/api/tokens/usage
```

### Pattern 2: Debug Test Failure

```bash
# Run specific test with output
cargo test test_name -- --nocapture

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run with nextest (better output)
cargo nextest run test_name
```

### Pattern 3: Inspect Test Database

Tests use temporary directories - they're deleted automatically. To inspect:

```rust
#[tokio::test]
async fn test_debug_database()
{
  let ( pool, temp ) = create_test_db().await;

  // Print database path
  let db_path = temp.path().join( "test.db" );
  println!( "Database: {}", db_path.display() );

  // ... test code ...

  // Keep database alive for inspection
  std::thread::sleep( std::time::Duration::from_secs( 300 ) );
}
```

Then open in another terminal:
```bash
sqlite3 /tmp/.tmp<random>/test.db
```

### Pattern 4: Verify Migration State

```rust
// Check all guard tables exist
let guards = vec![
  "_migration_002_completed",
  "_migration_003_completed",
  // ...
];

for guard in guards {
  let exists: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE name = $1"
  )
  .bind( guard )
  .fetch_one( &pool )
  .await?;

  assert_eq!( exists, 1, "Guard table {guard} should exist" );
}
```

### Pattern 5: Test Seed Data Locally

```bash
# Create test database
./scripts/reset_and_seed.sh /tmp/test.db

# Verify data
sqlite3 /tmp/test.db "SELECT COUNT(*) FROM users;"
# Expected: 3

sqlite3 /tmp/test.db "SELECT COUNT(*) FROM api_tokens;"
# Expected: 3

# Clean up
rm /tmp/test.db
```

---

## Troubleshooting

### Issue: Tests fail with "database is locked"

**Cause:** SQLite doesn't support high concurrency.

**Solution:** Tests are isolated - this shouldn't happen. If it does:
```bash
# Check for zombie processes
ps aux | grep cargo

# Kill all cargo processes
pkill -9 cargo

# Run tests again
make test
```

### Issue: Seed script fails with "table has no column"

**Cause:** Seed script schema doesn't match migration schema.

**Solution:** Verify seed script matches migration files:
```bash
# Check actual schema
sqlite3 dev_tokens.db .schema users

# Compare with seed script
grep "INSERT INTO users" scripts/seed_dev_data.sh
```

**Fix:** Update seed script to match actual schema (see `migrations/*.sql`).

### Issue: Migration guard table already exists

**Cause:** Migration was partially applied.

**Solution:** Migrations are idempotent - just run again:
```rust
migrations::apply_all_migrations( &pool ).await?; // Safe to call multiple times
```

If still failing:
```bash
# Drop guard table manually
sqlite3 dev_tokens.db "DROP TABLE IF EXISTS _migration_XXX_completed;"

# Run migration again
make db-reset
```

### Issue: Test database not cleaned up

**Cause:** `TempDir` wasn't dropped (likely due to panic).

**Solution:** Temporary directories are in `/tmp/` - they'll be cleaned by OS eventually.

**Manual cleanup:**
```bash
find /tmp -name ".tmp*" -type d -mtime +1 -exec rm -rf {} \;
```

### Issue: Foreign key constraint violation

**Cause:** Trying to insert invalid foreign key reference.

**Example:**
```sql
INSERT INTO token_usage (token_id, ...) VALUES (999, ...);
-- Error: no token with id=999
```

**Solution:** Create parent record first:
```rust
// Create token first
let token_id = storage.create_token( &token, "user_001", None, None, None, None ).await?;

// Then create usage record
tracker.record_usage( token_id, "openai", "gpt-4", 100, 50, 150 ).await?;
```

### Issue: Index count mismatch

**Expected:** 32 indexes (across all migrations)

**Verification:**
```bash
sqlite3 dev_tokens.db \
  "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%';"
```

**If wrong count:**
1. Reset database: `make db-reset`
2. Verify migrations applied: check for guard tables
3. Re-run tests: `make test`

### Issue: Production database missing tables

**Cause:** Migrations not applied during initialization.

**Solution:** Ensure `TokenStorage::new()` is called (applies migrations automatically):
```rust
// This applies all migrations
let storage = TokenStorage::new( database_url ).await?;
```

**Manual migration:**
```rust
use iron_token_manager::migrations;
use sqlx::SqlitePool;

let pool = SqlitePool::connect( database_url ).await?;
migrations::apply_all_migrations( &pool ).await?;
```

---

## Validation Tests

Automated validation tests ensure these practices are followed:

**Test file:** `tests/database_initialization.rs`

**Coverage:**
- ✅ Migration idempotency (run 3x → same result)
- ✅ Test isolation (DB1 changes don't affect DB2)
- ✅ Schema consistency (production == test)
- ✅ Seed data idempotency (run 3x → no duplicates)
- ✅ Automatic cleanup (TempDir drop → file deleted)
- ✅ Migration guards (all migrations have guard tables)
- ✅ Foreign keys enabled (PRAGMA = ON)
- ✅ Seed data correctness (3 users, 3 tokens, 7 usage records)

**Run validation:**
```bash
cargo nextest run database_initialization
```

---

## References

- **Migration files:** `migrations/*.sql`
- **Migration helper:** `src/migrations.rs`
- **Test helpers:** `tests/common/mod.rs`
- **Seed scripts:** `scripts/seed_dev_data.sh`, `scripts/reset_dev_db.sh`
- **Makefile:** Database management commands
- **ADR-007:** NO MOCKING policy
- **Issue-003:** Migration guard pattern (prevents CASCADE DELETE data loss)

---

## Revision History

| Date | Version | Changes |
|------|---------|---------|
| 2024-12-11 | 1.0.0 | Initial documentation (Phase 2B) |
