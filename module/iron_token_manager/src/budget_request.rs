//! Budget Request Storage Layer
//!
//! Protocol 012: Budget Request Workflow - Create → approve/reject budget change requests
//! Protocol 017: Budget History - Immutable audit trail of budget modifications
//!
//! Manages budget change requests and modification history with full CRUD operations.

use sqlx::{ SqlitePool, Row };

/// Budget change request status
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum RequestStatus
{
  /// Request created, awaiting approval
  Pending,
  /// Request approved by administrator
  Approved,
  /// Request rejected by administrator
  Rejected,
  /// Request cancelled by requester
  Cancelled,
}

impl RequestStatus
{
  /// Convert status to database string representation
  #[ must_use ]
  pub fn to_db_string( &self ) -> &'static str
  {
    match self
    {
      Self::Pending => "pending",
      Self::Approved => "approved",
      Self::Rejected => "rejected",
      Self::Cancelled => "cancelled",
    }
  }

  /// Parse status from database string
  ///
  /// # Errors
  ///
  /// Returns error if status string is not valid (pending/approved/rejected/cancelled)
  pub fn from_db_string( s: &str ) -> Result< Self, String >
  {
    match s
    {
      "pending" => Ok( Self::Pending ),
      "approved" => Ok( Self::Approved ),
      "rejected" => Ok( Self::Rejected ),
      "cancelled" => Ok( Self::Cancelled ),
      _ => Err( format!( "Invalid request status: {s}" ) ),
    }
  }
}

/// Budget modification type for history tracking
#[ derive( Debug, Clone, Copy, PartialEq, Eq ) ]
pub enum ModificationType
{
  /// Budget increased
  Increase,
  /// Budget decreased
  Decrease,
  /// Budget reset to specific value
  Reset,
}

impl ModificationType
{
  /// Convert modification type to database string representation
  #[ must_use ]
  pub fn to_db_string( &self ) -> &'static str
  {
    match self
    {
      Self::Increase => "increase",
      Self::Decrease => "decrease",
      Self::Reset => "reset",
    }
  }

  /// Parse modification type from database string
  ///
  /// # Errors
  ///
  /// Returns error if modification type string is not valid (increase/decrease/reset)
  pub fn from_db_string( s: &str ) -> Result< Self, String >
  {
    match s
    {
      "increase" => Ok( Self::Increase ),
      "decrease" => Ok( Self::Decrease ),
      "reset" => Ok( Self::Reset ),
      _ => Err( format!( "Invalid modification type: {s}" ) ),
    }
  }
}

/// Budget change request (Protocol 012)
#[ derive( Debug, Clone ) ]
pub struct BudgetChangeRequest
{
  /// Request ID (primary key)
  pub id: String,
  /// Agent ID (foreign key to agents table)
  pub agent_id: i64,
  /// User ID who created the request
  pub requester_id: String,
  /// Current budget in microdollars (integer, avoids floating point)
  pub current_budget_micros: i64,
  /// Requested budget in microdollars
  pub requested_budget_micros: i64,
  /// Justification text (20-500 characters, enforced by DB constraint)
  pub justification: String,
  /// Request status
  pub status: RequestStatus,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last update timestamp (milliseconds since epoch)
  pub updated_at: i64,
}

/// Budget modification history entry (Protocol 017)
#[ derive( Debug, Clone ) ]
pub struct BudgetModificationHistory
{
  /// History entry ID (primary key)
  pub id: String,
  /// Agent ID (foreign key to agents table)
  pub agent_id: i64,
  /// Type of modification
  pub modification_type: ModificationType,
  /// Old budget value in microdollars
  pub old_budget_micros: i64,
  /// New budget value in microdollars
  pub new_budget_micros: i64,
  /// Change amount in microdollars (new - old)
  pub change_amount_micros: i64,
  /// User ID who made the modification
  pub modifier_id: String,
  /// Reason for modification (10-500 characters, enforced by DB constraint)
  pub reason: String,
  /// Optional link to budget change request that triggered this
  pub related_request_id: Option< String >,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
}

// Storage functions

/// Create a new budget change request
///
/// # Errors
///
/// Returns error if database constraint violations occur (e.g., invalid `agent_id`)
pub async fn create_budget_request(
  pool: &SqlitePool,
  request: &BudgetChangeRequest,
) -> Result< (), sqlx::Error >
{
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &request.id )
  .bind( request.agent_id )
  .bind( &request.requester_id )
  .bind( request.current_budget_micros )
  .bind( request.requested_budget_micros )
  .bind( &request.justification )
  .bind( request.status.to_db_string() )
  .bind( request.created_at )
  .bind( request.updated_at )
  .execute( pool )
  .await?;

  Ok( () )
}

