//! Budget request workflow API (Protocol 012)
//!
//! Budget change request approval workflow

use super::state::BudgetState;
use axum::
{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

// ============================================================================
// Protocol 012: Budget Request Workflow API
// ============================================================================

/// Create budget change request (Protocol 012)
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct CreateBudgetRequestRequest
{
  pub agent_id: i64,
  pub requester_id: String,
  pub requested_budget_usd: f64,
  pub justification: String,
}

impl CreateBudgetRequestRequest
{
  /// Minimum justification length (database constraint from migration 011)
  const MIN_JUSTIFICATION_LENGTH: usize = 20;

  /// Maximum justification length (database constraint from migration 011)
  const MAX_JUSTIFICATION_LENGTH: usize = 500;

  /// Maximum budget in USD for pilot deployment ($10,000 = pilot operational limit)
  ///
  /// Rationale: Pilot phase restricts individual budget requests to $10K to limit
  /// financial exposure during initial validation. This is significantly below the
  /// technical limit of ~9.2 quintillion microdollars (i64::MAX) which allows for
  /// safe production scaling after pilot phase completes.
  ///
  /// Technical note: Microdollar conversion (×1,000,000) means i64::MAX supports
  /// up to ~$9.2 trillion USD before overflow. Current $10K limit leaves substantial
  /// headroom for future production use.
  ///
  /// GAP-002: Protocol 012 compliance - max budget lowered from $1T to $10K for pilot.
  const MAX_BUDGET_USD: f64 = 10_000.0;  // $10K pilot limit

  /// Validate create budget request parameters
  ///
  /// Fix(issue-003): Added `is_finite()` check to reject NaN and Infinity values.
  ///
  /// Root cause: Original validation used `requested_budget_usd <= 0.0` which doesnt
  /// catch NaN (comparison returns false) or Infinity (comparison passes). NaN and
  /// Infinity are nonsensical values for currency but could bypass validation.
  ///
  /// Pitfall: Always check `is_finite()` before other numeric validations on f64/f32
  /// inputs. NaN bypasses comparisons (NaN != NaN), Infinity passes positive checks.
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate agent_id is positive
    if self.agent_id <= 0
    {
      return Err( "agent_id must be positive".to_string() );
    }

    // Validate requester_id is not empty
    if self.requester_id.trim().is_empty()
    {
      return Err( "requester_id cannot be empty".to_string() );
    }

    // Validate requested_budget_usd is finite (not NaN or Infinity)
    if !self.requested_budget_usd.is_finite()
    {
      return Err( "requested_budget_usd must be a valid number".to_string() );
    }

    // Validate requested_budget_usd is positive
    if self.requested_budget_usd <= 0.0
    {
      return Err( "requested_budget_usd must be positive".to_string() );
    }

    // Fix(issue-TBD): Validate requested_budget_usd does not exceed safe maximum
    //
    // Root cause: Microdollar conversion (USD × 1,000,000) can overflow i64::MAX
    // for extremely large f64 values. Values like f64::MAX (~1.8e308) cause undefined
    // behavior when cast to i64 after multiplication. This was not validated, allowing
    // dangerous values to reach conversion code (budget.rs:1017).
    //
    // Pitfall: Floating-point to integer casts with out-of-range values produce
    // undefined behavior in Rust. Always validate numeric bounds before conversion.
    // Don't assume is_finite() is sufficient - finite doesn't mean reasonable.
    if self.requested_budget_usd > Self::MAX_BUDGET_USD
    {
      return Err( format!(
        "requested_budget_usd exceeds maximum allowed budget of {} USD",
        Self::MAX_BUDGET_USD
      ) );
    }

    // Validate justification length
    let justification_len = self.justification.trim().len();
    if justification_len < Self::MIN_JUSTIFICATION_LENGTH
    {
      return Err( format!(
        "justification too short (min {} characters)",
        Self::MIN_JUSTIFICATION_LENGTH
      ) );
    }

    if justification_len > Self::MAX_JUSTIFICATION_LENGTH
    {
      return Err( format!(
        "justification too long (max {} characters)",
        Self::MAX_JUSTIFICATION_LENGTH
      ) );
    }

    Ok( () )
  }
}

