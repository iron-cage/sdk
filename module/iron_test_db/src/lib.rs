//! Test database infrastructure for Iron Runtime crates
//!
//! Provides ergonomic builders for creating isolated test databases
//! with automatic cleanup, migration management, and seed data.
//!
//! # Examples
//!
//! ```no_run
//! use iron_test_db::{ TestDatabaseBuilder, StorageMode };
//!
//! #[ tokio::test ]
//! async fn test_with_database()
//! {
//!   let db = TestDatabaseBuilder::new()
//!     .storage_mode( StorageMode::InMemory )
//!     .build()
//!     .await
//!     .expect( "Failed to create test database" );
//!
//!   let pool = db.pool();
//!   // Use pool for testing...
//! }
//! ```

mod builder;
mod error;
mod migrations;
mod wipe;

pub use builder::{ TestDatabaseBuilder, StorageMode };
pub use error::{ TestDbError, Result };
pub use migrations::{ MigrationRegistry, Migration, MigrationRecord };
pub use wipe::{ discover_table_dependencies, wipe_all_tables, topological_sort_reverse };

use sqlx::SqlitePool;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test database handle with automatic cleanup
pub struct TestDatabase
{
  pool: SqlitePool,
  _temp: Option< TempDir >,
  storage_mode: StorageMode,
  path: Option< PathBuf >,
}

impl TestDatabase
{
  /// Get reference to database connection pool
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }

  /// Wipe all data from all tables (respects foreign keys)
  pub async fn wipe( &self ) -> Result< () >
  {
    wipe_all_tables( &self.pool ).await
  }

  /// Get storage mode
  pub fn storage_mode( &self ) -> &StorageMode
  {
    &self.storage_mode
  }

  /// Get database file path (None for in-memory databases)
  ///
  /// For CI environments, this returns the workspace-relative path where
  /// the test database is stored for post-failure inspection.
  /// For local environments with TempFile, returns the temporary path.
  /// For InMemory/SharedInMemory, returns None.
  pub fn path( &self ) -> Option< &PathBuf >
  {
    self.path.as_ref()
  }
}

// Automatic cleanup via Drop (in-memory DBs auto-cleanup, file DBs via TempDir)
impl Drop for TestDatabase
{
  fn drop( &mut self )
  {
    // TempDir cleanup happens automatically
    // In-memory databases are cleaned up when pool is closed
  }
}
