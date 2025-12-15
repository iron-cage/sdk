//! Budget refresh endpoint (Protocol 005)
//!
//! Request additional budget allocation

use super::state::BudgetState;
use crate::error::ValidationError;
use axum::
{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

/// Budget refresh request (Step 3: Request More Budget)
#[ derive( Debug, Deserialize ) ]
pub struct BudgetRefreshRequest
{
  pub ic_token: String,
  pub current_lease_id: String,
  pub requested_budget: Option< i64 >,
}

impl BudgetRefreshRequest
{
  /// Maximum IC Token length
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Maximum lease_id length
  const MAX_LEASE_ID_LENGTH: usize = 100;

  /// Maximum budget request (microdollars)
  const MAX_BUDGET_REQUEST: i64 = 1_000_000_000; // 1000 USD

  /// Default budget refresh amount (microdollars)
  const DEFAULT_REFRESH_BUDGET: i64 = 10_000_000; // 10 USD

  /// Validate budget refresh request parameters
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    // Validate ic_token
    if self.ic_token.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "ic_token".to_string() ) );
    }

    if self.ic_token.len() > Self::MAX_IC_TOKEN_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "ic_token".to_string(),
        max_length: Self::MAX_IC_TOKEN_LENGTH,
      } );
    }

    // Validate current_lease_id
    if self.current_lease_id.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "current_lease_id".to_string() ) );
    }

    if self.current_lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "current_lease_id".to_string(),
        max_length: Self::MAX_LEASE_ID_LENGTH,
      } );
    }

    // Validate requested_budget if provided
    if let Some( budget ) = self.requested_budget
    {
      if budget <= 0
      {
        return Err( ValidationError::InvalidValue
        {
          field: "requested_budget".to_string(),
          reason: "must be positive".to_string(),
        } );
      }

      if budget > Self::MAX_BUDGET_REQUEST
      {
        return Err( ValidationError::InvalidValue
        {
          field: "requested_budget".to_string(),
          reason: format!( "too large (max {} microdollars)", Self::MAX_BUDGET_REQUEST ),
        } );
      }
    }

    Ok( () )
  }

  /// Get requested budget or default
  pub fn get_requested_budget( &self ) -> i64
  {
    self.requested_budget.unwrap_or( Self::DEFAULT_REFRESH_BUDGET )
  }
}

/// Budget refresh response (approved)
#[ derive( Debug, Serialize ) ]
pub struct BudgetRefreshResponse
{
  pub status: String,
  pub budget_granted: Option< i64 >,
  pub budget_remaining: i64,
  pub lease_id: Option< String >,
  pub reason: Option< String >,
}

