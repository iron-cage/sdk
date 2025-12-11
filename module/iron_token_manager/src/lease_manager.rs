//! Budget Lease Manager
//!
//! Protocol 005: Budget Control Protocol - Lease Management
//!
//! Manages budget leases for agent sessions. Each lease represents a temporary
//! budget allocation that an agent can spend during a session.
//!
//! Leases have:
//! - Unique ID (format: lease_<uuid>)
//! - Budget granted (USD allocated for this session)
//! - Budget spent (USD consumed so far)
//! - Status (active, expired, revoked)
//! - Optional expiration time

use sqlx::{ SqlitePool, Row };
use std::time::{ SystemTime, UNIX_EPOCH };

/// Budget lease record
#[ derive( Debug, Clone ) ]
pub struct BudgetLease
{
  /// Lease ID (format: lease_<uuid>)
  pub id: String,
  /// Agent database ID
  pub agent_id: i64,
  /// Budget database ID
  pub budget_id: i64,
  /// USD allocated for this lease
  pub budget_granted: f64,
  /// USD spent in this lease
  pub budget_spent: f64,
  /// Lease status (active, expired, revoked)
  pub lease_status: String,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Expiration timestamp (milliseconds since epoch, None for no expiration)
  pub expires_at: Option< i64 >,
}

/// Lease manager for budget lease CRUD operations
#[ derive( Debug, Clone ) ]
pub struct LeaseManager
{
  pool: SqlitePool,
}

impl LeaseManager
{
  /// Create new lease manager from existing pool
  ///
  /// # Arguments
  ///
  /// * `pool` - Existing database connection pool
  #[ must_use ]
  pub fn from_pool( pool: SqlitePool ) -> Self
  {
    Self { pool }
  }

  /// Create new budget lease
  ///
  /// # Arguments
  ///
  /// * `lease_id` - Unique lease ID (format: lease_<uuid>)
  /// * `agent_id` - Agent database ID
  /// * `budget_id` - Budget database ID (same as `agent_id` for 1:1 relationship)
  /// * `budget_granted` - USD allocated for this lease
  /// * `expires_at` - Optional expiration timestamp (milliseconds)
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn create_lease(
    &self,
    lease_id: &str,
    agent_id: i64,
    budget_id: i64,
    budget_granted: f64,
    expires_at: Option< i64 >,
  ) -> Result< (), sqlx::Error >
  {
    #[ allow( clippy::cast_possible_truncation ) ]
    let now = SystemTime::now()
      .duration_since( UNIX_EPOCH )
      .expect( "Time went backwards" )
      .as_millis() as i64;

    sqlx::query(
      "INSERT INTO budget_leases
      (id, agent_id, budget_id, budget_granted, budget_spent, lease_status, created_at, expires_at)
      VALUES (?, ?, ?, ?, 0.0, 'active', ?, ?)"
    )
    .bind( lease_id )
    .bind( agent_id )
    .bind( budget_id )
    .bind( budget_granted )
    .bind( now )
    .bind( expires_at )
    .execute( &self.pool )
    .await?;

    Ok( () )
  }

  /// Get lease by ID
  ///
  /// # Arguments
  ///
  /// * `lease_id` - Lease ID to fetch
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_lease( &self, lease_id: &str ) -> Result< Option< BudgetLease >, sqlx::Error >
  {
    let row = sqlx::query(
      "SELECT id, agent_id, budget_id, budget_granted, budget_spent, lease_status, created_at, expires_at
      FROM budget_leases WHERE id = ?"
    )
    .bind( lease_id )
    .fetch_optional( &self.pool )
    .await?;

    Ok( row.map( | r | BudgetLease {
      id: r.get( "id" ),
      agent_id: r.get( "agent_id" ),
      budget_id: r.get( "budget_id" ),
      budget_granted: r.get( "budget_granted" ),
      budget_spent: r.get( "budget_spent" ),
      lease_status: r.get( "lease_status" ),
      created_at: r.get( "created_at" ),
      expires_at: r.get( "expires_at" ),
    } ) )
  }

  /// Record usage for a lease
  ///
  /// Updates `budget_spent` for the lease.
  ///
  /// # Arguments
  ///
  /// * `lease_id` - Lease ID
  /// * `cost_usd` - Cost to add to `budget_spent`
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn record_usage( &self, lease_id: &str, cost_usd: f64 ) -> Result< (), sqlx::Error >
  {
    sqlx::query( "UPDATE budget_leases SET budget_spent = budget_spent + ? WHERE id = ?" )
      .bind( cost_usd )
      .bind( lease_id )
      .execute( &self.pool )
      .await?;

    Ok( () )
  }

  /// Update lease budget (for budget refresh)
  ///
  /// Increases `budget_granted` by the specified amount.
  ///
  /// # Arguments
  ///
  /// * `lease_id` - Lease ID
  /// * `additional_budget` - USD to add to `budget_granted`
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn add_budget( &self, lease_id: &str, additional_budget: f64 ) -> Result< (), sqlx::Error >
  {
    sqlx::query( "UPDATE budget_leases SET budget_granted = budget_granted + ? WHERE id = ?" )
      .bind( additional_budget )
      .bind( lease_id )
      .execute( &self.pool )
      .await?;

    Ok( () )
  }

  /// Expire a lease (set status to 'expired')
  ///
  /// # Arguments
  ///
  /// * `lease_id` - Lease ID to expire
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn expire_lease( &self, lease_id: &str ) -> Result< (), sqlx::Error >
  {
    sqlx::query( "UPDATE budget_leases SET lease_status = 'expired' WHERE id = ?" )
      .bind( lease_id )
      .execute( &self.pool )
      .await?;

    Ok( () )
  }

  /// Get all active leases for an agent
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_leases( &self, agent_id: i64 ) -> Result< Vec< BudgetLease >, sqlx::Error >
  {
    let rows = sqlx::query(
      "SELECT id, agent_id, budget_id, budget_granted, budget_spent, lease_status, created_at, expires_at
      FROM budget_leases WHERE agent_id = ? AND lease_status = 'active'"
    )
    .bind( agent_id )
    .fetch_all( &self.pool )
    .await?;

    Ok( rows.into_iter().map( | r | BudgetLease {
      id: r.get( "id" ),
      agent_id: r.get( "agent_id" ),
      budget_id: r.get( "budget_id" ),
      budget_granted: r.get( "budget_granted" ),
      budget_spent: r.get( "budget_spent" ),
      lease_status: r.get( "lease_status" ),
      created_at: r.get( "created_at" ),
      expires_at: r.get( "expires_at" ),
    } ).collect() )
  }
}
