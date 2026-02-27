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
//! - Guard tables must not be deleted manually
//! - Foreign key pragma must run before migrations

use std::{fs, path::Path};

use sqlx::SqlitePool;

use crate::error::{Result, TokenError};

/// Applies all migrations to the database pool.
///
/// All `.sql` files in the `migrations/` directory are applied in sorted order.
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
pub async fn apply_all_migrations(pool: &SqlitePool) -> Result<()> {
  // Enable foreign keys (must be first)
  sqlx::query("PRAGMA foreign_keys = ON")
    .execute(pool)
    .await
    .map_err(|e| {
      eprintln!("PRAGMA foreign_keys failed: {e:?}");
      TokenError::Generic
    })?;

  // Discover and apply all .sql migrations in sorted order
  let migrations_dir = format!("{}/migrations", env!("CARGO_MANIFEST_DIR"));
  let mut filenames = fs::read_dir(&migrations_dir)
    .map_err(|e| {
      eprintln!("Failed to read migrations directory: {e}");
      TokenError::Generic
    })?
    .filter_map(|entry| {
      let name = entry.ok()?.file_name().into_string().ok()?;
      Path::new(&name)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("sql"))
        .then_some(name)
    })
    .collect::<Vec<_>>();
  filenames.sort();

  for filename in &filenames {
    apply_guarded_migration(pool, filename).await?;
  }

  Ok(())
}

/// Apply a single guarded migration.
///
/// Checks the guard table `_migration_{number}_completed` before running.
/// If the guard table exists, the migration is skipped (idempotent).
///
/// The migration number is extracted from the first 3 characters of `filename`
/// (e.g. `"003_create_users_table.sql"` → `"003"`).
async fn apply_guarded_migration(pool: &SqlitePool, filename: &str) -> Result<()> {
  let number = &filename[..3];
  let path = format!("{}/migrations/{filename}", env!("CARGO_MANIFEST_DIR"));
  let sql = fs::read_to_string(&path).map_err(|e| {
    eprintln!("Failed to read migration {filename}: {e}");
    TokenError::Generic
  })?;

  let guard_table = format!("_migration_{number}_completed");
  let check_sql =
    format!("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{guard_table}'");

  let completed: i64 = sqlx::query_scalar(&check_sql)
    .fetch_one(pool)
    .await
    .map_err(|_| TokenError::Generic)?;

  if completed == 0 {
    sqlx::raw_sql(&sql).execute(pool).await.map_err(|e| {
      eprintln!("Migration {number} failed: {e:?}");
      TokenError::Generic
    })?;
  }

  Ok(())
}
