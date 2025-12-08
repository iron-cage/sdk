// qqq : implement trace storage
//! Trace storage service
//!
//! Stores and queries detailed API call traces for debugging and cost analysis.

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions, Row };
use crate::error::Result;

/// API call trace record from database
#[ derive( Debug, Clone ) ]
pub struct TraceRecord
{
  /// Database ID
  pub id: i64,
  /// Token ID this trace belongs to
  pub token_id: i64,
  /// Provider name (openai, anthropic, gemini)
  pub provider: String,
  /// Model name (gpt-4, claude-sonnet-4-5-20250929, etc.)
  pub model: String,
  /// API endpoint path
  pub endpoint: String,
  /// HTTP response status code
  pub response_status: i32,
  /// Request duration in milliseconds
  pub duration_ms: i64,
  /// Input tokens consumed
  pub input_tokens: i64,
  /// Output tokens generated
  pub output_tokens: i64,
  /// Total tokens (input + output)
  pub total_tokens: i64,
  /// Cost in cents
  pub cost_cents: i64,
  /// Timestamp when trace was recorded (milliseconds since epoch)
  pub traced_at: i64,
}

/// Trace storage
///
/// Stores and queries API call traces with real database persistence.
#[ derive( Debug, Clone ) ]
pub struct TraceStorage
{
  pool: SqlitePool,
}

impl TraceStorage
{
  /// Create new trace storage
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

    // Run migrations
    let migration_sql = include_str!( "../migrations/001_initial_schema.sql" );
    sqlx::raw_sql( migration_sql )
      .execute( &pool )
      .await
      .map_err( |_| crate::error::TokenError )?;

    Ok( Self { pool } )
  }

  /// Get all trace records
  ///
  /// # Returns
  ///
  /// Vector of all trace records ordered by timestamp (newest first)
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_all_traces( &self ) -> Result< Vec< TraceRecord > >
  {
    let rows = sqlx::query(
      "SELECT id, token_id, provider, model, endpoint, response_status, duration_ms, \
       input_tokens, output_tokens, (input_tokens + output_tokens) as total_tokens, cost_cents, traced_at \
       FROM api_call_traces ORDER BY traced_at DESC"
    )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| TraceRecord {
        id: row.get( "id" ),
        token_id: row.get( "token_id" ),
        provider: row.get( "provider" ),
        model: row.get( "model" ),
        endpoint: row.get( "endpoint" ),
        response_status: row.get( "response_status" ),
        duration_ms: row.get( "duration_ms" ),
        input_tokens: row.get( "input_tokens" ),
        output_tokens: row.get( "output_tokens" ),
        total_tokens: row.get( "total_tokens" ),
        cost_cents: row.get( "cost_cents" ),
        traced_at: row.get( "traced_at" ),
      } ).collect()
    )
  }

  /// Get trace by ID
  ///
  /// # Arguments
  ///
  /// * `id` - Trace ID
  ///
  /// # Returns
  ///
  /// Trace record
  ///
  /// # Errors
  ///
  /// Returns error if trace not found or database query fails
  pub async fn get_trace_by_id( &self, id: i64 ) -> Result< TraceRecord >
  {
    let row = sqlx::query(
      "SELECT id, token_id, provider, model, endpoint, response_status, duration_ms, \
       input_tokens, output_tokens, (input_tokens + output_tokens) as total_tokens, cost_cents, traced_at \
       FROM api_call_traces WHERE id = $1"
    )
    .bind( id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?
    .ok_or( crate::error::TokenError )?;

    Ok( TraceRecord {
      id: row.get( "id" ),
      token_id: row.get( "token_id" ),
      provider: row.get( "provider" ),
      model: row.get( "model" ),
      endpoint: row.get( "endpoint" ),
      response_status: row.get( "response_status" ),
      duration_ms: row.get( "duration_ms" ),
      input_tokens: row.get( "input_tokens" ),
      output_tokens: row.get( "output_tokens" ),
      total_tokens: row.get( "total_tokens" ),
      cost_cents: row.get( "cost_cents" ),
      traced_at: row.get( "traced_at" ),
    } )
  }

  /// Get traces for specific token
  ///
  /// # Arguments
  ///
  /// * `token_id` - Token ID
  ///
  /// # Returns
  ///
  /// Vector of trace records for the token
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_traces_for_token( &self, token_id: i64 ) -> Result< Vec< TraceRecord > >
  {
    let rows = sqlx::query(
      "SELECT id, token_id, provider, model, endpoint, response_status, duration_ms, \
       input_tokens, output_tokens, (input_tokens + output_tokens) as total_tokens, cost_cents, traced_at \
       FROM api_call_traces WHERE token_id = $1 ORDER BY traced_at DESC"
    )
    .bind( token_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| TraceRecord {
        id: row.get( "id" ),
        token_id: row.get( "token_id" ),
        provider: row.get( "provider" ),
        model: row.get( "model" ),
        endpoint: row.get( "endpoint" ),
        response_status: row.get( "response_status" ),
        duration_ms: row.get( "duration_ms" ),
        input_tokens: row.get( "input_tokens" ),
        output_tokens: row.get( "output_tokens" ),
        total_tokens: row.get( "total_tokens" ),
        cost_cents: row.get( "cost_cents" ),
        traced_at: row.get( "traced_at" ),
      } ).collect()
    )
  }
}
