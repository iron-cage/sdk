//! Common test utilities
//!
//! Shared test helpers for creating temporary databases with proper schema.
//! All helpers use the unified migrations module to ensure test databases
//! match production schema exactly.
//!
//! # Migration to `iron_test_db`
//!
//! This module is being incrementally migrated to use the new `iron_test_db` crate.
//! New functions use `_v2` suffix and return `TestDatabase` instead of `( pool, TempDir )`.
//!
//! Prefer using the new `*_v2()` functions for new tests. Old functions maintained
//! for backward compatibility.

#![ allow( dead_code ) ]

use iron_token_manager::limit_enforcer::LimitEnforcer;
use iron_token_manager::storage::TokenStorage;
use iron_token_manager::usage_tracker::UsageTracker;
use iron_test_db::{ TestDatabase, TestDatabaseBuilder };
use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use tempfile::TempDir;

/// Create test database pool with all migrations applied
///
/// Returns `SQLite` connection pool and temporary directory.
/// Database is automatically deleted when `TempDir` is dropped.
///
/// # Example
///
/// ```ignore
/// let ( pool, _temp ) = create_test_db().await;
/// // Use pool for direct SQL queries
/// ```
pub async fn create_test_db() -> ( SqlitePool, TempDir )
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test.db" );
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );

  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( &db_url )
    .await
    .expect( "Failed to connect to test database" );

  // Apply all migrations using unified helper
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect( "Failed to apply migrations" );

  ( pool, temp_dir )
}

/// Create test token storage with initialized database
///
/// Returns `TokenStorage` instance and temporary directory.
/// Database has all migrations applied and is ready for use.
///
/// # Example
///
/// ```ignore
/// let ( storage, _temp ) = create_test_storage().await;
/// storage.create_token( "token", "user", None, None, None, None ).await?;
/// ```
pub async fn create_test_storage() -> ( TokenStorage, TempDir )
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test.db" );
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );

  let storage = TokenStorage::new( &db_url )
    .await
    .expect( "Failed to create storage" );

  ( storage, temp_dir )
}

/// Create test limit enforcer with initialized database
///
/// Returns `LimitEnforcer`, `TokenStorage`, and temporary directory.
/// Both components share the same database connection.
///
/// # Example
///
/// ```ignore
/// let ( enforcer, storage, _temp ) = create_test_enforcer().await;
/// enforcer.create_limit( "user", None, Some( 1000 ), None, None ).await?;
/// ```
pub async fn create_test_enforcer() -> ( LimitEnforcer, TokenStorage, TempDir )
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test.db" );
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );

  let enforcer = LimitEnforcer::new( &db_url )
    .await
    .expect( "Failed to create enforcer" );

  let storage = TokenStorage::new( &db_url )
    .await
    .expect( "Failed to create storage" );

  ( enforcer, storage, temp_dir )
}

/// Create test usage tracker with initialized database
///
/// Returns `UsageTracker`, `TokenStorage`, and temporary directory.
/// Both components share the same database connection.
///
/// # Example
///
/// ```ignore
/// let ( tracker, storage, _temp ) = create_test_tracker().await;
/// let token_id = storage.create_token( "token", "user", None, None, None, None ).await?;
/// tracker.record_usage( token_id, "openai", "gpt-4", 100, 50, 25 ).await?;
/// ```
pub async fn create_test_tracker() -> ( UsageTracker, TokenStorage, TempDir )
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let db_path = temp_dir.path().join( "test.db" );
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );

  let storage = TokenStorage::new( &db_url )
    .await
    .expect( "Failed to create storage" );

  let tracker = UsageTracker::new( &db_url )
    .await
    .expect( "Failed to create tracker" );

  ( tracker, storage, temp_dir )
}

// ============================================================================
// New iron_test_db-based helpers (v2)
// ============================================================================