/// Create budget request response
#[ derive( Debug, Serialize ) ]
pub struct CreateBudgetRequestResponse
{
  pub request_id: String,
  pub status: String,
  pub created_at: i64,
}

/// POST /api/v1/budget/requests
///
/// Create a new budget change request (Protocol 012)
///
/// # Arguments
///
/// * `state` - Budget protocol state (database, managers)
/// * `user` - Authenticated user from JWT
/// * `request` - Budget request parameters
///
/// # Returns
///
/// - 201 Created with request_id if successful
/// - 400 Bad Request if validation fails
/// - 403 Forbidden if user doesn't own agent
/// - 404 Not Found if agent doesnt exist
/// - 500 Internal Server Error if database fails
pub async fn create_budget_request(
  State( state ): State< BudgetState >,
  user: crate::jwt_auth::AuthenticatedUser,
  Json( request ): Json< CreateBudgetRequestRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
    {
      "error": validation_error
    } ) ) ).into_response();
  }

  // Check if agent exists and verify ownership
  let agent_owner_result = sqlx::query_scalar::<sqlx::Sqlite, String>(
    "SELECT owner_id FROM agents WHERE id = ?"
  )
  .bind( request.agent_id )
  .fetch_optional( &state.db_pool )
  .await;

  let agent_owner = match agent_owner_result
  {
    Ok( owner ) => owner,
    Err( err ) =>
    {
      tracing::error!( "Database error checking agent: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  match agent_owner
  {
    None =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!(
      {
        "error": "Agent not found"
      } ) ) ).into_response();
    }
    Some( owner_id ) if user.0.role != "admin" && owner_id != user.0.sub =>
    {
      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!({ "error": "You don't own this agent" }) ),
      )
        .into_response();
    }
    Some( _ ) =>
    {
      // Authorized - user owns the agent or is admin
    }
  }

  // Get current agent budget
  let current_budget_result = state
    .agent_budget_manager
    .get_budget_status( request.agent_id )
    .await;

  let current_budget_micros = match current_budget_result
  {
    Ok( Some( budget ) ) => budget.budget_remaining,
    Ok( None ) => 0, // No budget record = $0.00
    Err( err ) =>
    {
      tracing::error!( "Database error fetching agent budget: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to fetch agent budget" }) ),
      )
        .into_response();
    }
  };

  // Generate unique request ID
  let request_id = format!( "breq_{}", Uuid::new_v4() );
  let now_ms = chrono::Utc::now().timestamp_millis();
  let requested_budget_micros = ( request.requested_budget_usd * 1_000_000.0 ) as i64;

  // Fix(issue-004): Validate requested budget differs from current budget
  //
  // Root cause: Original implementation allowed creating budget change requests where
  // requested budget equals current budget, causing nonsensical workflow operations.
  //
  // Pitfall: Business logic validation belongs in API layer after fetching related data.
  // Validating that operations make logical sense ("budget change must change budget")
  // prevents wasted approval cycles and database clutter.
  if requested_budget_micros == current_budget_micros
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
    {
      "error": "requested_budget_usd must differ from current budget"
    } ) ) ).into_response();
  }

  // Create budget request in database using storage layer
  let budget_request = iron_token_manager::budget_request::BudgetChangeRequest
  {
    id: request_id.clone(),
    agent_id: request.agent_id,
    requester_id: request.requester_id.clone(),
    current_budget_micros,
    requested_budget_micros,
    justification: request.justification.clone(),
    status: iron_token_manager::budget_request::RequestStatus::Pending,
    created_at: now_ms,
    updated_at: now_ms,
  };

  if let Err( err ) = iron_token_manager::budget_request::create_budget_request( &state.db_pool, &budget_request ).await
  {
    tracing::error!( "Database error creating budget request: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to create budget request" }) ),
    )
      .into_response();
  }

  // Return success response
  (
    StatusCode::CREATED,
    Json( CreateBudgetRequestResponse
    {
      request_id,
      status: "pending".to_string(),
      created_at: now_ms,
    } ),
  )
    .into_response()
}

/// Get budget request response
#[ derive( Debug, Serialize ) ]
pub struct GetBudgetRequestResponse
{
  pub id: String,
  pub agent_id: i64,
  pub requester_id: String,
  pub current_budget_usd: f64,
  pub requested_budget_usd: f64,
  pub justification: String,
  pub status: String,
  pub created_at: i64,
  pub updated_at: i64,
}

