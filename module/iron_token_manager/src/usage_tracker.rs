// qqq : implement usage tracking
//! Usage tracking service
//!
//! Tracks LLM API usage (tokens, requests, costs) per user/project/token.

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions, Row };
use crate::error::Result;

/// Usage record from database
#[ derive( Debug, Clone ) ]
pub struct UsageRecord
{
  /// Database ID
  pub id: i64,
  /// Token ID this usage belongs to
  pub token_id: i64,
  /// Provider name (openai, anthropic, gemini)
  pub provider: String,
  /// Model name (gpt-4, claude-sonnet-4-5-20250929, etc.)
  pub model: String,
  /// Input tokens consumed
  pub input_tokens: i64,
  /// Output tokens generated
  pub output_tokens: i64,
  /// Total tokens (input + output)
  pub total_tokens: i64,
  /// Number of requests
  pub requests_count: i64,
  /// Cost in cents
  pub cost_cents: i64,
  /// Timestamp when usage was recorded (milliseconds since epoch)
  pub recorded_at: i64,
}

/// Aggregated usage statistics
#[ derive( Debug, Clone ) ]
pub struct AggregateUsage
{
  /// Total input tokens
  pub input_tokens: i64,
  /// Total output tokens
  pub output_tokens: i64,
  /// Total tokens (input + output)
  pub total_tokens: i64,
  /// Total number of requests
  pub total_requests: i64,
  /// Total cost in cents
  pub total_cost_cents: i64,
}

/// Usage tracker
///
/// Tracks LLM API usage with real database persistence.
#[ derive( Debug, Clone ) ]
pub struct UsageTracker
{
  pool: SqlitePool,
}

impl UsageTracker
{
  /// Create new usage tracker from existing pool
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
  /// Initialized tracker using provided pool
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use iron_test_db::TestDatabaseBuilder;
  /// use iron_token_manager::usage_tracker::UsageTracker;
  ///
  /// let db = TestDatabaseBuilder::new().in_memory().build().await?;
  /// let tracker = UsageTracker::from_pool( db.pool().clone() );
  /// ```
  #[ must_use ]
  pub fn from_pool( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new usage tracker
  ///
  /// # Arguments
  ///
  /// * `database_url` - Database connection string
  ///
  /// # Returns
  ///
  /// Initialized tracker with database schema applied
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

  /// Record usage without cost
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  /// * `provider` - Provider name (openai, anthropic, gemini)
  /// * `model` - Model name
  /// * `input_tokens` - Input tokens consumed
  /// * `output_tokens` - Output tokens generated
  /// * `total_tokens` - Total tokens (input + output)
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn record_usage(
    &self,
    token_id: i64,
    provider: &str,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    total_tokens: i64,
  ) -> Result< () >
  {
    self.record_usage_with_cost( token_id, provider, model, input_tokens, output_tokens, total_tokens, 0 ).await
  }

  /// Record usage with cost
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  /// * `provider` - Provider name (openai, anthropic, gemini)
  /// * `model` - Model name
  /// * `input_tokens` - Input tokens consumed
  /// * `output_tokens` - Output tokens generated
  /// * `total_tokens` - Total tokens (input + output)
  /// * `cost_cents` - Cost in cents
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  #[ allow( clippy::too_many_arguments ) ]
  pub async fn record_usage_with_cost(
    &self,
    token_id: i64,
    provider: &str,
    model: &str,
    input_tokens: i64,
    output_tokens: i64,
    total_tokens: i64,
    cost_cents: i64,
  ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "INSERT INTO token_usage (token_id, provider, model, input_tokens, output_tokens, total_tokens, cost_cents, recorded_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8)"
    )
    .bind( token_id )
    .bind( provider )
    .bind( model )
    .bind( input_tokens )
    .bind( output_tokens )
    .bind( total_tokens )
    .bind( cost_cents )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( () )
  }

