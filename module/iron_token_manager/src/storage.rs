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
  /// Optional agent ID
  pub agent_id: Option< i64 >,
  /// Optional provider
  pub provider: Option< String >,
  /// Whether token is active
  pub is_active: bool,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last used timestamp (milliseconds since epoch)
  pub last_used_at: Option< i64 >,
  /// Expiration timestamp (milliseconds since epoch)
  pub expires_at: Option< i64 >,
  /// Revocation timestamp (milliseconds since epoch, NULL if rotated/deactivated)
  pub revoked_at: Option< i64 >,
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
  /// Create new token storage from existing pool
  ///
  /// Preferred constructor for test environments using `iron_test_db`.
  /// Does NOT apply migrations - caller is responsible for schema setup.
  ///
  /// # Arguments
  ///
  /// * `pool` - Existing database connection pool
  ///
  /// # Returns
  ///
  /// Initialized storage using provided pool
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use iron_test_db::TestDatabaseBuilder;
  /// use iron_token_manager::storage::TokenStorage;
  ///
  /// let db = TestDatabaseBuilder::new().in_memory().build().await?;
  /// let storage = TokenStorage::from_pool( db.pool().clone() );
  /// ```
  #[ must_use ]
  pub fn from_pool( pool: SqlitePool ) -> Self
  {
    Self {
      pool,
      generator: TokenGenerator::new(),
    }
  }

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
      .map_err( crate::error::TokenError::Database )?;

    // Apply all migrations using unified helper
    crate::migrations::apply_all_migrations( &pool ).await?;
    Ok( Self {
      pool,
      generator: TokenGenerator::new(),
    } )
  }

  /// Create new token storage from configuration file
  ///
  /// This is the preferred initialization method for production use.
  /// Supports configuration files and environment variable overrides.
  ///
  /// # Returns
  ///
  /// Initialized storage with database schema applied according to config
  ///
  /// # Errors
  ///
  /// Returns error if config loading fails, database connection fails, or migration fails
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use iron_token_manager::storage::TokenStorage;
  ///
  /// // Load from default environment (IRON_ENV or "development")
  /// let storage = TokenStorage::from_config().await?;
  ///
  /// // Override via environment variable
  /// std::env::set_var("IRON_ENV", "production");
  /// let storage = TokenStorage::from_config().await?;
  /// ```
  pub async fn from_config() -> Result< Self >
  {
    let config = crate::config::Config::load()?;
    Self::from_config_object( &config ).await
  }

  /// Create new token storage from configuration object
  ///
  /// # Arguments
  ///
  /// * `config` - Configuration object
  ///
  /// # Returns
  ///
  /// Initialized storage with database schema applied according to config
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails or migration fails
  pub async fn from_config_object( config: &crate::config::Config ) -> Result< Self >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( config.database.max_connections )
      .connect( &config.database.url )
      .await
      .map_err( crate::error::TokenError::Database )?;

    // Apply migrations if configured
    if config.database.auto_migrate
    {
      crate::migrations::apply_all_migrations( &pool ).await?;
    }

    // Wipe and seed if configured (development/test only)
    let should_wipe_and_seed = config.development
      .as_ref()
      .map( |d| d.wipe_and_seed )
      .or_else( || config.test.as_ref().map( |t| t.wipe_and_seed ) )
      .unwrap_or( false );

    if should_wipe_and_seed
    {
      crate::seed::wipe_database( &pool ).await?;
      crate::seed::seed_all( &pool ).await?;
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
    agent_id: Option< i64 >,
    provider: Option< &str >,
  ) -> Result< i64 >
  {
    let now_ms = current_time_ms();
    let token_hash = self.generator.hash_token( plaintext_token );

    let result = sqlx::query(
      "INSERT INTO api_tokens (token_hash, user_id, project_id, name, agent_id, provider, created_at) \
       VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( &token_hash )
    .bind( user_id )
    .bind( project_id )
    .bind( name )
    .bind( agent_id )
    .bind( provider )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .unwrap();
    // .map_err( crate::error::TokenError::Database )?;

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
    .map_err( crate::error::TokenError::Database )?;

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
    .map_err( crate::error::TokenError::Database )?;

    row
      .map( |r| r.get::< i64, _ >( "id" ) )
      .ok_or( crate::error::TokenError::Generic )
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
      .map_err( crate::error::TokenError::Database )?;

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
      "SELECT id, user_id, project_id, name, agent_id, provider, is_active, created_at, last_used_at, expires_at, revoked_at \
       FROM api_tokens WHERE id = $1"
    )
    .bind( token_id )
    .fetch_one( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    Ok( TokenMetadata {
      id: row.get( "id" ),
      user_id: row.get( "user_id" ),
      project_id: row.get( "project_id" ),
      name: row.get( "name" ),
      agent_id: row.get( "agent_id" ),
      provider: row.get( "provider" ),
      is_active: row.get::< bool, _ >( "is_active" ),
      created_at: row.get( "created_at" ),
      last_used_at: row.get( "last_used_at" ),
      expires_at: row.get( "expires_at" ),
      revoked_at: row.get( "revoked_at" ),
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
      .map_err( crate::error::TokenError::Database )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError::Generic );
    }

    Ok( () )
  }

  /// Revoke token (deactivate and mark as explicitly revoked)
  ///
  /// Sets both `is_active = 0` and `revoked_at = current_timestamp`.
  /// Enables distinguishing explicit revocations (409) from rotations (404).
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token to revoke
  ///
  /// # Errors
  ///
  /// Returns error if database update fails or token not found (0 rows affected)
  pub async fn revoke_token( &self, token_id: i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();

    let result = sqlx::query(
      "UPDATE api_tokens SET is_active = 0, revoked_at = $1 WHERE id = $2 AND is_active = 1"
    )
    .bind( now_ms )
    .bind( token_id )
    .execute( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError::Generic );
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
      .map_err( crate::error::TokenError::Database )?;

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
      "SELECT id, user_id, project_id, name, agent_id, provider, is_active, created_at, last_used_at, expires_at, revoked_at \
       FROM api_tokens WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind( user_id )
    .fetch_all( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    Ok(
      rows.iter().map( |row| TokenMetadata {
        id: row.get( "id" ),
        user_id: row.get( "user_id" ),
        project_id: row.get( "project_id" ),
        name: row.get( "name" ),
        agent_id: row.get( "agent_id" ),
        provider: row.get( "provider" ),
        is_active: row.get::< bool, _ >( "is_active" ),
        created_at: row.get( "created_at" ),
        last_used_at: row.get( "last_used_at" ),
        expires_at: row.get( "expires_at" ),
        revoked_at: row.get( "revoked_at" ),
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
      .map_err( crate::error::TokenError::Database )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError::Generic );
    }

    Ok( () )
  }

  /// Update token provider
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token to update
  /// * `provider` - New provider for token
  ///
  /// # Errors
  ///
  /// Returns error if database update fails or token not found (0 rows affected)
  pub async fn update_token_provider( &self, token_id: i64, provider: &str ) -> Result< () >
  {
    let result = sqlx::query( "UPDATE api_tokens SET provider = $2 WHERE id = $1" )
      .bind( token_id )
      .bind( provider )
      .execute( &self.pool )
      .await
      .map_err( crate::error::TokenError::Database )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError::Generic );
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

  /// Get token generator for hash operations
  ///
  /// # Returns
  ///
  /// Reference to the token generator used for hashing token values
  #[ must_use ]
  pub fn generator( &self ) -> &TokenGenerator
  {
    &self.generator
  }

  /// Log audit event to `audit_log` table
  ///
  /// # Arguments
  ///
  /// * `entity_type` - Type of entity ("token", "limit", "usage")
  /// * `entity_id` - ID of the entity
  /// * `action` - Action performed ("created", "revoked", "updated", etc.)
  /// * `actor_user_id` - User who performed the action
  /// * `changes` - JSON object with before/after values (optional)
  ///
  /// # Returns
  ///
  /// Ok if audit log entry was created successfully
  ///
  /// # Errors
  ///
  /// Returns error if database write fails
  ///
  /// # Security
  ///
  /// - Never log plaintext token values in changes field
  /// - Always log who performed the action for accountability
  pub async fn log_audit_event(
    &self,
    entity_type: &str,
    entity_id: i64,
    action: &str,
    actor_user_id: &str,
    changes: Option< &str >,
  ) -> Result< () >
  {
    let logged_at = current_time_ms();

    sqlx::query(
      "INSERT INTO audit_log (entity_type, entity_id, action, actor_user_id, changes, logged_at)
       VALUES (?, ?, ?, ?, ?, ?)"
    )
    .bind( entity_type )
    .bind( entity_id )
    .bind( action )
    .bind( actor_user_id )
    .bind( changes )
    .bind( logged_at )
    .execute( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    Ok( () )
  }

  /// Count active tokens for a user
  ///
  /// Used for enforcing max active tokens limit (Protocol 014: max 10 per user).
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID to check
  ///
  /// # Returns
  ///
  /// Number of active tokens for this user
  ///
  /// # Errors
  ///
  /// Returns `TokenError` if database query fails
  pub async fn count_active_tokens_for_user( &self, user_id: &str ) -> Result< i64 >
  {
    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM api_tokens WHERE user_id = ? AND is_active = 1"
    )
    .bind( user_id )
    .fetch_one( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    Ok( count )
  }

  /// Count token creations in the last minute for a user
  ///
  /// Used for rate limiting (Protocol 014: max 10 creates/min per user).
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID to check
  ///
  /// # Returns
  ///
  /// Number of tokens created in the last 60 seconds
  ///
  /// # Errors
  ///
  /// Returns `TokenError` if database query fails
  pub async fn count_recent_token_creations( &self, user_id: &str ) -> Result< i64 >
  {
    let one_minute_ago = current_time_ms() - 60_000;  // 60 seconds in milliseconds

    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM api_tokens WHERE user_id = ? AND created_at > ?"
    )
    .bind( user_id )
    .bind( one_minute_ago )
    .fetch_one( &self.pool )
    .await
    .map_err( crate::error::TokenError::Database )?;

    Ok( count )
  }
}

/// Get current time in milliseconds since UNIX epoch
#[ allow( clippy::cast_possible_truncation ) ]
pub( crate ) fn current_time_ms() -> i64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "LOUD FAILURE: Time went backwards" )
    .as_millis() as i64
}