/// Get a budget change request by ID
///
/// # Errors
///
/// Returns error if database query fails
pub async fn get_budget_request(
  pool: &SqlitePool,
  id: &str,
) -> Result< Option< BudgetChangeRequest >, sqlx::Error >
{
  let row = sqlx::query(
    "SELECT id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
            justification, status, created_at, updated_at
     FROM budget_change_requests
     WHERE id = ?"
  )
  .bind( id )
  .fetch_optional( pool )
  .await?;

  match row
  {
    Some( r ) =>
    {
      let status_str: String = r.get( "status" );
      let status = RequestStatus::from_db_string( &status_str )
        .map_err( | e | sqlx::Error::Decode( Box::new( std::io::Error::new( std::io::ErrorKind::InvalidData, e ) ) ) )?;

      Ok( Some( BudgetChangeRequest
      {
        id: r.get( "id" ),
        agent_id: r.get( "agent_id" ),
        requester_id: r.get( "requester_id" ),
        current_budget_micros: r.get( "current_budget_micros" ),
        requested_budget_micros: r.get( "requested_budget_micros" ),
        justification: r.get( "justification" ),
        status,
        created_at: r.get( "created_at" ),
        updated_at: r.get( "updated_at" ),
      } ) )
    }
    None => Ok( None ),
  }
}

/// List all budget change requests with a specific status
///
/// # Errors
///
/// Returns error if database query fails
pub async fn list_budget_requests_by_status(
  pool: &SqlitePool,
  status: RequestStatus,
) -> Result< Vec< BudgetChangeRequest >, sqlx::Error >
{
  let rows = sqlx::query(
    "SELECT id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
            justification, status, created_at, updated_at
     FROM budget_change_requests
     WHERE status = ?
     ORDER BY created_at DESC"
  )
  .bind( status.to_db_string() )
  .fetch_all( pool )
  .await?;

  let mut requests = Vec::new();
  for row in rows
  {
    let status_str: String = row.get( "status" );
    let status = RequestStatus::from_db_string( &status_str )
      .map_err( | e | sqlx::Error::Decode( Box::new( std::io::Error::new( std::io::ErrorKind::InvalidData, e ) ) ) )?;

    requests.push( BudgetChangeRequest
    {
      id: row.get( "id" ),
      agent_id: row.get( "agent_id" ),
      requester_id: row.get( "requester_id" ),
      current_budget_micros: row.get( "current_budget_micros" ),
      requested_budget_micros: row.get( "requested_budget_micros" ),
      justification: row.get( "justification" ),
      status,
      created_at: row.get( "created_at" ),
      updated_at: row.get( "updated_at" ),
    } );
  }

  Ok( requests )
}

/// List all budget change requests for a specific agent
///
/// # Errors
///
/// Returns error if database query fails
pub async fn list_budget_requests_by_agent(
  pool: &SqlitePool,
  agent_id: i64,
) -> Result< Vec< BudgetChangeRequest >, sqlx::Error >
{
  let rows = sqlx::query(
    "SELECT id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
            justification, status, created_at, updated_at
     FROM budget_change_requests
     WHERE agent_id = ?
     ORDER BY created_at DESC"
  )
  .bind( agent_id )
  .fetch_all( pool )
  .await?;

  let mut requests = Vec::new();
  for row in rows
  {
    let status_str: String = row.get( "status" );
    let status = RequestStatus::from_db_string( &status_str )
      .map_err( | e | sqlx::Error::Decode( Box::new( std::io::Error::new( std::io::ErrorKind::InvalidData, e ) ) ) )?;

    requests.push( BudgetChangeRequest
    {
      id: row.get( "id" ),
      agent_id: row.get( "agent_id" ),
      requester_id: row.get( "requester_id" ),
      current_budget_micros: row.get( "current_budget_micros" ),
      requested_budget_micros: row.get( "requested_budget_micros" ),
      justification: row.get( "justification" ),
      status,
      created_at: row.get( "created_at" ),
      updated_at: row.get( "updated_at" ),
    } );
  }

  Ok( requests )
}

