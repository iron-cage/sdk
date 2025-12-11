//! Common test utilities
//!
//! Shared test helpers for creating temporary databases with proper schema.
//! All helpers use the unified migrations module to ensure test databases
//! match production schema exactly.

#![ allow( dead_code ) ]

use iron_token_manager::limit_enforcer::LimitEnforcer;
use iron_token_manager::storage::TokenStorage;
use iron_token_manager::usage_tracker::UsageTracker;
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
