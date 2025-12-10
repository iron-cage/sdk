// qqq : implement database storage
//! Database storage layer
//!
//! `SQLite`/`PostgreSQL` persistence for tokens, usage, limits, traces, audit logs.

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions, Row };
use crate::token_generator::TokenGenerator;
use crate::error::Result;

/// Token metadata returned from storage
#[ derive( Debug, Clone ) ]
pub struct TokenMetadata
{
  /// Database ID
  pub id: i64,
  /// User ID who owns this token
  pub user_id: String,
  /// Optional project ID
  pub project_id: Option< String >,
  /// Optional human-friendly name
  pub name: Option< String >,
  /// Whether token is active
  pub is_active: bool,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last used timestamp (milliseconds since epoch)
  pub last_used_at: Option< i64 >,
  /// Expiration timestamp (milliseconds since epoch)
  pub expires_at: Option< i64 >,
}

/// Token storage layer
///
/// Manages token persistence in `SQLite` database.
/// Stores SHA-256 hashes, never plaintext tokens.
#[ derive( Debug, Clone ) ]
pub struct TokenStorage
{
  pool: SqlitePool,
  generator: TokenGenerator,
}

impl TokenStorage
{
  /// Create new token storage
  ///
  /// # Arguments
  ///
  /// * `database_url` - Database connection string
  ///
  /// # Returns
  ///
  /// Initialized storage with database schema applied
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails or migration fails
  pub async fn new( database_url: &str ) -> Result< Self >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( 5 )
      .connect( database_url )
      .await
      .map_err( |_| crate::error::TokenError )?;

    // Run migrations in order
    // Migration 001: Initial schema (tables, indexes, foreign keys)
    let migration_001 = include_str!( "../migrations/001_initial_schema.sql" );
    sqlx::raw_sql( migration_001 )
      .execute( &pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    // Migration 002: Length constraints (defense-in-depth for issue-001)
    // Fix(issue-003): Check if migration already applied to prevent CASCADE DELETE data loss
    // Root cause: Migration dropped api_tokens table on every run, cascading to token_usage deletion
    // Pitfall: Always check migration guard tables before running destructive schema changes
    let migration_002_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_002_completed'"
    )
    .fetch_one( &pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Only run migration if guard table doesn't exist
    if migration_002_completed == 0
    {
      let migration_002 = include_str!( "../migrations/002_add_length_constraints.sql" );
      sqlx::raw_sql( migration_002 )
        .execute( &pool )
        .await
        .map_err( |_| crate::error::TokenError )?;
    }

    // Migration 003: Users table for authentication
    let migration_003_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_003_completed'"
    )
    .fetch_one( &pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    if migration_003_completed == 0
    {
      let migration_003 = include_str!( "../migrations/003_create_users_table.sql" );
      sqlx::raw_sql( migration_003 )
        .execute( &pool )
        .await
        .map_err( |_| crate::error::TokenError )?;
    }

    // Migration 004: AI provider keys table
    let migration_004_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_004_completed'"
    )
    .fetch_one( &pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    if migration_004_completed == 0
    {
      let migration_004 = include_str!( "../migrations/004_create_ai_provider_keys.sql" );
      sqlx::raw_sql( migration_004 )
        .execute( &pool )
        .await
        .map_err( |_| crate::error::TokenError )?;
    }

    // Migration 005: Enhance users table for user management
    let migration_005_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_005_completed'"
    )
    .fetch_one( &pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    if migration_005_completed == 0
    {
      let migration_005 = include_str!( "../migrations/005_enhance_users_table.sql" );
      sqlx::raw_sql( migration_005 )
        .execute( &pool )
        .await
        .map_err( |_| crate::error::TokenError )?;
    }

    // Migration 006: Create user audit log table
    let migration_006_completed : i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_006_completed'"
    )
    .fetch_one( &pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    if migration_006_completed == 0
    {
      let migration_006 = include_str!( "../migrations/006_create_user_audit_log.sql" );
      sqlx::raw_sql( migration_006 )
        .execute( &pool )
        .await
        .map_err( |_| crate::error::TokenError )?;
    }

    Ok( Self {
      pool,
      generator: TokenGenerator::new(),
    } )
  }