/// Update the status of a budget change request with optimistic locking
///
/// Fix(issue-002): Added optimistic locking to prevent race conditions in generic status updates.
///
/// Root cause: Function allowed unconditional status updates without checking current state,
/// enabling invalid transitions and concurrent modification issues.
///
/// Pitfall: Generic update functions are convenient but dangerous without state validation.
/// Prefer specific transition functions (approve, reject, cancel) with explicit preconditions.
///
/// # Errors
///
/// Returns error if database update fails or if request is not in pending state
/// (returns `RowNotFound` for optimistic lock failure)
pub async fn update_budget_request_status(
  pool: &SqlitePool,
  id: &str,
  status: RequestStatus,
  updated_at: i64,
) -> Result< u64, sqlx::Error >
{
  // Fetch current request to validate state
  let current_request = sqlx::query(
    "SELECT status FROM budget_change_requests WHERE id = ?"
  )
  .bind( id )
  .fetch_optional( pool )
  .await?;

  let current_status = match current_request
  {
    Some( row ) =>
    {
      row.try_get::< String, _ >( "status" )?
    }
    None =>
    {
      return Err( sqlx::Error::RowNotFound );
    }
  };

  // Only allow updates from pending status (optimistic locking precondition)
  if current_status != "pending"
  {
    return Err( sqlx::Error::RowNotFound ); // Simulate optimistic lock failure
  }

  // Update with optimistic locking WHERE clause
  let result = sqlx::query(
    "UPDATE budget_change_requests
     SET status = ?, updated_at = ?
     WHERE id = ? AND status = 'pending'"
  )
  .bind( status.to_db_string() )
  .bind( updated_at )
  .bind( id )
  .execute( pool )
  .await?;

  // If no rows affected, concurrent modification occurred
  if result.rows_affected() == 0
  {
    return Err( sqlx::Error::RowNotFound ); // Optimistic lock failed
  }

  Ok( result.rows_affected() )
}

/// Approve a budget change request and apply the budget change
///
/// This function atomically:
/// 1. Updates request status to 'approved' (with optimistic locking - only if status='pending')
/// 2. Updates agent budget to the requested amount
/// 3. Records the change in `budget_modification_history`
///
/// Uses a database transaction to ensure atomicity. If the request is not in pending status,
/// returns an error (preventing double approval).
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `id` - Budget request ID
/// * `approver_id` - ID of the user approving the request
/// * `updated_at` - Timestamp of approval (milliseconds since epoch)
///
/// # Errors
///
/// Returns error if:
/// - Database transaction fails
/// - Request not found
/// - Request not in pending status (optimistic lock failure)
/// - Budget update fails
/// - History recording fails
pub async fn approve_budget_request(
  pool: &SqlitePool,
  id: &str,
  approver_id: &str,
  updated_at: i64,
) -> Result< (), sqlx::Error >
{
  // Start transaction
  let mut tx = pool.begin().await?;

  // Fetch the request to get current and requested budget
  let request_result = sqlx::query(
    "SELECT agent_id, current_budget_micros, requested_budget_micros, status
     FROM budget_change_requests
     WHERE id = ?"
  )
  .bind( id )
  .fetch_optional( &mut *tx )
  .await?;

  let Some( request_row ) = request_result else
  {
    return Err( sqlx::Error::RowNotFound );
  };

  let agent_id: i64 = request_row.get( "agent_id" );
  let current_budget_micros: i64 = request_row.get( "current_budget_micros" );
  let requested_budget_micros: i64 = request_row.get( "requested_budget_micros" );
  let current_status: String = request_row.get( "status" );

  // Check if request is pending (optimistic locking)
  if current_status != "pending"
  {
    return Err( sqlx::Error::RowNotFound ); // Simulate optimistic lock failure
  }

  // Update request status to approved (with optimistic locking WHERE clause)
  let update_result = sqlx::query(
    "UPDATE budget_change_requests
     SET status = ?,
         updated_at = ?
     WHERE id = ? AND status = 'pending'"
  )
  .bind( "approved" )
  .bind( updated_at )
  .bind( id )
  .execute( &mut *tx )
  .await?;

  // If no rows affected, concurrent modification occurred
  if update_result.rows_affected() == 0
  {
    return Err( sqlx::Error::RowNotFound ); // Optimistic lock failed
  }

  // Calculate budget delta (requested - current)
  let delta_micros = requested_budget_micros - current_budget_micros;
  let delta_usd = delta_micros as f64 / 1_000_000.0;

  // Update agent budget
  sqlx::query(
    "UPDATE agent_budgets
     SET total_allocated = total_allocated + ?,
         budget_remaining = budget_remaining + ?,
         updated_at = ?
     WHERE agent_id = ?"
  )
  .bind( delta_usd )
  .bind( delta_usd )
  .bind( updated_at )
  .bind( agent_id )
  .execute( &mut *tx )
  .await?;

  // Record in budget_modification_history
  let history_id = format!( "bhist_{}", uuid::Uuid::new_v4() );
  let change_amount_micros = requested_budget_micros - current_budget_micros;

  // Determine modification type based on budget change
  use core::cmp::Ordering;
  let modification_type = match change_amount_micros.cmp( &0 )
  {
    Ordering::Greater => "increase",
    Ordering::Less => "decrease",
    Ordering::Equal => "reset",
  };

  sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, related_request_id, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &history_id )
  .bind( agent_id )
  .bind( modification_type )
  .bind( current_budget_micros )
  .bind( requested_budget_micros )
  .bind( change_amount_micros )
  .bind( approver_id )
  .bind( format!( "Budget request {id} approved" ) )
  .bind( id )
  .bind( updated_at )
  .execute( &mut *tx )
  .await?;

  // Commit transaction
  tx.commit().await?;

  Ok( () )
}

