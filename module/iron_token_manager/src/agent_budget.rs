//! Agent Budget Manager
//!
//! Protocol 005: Budget Control Protocol - Agent Budget Management
//!
//! Manages per-agent budget allocations. Each agent has exactly one budget (1:1 relationship)
//! that tracks total allocated, total spent, and remaining budget across all leases.
//!
//! Budget Invariant: `total_allocated` = `total_spent` + `budget_remaining`

use sqlx::{ SqlitePool, Row };
use std::time::{ SystemTime, UNIX_EPOCH };

/// Agent budget record
#[ derive( Debug, Clone ) ]
pub struct AgentBudget
{
  /// Agent database ID (1:1 relationship with agents table)
  pub agent_id: i64,
  /// Total microdollars budget allocated to this agent
  pub total_allocated: i64,
  /// Total microdollars spent across all leases
  pub total_spent: i64,
  /// Microdollars remaining (`total_allocated` - `total_spent`)
  pub budget_remaining: i64,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last update timestamp (milliseconds since epoch)
  pub updated_at: i64,
}

/// Agent budget manager for budget CRUD operations
#[ derive( Debug, Clone ) ]
pub struct AgentBudgetManager
{
  pool: SqlitePool,
}

impl AgentBudgetManager
{
  /// Create new agent budget manager from existing pool
  ///
  /// # Arguments
  ///
  /// * `pool` - Existing database connection pool
  #[ must_use ]
  pub fn from_pool( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new agent budget
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `total_allocated` - Total microdollars budget allocated to this agent
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn create_budget( &self, agent_id: i64, total_allocated: i64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "LOUD FAILURE: Time went backwards" )
      .as_millis() as i64;

    sqlx::query(
      "INSERT INTO agent_budgets
      (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
      VALUES (?, ?, 0, ?, ?, ?)"
    )
    .bind( agent_id )
    .bind( total_allocated )
    .bind( total_allocated )  // budget_remaining = total_allocated initially
    .bind( now )
    .bind( now )
    .execute( &self.pool )
    .await?;

    Ok( () )
  }

  /// Get agent budget status
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_budget_status( &self, agent_id: i64 ) -> Result< Option< AgentBudget >, sqlx::Error >
  {
    let row = sqlx::query(
      "SELECT agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at
      FROM agent_budgets WHERE agent_id = ?"
    )
    .bind( agent_id )
    .fetch_optional( &self.pool )
    .await?;

    Ok( row.map( | r | AgentBudget {
      agent_id: r.get( "agent_id" ),
      total_allocated: r.get( "total_allocated" ),
      total_spent: r.get( "total_spent" ),
      budget_remaining: r.get( "budget_remaining" ),
      created_at: r.get( "created_at" ),
      updated_at: r.get( "updated_at" ),
    } ) )
  }

  /// Record spending against agent budget
  ///
  /// Updates `total_spent` and `budget_remaining`.
  /// Maintains invariant: `total_allocated` = `total_spent` + `budget_remaining`
  ///
  /// Fix(issue-budget-003): Use explicit transaction for atomic concurrent updates
  ///
  /// Root cause: Direct UPDATE statements from concurrent requests can cause lost updates
  /// in `SQLite` when using connection pooling. Same issue as `lease_manager::record_usage()`.
  /// Without explicit transaction control, concurrent updates to `total_spent` and
  /// `budget_remaining` can cause inconsistent state or lost spending records.
  ///
  /// Pitfall: Same as `lease_manager::record_usage()` - never rely on implicit atomicity
  /// for read-modify-write SQL operations. Always wrap in explicit transactions.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `cost_microdollars` - Cost to add to `total_spent` (in microdollars)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn record_spending( &self, agent_id: i64, cost_microdollars: i64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "LOUD FAILURE: Time went backwards" )
      .as_millis() as i64;

    // Use explicit transaction with IMMEDIATE locking for atomic updates
    let mut tx = self.pool.begin().await?;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent + ?,
          budget_remaining = budget_remaining - ?,
          updated_at = ?
      WHERE agent_id = ?"
    )
    .bind( cost_microdollars )
    .bind( cost_microdollars )
    .bind( now )
    .bind( agent_id )
    .execute( &mut *tx )
    .await?;

    tx.commit().await?;

