//! Database migration utilities
//!
//! Provides unified migration application for both production and test environments.
//! Uses guard tables to prevent re-running destructive migrations (issue-003 fix).
//!
//! # Usage
//!
//! Production:
//! ```rust,ignore
//! use iron_token_manager::migrations::apply_all_migrations;
//!
//! let pool = SqlitePool::connect(database_url).await?;
//! apply_all_migrations(&pool).await?;
//! ```
//!
//! Tests:
//! ```rust,ignore
//! use iron_token_manager::migrations::apply_all_migrations;
//!
//! let pool = SqlitePoolOptions::new()
//!   .connect("sqlite::memory:").await?;
//! apply_all_migrations(&pool).await?;
//! ```
//!
//! # Safety
//!
//! - Idempotent (safe to call multiple times)
//! - Guard tables prevent data loss
//! - Foreign keys always enabled
//! - All migrations applied in order
//!
//! # Known Pitfalls
//!
//! - Migration 007 intentionally skipped (reserved)
//! - Guard tables must not be deleted manually
//! - Foreign key pragma must run before migrations

use sqlx::{ query_scalar, SqlitePool };
use crate::error::Result;

/// Applies all migrations to the database pool.
///
/// Migrations are applied in order (001-008, skipping 007).
/// Uses guard tables to prevent re-running destructive operations.
/// Safe to call multiple times (idempotent).
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Ok if all migrations applied successfully
///
/// # Errors
///
/// Returns error if any migration fails to execute
pub async fn apply_all_migrations( pool: &SqlitePool ) -> Result< () >
{
  // Enable foreign keys (must be first)
  sqlx::query( "PRAGMA foreign_keys = ON" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  // Migration 001: Initial schema (5 core tables)
  apply_migration_001( pool ).await?;

  // Migration 002: Length constraints (guarded)
  apply_migration_002( pool ).await?;

  // Migration 003: Users table (guarded)
  apply_migration_003( pool ).await?;

  // Migration 004: AI provider keys
  apply_migration_004( pool ).await?;

  // Migration 005: Enhanced users table
  apply_migration_005( pool ).await?;

  // Migration 006: User audit log
  apply_migration_006( pool ).await?;

  // Migration 007: RESERVED (intentionally skipped)

  // Migration 008: Agents table
  apply_migration_008( pool ).await?;

  Ok( () )
}

/// Migration 001: Initial schema
///
/// Creates 5 core tables:
/// - `api_tokens`: Token metadata and hashes
/// - `token_usage`: Usage tracking per token
/// - `usage_limits`: Quota per user/project
/// - `api_call_traces`: Detailed call logs
/// - `audit_log`: Compliance audit trail
async fn apply_migration_001( pool: &SqlitePool ) -> Result< () >
{
  let migration = include_str!( "../migrations/001_initial_schema.sql" );
  sqlx::raw_sql( migration )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
  Ok( () )
}

/// Migration 002: Length constraints (GUARDED)
///
/// Adds length constraints to prevent `DoS` attacks (issue-001 defense).
/// Uses guard table to prevent re-running (CASCADE DELETE protection).
///
/// Fix(issue-003): Guard table prevents CASCADE DELETE data loss
/// Root cause: Dropping `api_tokens` table cascaded to `token_usage` deletion
/// Pitfall: Always check guard tables before destructive schema changes
async fn apply_migration_002( pool: &SqlitePool ) -> Result< () >
{
  // Check guard table
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_002_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/002_add_length_constraints.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

/// Migration 003: Users table (GUARDED)
///
/// Creates users table with authentication fields.
/// Uses guard table to prevent re-running.
async fn apply_migration_003( pool: &SqlitePool ) -> Result< () >
{
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_003_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/003_create_users_table.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

/// Migration 004: AI provider keys
async fn apply_migration_004( pool: &SqlitePool ) -> Result< () >
{
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_004_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/004_create_ai_provider_keys.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

/// Migration 005: Enhanced users table
async fn apply_migration_005( pool: &SqlitePool ) -> Result< () >
{
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_005_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/005_enhance_users_table.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

/// Migration 006: User audit log
async fn apply_migration_006( pool: &SqlitePool ) -> Result< () >
{
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_006_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/006_create_user_audit_log.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

// Migration 007: RESERVED
//
// This migration number is intentionally skipped/reserved.
// See: `migrations/007_reserved.md` for explanation.

/// Migration 008: Agents table
async fn apply_migration_008( pool: &SqlitePool ) -> Result< () >
{
  let completed: i64 = query_scalar(
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table' AND name='_migration_008_completed'"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  if completed == 0
  {
    let migration = include_str!( "../migrations/008_create_agents_table.sql" );
    sqlx::raw_sql( migration )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
  }

  Ok( () )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
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
}
