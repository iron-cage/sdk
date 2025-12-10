//! AI Provider Key storage layer
//!
//! Manages encrypted storage of AI provider API keys (OpenAI, Anthropic).

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions, Row };
use crate::error::Result;

/// Provider type enum
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ProviderType
{
  /// OpenAI provider
  OpenAI,
  /// Anthropic provider
  Anthropic,
}

impl ProviderType
{
  /// Convert to database string representation
  pub fn as_str( &self ) -> &'static str
  {
    match self
    {
      Self::OpenAI => "openai",
      Self::Anthropic => "anthropic",
    }
  }

  /// Parse from database string
  pub fn from_str( s : &str ) -> Option< Self >
  {
    match s
    {
      "openai" => Some( Self::OpenAI ),
      "anthropic" => Some( Self::Anthropic ),
      _ => None,
    }
  }
}

impl std::fmt::Display for ProviderType
{
  fn fmt( &self, f : &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    write!( f, "{}", self.as_str() )
  }
}

/// Provider key metadata (excludes encrypted key)
#[ derive( Debug, Clone ) ]
pub struct ProviderKeyMetadata
{
  /// Database ID
  pub id : i64,
  /// Provider type
  pub provider : ProviderType,
  /// Optional custom base URL
  pub base_url : Option< String >,
  /// Human-friendly description
  pub description : Option< String >,
  /// Whether key is enabled
  pub is_enabled : bool,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at : i64,
  /// Last used timestamp (milliseconds since epoch)
  pub last_used_at : Option< i64 >,
  /// Balance in cents
  pub balance_cents : Option< i64 >,
  /// When balance was last updated
  pub balance_updated_at : Option< i64 >,
  /// User ID who owns this key
  pub user_id : String,
}

/// Full provider key record (includes encrypted data)
#[ derive( Debug, Clone ) ]
pub struct ProviderKeyRecord
{
  /// Metadata
  pub metadata : ProviderKeyMetadata,
  /// Encrypted API key (base64)
  pub encrypted_api_key : String,
  /// Encryption nonce (base64)
  pub encryption_nonce : String,
}

/// Provider key storage layer
#[ derive( Debug, Clone ) ]
pub struct ProviderKeyStorage
{
  pool : SqlitePool,
}

impl ProviderKeyStorage
{
  /// Create new provider key storage from existing pool
  pub fn new( pool : SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new provider key storage with database URL
  ///
  /// # Arguments
  ///
  /// * `database_url` - Database connection string
  ///
  /// # Errors
  ///
  /// Returns error if database connection or migration fails
  pub async fn connect( database_url : &str ) -> Result< Self >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( 5 )
      .connect( database_url )
      .await
      .map_err( |_| crate::error::TokenError )?;

    // Run migration 004 if not already applied
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

    Ok( Self { pool } )
  }

