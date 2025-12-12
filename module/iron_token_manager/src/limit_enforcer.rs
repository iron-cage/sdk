// qqq : implement limit enforcement
//! Limit enforcement service
//!
//! Enforces hard limits (token quotas, request rates, cost caps) per user/project.

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions, Row };
use crate::error::Result;

/// Usage limit configuration
#[ derive( Debug, Clone ) ]
pub struct UsageLimit
{
  /// Database ID
  pub id: i64,
  /// User ID
  pub user_id: String,
  /// Project ID (nullable for user-level limits)
  pub project_id: Option< String >,
  /// Max tokens per day (NULL = unlimited)
  pub max_tokens_per_day: Option< i64 >,
  /// Max requests per minute (NULL = unlimited)
  pub max_requests_per_minute: Option< i64 >,
  /// Max cost in cents per month (NULL = unlimited)
  pub max_cost_cents_per_month: Option< i64 >,
  /// Current tokens used today
  pub current_tokens_today: i64,
  /// Current requests this minute
  pub current_requests_this_minute: i64,
  /// Current cost in cents this month
  pub current_cost_cents_this_month: i64,
  /// Last daily reset timestamp
  pub tokens_reset_at: Option< i64 >,
  /// Last minute reset timestamp
  pub requests_reset_at: Option< i64 >,
  /// Last monthly reset timestamp
  pub cost_reset_at: Option< i64 >,
  /// Created timestamp
  pub created_at: i64,
  /// Updated timestamp
  pub updated_at: i64,
}

/// Limit enforcer
///
/// Enforces usage limits with real database persistence.
#[ derive( Debug, Clone ) ]
pub struct LimitEnforcer
{
  pool: SqlitePool,
}

impl LimitEnforcer
{
  /// Create new limit enforcer from existing pool
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
  /// Initialized enforcer using provided pool
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// use iron_test_db::TestDatabaseBuilder;
  /// use iron_token_manager::limit_enforcer::LimitEnforcer;
  ///
  /// let db = TestDatabaseBuilder::new().in_memory().build().await?;
  /// let enforcer = LimitEnforcer::from_pool( db.pool().clone() );
  /// ```
  #[ must_use ]
  pub fn from_pool( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new limit enforcer
  ///
  /// # Arguments
  ///
  /// * `database_url` - Database connection string
  ///
  /// # Returns
  ///
  /// Initialized enforcer with database schema applied
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
      .map_err( |_| crate::error::TokenError::Generic )?;

    // Run migrations
    let migration_sql = include_str!( "../migrations/001_initial_schema.sql" );
    sqlx::raw_sql( migration_sql )
      .execute( &pool )
      .await
      .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( Self { pool } )
  }