  /// Get all usage records for a token
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  ///
  /// # Returns
  ///
  /// Vector of usage records
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_token_usage( &self, token_id: i64 ) -> Result< Vec< UsageRecord > >
  {
    let rows = sqlx::query(
      "SELECT id, token_id, provider, model, input_tokens, output_tokens, total_tokens, requests_count, cost_cents, recorded_at \
       FROM token_usage WHERE token_id = $1 ORDER BY recorded_at DESC"
    )
    .bind( token_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| UsageRecord {
        id: row.get( "id" ),
        token_id: row.get( "token_id" ),
        provider: row.get( "provider" ),
        model: row.get( "model" ),
        input_tokens: row.get( "input_tokens" ),
        output_tokens: row.get( "output_tokens" ),
        total_tokens: row.get( "total_tokens" ),
        requests_count: row.get( "requests_count" ),
        cost_cents: row.get( "cost_cents" ),
        recorded_at: row.get( "recorded_at" ),
      } ).collect()
    )
  }

  /// Get usage records filtered by provider
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  /// * `provider` - Provider name to filter by
  ///
  /// # Returns
  ///
  /// Vector of usage records for specified provider
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_by_provider( &self, token_id: i64, provider: &str ) -> Result< Vec< UsageRecord > >
  {
    let rows = sqlx::query(
      "SELECT id, token_id, provider, model, input_tokens, output_tokens, total_tokens, requests_count, cost_cents, recorded_at \
       FROM token_usage WHERE token_id = $1 AND provider = $2 ORDER BY recorded_at DESC"
    )
    .bind( token_id )
    .bind( provider )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| UsageRecord {
        id: row.get( "id" ),
        token_id: row.get( "token_id" ),
        provider: row.get( "provider" ),
        model: row.get( "model" ),
        input_tokens: row.get( "input_tokens" ),
        output_tokens: row.get( "output_tokens" ),
        total_tokens: row.get( "total_tokens" ),
        requests_count: row.get( "requests_count" ),
        cost_cents: row.get( "cost_cents" ),
        recorded_at: row.get( "recorded_at" ),
      } ).collect()
    )
  }

  /// Get aggregated usage statistics
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  ///
  /// # Returns
  ///
  /// Aggregated usage statistics
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_aggregate_usage( &self, token_id: i64 ) -> Result< AggregateUsage >
  {
    let row = sqlx::query(
      "SELECT \
       COALESCE(SUM(input_tokens), 0) as input_tokens, \
       COALESCE(SUM(output_tokens), 0) as output_tokens, \
       COALESCE(SUM(total_tokens), 0) as total_tokens, \
       COALESCE(SUM(requests_count), 0) as total_requests, \
       COALESCE(SUM(cost_cents), 0) as total_cost_cents \
       FROM token_usage WHERE token_id = $1"
    )
    .bind( token_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( AggregateUsage {
      input_tokens: row.get( "input_tokens" ),
      output_tokens: row.get( "output_tokens" ),
      total_tokens: row.get( "total_tokens" ),
      total_requests: row.get( "total_requests" ),
      total_cost_cents: row.get( "total_cost_cents" ),
    } )
  }

  /// Get usage records within time range
  ///
  /// # Arguments
  ///
  /// * `token_id` - Database ID of token
  /// * `start_time` - Start timestamp (milliseconds since epoch)
  /// * `end_time` - End timestamp (milliseconds since epoch)
  ///
  /// # Returns
  ///
  /// Vector of usage records within time range
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_in_range(
    &self,
    token_id: i64,
    start_time: i64,
    end_time: i64,
  ) -> Result< Vec< UsageRecord > >
  {
    let rows = sqlx::query(
      "SELECT id, token_id, provider, model, input_tokens, output_tokens, total_tokens, requests_count, cost_cents, recorded_at \
       FROM token_usage WHERE token_id = $1 AND recorded_at >= $2 AND recorded_at <= $3 ORDER BY recorded_at DESC"
    )
    .bind( token_id )
    .bind( start_time )
    .bind( end_time )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| UsageRecord {
        id: row.get( "id" ),
        token_id: row.get( "token_id" ),
        provider: row.get( "provider" ),
        model: row.get( "model" ),
        input_tokens: row.get( "input_tokens" ),
        output_tokens: row.get( "output_tokens" ),
        total_tokens: row.get( "total_tokens" ),
        requests_count: row.get( "requests_count" ),
        cost_cents: row.get( "cost_cents" ),
        recorded_at: row.get( "recorded_at" ),
      } ).collect()
    )
  }

  /// Get aggregated usage statistics across all tokens
  ///
  /// # Returns
  ///
  /// Aggregated usage statistics for all tokens
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_all_aggregate_usage( &self ) -> Result< AggregateUsage >
  {
    let row = sqlx::query(
      "SELECT \
       COALESCE(SUM(input_tokens), 0) as input_tokens, \
       COALESCE(SUM(output_tokens), 0) as output_tokens, \
       COALESCE(SUM(total_tokens), 0) as total_tokens, \
       COALESCE(SUM(requests_count), 0) as total_requests, \
       COALESCE(SUM(cost_cents), 0) as total_cost_cents \
       FROM token_usage"
    )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( AggregateUsage {
      input_tokens: row.get( "input_tokens" ),
      output_tokens: row.get( "output_tokens" ),
      total_tokens: row.get( "total_tokens" ),
      total_requests: row.get( "total_requests" ),
      total_cost_cents: row.get( "total_cost_cents" ),
    } )
  }

  /// Get usage statistics grouped by provider across all tokens
  ///
  /// # Returns
  ///
  /// Vector of (provider, `AggregateUsage`) tuples
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_by_provider_all( &self ) -> Result< Vec< ( String, AggregateUsage ) > >
  {
    let rows = sqlx::query(
      "SELECT provider, \
       COALESCE(SUM(input_tokens), 0) as input_tokens, \
       COALESCE(SUM(output_tokens), 0) as output_tokens, \
       COALESCE(SUM(total_tokens), 0) as total_tokens, \
       COALESCE(SUM(requests_count), 0) as total_requests, \
       COALESCE(SUM(cost_cents), 0) as total_cost_cents \
       FROM token_usage \
       GROUP BY provider \
       ORDER BY total_cost_cents DESC"
    )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| {
        let provider: String = row.get( "provider" );
        let usage = AggregateUsage {
          input_tokens: row.get( "input_tokens" ),
          output_tokens: row.get( "output_tokens" ),
          total_tokens: row.get( "total_tokens" ),
          total_requests: row.get( "total_requests" ),
          total_cost_cents: row.get( "total_cost_cents" ),
        };
        ( provider, usage )
      } ).collect()
    )
  }

  /// Get usage statistics for a specific provider across all tokens
  ///
  /// # Arguments
  ///
  /// * `provider` - Provider name to filter by
  ///
  /// # Returns
  ///
  /// Aggregated usage for the provider
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_all_usage_for_provider( &self, provider: &str ) -> Result< AggregateUsage >
  {
    let row = sqlx::query(
      "SELECT \
       COALESCE(SUM(input_tokens), 0) as input_tokens, \
       COALESCE(SUM(output_tokens), 0) as output_tokens, \
       COALESCE(SUM(total_tokens), 0) as total_tokens, \
       COALESCE(SUM(requests_count), 0) as total_requests, \
       COALESCE(SUM(cost_cents), 0) as total_cost_cents \
       FROM token_usage \
       WHERE provider = $1"
    )
    .bind( provider )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( AggregateUsage {
      input_tokens: row.get( "input_tokens" ),
      output_tokens: row.get( "output_tokens" ),
      total_tokens: row.get( "total_tokens" ),
      total_requests: row.get( "total_requests" ),
      total_cost_cents: row.get( "total_cost_cents" ),
    } )
  }

  /// Get usage statistics for a specific project
  ///
  /// # Arguments
  ///
  /// * `project_id` - Project identifier
  ///
  /// # Returns
  ///
  /// Aggregated usage for the project
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_by_project( &self, project_id: &str ) -> Result< AggregateUsage >
  {
    let row = sqlx::query(
      "SELECT \
       COALESCE(SUM(tu.input_tokens), 0) as input_tokens, \
       COALESCE(SUM(tu.output_tokens), 0) as output_tokens, \
       COALESCE(SUM(tu.total_tokens), 0) as total_tokens, \
       COALESCE(SUM(tu.requests_count), 0) as total_requests, \
       COALESCE(SUM(tu.cost_cents), 0) as total_cost_cents \
       FROM token_usage tu \
       INNER JOIN api_tokens t ON tu.token_id = t.id \
       WHERE t.project_id = $1"
    )
    .bind( project_id )
    .fetch_one( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok( AggregateUsage {
      input_tokens: row.get( "input_tokens" ),
      output_tokens: row.get( "output_tokens" ),
      total_tokens: row.get( "total_tokens" ),
      total_requests: row.get( "total_requests" ),
      total_cost_cents: row.get( "total_cost_cents" ),
    } )
  }

  /// Get usage by provider for specific project
  ///
  /// # Arguments
  ///
  /// * `project_id` - Project identifier
  ///
  /// # Returns
  ///
  /// Vector of (provider, `AggregateUsage`) tuples for the project
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_by_provider_for_project( &self, project_id: &str ) -> Result< Vec< ( String, AggregateUsage ) > >
  {
    let rows = sqlx::query(
      "SELECT tu.provider, \
       COALESCE(SUM(tu.input_tokens), 0) as input_tokens, \
       COALESCE(SUM(tu.output_tokens), 0) as output_tokens, \
       COALESCE(SUM(tu.total_tokens), 0) as total_tokens, \
       COALESCE(SUM(tu.requests_count), 0) as total_requests, \
       COALESCE(SUM(tu.cost_cents), 0) as total_cost_cents \
       FROM token_usage tu \
       INNER JOIN api_tokens t ON tu.token_id = t.id \
       WHERE t.project_id = $1 \
       GROUP BY tu.provider"
    )
    .bind( project_id )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    Ok(
      rows.iter().map( |row| {
        let provider: String = row.get( "provider" );
        let usage = AggregateUsage {
          input_tokens: row.get( "input_tokens" ),
          output_tokens: row.get( "output_tokens" ),
          total_tokens: row.get( "total_tokens" ),
          total_requests: row.get( "total_requests" ),
          total_cost_cents: row.get( "total_cost_cents" ),
        };
        ( provider, usage )
      } ).collect()
    )
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