  /// Get the underlying pool for sharing with other storage types
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }

  /// Create a new provider key
  ///
  /// # Arguments
  ///
  /// * `provider` - Provider type (openai, anthropic)
  /// * `encrypted_api_key` - Encrypted API key (base64)
  /// * `encryption_nonce` - Encryption nonce (base64)
  /// * `base_url` - Optional custom base URL
  /// * `description` - Optional description
  /// * `user_id` - Owner user ID
  ///
  /// # Returns
  ///
  /// Database ID of created key
  pub async fn create_key(
    &self,
    provider : ProviderType,
    encrypted_api_key : &str,
    encryption_nonce : &str,
    base_url : Option< &str >,
    description : Option< &str >,
    user_id : &str,
  ) -> Result< i64 >
  {
    let now_ms = current_time_ms();

    let result = sqlx::query(
      "INSERT INTO ai_provider_keys \
       ( provider, encrypted_api_key, encryption_nonce, base_url, description, user_id, created_at ) \
       VALUES ( $1, $2, $3, $4, $5, $6, $7 )"
    )
    .bind( provider.as_str() )
    .bind( encrypted_api_key )
    .bind( encryption_nonce )
    .bind( base_url )
    .bind( description )
    .bind( user_id )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( result.last_insert_rowid() )
  }

  /// Get a provider key by ID (includes encrypted data)
  ///
  /// # Arguments
  ///
  /// * `key_id` - Database ID
  ///
  /// # Returns
  ///
  /// Full key record with encrypted data
  pub async fn get_key( &self, key_id : i64 ) -> Result< ProviderKeyRecord >
  {
    let row = sqlx::query(
      "SELECT id, provider, encrypted_api_key, encryption_nonce, base_url, \
       description, is_enabled, created_at, last_used_at, balance_cents, \
       balance_updated_at, user_id \
       FROM ai_provider_keys WHERE id = $1"
    )
    .bind( key_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( row_to_record( &row ) )
  }

  /// Get a provider key by ID (metadata only, no encrypted data)
  pub async fn get_key_metadata( &self, key_id : i64 ) -> Result< ProviderKeyMetadata >
  {
    let row = sqlx::query(
      "SELECT id, provider, base_url, description, is_enabled, created_at, \
       last_used_at, balance_cents, encrypted_api_key, balance_updated_at, user_id \
       FROM ai_provider_keys WHERE id = $1"
    )
    .bind( key_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( row_to_metadata( &row ) )
  }

  /// List all keys for a user (metadata only)
  ///
  /// # Arguments
  ///
  /// * `user_id` - Owner user ID
  ///
  /// # Returns
  ///
  /// List of key metadata (no encrypted data)
  pub async fn list_keys( &self, user_id : &str ) -> Result< Vec< ProviderKeyMetadata > >
  {
    let rows = sqlx::query(
      "SELECT id, provider, base_url, description, is_enabled, created_at, \
       last_used_at, balance_cents, balance_updated_at, user_id \
       FROM ai_provider_keys WHERE user_id = $1 ORDER BY created_at DESC"
    )
    .bind( user_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( rows.iter().map( |row| row_to_metadata( row ) ).collect() )
  }

  /// Set key enabled/disabled status
  pub async fn set_enabled( &self, key_id : i64, enabled : bool ) -> Result< () >
  {
    sqlx::query( "UPDATE ai_provider_keys SET is_enabled = $1 WHERE id = $2" )
      .bind( enabled )
      .bind( key_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Update description
  pub async fn update_description( &self, key_id : i64, description : Option< &str > ) -> Result< () >
  {
    sqlx::query( "UPDATE ai_provider_keys SET description = $1 WHERE id = $2" )
      .bind( description )
      .bind( key_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Update base URL
  pub async fn update_base_url( &self, key_id : i64, base_url : Option< &str > ) -> Result< () >
  {
    sqlx::query( "UPDATE ai_provider_keys SET base_url = $1 WHERE id = $2" )
      .bind( base_url )
      .bind( key_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Update balance
  pub async fn update_balance( &self, key_id : i64, balance_cents : i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();
    sqlx::query(
      "UPDATE ai_provider_keys SET balance_cents = $1, balance_updated_at = $2 WHERE id = $3"
    )
    .bind( balance_cents )
    .bind( now_ms )
    .bind( key_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Update last used timestamp
  pub async fn update_last_used( &self, key_id : i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();
    sqlx::query( "UPDATE ai_provider_keys SET last_used_at = $1 WHERE id = $2" )
      .bind( now_ms )
      .bind( key_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Delete a key
  pub async fn delete_key( &self, key_id : i64 ) -> Result< () >
  {
    let result = sqlx::query( "DELETE FROM ai_provider_keys WHERE id = $1" )
      .bind( key_id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    if result.rows_affected() == 0
    {
      return Err( crate::error::TokenError );
    }
    Ok( () )
  }

  /// Assign a key to a project
  pub async fn assign_to_project( &self, key_id : i64, project_id : &str ) -> Result< () >
  {
    let now_ms = current_time_ms();
    sqlx::query(
      "INSERT OR REPLACE INTO project_provider_key_assignments \
       ( project_id, provider_key_id, assigned_at ) VALUES ( $1, $2, $3 )"
    )
    .bind( project_id )
    .bind( key_id )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Remove key assignment from a project
  pub async fn unassign_from_project( &self, key_id : i64, project_id : &str ) -> Result< () >
  {
    sqlx::query(
      "DELETE FROM project_provider_key_assignments \
       WHERE project_id = $1 AND provider_key_id = $2"
    )
    .bind( project_id )
    .bind( key_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
    Ok( () )
  }

  /// Get key assigned to a project
  pub async fn get_project_key( &self, project_id : &str ) -> Result< Option< i64 > >
  {
    let row : Option< ( i64, ) > = sqlx::query_as(
      "SELECT provider_key_id FROM project_provider_key_assignments WHERE project_id = $1"
    )
    .bind( project_id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( row.map( |r| r.0 ) )
  }

  /// Get all project assignments for a key
  pub async fn get_key_projects( &self, key_id : i64 ) -> Result< Vec< String > >
  {
    let rows : Vec< ( String, ) > = sqlx::query_as(
      "SELECT project_id FROM project_provider_key_assignments WHERE provider_key_id = $1"
    )
    .bind( key_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( rows.into_iter().map( |r| r.0 ).collect() )
  }
}

fn current_time_ms() -> i64
{
  std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64
}

fn row_to_metadata( row : &sqlx::sqlite::SqliteRow ) -> ProviderKeyMetadata
{
  let provider_str : String = row.get( "provider" );
  ProviderKeyMetadata
  {
    id : row.get( "id" ),
    provider : ProviderType::from_str( &provider_str ).unwrap_or( ProviderType::OpenAI ),
    base_url : row.get( "base_url" ),
    description : row.get( "description" ),
    is_enabled : row.get( "is_enabled" ),
    created_at : row.get( "created_at" ),
    last_used_at : row.get( "last_used_at" ),
    balance_cents : row.get( "balance_cents" ),
    balance_updated_at : row.get( "balance_updated_at" ),
    user_id : row.get( "user_id" ),
  }
}

fn row_to_record( row : &sqlx::sqlite::SqliteRow ) -> ProviderKeyRecord
{
  let provider_str : String = row.get( "provider" );
  ProviderKeyRecord
  {
    metadata : ProviderKeyMetadata
    {
      id : row.get( "id" ),
      provider : ProviderType::from_str( &provider_str ).unwrap_or( ProviderType::OpenAI ),
      base_url : row.get( "base_url" ),
      description : row.get( "description" ),
      is_enabled : row.get( "is_enabled" ),
      created_at : row.get( "created_at" ),
      last_used_at : row.get( "last_used_at" ),
      balance_cents : row.get( "balance_cents" ),
      balance_updated_at : row.get( "balance_updated_at" ),
      user_id : row.get( "user_id" ),
    },
    encrypted_api_key : row.get( "encrypted_api_key" ),
    encryption_nonce : row.get( "encryption_nonce" ),
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ tokio::test ]
  async fn create_and_get_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();

    let key_id = storage.create_key(
      ProviderType::OpenAI,
      "encrypted_data_base64",
      "nonce_base64",
      None,
      Some( "Test key" ),
      "user_123",
    ).await.unwrap();

    let record = storage.get_key( key_id ).await.unwrap();
    assert_eq!( record.metadata.provider, ProviderType::OpenAI );
    assert_eq!( record.metadata.description, Some( "Test key".to_string() ) );
    assert_eq!( record.metadata.user_id, "user_123" );
    assert!( record.metadata.is_enabled );
    assert_eq!( record.encrypted_api_key, "encrypted_data_base64" );
    assert_eq!( record.encryption_nonce, "nonce_base64" );
  }

  #[ tokio::test ]
  async fn list_keys_by_user()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();

    storage.create_key( ProviderType::OpenAI, "enc1", "nonce1", None, Some( "Key 1" ), "user_a" ).await.unwrap();
    storage.create_key( ProviderType::Anthropic, "enc2", "nonce2", None, Some( "Key 2" ), "user_a" ).await.unwrap();
    storage.create_key( ProviderType::OpenAI, "enc3", "nonce3", None, Some( "Key 3" ), "user_b" ).await.unwrap();

    let user_a_keys = storage.list_keys( "user_a" ).await.unwrap();
    assert_eq!( user_a_keys.len(), 2 );

    let user_b_keys = storage.list_keys( "user_b" ).await.unwrap();
    assert_eq!( user_b_keys.len(), 1 );
  }

  #[ tokio::test ]
  async fn enable_disable_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Initially enabled
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.is_enabled );

    // Disable
    storage.set_enabled( key_id, false ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( !meta.is_enabled );

    // Enable again
    storage.set_enabled( key_id, true ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.is_enabled );
  }

  #[ tokio::test ]
  async fn update_balance()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Initially no balance
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert!( meta.balance_cents.is_none() );

    // Update balance
    storage.update_balance( key_id, 10000 ).await.unwrap();
    let meta = storage.get_key_metadata( key_id ).await.unwrap();
    assert_eq!( meta.balance_cents, Some( 10000 ) );
    assert!( meta.balance_updated_at.is_some() );
  }

  #[ tokio::test ]
  async fn delete_key()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // Delete
    storage.delete_key( key_id ).await.unwrap();

    // Should fail to get
    let result = storage.get_key( key_id ).await;
    assert!( result.is_err() );
  }

  #[ tokio::test ]
  async fn project_assignment()
  {
    let storage = ProviderKeyStorage::connect( "sqlite::memory:" ).await.unwrap();
    let key_id = storage.create_key( ProviderType::OpenAI, "enc", "nonce", None, None, "user" ).await.unwrap();

    // No assignment initially
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert!( assigned.is_none() );

    // Assign
    storage.assign_to_project( key_id, "project_abc" ).await.unwrap();
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert_eq!( assigned, Some( key_id ) );

    // Get projects for key
    let projects = storage.get_key_projects( key_id ).await.unwrap();
    assert_eq!( projects, vec![ "project_abc".to_string() ] );

    // Unassign
    storage.unassign_from_project( key_id, "project_abc" ).await.unwrap();
    let assigned = storage.get_project_key( "project_abc" ).await.unwrap();
    assert!( assigned.is_none() );
  }
}
