# Database Quick Reference Card

One-page reference for common database workflows in `iron_token_manager`.

## 🚀 Quick Start

```bash
# Set up development database with test data
make db-reset-seed

# Run all tests
make test

# Start coding!
```

## 📋 Common Commands

### Development Database

| Command | Purpose | Safe? |
|---------|---------|-------|
| `make db-reset-seed` | Wipe + populate test data | ✅ Yes (creates backup) |
| `make db-reset` | Wipe only (clean schema) | ✅ Yes (creates backup) |
| `make db-seed` | Add test data to existing DB | ✅ Yes (idempotent) |
| `make db-backup` | Manual backup | ✅ Yes |
| `make db-inspect` | Open SQLite shell | ✅ Read-only |

### Testing

| Command | Purpose | Speed |
|---------|---------|-------|
| `make test` | All tests + doctests + clippy | Slow (Level 3) |
| `make test-quick` | Unit tests only | Fast (Level 1) |
| `cargo nextest run <name>` | Specific test | Fastest |
| `cargo test --doc` | Doc tests only | Fast |

### Verification

| Command | Purpose |
|---------|---------|
| `make clippy` | Lint check |
| `make check` | Fast compilation check |
| `make fmt-check` | Format check (no changes) |

## 🔑 Test Credentials

**Users (password: `password123`):**
- `admin` (role: admin, active)
- `developer` (role: user, active)
- `viewer` (role: user, inactive)

**API Tokens:**
- Admin token (never expires)
- Developer token (expires in 30 days)
- Project alpha token
- Inactive token
- Expired token

**Projects:**
- `project_alpha` (assigned OpenAI + Anthropic keys)

**Provider Keys:**
- OpenAI (enabled, $50 balance)
- Anthropic (enabled, $100 balance)

## 📁 File Locations

| File | Purpose |
|------|---------|
| `dev_tokens.db` | Development database (default) |
| `config.dev.toml` | Development configuration |
| `config.test.toml` | Test configuration |
| `config.prod.toml.example` | Production config template |
| `backups/` | Timestamped backups |
| `migrations/*.sql` | Schema definitions |
| `scripts/seed_dev_data.sh` | Test data population |
| `tests/common/mod.rs` | Test helper functions |

## ⚙️ Configuration

### Load from Config File (Recommended)

```rust
use iron_token_manager::storage::TokenStorage;

// Load from IRON_ENV (default: "development")
let storage = TokenStorage::from_config().await?;

// Or specify environment
std::env::set_var("IRON_ENV", "production");
let storage = TokenStorage::from_config().await?;
```

### Automatic Wipe and Seed (Development)

Enable automatic database reset on every startup:

```toml
# config.dev.toml
[development]
wipe_and_seed = true
```

This will:
1. Wipe ALL data from database (DESTRUCTIVE!)
2. Re-apply migrations
3. Seed with sample test data
4. Run on EVERY startup

⚠️ **Never enable in production!**

### Manual Seed Data

```rust
use iron_token_manager::seed::{wipe_database, seed_all};

let pool = storage.pool();
wipe_database(pool).await?;  // Optional: wipe first
seed_all(pool).await?;
```

### Environment Overrides

```bash
export DATABASE_URL="sqlite:///custom.db?mode=rwc"
export DATABASE_MAX_CONNECTIONS=20
```

## 🧪 Test Patterns

### Basic Test

```rust
mod common;
use common::create_test_db;

#[tokio::test]
async fn test_example()
{
  let ( pool, _temp ) = create_test_db().await;
  // Database auto-created and auto-cleaned
}
```

### With TokenStorage

```rust
use common::create_test_storage;

#[tokio::test]
async fn test_tokens()
{
  let ( storage, _temp ) = create_test_storage().await;
  // Ready to use!
}
```

## 📊 Expected Schema Counts

| Metric | Count |
|--------|-------|
| Application tables | 11 |
| Migration guard tables | 6 |
| Indexes (idx_*) | 32 |
| Test users (after seed) | 3 |
| Test tokens (after seed) | 5 |
| Test provider keys (after seed) | 2 |
| Test usage limits (after seed) | 3 |
| Test project assignments (after seed) | 2 |

## ⚠️ Common Issues

### "database is locked"
```bash
pkill -9 cargo  # Kill zombie processes
make test       # Try again
```

### Tests fail after schema change
```bash
make db-reset   # Reset development DB
cargo test      # Test DB auto-updates
```

### Seed script fails
```bash
# Check schema matches migrations
sqlite3 dev_tokens.db .schema users
grep "INSERT INTO users" scripts/seed_dev_data.sh
```

## 🔍 Database Inspection

```bash
# Quick stats
make db-inspect

# Or manually:
sqlite3 dev_tokens.db

sqlite> SELECT COUNT(*) FROM users;
sqlite> SELECT username, role FROM users;
sqlite> .schema api_tokens
sqlite> .quit
```

## 🎯 Best Practices

✅ **Do:**
- Use `create_test_db()` for every test
- Run `make db-reset-seed` when switching branches
- Let `TempDir` auto-cleanup test databases
- Use real SQLite (no mocks - ADR-007)

❌ **Don't:**
- Share databases between tests
- Mock database interactions
- Skip migrations
- Commit `dev_tokens.db` to git

## 📚 Full Documentation

See: `docs/database_initialization.md` for comprehensive guide.

## 🐛 Debug Test

```rust
#[tokio::test]
async fn debug_test()
{
  let ( pool, temp ) = create_test_db().await;

  let db_path = temp.path().join( "test.db" );
  println!( "Database: {}", db_path.display() );

  // Add breakpoint or sleep
  std::thread::sleep( std::time::Duration::from_secs( 300 ) );
}
```

Then inspect:
```bash
sqlite3 <path_from_output>
```

## 🔄 Migration Workflow

```rust
// Migrations auto-applied in TokenStorage::new()
let storage = TokenStorage::new( db_url ).await?;

// Or manually:
use iron_token_manager::migrations;
migrations::apply_all_migrations( &pool ).await?;
```

All migrations are **idempotent** (safe to run multiple times).
