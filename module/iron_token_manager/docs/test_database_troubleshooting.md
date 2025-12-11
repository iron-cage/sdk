# Test Database Troubleshooting Guide

This guide covers common issues when working with the iron_test_db infrastructure.

## Common Issues

### 1. Wipe Cycle Detection Error

**Error:**
```
LOUD FAILURE: Failed to wipe database: DependencyCycle("Cycle detected involving table: users")
```

**Cause:**
The automatic dependency detection in `iron_test_db` found circular foreign key relationships that it cannot resolve automatically.

**Solution:**
Use the domain-specific wipe function instead of the automatic wipe:

```rust
// Instead of:
db.wipe().await?;

// Use:
iron_token_manager::seed::wipe_database( db.pool() ).await?;
```

**Why:**
The domain-specific wipe function has manual FK ordering that handles complex circular relationships correctly.

---

### 2. Seed Data Doesn't Match Documentation

**Error:**
Validation tests fail with unexpected counts or missing entities.

**Cause:**
Seed data has been modified but documentation wasn't updated.

**Solution:**
1. Run seed data validation tests: `cargo nextest run seed_data_validation`
2. Check failing test messages for mismatches
3. Update either the seed data or documentation to match
4. Ensure `tests/fixtures/seed_data_reference.md` is accurate

**Prevention:**
The validation tests in `seed_data_validation.rs` automatically catch documentation drift.

---

### 3. Foreign Key Integrity Violations

**Error:**
```
LOUD FAILURE: No provider keys should be orphaned
  left: 2
 right: 0
```

**Cause:**
Seed data references user_ids that don't exist in the users table, or uses wrong field names.

**Solution:**
Check that seed data user_id references match actual usernames in users table:

```rust
// Users table has usernames:
"admin", "developer", "viewer"

// Foreign keys should reference these exactly:
.bind( "admin" )      // ✓ Correct
.bind( "developer" )  // ✓ Correct
.bind( "viewer" )     // ✓ Correct

// NOT:
.bind( "user_admin" )     // ❌ Wrong
.bind( "user_developer" ) // ❌ Wrong
```

---

### 4. Private Function Access Errors

**Error:**
```
error[E0603]: function `current_time_ms` is private
```

**Cause:**
Attempting to use `pub(crate)` functions from test code (which is outside the crate).

**Solution:**
Use standard library equivalents directly:

```rust
// Instead of:
let now_ms = iron_token_manager::storage::current_time_ms();

// Use:
let now_ms = std::time::SystemTime::now()
  .duration_since( std::time::UNIX_EPOCH )
  .unwrap()
  .as_millis() as i64;
```

---

### 5. Pool Sharing Issues

**Problem:**
Tests using `/tmp` file paths to work around lack of pool-sharing constructors.

**Solution:**
Use the v2 helpers that leverage `from_pool()` constructors:

```rust
// Old approach (workaround):
let db = create_test_db_v2().await;
let db_path = "/tmp/test_storage_temp.db";
let db_url = format!( "sqlite://{}?mode=rwc", db_path );
let storage = TokenStorage::new( &db_url ).await?;

// New approach (clean):
let ( storage, db ) = create_test_storage_v2().await;
// Storage and db share the same pool
```

**Benefits:**
- No temporary files
- True pool sharing
- Better performance
- Cleaner code

---

### 6. Test Isolation Problems

**Problem:**
Tests affect each other's data or state.

**Cause:**
Tests might be sharing a database or not cleaning up properly.

**Solution:**
Ensure each test creates its own database:

```rust
#[ tokio::test ]
async fn test_isolated()
{
  // Each call creates a NEW isolated database
  let db = create_test_db_v2().await;

  // Modify data freely - won't affect other tests
  sqlx::query( "DELETE FROM users" ).execute( db.pool() ).await?;

  // Automatic cleanup when `db` drops
}
```

**Guarantee:**
`iron_test_db` ensures complete isolation - each test gets its own database that's automatically cleaned up.

---

### 7. Seed Data Not Populating

**Problem:**
`create_test_db_with_seed()` returns database but queries show empty tables.

**Cause:**
Seed function might have failed silently or migrations weren't applied.

**Solution:**
Check that:
1. Migrations run before seeding
2. Seed function returns Ok
3. All foreign key dependencies are seeded in correct order

```rust
pub async fn create_test_db_with_seed() -> TestDatabase
{
  let db = create_test_db_v2().await;

  // MUST apply migrations first
  iron_token_manager::migrations::apply_all_migrations( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to apply migrations" );

  // Then seed
  iron_token_manager::seed::seed_all( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to seed database" );

  db
}
```

---

### 8. Method Not Found Errors

**Error:**
```
error[E0599]: no method named `get_token_by_id` found for struct `TokenStorage`
```

**Cause:**
Using incorrect method names or old API.

**Solution:**
Check actual API in source code:

- `get_token_metadata()` not `get_token_by_id()`
- `get_token_usage( token_id )` not `get_token_usage( token_id, None, None )`

Always verify method signatures in the implementation before writing tests.

---

## Best Practices

### 1. Use v2 Helpers

Prefer `*_v2()` helpers over v1 for new tests:
- `create_test_db_v2()` instead of `create_test_db()`
- `create_test_storage_v2()` instead of `create_test_storage()`
- etc.

### 2. Validate Seed Data

Add validation tests for any new seed data to prevent documentation drift:

```rust
#[ tokio::test ]
async fn validate_new_entity_count()
{
  let db = create_test_db_with_seed().await;

  let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM new_table" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count entities" );

  assert_eq!( count, EXPECTED_COUNT, "Seed should create N entities" );
}
```

### 3. Use Loud Failures

Always use descriptive error messages with "LOUD FAILURE:" prefix:

```rust
.expect( "LOUD FAILURE: Failed to create token" )
```

Not:
```rust
.unwrap()  // ❌ No context
.expect( "error" )  // ❌ Not loud enough
```

### 4. Keep Seed Data Documented

Update `tests/fixtures/seed_data_reference.md` whenever seed data changes.

The validation tests will catch mismatches automatically.

---

## Getting Help

If you encounter issues not covered here:

1. Check existing tests in `tests/` for examples
2. Review `tests/-example_v2_helpers.rs` for usage patterns
3. Read `tests/fixtures/seed_data_reference.md` for seed data details
4. Run validation tests to identify data issues: `cargo nextest run seed_data_validation`
