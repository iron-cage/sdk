//! Test database builder with fluent API

use crate::{
  error::{Result, TestDbError},
  TestDatabase,
};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tempfile::TempDir;
use workspace_tools::workspace;

/// Storage mode for test database
#[derive(Debug, Clone)]
pub enum StorageMode {
  /// In-memory database (fast, no cleanup needed)
  InMemory,
  /// File-based database in temporary directory (realistic, automatic cleanup)
  TempFile,
  /// Shared in-memory database for read-heavy tests (fastest)
  SharedInMemory {
    /// Named shared memory URI identifier.
    name: String,
  },
}

/// Builder for test databases with fluent API
#[derive(Debug)]
pub struct TestDatabaseBuilder {
  storage_mode: StorageMode,
  pool_size: u32,
}

impl TestDatabaseBuilder {
  /// Create new builder with default settings
  #[must_use]
  pub fn new() -> Self {
    Self {
      storage_mode: StorageMode::InMemory,
      pool_size: 5,
    }
  }

  /// Set storage mode
  #[must_use]
  pub fn storage_mode(mut self, mode: StorageMode) -> Self {
    self.storage_mode = mode;
    self
  }

  /// Use in-memory database (default)
  #[must_use]
  pub fn in_memory(mut self) -> Self {
    self.storage_mode = StorageMode::InMemory;
    self
  }

  /// Use temporary file database
  #[must_use]
  pub fn temp_file(mut self) -> Self {
    self.storage_mode = StorageMode::TempFile;
    self
  }

  /// Use shared in-memory database
  #[must_use]
  pub fn shared_memory(mut self, name: impl Into<String>) -> Self {
    self.storage_mode = StorageMode::SharedInMemory { name: name.into() };
    self
  }

  /// Set connection pool size (default: 5)
  #[must_use]
  pub fn pool_size(mut self, size: u32) -> Self {
    self.pool_size = size;
    self
  }

  /// Build the test database
  ///
  /// # Errors
  ///
  /// Returns an error if the `SQLite` connection pool cannot be created or migrations fail.
  ///
  /// # Panics
  ///
  /// Panics if the system clock is set before the Unix epoch.
  pub async fn build(self) -> Result<TestDatabase> {
    let (pool, temp_dir, path) = match &self.storage_mode {
      StorageMode::InMemory => {
        let pool = self.create_pool("sqlite::memory:").await?;
        (pool, None, None)
      }
      StorageMode::TempFile => {
        // CI environment: Use workspace storage for debugging
        // Local environment: Use TempDir for automatic cleanup
        let is_ci = std::env::var("CI").is_ok();

        if is_ci {
          // CI: Store in workspace for post-failure inspection
          let ws = workspace().map_err(|e| {
            TestDbError::Io(std::io::Error::other(format!(
              "Failed to detect workspace: {e}"
            )))
          })?;

          let test_db_dir = ws.root().join("target").join("test_databases");
          std::fs::create_dir_all(&test_db_dir).map_err(TestDbError::Io)?;

          // Use timestamp for unique test database names
          let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis();
          let db_path = test_db_dir.join(format!("test_{timestamp}.db"));
          let db_url = format!("sqlite://{}?mode=rwc", db_path.display());

          eprintln!("Test database at: {}", db_path.display());

          let pool = self.create_pool(&db_url).await?;
          (pool, None, Some(db_path))
        } else {
          // Local: Use TempDir for automatic cleanup
          let temp_dir = TempDir::new().map_err(TestDbError::Io)?;
          let db_path = temp_dir.path().join("test.db");
          let db_url = format!("sqlite://{}?mode=rwc", db_path.display());
          let pool = self.create_pool(&db_url).await?;
          (pool, Some(temp_dir), Some(db_path))
        }
      }
      StorageMode::SharedInMemory { name } => {
        let db_url = format!("sqlite:file:{name}?mode=memory&cache=shared");
        let pool = self.create_pool(&db_url).await?;
        (pool, None, None)
      }
    };

    // Enable foreign keys (critical for CASCADE DELETE)
    sqlx::query("PRAGMA foreign_keys = ON")
      .execute(&pool)
      .await?;

    Ok(TestDatabase {
      pool,
      _temp: temp_dir,
      storage_mode: self.storage_mode.clone(),
      path,
    })
  }

  async fn create_pool(&self, url: &str) -> Result<SqlitePool> {
    let pool = SqlitePoolOptions::new()
      .max_connections(self.pool_size)
      .connect(url)
      .await
      .map_err(TestDbError::Database)?;

    Ok(pool)
  }
}

impl Default for TestDatabaseBuilder {
  fn default() -> Self {
    Self::new()
  }
}