/// GET /api/v1/budget/requests/:id
///
/// Get a budget change request by ID (Protocol 012)
///
/// # Arguments
///
/// * `state` - Budget protocol state (database, managers)
/// * `request_id` - Budget request ID from path parameter
///
/// # Returns
///
/// - 200 OK with request details if found
/// - 404 Not Found if request doesnt exist
/// - 500 Internal Server Error if database fails
pub async fn get_budget_request(
  State( state ): State< BudgetState >,
  axum::extract::Path( request_id ): axum::extract::Path< String >,
) -> impl IntoResponse
{
  // Fetch request from database using storage layer
  let budget_request_result = iron_token_manager::budget_request::get_budget_request( &state.db_pool, &request_id ).await;

  match budget_request_result
  {
    Ok( Some( request ) ) =>
    {
      // Convert microdollars to USD
      let current_budget_usd = request.current_budget_micros as f64 / 1_000_000.0;
      let requested_budget_usd = request.requested_budget_micros as f64 / 1_000_000.0;

      // Return success response
      (
        StatusCode::OK,
        Json( GetBudgetRequestResponse
        {
          id: request.id,
          agent_id: request.agent_id,
          requester_id: request.requester_id,
          current_budget_usd,
          requested_budget_usd,
          justification: request.justification,
          status: request.status.to_db_string().to_string(),
          created_at: request.created_at,
          updated_at: request.updated_at,
        } ),
      )
        .into_response()
    }
    Ok( None ) =>
    {
      // Request not found
      ( StatusCode::NOT_FOUND, Json( serde_json::json!(
      {
        "error": "Budget request not found"
      } ) ) ).into_response()
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching budget request: {}", err );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response()
    }
  }
}

/// Query parameters for listing budget requests
#[ derive( Debug, Deserialize ) ]
pub struct ListBudgetRequestsQuery
{
  pub agent_id: Option< i64 >,
  pub status: Option< String >,
}

/// List budget requests response
#[ derive( Debug, Serialize ) ]
pub struct ListBudgetRequestsResponse
{
  pub requests: Vec< GetBudgetRequestResponse >,
}