  /// Create new usage limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `max_tokens_per_day` - Max tokens per day (None = unlimited)
  /// * `max_requests_per_minute` - Max requests per minute (None = unlimited)
  /// * `max_cost_cents_per_month` - Max cost in cents per month (None = unlimited)
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn create_limit(
    &self,
    user_id: &str,
    project_id: Option< &str >,
    max_tokens_per_day: Option< i64 >,
    max_requests_per_minute: Option< i64 >,
    max_cost_cents_per_month: Option< i64 >,
  ) -> Result< i64 >
  {
    let now_ms = current_time_ms();

    let result = sqlx::query(
      "INSERT INTO usage_limits \
       (user_id, project_id, max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month, created_at, updated_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind( user_id )
    .bind( project_id )
    .bind( max_tokens_per_day )
    .bind( max_requests_per_minute )
    .bind( max_cost_cents_per_month )
    .bind( now_ms )
    .bind( now_ms )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( result.last_insert_rowid() )
  }

  /// Get usage limit for user/project
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Returns
  ///
  /// Usage limit configuration
  ///
  /// # Errors
  ///
  /// Returns error if limit not found or database query fails
  pub async fn get_limit( &self, user_id: &str, project_id: Option< &str > ) -> Result< UsageLimit >
  {
    let row = sqlx::query(
      "SELECT id, user_id, project_id, max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month, \
       current_tokens_today, current_requests_this_minute, current_cost_cents_this_month, \
       tokens_reset_at, requests_reset_at, cost_reset_at, created_at, updated_at \
       FROM usage_limits WHERE user_id = $1 AND (project_id = $2 OR (project_id IS NULL AND $2 IS NULL))"
    )
    .bind( user_id )
    .bind( project_id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?
    .ok_or( crate::error::TokenError::Generic )?;

    Ok( UsageLimit {
      id: row.get( "id" ),
      user_id: row.get( "user_id" ),
      project_id: row.get( "project_id" ),
      max_tokens_per_day: row.get( "max_tokens_per_day" ),
      max_requests_per_minute: row.get( "max_requests_per_minute" ),
      max_cost_cents_per_month: row.get( "max_cost_cents_per_month" ),
      current_tokens_today: row.get( "current_tokens_today" ),
      current_requests_this_minute: row.get( "current_requests_this_minute" ),
      current_cost_cents_this_month: row.get( "current_cost_cents_this_month" ),
      tokens_reset_at: row.get( "tokens_reset_at" ),
      requests_reset_at: row.get( "requests_reset_at" ),
      cost_reset_at: row.get( "cost_reset_at" ),
      created_at: row.get( "created_at" ),
      updated_at: row.get( "updated_at" ),
    } )
  }

  /// Check if tokens are allowed without exceeding limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `tokens` - Number of tokens to check
  ///
  /// # Returns
  ///
  /// True if allowed, false if would exceed limit
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn check_tokens_allowed( &self, user_id: &str, project_id: Option< &str >, tokens: i64 ) -> Result< bool >
  {
    let limit = self.get_limit( user_id, project_id ).await?;

    // If no limit set, allow unlimited
    let Some( max_tokens ) = limit.max_tokens_per_day else { return Ok( true ) };

    Ok( limit.current_tokens_today + tokens <= max_tokens )
  }

  /// Check if request is allowed without exceeding rate limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Returns
  ///
  /// True if allowed, false if would exceed limit
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn check_request_allowed( &self, user_id: &str, project_id: Option< &str > ) -> Result< bool >
  {
    let limit = self.get_limit( user_id, project_id ).await?;

    // If no limit set, allow unlimited
    let Some( max_requests ) = limit.max_requests_per_minute else { return Ok( true ) };

    Ok( limit.current_requests_this_minute < max_requests )
  }

  /// Check if cost is allowed without exceeding limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `cost_cents` - Cost in cents to check
  ///
  /// # Returns
  ///
  /// True if allowed, false if would exceed limit
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn check_cost_allowed( &self, user_id: &str, project_id: Option< &str >, cost_cents: i64 ) -> Result< bool >
  {
    let limit = self.get_limit( user_id, project_id ).await?;

    // If no limit set, allow unlimited
    let Some( max_cost ) = limit.max_cost_cents_per_month else { return Ok( true ) };

    Ok( limit.current_cost_cents_this_month + cost_cents <= max_cost )
  }

  /// Increment token usage counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `tokens` - Number of tokens to add
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn increment_tokens( &self, user_id: &str, project_id: Option< &str >, tokens: i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_tokens_today = current_tokens_today + $1, updated_at = $2 \
       WHERE user_id = $3 AND (project_id = $4 OR (project_id IS NULL AND $4 IS NULL))"
    )
    .bind( tokens )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Increment request counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn increment_requests( &self, user_id: &str, project_id: Option< &str > ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_requests_this_minute = current_requests_this_minute + 1, updated_at = $1 \
       WHERE user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))"
    )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Increment cost counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `cost_cents` - Cost in cents to add
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn increment_cost( &self, user_id: &str, project_id: Option< &str >, cost_cents: i64 ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_cost_cents_this_month = current_cost_cents_this_month + $1, updated_at = $2 \
       WHERE user_id = $3 AND (project_id = $4 OR (project_id IS NULL AND $4 IS NULL))"
    )
    .bind( cost_cents )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Reset daily token counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn reset_daily_tokens( &self, user_id: &str, project_id: Option< &str > ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_tokens_today = 0, tokens_reset_at = $1, updated_at = $1 \
       WHERE user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))"
    )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Reset per-minute request counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn reset_minute_requests( &self, user_id: &str, project_id: Option< &str > ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_requests_this_minute = 0, requests_reset_at = $1, updated_at = $1 \
       WHERE user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))"
    )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Reset monthly cost counter
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn reset_monthly_cost( &self, user_id: &str, project_id: Option< &str > ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET current_cost_cents_this_month = 0, cost_reset_at = $1, updated_at = $1 \
       WHERE user_id = $2 AND (project_id = $3 OR (project_id IS NULL AND $3 IS NULL))"
    )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Update existing limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  /// * `max_tokens_per_day` - Max tokens per day (None = unlimited)
  /// * `max_requests_per_minute` - Max requests per minute (None = unlimited)
  /// * `max_cost_cents_per_month` - Max cost in cents per month (None = unlimited)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  #[ allow( clippy::too_many_arguments ) ]
  pub async fn update_limit(
    &self,
    user_id: &str,
    project_id: Option< &str >,
    max_tokens_per_day: Option< i64 >,
    max_requests_per_minute: Option< i64 >,
    max_cost_cents_per_month: Option< i64 >,
  ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET max_tokens_per_day = $1, max_requests_per_minute = $2, max_cost_cents_per_month = $3, updated_at = $4 \
       WHERE user_id = $5 AND (project_id = $6 OR (project_id IS NULL AND $6 IS NULL))"
    )
    .bind( max_tokens_per_day )
    .bind( max_requests_per_minute )
    .bind( max_cost_cents_per_month )
    .bind( now_ms )
    .bind( user_id )
    .bind( project_id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Get limit by ID
  ///
  /// # Arguments
  ///
  /// * `id` - Limit ID
  ///
  /// # Returns
  ///
  /// Usage limit configuration
  ///
  /// # Errors
  ///
  /// Returns error if limit not found or database query fails
  pub async fn get_limit_by_id( &self, id: i64 ) -> Result< UsageLimit >
  {
    let row = sqlx::query(
      "SELECT id, user_id, project_id, max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month, \
       current_tokens_today, current_requests_this_minute, current_cost_cents_this_month, \
       tokens_reset_at, requests_reset_at, cost_reset_at, created_at, updated_at \
       FROM usage_limits WHERE id = $1"
    )
    .bind( id )
    .fetch_optional( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?
    .ok_or( crate::error::TokenError::Generic )?;

    Ok( UsageLimit {
      id: row.get( "id" ),
      user_id: row.get( "user_id" ),
      project_id: row.get( "project_id" ),
      max_tokens_per_day: row.get( "max_tokens_per_day" ),
      max_requests_per_minute: row.get( "max_requests_per_minute" ),
      max_cost_cents_per_month: row.get( "max_cost_cents_per_month" ),
      current_tokens_today: row.get( "current_tokens_today" ),
      current_requests_this_minute: row.get( "current_requests_this_minute" ),
      current_cost_cents_this_month: row.get( "current_cost_cents_this_month" ),
      tokens_reset_at: row.get( "tokens_reset_at" ),
      requests_reset_at: row.get( "requests_reset_at" ),
      cost_reset_at: row.get( "cost_reset_at" ),
      created_at: row.get( "created_at" ),
      updated_at: row.get( "updated_at" ),
    } )
  }

  /// List all usage limits
  ///
  /// # Returns
  ///
  /// Vector of all usage limits
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn list_all_limits( &self ) -> Result< Vec< UsageLimit > >
  {
    let rows = sqlx::query(
      "SELECT id, user_id, project_id, max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month, \
       current_tokens_today, current_requests_this_minute, current_cost_cents_this_month, \
       tokens_reset_at, requests_reset_at, cost_reset_at, created_at, updated_at \
       FROM usage_limits ORDER BY created_at DESC"
    )
    .fetch_all( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok(
      rows.iter().map( |row| UsageLimit {
        id: row.get( "id" ),
        user_id: row.get( "user_id" ),
        project_id: row.get( "project_id" ),
        max_tokens_per_day: row.get( "max_tokens_per_day" ),
        max_requests_per_minute: row.get( "max_requests_per_minute" ),
        max_cost_cents_per_month: row.get( "max_cost_cents_per_month" ),
        current_tokens_today: row.get( "current_tokens_today" ),
        current_requests_this_minute: row.get( "current_requests_this_minute" ),
        current_cost_cents_this_month: row.get( "current_cost_cents_this_month" ),
        tokens_reset_at: row.get( "tokens_reset_at" ),
        requests_reset_at: row.get( "requests_reset_at" ),
        cost_reset_at: row.get( "cost_reset_at" ),
        created_at: row.get( "created_at" ),
        updated_at: row.get( "updated_at" ),
      } ).collect()
    )
  }

  /// Update limit by ID
  ///
  /// # Arguments
  ///
  /// * `id` - Limit ID
  /// * `max_tokens_per_day` - Max tokens per day (None = unlimited)
  /// * `max_requests_per_minute` - Max requests per minute (None = unlimited)
  /// * `max_cost_cents_per_month` - Max cost in cents per month (None = unlimited)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_limit_by_id(
    &self,
    id: i64,
    max_tokens_per_day: Option< i64 >,
    max_requests_per_minute: Option< i64 >,
    max_cost_cents_per_month: Option< i64 >,
  ) -> Result< () >
  {
    let now_ms = current_time_ms();

    sqlx::query(
      "UPDATE usage_limits SET max_tokens_per_day = $1, max_requests_per_minute = $2, max_cost_cents_per_month = $3, updated_at = $4 \
       WHERE id = $5"
    )
    .bind( max_tokens_per_day )
    .bind( max_requests_per_minute )
    .bind( max_cost_cents_per_month )
    .bind( now_ms )
    .bind( id )
    .execute( &self.pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
  }

  /// Delete limit by ID
  ///
  /// # Arguments
  ///
  /// * `id` - Limit ID
  ///
  /// # Errors
  ///
  /// Returns error if database delete fails
  pub async fn delete_limit( &self, id: i64 ) -> Result< () >
  {
    sqlx::query( "DELETE FROM usage_limits WHERE id = $1" )
      .bind( id )
      .execute( &self.pool )
      .await
      .map_err( |_| crate::error::TokenError::Generic )?;

    Ok( () )
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
