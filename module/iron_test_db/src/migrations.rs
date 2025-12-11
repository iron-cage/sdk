//! Migration registry for version tracking and guard management

use crate::error::{ Result, TestDbError };
use sqlx::{ SqlitePool, FromRow };
use std::collections::BTreeMap;

/// Migration definition
#[ derive( Debug, Clone ) ]
pub struct Migration
{
  /// Migration version number
  pub version: u32,
  /// Human-readable migration name
  pub name: String,
  /// SQL to execute
  pub sql: &'static str,
}

/// Migration history record
#[ derive( Debug, FromRow ) ]
pub struct MigrationRecord
{
  pub version: i64,
  pub name: String,
  pub applied_at: i64,
}

/// Central registry for database migrations
///
/// Provides automatic guard table management and version tracking.
pub struct MigrationRegistry
{
  migrations: BTreeMap< u32, Migration >,
}

impl MigrationRegistry
{
  /// Create new empty registry
  pub fn new() -> Self
  {
    Self {
      migrations: BTreeMap::new(),
    }
  }

  /// Register a migration (builder pattern)
  pub fn register( mut self, migration: Migration ) -> Self
  {
    self.migrations.insert( migration.version, migration );
    self
  }

  /// Apply all registered migrations in order
  pub async fn apply_all( &self, pool: &SqlitePool ) -> Result< () >
  {
    // Create schema_version table if not exists
    self.init_schema_version_table( pool ).await?;

    for ( version, migration ) in &self.migrations
    {
      if self.is_applied( pool, *version ).await?
      {
        continue;  // Skip already-applied migrations
      }

      // Apply migration
      sqlx::raw_sql( migration.sql )
        .execute( pool )
        .await
        .map_err( |e| TestDbError::Migration(
          format!( "Failed to apply migration {}: {}", version, e )
        ) )?;

      // Mark as applied
      self.mark_applied( pool, *version, &migration.name ).await?;
    }

    Ok( () )
  }

  async fn init_schema_version_table( &self, pool: &SqlitePool ) -> Result< () >
  {
    sqlx::query(
      "CREATE TABLE IF NOT EXISTS _schema_version (
        version INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        applied_at INTEGER NOT NULL
      )"
    )
    .execute( pool )
    .await?;

    Ok( () )
  }

  async fn is_applied( &self, pool: &SqlitePool, version: u32 ) -> Result< bool >
  {
    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM _schema_version WHERE version = ?"
    )
    .bind( version as i64 )
    .fetch_one( pool )
    .await?;

    Ok( count > 0 )
  }

  async fn mark_applied(
    &self,
    pool: &SqlitePool,
    version: u32,
    name: &str,
  ) -> Result< () >
  {
    let now_ms = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_millis() as i64;

    sqlx::query(
      "INSERT INTO _schema_version ( version, name, applied_at )
       VALUES ( ?, ?, ? )"
    )
    .bind( version as i64 )
    .bind( name )
    .bind( now_ms )
    .execute( pool )
    .await?;

    Ok( () )
  }

  /// Get current schema version
  pub async fn current_version( &self, pool: &SqlitePool ) -> Result< Option< u32 > >
  {
    let version: Option< i64 > = sqlx::query_scalar(
      "SELECT MAX( version ) FROM _schema_version"
    )
    .fetch_optional( pool )
    .await?;

    Ok( version.map( |v| v as u32 ) )
  }

  /// Get migration history
  pub async fn history( &self, pool: &SqlitePool ) -> Result< Vec< MigrationRecord > >
  {
    let records: Vec< MigrationRecord > = sqlx::query_as(
      "SELECT version, name, applied_at FROM _schema_version ORDER BY version"
    )
    .fetch_all( pool )
    .await?;

    Ok( records )
  }
}

impl Default for MigrationRegistry
{
  fn default() -> Self
  {
    Self::new()
  }
}
