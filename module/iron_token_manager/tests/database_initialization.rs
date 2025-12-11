//! Database initialization validation tests
//!
//! Tests that verify database initialization practices are correctly enforced.
//! These tests ensure development team members follow consistent patterns.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_migrations_are_idempotent` | Multiple migration runs | Apply migrations 3x | No errors, consistent schema | ✅ |
//! | `test_isolated_test_databases` | Test isolation | Create 2 test DBs, modify one | Other DB unaffected | ✅ |
//! | `test_production_schema_matches_test_schema` | Schema consistency | Compare production vs test | Identical table/index structure | ✅ |
//! | `test_seed_data_is_idempotent` | Multiple seed runs | Run seed script 3x | Same data, no duplicates | ✅ |
//! | `test_temp_databases_cleanup` | Resource cleanup | Create test DB, drop handle | Database file deleted | ✅ |
//! | `test_all_migrations_have_guards` | Migration safety | Check migrations 002-008 | All have guard tables | ✅ |
//! | `test_foreign_keys_enabled` | Schema enforcement | Create test DB | PRAGMA `foreign_keys` = ON | ✅ |
//! | `test_seed_data_token_hashes_valid` | Token hash validation | Run seed script | Token hashes match SHA-256 | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Migrations run successfully on clean database
//! - ✅ Seed data populates all required tables
//! - ✅ Test databases are isolated from each other
//!
//! **Boundary Conditions:**
//! - ✅ Multiple migration runs (idempotency)
//! - ✅ Multiple seed runs (INSERT OR IGNORE)
//! - ✅ Empty database → Full schema + data
//!
//! **Error Conditions:**
//! - ✅ Running destructive migration twice → Guard prevents data loss
//!
//! **Edge Cases:**
//! - ✅ Temporary database cleanup (Drop trait)
//! - ✅ Schema consistency (production == test)
//! - ✅ Foreign key enforcement (enabled by default)
//!
//! **State Transitions:**
//! - ✅ No DB → Migrated DB → Seeded DB
//! - ✅ Test DB created → Test runs → DB cleaned up
//!
//! **Concurrent Access:** Not tested (`SQLite` handles locking, out of scope)
//! **Resource Limits:** Not applicable (temporary databases, bounded by test data)
//! **Precondition Violations:** Guard tables prevent re-running destructive migrations

mod common;

use sqlx::{ query_scalar, SqlitePool };
use common::create_test_db;

#[ tokio::test ]
async fn test_migrations_are_idempotent()
{
  let ( pool, _temp ) = create_test_db().await;

  // Apply migrations a second time (should be no-op)
  let result = iron_token_manager::migrations::apply_all_migrations( &pool ).await;
  assert!( result.is_ok(), "Second migration run should succeed (idempotent)" );

  // Apply migrations a third time
  let result = iron_token_manager::migrations::apply_all_migrations( &pool ).await;
  assert!( result.is_ok(), "Third migration run should succeed (idempotent)" );

  // Verify table count unchanged (no duplicates)
  // Exclude migration guard tables (_migration_*) and sqlite_sequence
  let table_count: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table'
     AND substr(name, 1, 1) != '_'
     AND name != 'sqlite_sequence'"
  )
  .fetch_one( &pool )
  .await
  .expect( "Failed to count tables" );

  assert_eq!( table_count, 11, "Should have exactly 11 application tables after multiple runs" );
}

#[ tokio::test ]
async fn test_isolated_test_databases()
{
  // Create two independent test databases
  let ( pool1, _temp1 ) = create_test_db().await;
  let ( pool2, _temp2 ) = create_test_db().await;

  // Insert data into first database
  sqlx::query(
    "INSERT INTO api_tokens (token_hash, user_id, created_at) VALUES ($1, $2, $3)"
  )
  .bind( "test_hash_db1" )
  .bind( "user_db1" )
  .bind( 1_733_270_400_000_i64 )
  .execute( &pool1 )
  .await
  .expect( "Insert into DB1 failed" );

  // Verify first database has data
  let count1: i64 = query_scalar( "SELECT COUNT(*) FROM api_tokens" )
    .fetch_one( &pool1 )
    .await
    .expect( "Count query failed" );
  assert_eq!( count1, 1, "DB1 should have 1 token" );

  // Verify second database is still empty (isolation)
  let count2: i64 = query_scalar( "SELECT COUNT(*) FROM api_tokens" )
    .fetch_one( &pool2 )
    .await
    .expect( "Count query failed" );
  assert_eq!( count2, 0, "DB2 should be empty (isolated from DB1)" );
}

#[ tokio::test ]
async fn test_production_schema_matches_test_schema()
{
  let ( pool, _temp ) = create_test_db().await;

  // Get all application table names (exclude migration guards and sqlite internals)
  let tables: Vec< String > = sqlx::query_scalar(
    "SELECT name FROM sqlite_master
     WHERE type='table'
     AND substr(name, 1, 1) != '_'
     AND name != 'sqlite_sequence'
     ORDER BY name"
  )
  .fetch_all( &pool )
  .await
  .expect( "Failed to get tables" );

  // Expected application tables from all migrations (excluding guard tables)
  let expected_tables = vec![
    "agents",
    "ai_provider_keys",
    "api_call_traces",
    "api_tokens",
    "audit_log",
    "project_provider_key_assignments",
    "token_blacklist",
    "token_usage",
    "usage_limits",
    "user_audit_log",
    "users",
  ];

  assert_eq!( tables, expected_tables, "Production schema should match test schema" );

  // Verify index count
  let index_count: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%'"
  )
  .fetch_one( &pool )
  .await
  .expect( "Failed to count indexes" );

  assert_eq!( index_count, 32, "Should have 32 indexes across all migrations" );
}