  /// Create new token in database
  ///
  /// # Arguments
  ///
  /// * `plaintext_token` - Token to store (will be hashed)
  /// * `user_id` - User who owns this token
  /// * `project_id` - Optional project ID
  /// * `name` - Optional human-friendly name
  ///
  /// # Returns
  ///
  /// Database ID of created token
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn create_token(
    &self,
    plaintext_token: &str,
    user_id: &str,
    project_id: Option< &str >,
    name: Option< &str >,
  ) -> Result< i64 >
  {
    let now_ms = current_time_ms();
    let token_hash = self.generator.hash_token( plaintext_token );

    let result = sqlx::query(
      "INSERT INTO api_tokens (token_hash, user_id, project_id, name, created_at) \
       VALUES ($1, $2, $3, $4, $5)"
    )
    .bind( &token_hash )
    .bind( user_id )
    .bind( project_id )
    .bind( name )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( result.last_insert_rowid() )
  }

  /// Create token with custom expiration
  ///
  /// # Arguments
  ///
  /// * `plaintext_token` - Token to store (will be hashed)
  /// * `user_id` - User who owns this token
  /// * `project_id` - Optional project ID
  /// * `name` - Optional human-friendly name
  /// * `expires_at` - Expiration timestamp (milliseconds since epoch)
  ///
  /// # Returns
  ///
  /// Database ID of created token
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn create_token_with_expiry(
    &self,
    plaintext_token: &str,
    user_id: &str,
    project_id: Option< &str >,
    name: Option< &str >,
    expires_at: Option< i64 >,
  ) -> Result< i64 >
  {
    let now_ms = current_time_ms();
    let token_hash = self.generator.hash_token( plaintext_token );

    let result = sqlx::query(
      "INSERT INTO api_tokens (token_hash, user_id, project_id, name, created_at, expires_at) \
       VALUES ($1, $2, $3, $4, $5, $6)"
    )
    .bind( &token_hash )
    .bind( user_id )
    .bind( project_id )
    .bind( name )
    .bind( now_ms )
    .bind( expires_at )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( result.last_insert_rowid() )
  }

