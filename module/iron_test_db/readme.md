# iron_test_db

Test database infrastructure for Iron Runtime crates.

[![Documentation](https://img.shields.io/badge/docs-ironcage.ai-E5E7EB.svg)](https://ironcage.ai/docs)

## Installation

```toml
[dev-dependencies]
iron_test_db = { path = "../iron_test_db" }
```


## Features

- **Fluent Builder API**: Ergonomic database creation with `TestDatabaseBuilder`
- **Automatic Cleanup**: RAII-based cleanup via `TempDir` and `Drop`
- **Three Storage Modes**: InMemory (fast), TempFile (realistic I/O), SharedMemory (read-only sharing)
- **Migration Registry**: Centralized migration management with automatic guards
- **Automatic Table Wiping**: Topological sort respects foreign key dependencies
- **No Mocking**: Real SQLite databases for all tests


## Quick Start

### Basic Test Database

```rust
use iron_test_db::TestDatabaseBuilder;

#[ tokio::test ]
async fn test_with_database()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "Failed to create test database" );

  let pool = db.pool();
  // Use pool for testing...
}
```

### With Migrations

```rust
use iron_test_db::{ TestDatabaseBuilder, Migration, MigrationRegistry };

#[ tokio::test ]
async fn test_with_migrations()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "Failed to create database" );

  let registry = MigrationRegistry::new()
    .register( Migration {
      version: 1,
      name: "create_users".to_string(),
      sql: "CREATE TABLE users ( id INTEGER PRIMARY KEY, name TEXT )",
    } );

  registry.apply_all( db.pool() )
    .await
    .expect( "Failed to apply migrations" );

  // Test with migrated schema...
}
```

### Wiping Data

```rust
#[ tokio::test ]
async fn test_with_cleanup()
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "Failed to create database" );

  // ... insert test data ...

  // Wipe all tables (respects foreign keys automatically)
  db.wipe()
    .await
    .expect( "Failed to wipe tables" );

  // Database now empty, ready for next test
}
```

### Storage Modes

```rust
// In-memory (fastest, default)
let db = TestDatabaseBuilder::new()
  .in_memory()
  .build()
  .await?;

// Temporary file (realistic I/O)
let db = TestDatabaseBuilder::new()
  .temp_file()
  .build()
  .await?;

// Shared memory (read-only sharing)
let db = TestDatabaseBuilder::new()
  .shared_memory( "shared_db" )
  .build()
  .await?;
```


## Key Innovations

### Automatic Dependency Detection

No need to manually track foreign key order when wiping tables:

```rust
// Old way (manual, fragile)
sqlx::query( "DELETE FROM token_usage" ).execute( pool ).await?;  // Child
sqlx::query( "DELETE FROM api_tokens" ).execute( pool ).await?;   // Parent
// Adding new FK breaks this!

// New way (automatic, maintenance-free)
db.wipe().await?;  // Automatically detects and orders deletions
```

### Migration Registry

Eliminates guard table boilerplate:

```rust
// Old way: 50 lines of boilerplate per migration
let completed: i64 = query_scalar(
  "SELECT COUNT(*) FROM sqlite_master WHERE name='_migration_002_completed'"
).fetch_one( pool ).await?;

if completed == 0 {
  // Apply migration...
}

// New way: Automatic guard management
let registry = MigrationRegistry::new()
  .register( migration_001 )
  .register( migration_002 );

registry.apply_all( pool ).await?;  // Automatically tracked
```


## Design Principles

1. **No Mocking**: Use real SQLite databases, not mocks
2. **Automatic Cleanup**: RAII-based, no manual cleanup required
3. **Loud Failures**: Explicit error messages for debugging
4. **Zero Configuration**: Sensible defaults, customize when needed
5. **Test Isolation**: Each test gets fresh database (no shared state)


## Performance

- **In-memory databases**: ~5ms setup overhead per test
- **File-based databases**: ~65ms setup overhead per test
- **Shared memory**: ~2ms setup overhead per test (amortized)


<details>
<summary>Integration with Existing Crates</summary>

Add to `Cargo.toml`:

```toml
[dev-dependencies]
iron_test_db = { path = "../iron_test_db" }
```

Migrate tests incrementally:

```rust
// Old helper (keep during transition)
#[ allow( deprecated ) ]
pub async fn create_test_db() -> ( SqlitePool, TempDir )
{
  // Original implementation
}

// New helper
pub async fn create_test_db_v2() -> iron_test_db::TestDatabase
{
  TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "Failed to create test database" )
}
```

</details>


<details>
<summary>Testing the Tests</summary>

The crate includes comprehensive infrastructure tests:

```bash
# Run all tests
cargo test --all-features

# Run specific test
cargo test --test infrastructure_tests test_wipe_respects_foreign_keys
```

</details>


<details>
<summary>Known Limitations</summary>

- **SQLite only**: PostgreSQL/MySQL support planned for future
- **No transaction rollback**: SQLite savepoint limitations
- **No parallel writes**: Shared memory databases are read-only for parallel tests

</details>


<details>
<summary>Scope & Boundaries</summary>

**Responsibilities:**
Provides ergonomic builders for creating isolated test databases with automatic cleanup, migration management, and table wiping.

**In Scope:**
- Fluent Builder API
- Automatic Cleanup (RAII)
- Three Storage Modes (InMemory, TempFile, SharedMemory)
- Migration Registry
- Automatic Table Wiping with dependency detection

**Out of Scope:**
- PostgreSQL/MySQL support (future)
- Production database management
- Database migrations for production schemas

</details>


<details>
<summary>Directory Structure</summary>

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Test database infrastructure for Iron Runtime crates |
| builder.rs | Test database builder with fluent API |
| error.rs | Error types for test database operations |
| migrations.rs | Migration registry for version tracking and guard management |
| wipe.rs | Automatic table wiping with dependency detection |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

</details>


## Contributing

When adding features:

1. Add tests to `tests/infrastructure_tests.rs`
2. Update this readme
3. Verify all tests pass: `cargo test --all-features`
4. Run clippy: `cargo clippy --all-targets --all-features`


## License

MIT OR Apache-2.0
