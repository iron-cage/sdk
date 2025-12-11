//! Test database builder with fluent API

use crate::{ TestDatabase, error::{ Result, TestDbError } };
use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use tempfile::TempDir;

/// Storage mode for test database
#[ derive( Debug, Clone ) ]
pub enum StorageMode
{
  /// In-memory database (fast, no cleanup needed)
  InMemory,
  /// File-based database in temporary directory (realistic, automatic cleanup)
  TempFile,
  /// Shared in-memory database for read-heavy tests (fastest)
  SharedInMemory { name: String },
}

/// Builder for test databases with fluent API
pub struct TestDatabaseBuilder
{
  storage_mode: StorageMode,
  pool_size: u32,
}

impl TestDatabaseBuilder
{
  /// Create new builder with default settings
  pub fn new() -> Self
  {
    Self {
      storage_mode: StorageMode::InMemory,
      pool_size: 5,
    }
  }

  /// Set storage mode
  pub fn storage_mode( mut self, mode: StorageMode ) -> Self
  {
    self.storage_mode = mode;
    self
  }

  /// Use in-memory database (default)
  pub fn in_memory( mut self ) -> Self
  {
    self.storage_mode = StorageMode::InMemory;
    self
  }

  /// Use temporary file database
  pub fn temp_file( mut self ) -> Self
  {
    self.storage_mode = StorageMode::TempFile;
    self
  }

  /// Use shared in-memory database
  pub fn shared_memory( mut self, name: impl Into< String > ) -> Self
  {
    self.storage_mode = StorageMode::SharedInMemory { name: name.into() };
    self
  }

  /// Set connection pool size (default: 5)
  pub fn pool_size( mut self, size: u32 ) -> Self
  {
    self.pool_size = size;
    self
  }

  /// Build the test database
  pub async fn build( self ) -> Result< TestDatabase >
  {
    let ( pool, temp_dir ) = match &self.storage_mode
    {
      StorageMode::InMemory => {
        let pool = self.create_pool( "sqlite::memory:" ).await?;
        ( pool, None )
      },
      StorageMode::TempFile => {
        let temp_dir = TempDir::new()
          .map_err( TestDbError::Io )?;
        let db_path = temp_dir.path().join( "test.db" );
        let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );
        let pool = self.create_pool( &db_url ).await?;
        ( pool, Some( temp_dir ) )
      },
      StorageMode::SharedInMemory { name } => {
        let db_url = format!( "sqlite:file:{}?mode=memory&cache=shared", name );
        let pool = self.create_pool( &db_url ).await?;
        ( pool, None )
      },
    };

    // Enable foreign keys (critical for CASCADE DELETE)
    sqlx::query( "PRAGMA foreign_keys = ON" )
      .execute( &pool )
      .await?;

    Ok( TestDatabase {
      pool,
      _temp: temp_dir,
      storage_mode: self.storage_mode.clone(),
    } )
  }

  async fn create_pool( &self, url: &str ) -> Result< SqlitePool >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( self.pool_size )
      .connect( url )
      .await
      .map_err( TestDbError::Database )?;

    Ok( pool )
  }
}

impl Default for TestDatabaseBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}