    Ok( () )
  }

  /// Atomically check and reserve budget for a request
  ///
  /// Fix(issue-budget-006): Prevent TOCTOU race in budget handshake
  ///
  /// Root cause: Handshake function checked `budget_remaining` with `get_budget_status()`,
  /// then separately called `record_spending()` in non-atomic operations. This created
  /// a race window where 2 concurrent requests could both pass the check before either
  /// recorded spending, allowing budget to go negative (violating budget invariant).
  ///
  /// Pitfall: Never split check-and-use into separate database operations for
  /// concurrent resource allocation. Use conditional UPDATE with WHERE clause that
  /// prevents negative budget, then verify `rows_affected` to detect race conditions.
  /// `SQLite`'s row-level write lock ensures the UPDATE is atomic even across concurrent
  /// requests - only one UPDATE can succeed when budget is insufficient for both.
  /// Under high concurrency (10+ simultaneous requests), `SQLite` may return deadlock
  /// errors - always implement retry logic with exponential backoff for database
  /// busy/locked/deadlocked errors.
  ///
  /// This method atomically:
  /// 1. Reads current `budget_remaining` within transaction
  /// 2. Calculates granted = min(requested, `budget_remaining`)
  /// 3. Updates budget only if granted > 0 AND wont go negative
  /// 4. Returns granted amount or 0
  ///
  /// Supports **partial grants**: If agent has $5 (`5_000_000` microdollars) and requests $10 (`10_000_000`), grants $5.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `requested_amount` - Microdollars amount requested
  ///
  /// # Returns
  ///
  /// * `Ok(granted_amount)` - Amount granted in microdollars (0 if no budget available)
  ///
  /// # Errors
  ///
  /// Returns error if database operation fails (not for insufficient budget)
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn check_and_reserve_budget( &self, agent_id: i64, requested_amount: i64 ) -> Result< i64, sqlx::Error >
  {
    // Retry logic for SQLite database busy/locked/deadlocked errors under high concurrency
    const MAX_RETRIES: u32 = 50;

    for attempt in 0..MAX_RETRIES
    {
      // Exponential backoff on retries
      if attempt > 0
      {
        let backoff_ms = 2_u64.pow( attempt.min( 8 ) ); // Cap at 256ms
        tokio::time::sleep( tokio::time::Duration::from_millis( backoff_ms ) ).await;
      }

      match self.try_reserve_budget_once( agent_id, requested_amount ).await
      {
        Ok( granted ) => return Ok( granted ),
        Err( e ) =>
        {
          // Check if error is database busy/locked/deadlocked - retry if so
          let err_msg = e.to_string().to_lowercase();
          let is_retryable = err_msg.contains( "database is locked" )
            || err_msg.contains( "database is busy" )
            || err_msg.contains( "deadlock" );

          if is_retryable && attempt < MAX_RETRIES - 1
          {
            // Retry on busy/deadlock error
          }
          else
          {
            // Not a retryable error, or max retries reached
            return Err( e );
          }
        }
      }
    }

    // Should never reach here
    Ok( 0 )
  }

  /// Single attempt to reserve budget (internal helper)
  async fn try_reserve_budget_once( &self, agent_id: i64, requested_amount: i64 ) -> Result< i64, sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "LOUD FAILURE: Time went backwards" )
      .as_millis() as i64;

    // Use explicit transaction for atomic check-and-reserve
    let mut tx = self.pool.begin().await?;

    // Read total_spent BEFORE update to calculate granted amount later
    let row = sqlx::query(
      "SELECT total_spent FROM agent_budgets WHERE agent_id = ?"
    )
    .bind( agent_id )
    .fetch_optional( &mut *tx )
    .await?;

    let spent_before = if let Some( r ) = row
    {
      r.get::< i64, _ >( "total_spent" )
    }
    else
    {
      // Agent doesnt exist
      tx.rollback().await?;
      return Ok( 0 );
    };

    // Single atomic UPDATE that calculates partial grant inline using CASE expression
    // This eliminates TOCTOU race by doing check+grant in one SQL statement
    //
    // CASE WHEN budget_remaining < requested THEN budget_remaining ELSE requested END
    // = min(budget_remaining, requested)
    //
    // WHERE budget_remaining > 0 ensures we only update if budget available
    let result = sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent +
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          budget_remaining = budget_remaining -
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          updated_at = ?
      WHERE agent_id = ? AND budget_remaining > 0"
    )
    .bind( requested_amount )
    .bind( requested_amount )
    .bind( requested_amount )
    .bind( requested_amount )
    .bind( now )
    .bind( agent_id )
    .execute( &mut *tx )
    .await?;

    // Calculate granted amount from change in total_spent
    let granted_amount = if result.rows_affected() == 1
    {
      // Read total_spent AFTER update
      let row = sqlx::query(
        "SELECT total_spent FROM agent_budgets WHERE agent_id = ?"
      )
      .bind( agent_id )
      .fetch_one( &mut *tx )
      .await?;

      let spent_after: i64 = row.get( "total_spent" );

      // Granted = difference in total_spent
      spent_after - spent_before
    }
    else
    {
      // UPDATE failed - no budget available
      0
    };

    tx.commit().await?;
    Ok( granted_amount )
  }

  /// Add budget to agent allocation
  ///
  /// Increases `total_allocated` and `budget_remaining`.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `additional_budget` - Microdollars to add to allocation
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn add_budget( &self, agent_id: i64, additional_budget: i64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "LOUD FAILURE: Time went backwards" )
      .as_millis() as i64;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_allocated = total_allocated + ?,
          budget_remaining = budget_remaining + ?,
          updated_at = ?
      WHERE agent_id = ?"
    )
    .bind( additional_budget )
    .bind( additional_budget )
    .bind( now )
    .bind( agent_id )
    .execute( &self.pool )
    .await?;

    Ok( () )
  }

  /// Check if agent has sufficient budget
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `required_amount` - Microdollars amount needed
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn has_sufficient_budget( &self, agent_id: i64, required_amount: i64 ) -> Result< bool, sqlx::Error >
  {
    let budget = self.get_budget_status( agent_id ).await?;

    match budget
    {
      Some( b ) => Ok( b.budget_remaining >= required_amount ),
      None => Ok( false ),
    }
  }
}