/// Create test database using `iron_test_db` infrastructure
///
/// Returns `TestDatabase` with all migrations applied and automatic cleanup.
/// Uses in-memory storage by default for maximum speed.
///
/// # Example
///
/// ```ignore
/// let db = create_test_db_v2().await;
/// let pool = db.pool();
/// // Use pool for direct SQL queries
/// ```
pub async fn create_test_db_v2() -> TestDatabase
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create test database" );

  // Apply migrations
  iron_token_manager::migrations::apply_all_migrations( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to apply migrations" );

  db
}

/// Create test database with seed data
///
/// Returns `TestDatabase` with migrations applied and seed data populated.
/// Useful for tests that need realistic data.
///
/// # Example
///
/// ```ignore
/// let db = create_test_db_with_seed().await;
/// // Database now contains users, tokens, limits, etc.
/// ```
pub async fn create_test_db_with_seed() -> TestDatabase
{
  let db = create_test_db_v2().await;

  // Seed database
  iron_token_manager::seed::seed_all( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to seed database" );

  db
}

/// Create test token storage using `iron_test_db`
///
/// Returns `TokenStorage` instance backed by `iron_test_db` infrastructure.
/// Database has all migrations applied and is ready for use.
///
/// # Example
///
/// ```ignore
/// let ( storage, db ) = create_test_storage_v2().await;
/// storage.create_token( "token", "user", None, None, None, None ).await?;
/// ```
pub async fn create_test_storage_v2() -> ( TokenStorage, TestDatabase )
{
  let db = create_test_db_v2().await;

  // Use from_pool to share the same database connection
  let storage = TokenStorage::from_pool( db.pool().clone() );

  ( storage, db )
}

/// Create test limit enforcer using `iron_test_db`
///
/// Returns `LimitEnforcer`, `TokenStorage`, and `TestDatabase`.
/// All components share the same database connection.
///
/// # Example
///
/// ```ignore
/// let ( enforcer, storage, db ) = create_test_enforcer_v2().await;
/// enforcer.create_limit( "user", None, Some( 1000 ), None, None ).await?;
/// ```
pub async fn create_test_enforcer_v2() -> ( LimitEnforcer, TokenStorage, TestDatabase )
{
  let db = create_test_db_v2().await;

  // Use from_pool to share the same database connection
  let enforcer = LimitEnforcer::from_pool( db.pool().clone() );
  let storage = TokenStorage::from_pool( db.pool().clone() );

  ( enforcer, storage, db )
}

/// Create test usage tracker using `iron_test_db`
///
/// Returns `UsageTracker`, `TokenStorage`, and `TestDatabase`.
/// All components share the same database connection.
///
/// # Example
///
/// ```ignore
/// let ( tracker, storage, db ) = create_test_tracker_v2().await;
/// let token_id = storage.create_token( "token", "user", None, None, None, None ).await?;
/// tracker.record_usage( token_id, "openai", "gpt-4", 100, 50, 25 ).await?;
/// ```
pub async fn create_test_tracker_v2() -> ( UsageTracker, TokenStorage, TestDatabase )
{
  let db = create_test_db_v2().await;

  // Use from_pool to share the same database connection
  let storage = TokenStorage::from_pool( db.pool().clone() );
  let tracker = UsageTracker::from_pool( db.pool().clone() );

  ( tracker, storage, db )
}

/// Seed test agent with budget (for budget management tests)
///
/// Creates test agent and associated budget in the database.
/// Uses INSERT OR IGNORE for idempotent seeding.
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `agent_id` - Agent ID to create
///
/// # Example
///
/// ```ignore
/// let db = create_test_db_v2().await;
/// seed_test_agent(db.pool(), 1).await;
/// // Agent 1 now exists with $100 budget
/// ```
pub async fn seed_test_agent( pool: &sqlx::SqlitePool, agent_id: i32 )
{
  sqlx::query( "INSERT OR IGNORE INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( "test-agent" )
    .bind( "[]" )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( pool )
    .await
    .expect( "Should insert test agent" );

  sqlx::query( "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 0.0 )
    .bind( 100.0 )
    .bind( chrono::Utc::now().timestamp_millis() )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( pool )
    .await
    .expect( "Should insert agent budget" );
}