/// Reject a budget change request with optimistic locking
///
/// Fix(issue-001): Added optimistic locking to prevent concurrent modification race condition.
///
/// Root cause: Original implementation called `update_budget_request_status` without WHERE clause
/// on current status, allowing concurrent approve+reject operations to both succeed.
///
/// Pitfall: API-layer status validation alone is insufficient. Database-level optimistic locking
/// (WHERE status='pending' + `rows_affected` check) is required for atomicity in concurrent environments.
///
/// # Errors
///
/// Returns error if database update fails or if request is not in pending state
/// (returns `RowNotFound` for optimistic lock failure)
pub async fn reject_budget_request(
  pool: &SqlitePool,
  id: &str,
  updated_at: i64,
) -> Result< u64, sqlx::Error >
{
  // Fetch current request to validate state
  let current_request = sqlx::query(
    "SELECT status FROM budget_change_requests WHERE id = ?"
  )
  .bind( id )
  .fetch_optional( pool )
  .await?;

  let current_status = match current_request
  {
    Some( row ) =>
    {
      row.try_get::< String, _ >( "status" )?
    }
    None =>
    {
      return Err( sqlx::Error::RowNotFound );
    }
  };

  // Check if request is pending (optimistic locking precondition)
  if current_status != "pending"
  {
    return Err( sqlx::Error::RowNotFound ); // Simulate optimistic lock failure
  }

  // Update with optimistic locking WHERE clause
  let update_result = sqlx::query(
    "UPDATE budget_change_requests
     SET status = ?,
         updated_at = ?
     WHERE id = ? AND status = 'pending'"
  )
  .bind( "rejected" )
  .bind( updated_at )
  .bind( id )
  .execute( pool )
  .await?;

  // If no rows affected, concurrent modification occurred
  if update_result.rows_affected() == 0
  {
    return Err( sqlx::Error::RowNotFound ); // Optimistic lock failed
  }

  Ok( update_result.rows_affected() )
}

/// Record a budget modification in history
///
/// # Errors
///
/// Returns error if database insertion fails or constraint violations occur
pub async fn record_budget_modification(
  pool: &SqlitePool,
  history: &BudgetModificationHistory,
) -> Result< (), sqlx::Error >
{
  sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, related_request_id, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &history.id )
  .bind( history.agent_id )
  .bind( history.modification_type.to_db_string() )
  .bind( history.old_budget_micros )
  .bind( history.new_budget_micros )
  .bind( history.change_amount_micros )
  .bind( &history.modifier_id )
  .bind( &history.reason )
  .bind( &history.related_request_id )
  .bind( history.created_at )
  .execute( pool )
  .await?;

  Ok( () )
}

/// Get budget modification history for an agent
///
/// # Errors
///
/// Returns error if database query fails
pub async fn get_budget_history(
  pool: &SqlitePool,
  agent_id: i64,
) -> Result< Vec< BudgetModificationHistory >, sqlx::Error >
{
  let rows = sqlx::query(
    "SELECT id, agent_id, modification_type, old_budget_micros, new_budget_micros,
            change_amount_micros, modifier_id, reason, related_request_id, created_at
     FROM budget_modification_history
     WHERE agent_id = ?
     ORDER BY created_at DESC"
  )
  .bind( agent_id )
  .fetch_all( pool )
  .await?;

  let mut history = Vec::new();
  for row in rows
  {
    let mod_type_str: String = row.get( "modification_type" );
    let modification_type = ModificationType::from_db_string( &mod_type_str )
      .map_err( | e | sqlx::Error::Decode( Box::new( std::io::Error::new( std::io::ErrorKind::InvalidData, e ) ) ) )?;

    history.push( BudgetModificationHistory
    {
      id: row.get( "id" ),
      agent_id: row.get( "agent_id" ),
      modification_type,
      old_budget_micros: row.get( "old_budget_micros" ),
      new_budget_micros: row.get( "new_budget_micros" ),
      change_amount_micros: row.get( "change_amount_micros" ),
      modifier_id: row.get( "modifier_id" ),
      reason: row.get( "reason" ),
      related_request_id: row.get( "related_request_id" ),
      created_at: row.get( "created_at" ),
    } );
  }

  Ok( history )
}