/// POST /api/budget/refresh
///
/// Request additional budget
///
/// # Arguments
///
/// * `state` - Budget protocol state
/// * `request` - Budget refresh request
///
/// # Returns
///
/// - 200 OK with approval/denial status
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if JWT authentication fails
/// - 404 Not Found if lease doesnt exist
/// - 500 Internal Server Error if database fails
pub async fn refresh_budget(
  State( state ): State< BudgetState >,
  authenticated_user: crate::jwt_auth::AuthenticatedUser,
  Json( request ): Json< BudgetRefreshRequest >,
) -> impl IntoResponse
{
  // Extract approver identity from JWT for audit trail (GAP-003)
  let approver_id = &authenticated_user.0.sub;
  tracing::info!(
    "Budget refresh requested by approver: {} (role: {})",
    approver_id,
    authenticated_user.0.role
  );

  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
    {
      "error": validation_error.to_string()
    } ) ) ).into_response();
  }

  // Verify IC Token
  let claims = match state.ic_token_manager.verify_token( &request.ic_token )
  {
    Ok( claims ) => claims,
    Err( _ ) =>
    {
      return ( StatusCode::UNAUTHORIZED, Json( serde_json::json!(
      {
        "error": "Invalid IC Token"
      } ) ) ).into_response();
    }
  };

  // Extract agent_id from IC Token claims
  // Claims.agent_id format: "agent_<id>"
  let agent_id = match claims.agent_id.strip_prefix( "agent_" )
  {
    Some( id_str ) => match id_str.parse::< i64 >()
    {
      Ok( id ) => id,
      Err( _ ) =>
      {
        return ( StatusCode::UNAUTHORIZED, Json( serde_json::json!(
        {
          "error": "Invalid agent ID in IC Token"
        } ) ) ).into_response();
      }
    },
    None =>
    {
      return ( StatusCode::UNAUTHORIZED, Json( serde_json::json!(
      {
        "error": "Invalid IC Token agent_id format"
      } ) ) ).into_response();
    }
  };

  // Get current lease
  let lease = match state.lease_manager.get_lease( &request.current_lease_id ).await
  {
    Ok( Some( lease ) ) => lease,
    Ok( None ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Lease not found" }) ),
      )
        .into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching lease: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Lease service unavailable" }) ),
      )
        .into_response();
    }
  };

  // Verify IC Token agent matches lease owner (authorization check)
  if lease.agent_id != agent_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!(
      {
        "error": "Unauthorized - lease belongs to different agent"
      } ) ),
    )
      .into_response();
  }

  // Fix(issue-budget-007): Missing lease revocation check in refresh_budget
  //
  // Root cause: Report usage endpoint had revocation check (issue-budget-001), but refresh_budget
  // endpoint was missing the same validation. When refresh was implemented, the developer copied
  // the authorization check pattern from report_usage but forgot to also copy the lease status
  // validation (expiry + revocation checks). This allowed revoked leases to refresh successfully.
  //
  // Pitfall: Incomplete validation copying - when copying validation patterns between similar
  // endpoints, ensure ALL relevant checks are copied, not just authorization. Pattern: endpoints
  // operating on same resource type (leases, tokens, sessions) should have consistent validation.
  // Use checklist: (1) existence, (2) authorization, (3) state (expiry/revocation/enabled),
  // (4) capacity/limits. Missing ANY check creates security gap.
  //
  // Check if lease has expired
  if let Some( expires_at ) = lease.expires_at
  {
    let now_ms = chrono::Utc::now().timestamp_millis();
    if expires_at < now_ms
    {
      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!({ "error": "Lease expired" }) ),
      )
        .into_response();
    }
  }

  // Check if lease has been revoked
  if lease.lease_status == "revoked"
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Lease has been revoked" }) ),
    )
      .into_response();
  }

  // Get agent budget
  let agent_budget = match state
    .agent_budget_manager
    .get_budget_status( lease.agent_id )
    .await
  {
    Ok( Some( budget ) ) => budget,
    Ok( None ) =>
    {
      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!({ "error": "No budget allocated" }) ),
      )
        .into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching agent budget: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Budget service unavailable" }) ),
      )
        .into_response();
    }
  };

  // Get requested budget amount (with default)
  let requested_budget = request.get_requested_budget();

  // Fix(issue-budget-008): Reserve budget atomically before creating new lease
  //
  // Root cause: Original implementation created new lease via `create_lease()` without
  // reserving budget from agent. `create_lease()` only creates the database record but
  // does NOT call `check_and_reserve_budget()` to update agent's total_spent and
  // budget_remaining. This violated the budget invariant and allowed unlimited lease
  // creation without budget enforcement.
  //
  // Pitfall: Never call `lease_manager.create_lease()` directly for user-facing endpoints.
  // Always reserve budget first via `agent_budget_manager.check_and_reserve_budget()`.
  // The lease record and agent budget must be updated atomically. Pattern: handshake
  // endpoint does this correctly (handshake.rs:256-258), refresh endpoint must match.

  // Atomically check and reserve budget for new lease
  let budget_granted = match state
    .agent_budget_manager
    .check_and_reserve_budget( lease.agent_id, requested_budget )
    .await
  {
    Ok( granted ) if granted > 0 => granted,
    Ok( _ ) =>
    {
      // Insufficient budget - deny request
      return ( StatusCode::OK, Json( BudgetRefreshResponse
      {
        status: "denied".to_string(),
        budget_granted: None,
        budget_remaining: agent_budget.budget_remaining,
        lease_id: None,
        reason: Some( "insufficient_budget".to_string() ),
      } ) )
        .into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error checking and reserving budget: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Budget service unavailable" }) ),
      )
        .into_response();
    }
  };

  // Create new lease with granted budget
  let new_lease_id = format!( "lease_{}", Uuid::new_v4() );

  if let Err( err ) = state
    .lease_manager
    .create_lease(
      &new_lease_id,
      lease.agent_id,
      lease.budget_id,
      budget_granted,
      None,
    )
    .await
  {
    tracing::error!( "Database error creating new lease: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to create new lease" }) ),
    )
      .into_response();
  }

  // Expire old lease
  if let Err( err ) = state.lease_manager.expire_lease( &request.current_lease_id ).await
  {
    tracing::error!( "Database error expiring old lease: {}", err );
    // Continue anyway - new lease was created
  }

  // Get updated budget status after reservation
  let updated_budget = match state
    .agent_budget_manager
    .get_budget_status( lease.agent_id )
    .await
  {
    Ok( Some( budget ) ) => budget,
    Ok( None ) =>
    {
      // Agent budget disappeared - should never happen
      tracing::error!( "Agent budget disappeared after refresh" );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Budget service unavailable" }) ),
      )
        .into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching updated budget: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Budget service unavailable" }) ),
      )
        .into_response();
    }
  };

  ( StatusCode::OK, Json( BudgetRefreshResponse
  {
    status: "approved".to_string(),
    budget_granted: Some( budget_granted ),
    budget_remaining: updated_budget.budget_remaining,
    lease_id: Some( new_lease_id ),
    reason: None,
  } ) )
    .into_response()
}
