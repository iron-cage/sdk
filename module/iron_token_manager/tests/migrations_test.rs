#![allow(missing_docs)]

use iron_token_manager::*;
use sqlx::SqlitePool;

#[ tokio::test ]
async fn test_apply_all_migrations_creates_tables()
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();

  apply_all_migrations( &pool ).await.unwrap();

  // Verify all expected tables exist
  let table_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table'"
  )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!(
    table_count >= 9,  // 9 core tables + guard tables
    "Must create all expected tables, got: {table_count}"
  );
}

#[ tokio::test ]
async fn test_apply_all_migrations_idempotent()
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();

  // Apply twice
  apply_all_migrations( &pool ).await.unwrap();
  apply_all_migrations( &pool ).await.unwrap();

  // Should succeed without errors (idempotent)

  // Verify no duplicate data
  let guard_002_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM _migration_002_completed"
  )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!(
    guard_002_count, 1,
    "Guard table must have single entry after re-application"
  );
}

#[ tokio::test ]
async fn test_foreign_keys_enabled_after_migrations()
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();

  apply_all_migrations( &pool ).await.unwrap();

  let fk_enabled: i64 = sqlx::query_scalar( "PRAGMA foreign_keys" )
    .fetch_one( &pool )
    .await
    .unwrap();

  assert_eq!( fk_enabled, 1, "Foreign keys must be enabled" );
}