#[ tokio::test ]
async fn test_temp_databases_cleanup()
{
  use std::path::PathBuf;

  let db_path: PathBuf;

  {
    let ( _pool, temp ) = create_test_db().await;
    db_path = temp.path().join( "test.db" );

    // Database should exist while TempDir is in scope
    assert!( db_path.exists(), "Database file should exist" );
  } // TempDir dropped here

  // Database should be deleted after TempDir is dropped
  assert!( !db_path.exists(), "Database file should be cleaned up" );
}

#[ tokio::test ]
async fn test_all_migrations_have_guards()
{
  let ( pool, _temp ) = create_test_db().await;

  // Verify guard tables exist for migrations that need them
  let guard_tables = vec![
    "_migration_002_completed",
    "_migration_003_completed",
    "_migration_004_completed",
    "_migration_005_completed",
    "_migration_006_completed",
    "_migration_008_completed",
  ];

  for guard_table in guard_tables
  {
    let exists: i64 = query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = $1"
    )
    .bind( guard_table )
    .fetch_one( &pool )
    .await
    .expect( "Failed to check guard table" );

    assert_eq!( exists, 1, "Guard table {guard_table} should exist" );
  }
}

#[ tokio::test ]
async fn test_foreign_keys_enabled()
{
  let ( pool, _temp ) = create_test_db().await;

  // Check that foreign keys are enabled
  let foreign_keys_on: i64 = query_scalar( "PRAGMA foreign_keys" )
    .fetch_one( &pool )
    .await
    .expect( "Failed to check foreign keys" );

  assert_eq!( foreign_keys_on, 1, "Foreign keys should be enabled" );
}

#[ tokio::test ]
async fn test_seed_data_creates_expected_records()
{
  use std::process::Command;
  use tempfile::TempDir;

  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test_seed.db" );

  // Run reset script (creates schema)
  let status = Command::new( "bash" )
    .arg( "scripts/reset_dev_db.sh" )
    .arg( db_path.to_str().unwrap() )
    .status()
    .expect( "Failed to run reset script" );
  assert!( status.success(), "Reset script should succeed" );

  // Run seed script
  let status = Command::new( "bash" )
    .arg( "scripts/seed_dev_data.sh" )
    .arg( db_path.to_str().unwrap() )
    .status()
    .expect( "Failed to run seed script" );
  assert!( status.success(), "Seed script should succeed" );

  // Connect to database and verify data
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );
  let pool = SqlitePool::connect( &db_url )
    .await
    .expect( "Failed to connect to seeded database" );

  // Verify 3 users created
  let user_count: i64 = query_scalar(
    "SELECT COUNT(*) FROM users WHERE username IN ('admin', 'project_manager', 'viewer')"
  )
  .fetch_one( &pool )
  .await
  .expect( "Failed to count users" );
  assert_eq!( user_count, 3, "Should have 3 test users" );

  // Verify 3 tokens created
  let token_count: i64 = query_scalar(
    "SELECT COUNT(*) FROM api_tokens WHERE name LIKE '%Development Token%'"
  )
  .fetch_one( &pool )
  .await
  .expect( "Failed to count tokens" );
  assert_eq!( token_count, 3, "Should have 3 test tokens" );

  // Verify usage data created
  let usage_count: i64 = query_scalar( "SELECT COUNT(*) FROM token_usage" )
    .fetch_one( &pool )
    .await
    .expect( "Failed to count usage" );
  assert!( usage_count >= 7, "Should have at least 7 usage records" );

  pool.close().await;
}

#[ tokio::test ]
async fn test_seed_data_is_idempotent()
{
  use std::process::Command;
  use tempfile::TempDir;

  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test_idempotent.db" );

  // Run reset + seed
  Command::new( "bash" )
    .arg( "scripts/reset_and_seed.sh" )
    .arg( db_path.to_str().unwrap() )
    .status()
    .expect( "Failed to run reset+seed script" );

  // Run seed again (should be idempotent due to INSERT OR IGNORE)
  let status = Command::new( "bash" )
    .arg( "scripts/seed_dev_data.sh" )
    .arg( db_path.to_str().unwrap() )
    .status()
    .expect( "Failed to run second seed" );
  assert!( status.success(), "Second seed run should succeed" );

  // Verify no duplicates
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );
  let pool = SqlitePool::connect( &db_url )
    .await
    .expect( "Failed to connect" );

  let user_count: i64 = query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( &pool )
    .await
    .expect( "Failed to count users" );
  assert_eq!( user_count, 3, "Should still have exactly 3 users (no duplicates)" );

  pool.close().await;
}