  /// Verify token and return its database ID
  ///
  /// # Arguments
  ///
  /// * `plaintext_token` - Token to verify
  ///
  /// # Returns
  ///
  /// Database ID if token is valid and active
  ///
  /// # Errors
  ///
  /// Returns error if token is invalid, inactive, or expired
  pub async fn verify_token( &self, plaintext_token: &str ) -> Result< i64 >
  {
    let token_hash = self.generator.hash_token( plaintext_token );
    let now_ms = current_time_ms();

    let row = sqlx::query(
      "SELECT id FROM api_tokens \
       WHERE token_hash = $1 \
       AND is_active = 1 \
       AND (expires_at IS NULL OR expires_at > $2)"
    )
    .bind( &token_hash )
    .bind( now_ms )
    .fetch_optional( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    row
      .map( |r| r.get::< i64, _ >( "id" ) )
      .ok_or( crate::error::TokenError )
  }

  /// Get token hash by ID
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  ///
  /// # Returns
  ///
  /// SHA-256 hash of token
  ///
  /// # Errors
  ///
  /// Returns error if token ID not found
  pub async fn get_token_hash( &self, token_id: i64 ) -> Result< String >
  {
    let row = sqlx::query( "SELECT token_hash FROM api_tokens WHERE id = $1" )
      .bind( token_id )
      .fetch_one( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    Ok( row.get::< String, _ >( "token_hash" ) )
  }

  /// Get token metadata by ID
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  ///
  /// # Returns
  ///
  /// Complete token metadata
  ///
  /// # Errors
  ///
  /// Returns error if token ID not found
  pub async fn get_token_metadata( &self, token_id: i64 ) -> Result< TokenMetadata >
  {
    let row = sqlx::query(
      "SELECT id, user_id, project_id, name, is_active, created_at, last_used_at, expires_at \
       FROM api_tokens WHERE id = $1"
    )
    .bind( token_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( TokenMetadata {
      id: row.get( "id" ),
      user_id: row.get( "user_id" ),
      project_id: row.get( "project_id" ),
      name: row.get( "name" ),
      is_active: row.get::< bool, _ >( "is_active" ),
      created_at: row.get( "created_at" ),
      last_used_at: row.get( "last_used_at" ),
      expires_at: row.get( "expires_at" ),
    } )
  }

  /// Deactivate token
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token to deactivate
  ///
  /// # Errors
  ///
  /// Returns error if database update fails or token not found (0 rows affected)
  pub async fn deactivate_token( &self, token_id: i64 ) -> Result< () >
  {
    let result = sqlx::query( "UPDATE api_tokens SET is_active = 0 WHERE id = $1 AND is_active = 1" )
      .bind( token_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError );
    }

    Ok( () )
  }

  /// Update last used timestamp
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_last_used( &self, token_id: i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query( "UPDATE api_tokens SET last_used_at = $1 WHERE id = $2" )
      .bind( now_ms )
      .bind( token_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    Ok( () )
  }

  /// List all tokens for a user
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID to list tokens for
  ///
  /// # Returns
  ///
  /// Vector of token metadata
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn list_user_tokens( &self, user_id: &str ) -> Result< Vec< TokenMetadata > >
  {
    let rows = sqlx::query(
      "SELECT id, user_id, project_id, name, is_active, created_at, last_used_at, expires_at \
       FROM api_tokens WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind( user_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| TokenMetadata {
        id: row.get( "id" ),
        user_id: row.get( "user_id" ),
        project_id: row.get( "project_id" ),
        name: row.get( "name" ),
        is_active: row.get::< bool, _ >( "is_active" ),
        created_at: row.get( "created_at" ),
        last_used_at: row.get( "last_used_at" ),
        expires_at: row.get( "expires_at" ),
      } ).collect()
    )
  }

  /// Delete token permanently
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token to delete
  ///
  /// # Errors
  ///
  /// Returns error if database delete fails or token not found (0 rows affected)
  pub async fn delete_token( &self, token_id: i64 ) -> Result< () >
  {
    let result = sqlx::query( "DELETE FROM api_tokens WHERE id = $1" )
      .bind( token_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError );
    }

    Ok( () )
  }

  // Fix(issue-003): Database pool accessor for test verification
  // Root cause: Tests needed direct database access to verify security properties (plaintext never stored, BCrypt used)
  // Pitfall: Only expose internal state for test verification, never bypass validation logic

  /// Get database pool for test verification only
  ///
  /// **Warning:** This method is intended for integration tests only. It exposes
  /// internal database state for verification of security properties. Do not use
  /// in production code.
  ///
  /// Allows tests to query database directly to verify security properties:
  /// - Token plaintext is never stored in database (P0-9)
  /// - Token hash uses `BCrypt` algorithm (P0-10)
  ///
  /// # Returns
  ///
  /// Reference to underlying `SQLite` connection pool
  ///
  /// # Example
  ///
  /// ```rust,ignore
  /// // In test: verify token plaintext never stored
  /// let row: (String,) = sqlx::query_as("SELECT hash FROM api_tokens WHERE id = ?")
  ///   .bind(token_id)
  ///   .fetch_one(storage.pool())
  ///   .await
  ///   .unwrap();
  /// assert_ne!(row.0, plaintext_token, "Plaintext must not be stored");
  /// ```
  #[ must_use ]
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }
}

/// Get current time in milliseconds since UNIX epoch
#[ allow( clippy::cast_possible_truncation ) ]
fn current_time_ms() -> i64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64
}
