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
  /// Total USD budget allocated to this agent
  pub total_allocated: f64,
  /// Total USD spent across all leases
  pub total_spent: f64,
  /// USD remaining (`total_allocated` - `total_spent`)
  pub budget_remaining: f64,
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
  /// * `total_allocated` - Total USD budget allocated to this agent
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn create_budget( &self, agent_id: i64, total_allocated: f64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "Time went backwards" )
      .as_millis() as i64;

    sqlx::query(
      "INSERT INTO agent_budgets
      (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
      VALUES (?, ?, 0.0, ?, ?, ?)"
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
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `cost_usd` - Cost to add to `total_spent`
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn record_spending( &self, agent_id: i64, cost_usd: f64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "Time went backwards" )
      .as_millis() as i64;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent + ?,
          budget_remaining = budget_remaining - ?,
          updated_at = ?
      WHERE agent_id = ?"
    )
    .bind( cost_usd )
    .bind( cost_usd )
    .bind( now )
    .bind( agent_id )
    .execute( &self.pool )
    .await?;

    Ok( () )
  }

  /// Add budget to agent allocation
  ///
  /// Increases `total_allocated` and `budget_remaining`.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `additional_budget` - USD to add to allocation
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn add_budget( &self, agent_id: i64, additional_budget: f64 ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "Time went backwards" )
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
  /// * `required_amount` - USD amount needed
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn has_sufficient_budget( &self, agent_id: i64, required_amount: f64 ) -> Result< bool, sqlx::Error >
  {
    let budget = self.get_budget_status( agent_id ).await?;

    match budget
    {
      Some( b ) => Ok( b.budget_remaining >= required_amount ),
      None => Ok( false ),
    }
  }
}
