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
//! - SQL is embedded at compile time via `include_str!()` (no filesystem access at runtime)
//!
//! # Known Pitfalls
//!
//! - Guard tables must not be deleted manually
//! - Foreign key pragma must run before migrations

use sqlx::SqlitePool;

use crate::error::{Result, TokenError};

/// Embeds a migration SQL file at compile time and pairs it with its filename.
/// The filename is parsed at runtime to extract the migration number (first 3 chars).
macro_rules! migration {
  ($name:literal) => {
    (
      $name,
      include_str!(concat!("../migrations/", $name, ".sql")),
    )
  };
}

/// All migrations embedded at compile time.
/// Each entry is `(name, sql)` - sorted by number prefix for deterministic ordering.
/// Using `include_str!()` ensures deployed binaries carry their own migrations
/// without depending on the build machine's filesystem paths.
static MIGRATIONS: &[(&str, &str)] = &[
  migration!("001_initial_schema"),
  migration!("002_add_length_constraints"),
  migration!("003_create_users_table"),
  migration!("004_create_ai_provider_keys"),
  migration!("005_enhance_users_table"),
  migration!("006_create_user_audit_log"),
  migration!("007_create_blacklist_table"),
  migration!("008_create_agents_table"),
  migration!("009_create_budget_leases"),
  migration!("010_create_agent_budgets"),
  migration!("011_create_budget_requests"),
  migration!("012_create_analytics_events"),
  migration!("013_create_budget_history"),
  migration!("014_add_api_tokens_fk"),
  migration!("015_add_agents_owner_id"),
  migration!("016_add_revoked_at"),
  migration!("017_add_lease_return_columns"),
  migration!("018_create_system_config"),
  migration!("019_convert_budgets_to_microdollars"),
  migration!("020_add_agent_provider_key_id"),
  migration!("021_add_account_lockout_fields"),
  migration!("022_add_ic_token_to_agents"),
  migration!("023_seed_agent1_ic_token"),
  migration!("024_add_ip_key_spending_cap"),
  migration!("025_rename_roles"),
  migration!("026_add_spending_constraints"),
  migration!("027_add_agent_token_index"),
];

/// Applies all migrations to the database pool.
///
/// All migrations are embedded at compile time and applied in sorted order.
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

  for (name, sql) in MIGRATIONS {
    apply_guarded_migration(pool, name, sql).await?;
  }

  Ok(())
}

/// Apply a single guarded migration.
///
/// Checks the guard table `_migration_{number}_completed` before running.
/// If the guard table exists, the migration is skipped (idempotent).
async fn apply_guarded_migration(pool: &SqlitePool, name: &str, sql: &str) -> Result<()> {
  let number = &name[..3];
  let guard_table = format!("_migration_{number}_completed");

  let completed: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name = ?")
      .bind(&guard_table)
      .fetch_one(pool)
      .await
      .map_err(|_| TokenError::Generic)?;

  if completed == 0 {
    sqlx::raw_sql(sql).execute(pool).await.map_err(|e| {
      eprintln!("Migration {name} failed: {e:?}");
      TokenError::Generic
    })?;
  }

  Ok(())
}