/// GET /api/v1/budget/requests
///
/// List budget change requests with optional filtering (Protocol 012)
///
/// # Arguments
///
/// * `state` - Budget protocol state (database, managers)
/// * `query` - Optional query parameters (agent_id, status)
///
/// # Query Parameters
///
/// * `agent_id` - Filter by agent ID (optional)
/// * `status` - Filter by status: pending/approved/rejected/cancelled (optional)
///
/// # Returns
///
/// - 200 OK with array of requests (empty array if no matches)
/// - 500 Internal Server Error if database fails
pub async fn list_budget_requests(
  State( state ): State< BudgetState >,
  axum::extract::Query( query ): axum::extract::Query< ListBudgetRequestsQuery >,
) -> impl IntoResponse
{
  // Determine which query to use based on filters
  let requests_result = match ( query.agent_id, query.status.as_deref() )
  {
    // Filter by both agent_id and status
    ( Some( agent_id ), Some( status_str ) ) =>
    {
      // Parse status
      let status = match iron_token_manager::budget_request::RequestStatus::from_db_string( status_str )
      {
        Ok( s ) => s,
        Err( err ) =>
        {
          return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
          {
            "error": format!( "Invalid status: {}", err )
          } ) ) ).into_response();
        }
      };

      // Get by agent first, then filter by status in memory
      match iron_token_manager::budget_request::list_budget_requests_by_agent( &state.db_pool, agent_id ).await
      {
        Ok( all_agent_requests ) =>
        {
          let filtered: Vec< _ > = all_agent_requests
            .into_iter()
            .filter( | r | r.status == status )
            .collect();
          Ok( filtered )
        }
        Err( e ) => Err( e ),
      }
    }

    // Filter by agent_id only
    ( Some( agent_id ), None ) =>
    {
      iron_token_manager::budget_request::list_budget_requests_by_agent( &state.db_pool, agent_id ).await
    }

    // Filter by status only
    ( None, Some( status_str ) ) =>
    {
      // Parse status
      let status = match iron_token_manager::budget_request::RequestStatus::from_db_string( status_str )
      {
        Ok( s ) => s,
        Err( err ) =>
        {
          return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
          {
            "error": format!( "Invalid status: {}", err )
          } ) ) ).into_response();
        }
      };

      iron_token_manager::budget_request::list_budget_requests_by_status( &state.db_pool, status ).await
    }

    // No filters - fetch all requests
    ( None, None ) =>
    {
      let rows = sqlx::query(
        "SELECT id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
                justification, status, created_at, updated_at
         FROM budget_change_requests
         ORDER BY created_at DESC"
      )
      .fetch_all( &state.db_pool )
      .await;

      match rows
      {
        Ok( rows ) =>
        {
          let mut requests = Vec::new();
          for row in rows
          {
            let status_str: String = sqlx::Row::get( &row, "status" );
            let status = match iron_token_manager::budget_request::RequestStatus::from_db_string( &status_str )
            {
              Ok( s ) => s,
              Err( e ) =>
              {
                tracing::error!( "Invalid status in database: {}", e );
                continue; // Skip invalid rows
              }
            };

            requests.push( iron_token_manager::budget_request::BudgetChangeRequest
            {
              id: sqlx::Row::get( &row, "id" ),
              agent_id: sqlx::Row::get( &row, "agent_id" ),
              requester_id: sqlx::Row::get( &row, "requester_id" ),
              current_budget_micros: sqlx::Row::get( &row, "current_budget_micros" ),
              requested_budget_micros: sqlx::Row::get( &row, "requested_budget_micros" ),
              justification: sqlx::Row::get( &row, "justification" ),
              status,
              created_at: sqlx::Row::get( &row, "created_at" ),
              updated_at: sqlx::Row::get( &row, "updated_at" ),
            } );
          }
          Ok( requests )
        }
        Err( e ) => Err( e ),
      }
    }
  };

  match requests_result
  {
    Ok( requests ) =>
    {
      // Convert to response format
      let response_requests: Vec< GetBudgetRequestResponse > = requests
        .into_iter()
        .map( | r |
        {
          GetBudgetRequestResponse
          {
            id: r.id,
            agent_id: r.agent_id,
            requester_id: r.requester_id,
            current_budget_usd: r.current_budget_micros as f64 / 1_000_000.0,
            requested_budget_usd: r.requested_budget_micros as f64 / 1_000_000.0,
            justification: r.justification,
            status: r.status.to_db_string().to_string(),
            created_at: r.created_at,
            updated_at: r.updated_at,
          }
        } )
        .collect();

      (
        StatusCode::OK,
        Json( ListBudgetRequestsResponse
        {
          requests: response_requests,
        } ),
      )
        .into_response()
    }
    Err( err ) =>
    {
      tracing::error!( "Database error listing budget requests: {}", err );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response()
    }
  }
}

/// Approve budget request response
#[ derive( Debug, Serialize ) ]
pub struct ApproveBudgetRequestResponse
{
  pub request_id: String,
  pub status: String,
  pub updated_at: i64,
}

/// PATCH /api/v1/budget/requests/:id/approve
///
/// Approve a budget change request (Protocol 012)
///
/// # Arguments
///
/// * `state` - Budget protocol state (database, managers)
/// * `request_id` - Budget request ID from path parameter
///
/// # Returns
///
/// - 200 OK with updated status if successful
/// - 404 Not Found if request doesnt exist
/// - 409 Conflict if request is not pending
/// - 500 Internal Server Error if database fails
pub async fn approve_budget_request(
  State( state ): State< BudgetState >,
  axum::extract::Path( request_id ): axum::extract::Path< String >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
) -> impl IntoResponse
{
  // Fetch request from database
  let request_result = iron_token_manager::budget_request::get_budget_request( &state.db_pool, &request_id ).await;

  let request = match request_result
  {
    Ok( Some( req ) ) => req,
    Ok( None ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!(
      {
        "error": "Budget request not found"
      } ) ) ).into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching budget request: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  // Check if request is in pending status
  if request.status != iron_token_manager::budget_request::RequestStatus::Pending
  {
    let error_msg = match request.status
    {
      iron_token_manager::budget_request::RequestStatus::Approved =>
      {
        "Budget request is already approved"
      }
      iron_token_manager::budget_request::RequestStatus::Rejected =>
      {
        "Cannot approve rejected budget request"
      }
      iron_token_manager::budget_request::RequestStatus::Cancelled =>
      {
        "Cannot approve cancelled budget request"
      }
      _ => "Budget request is not pending",
    };

    return ( StatusCode::CONFLICT, Json( serde_json::json!(
    {
      "error": error_msg
    } ) ) ).into_response();
  }

  // Update status to approved and apply budget change
  let now_ms = chrono::Utc::now().timestamp_millis();
  let approver_id = &claims.sub; // Extract user ID from JWT claims
  let update_result = iron_token_manager::budget_request::approve_budget_request( &state.db_pool, &request_id, approver_id, now_ms ).await;

  match update_result
  {
    Ok( () ) =>
    {
      // Approval succeeded - budget was updated atomically
      // Return success response
      (
        StatusCode::OK,
        Json( ApproveBudgetRequestResponse
        {
          request_id,
          status: "approved".to_string(),
          updated_at: now_ms,
        } ),
      )
        .into_response()
    }
    Err( err ) =>
    {
      tracing::error!( "Database error approving budget request: {}", err );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response()
    }
  }
}

/// Reject budget request response
#[ derive( Debug, Serialize ) ]
pub struct RejectBudgetRequestResponse
{
  pub request_id: String,
  pub status: String,
  pub updated_at: i64,
}

/// PATCH /api/v1/budget/requests/:id/reject
///
/// Rejects a budget change request (Protocol 012).
///
/// # Request
///
/// - Method: PATCH
/// - Path: `/api/v1/budget/requests/:id/reject`
/// - Path parameter: `id` - Budget request ID
///
/// # Response
///
/// Success (200 OK):
/// ```json
/// {
///   "request_id": "breq_...",
///   "status": "rejected",
///   "updated_at": 1234567890
/// }
/// ```
///
/// Errors:
/// - 404 Not Found: Request doesnt exist
/// - 409 Conflict: Request is not pending (already approved/rejected/cancelled)
/// - 500 Internal Server Error: Database error
pub async fn reject_budget_request(
  State( state ): State< BudgetState >,
  axum::extract::Path( request_id ): axum::extract::Path< String >,
) -> impl IntoResponse
{
  // Fetch request from database
  let request_result = iron_token_manager::budget_request::get_budget_request( &state.db_pool, &request_id ).await;

  let request = match request_result
  {
    Ok( Some( req ) ) => req,
    Ok( None ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!(
      {
        "error": "Budget request not found"
      } ) ) ).into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching budget request: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  // Check if request is in pending status
  if request.status != iron_token_manager::budget_request::RequestStatus::Pending
  {
    let error_msg = match request.status
    {
      iron_token_manager::budget_request::RequestStatus::Rejected =>
      {
        "Budget request is already rejected"
      }
      iron_token_manager::budget_request::RequestStatus::Approved =>
      {
        "Cannot reject approved budget request"
      }
      iron_token_manager::budget_request::RequestStatus::Cancelled =>
      {
        "Cannot reject cancelled budget request"
      }
      _ => "Budget request is not pending",
    };

    return ( StatusCode::CONFLICT, Json( serde_json::json!(
    {
      "error": error_msg
    } ) ) ).into_response();
  }

  // Update status to rejected
  let now_ms = chrono::Utc::now().timestamp_millis();
  let update_result = iron_token_manager::budget_request::reject_budget_request( &state.db_pool, &request_id, now_ms ).await;

  match update_result
  {
    Ok( rows_affected ) =>
    {
      if rows_affected == 0
      {
        // This shouldnt happen since we just fetched the request
        tracing::error!( "Failed to update budget request status - no rows affected" );
        return (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json( serde_json::json!({ "error": "Failed to update request status" }) ),
        )
          .into_response();
      }

      // Return success response
      (
        StatusCode::OK,
        Json( RejectBudgetRequestResponse
        {
          request_id,
          status: "rejected".to_string(),
          updated_at: now_ms,
        } ),
      )
        .into_response()
    }
    Err( err ) =>
    {
      tracing::error!( "Database error rejecting budget request: {}", err );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response()
    }
  }
}
