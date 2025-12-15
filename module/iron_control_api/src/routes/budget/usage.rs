//! Budget usage reporting and return endpoints (Protocol 005)
//!
//! Cost tracking and unused budget return

use super::state::BudgetState;
use crate::error::ValidationError;
use axum::
{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use serde::{ Deserialize, Serialize };

// ============================================================================
// Protocol 005: Usage Reporting (Step 2)
// ============================================================================

/// Usage report request (Step 2: Cost Tracking)
#[ derive( Debug, Deserialize ) ]
pub struct UsageReportRequest
{
  pub lease_id: String,
  pub request_id: String,
  pub tokens: i64,
  pub cost_microdollars: i64,
  pub model: String,
  pub provider: String,
}

impl UsageReportRequest
{
  /// Maximum lease_id length
  const MAX_LEASE_ID_LENGTH: usize = 100;

  /// Maximum request_id length (UUID + prefix)
  const MAX_REQUEST_ID_LENGTH: usize = 100;

  /// Maximum model name length
  const MAX_MODEL_LENGTH: usize = 100;

  /// Maximum provider name length
  const MAX_PROVIDER_LENGTH: usize = 50;

  /// Validate usage report request parameters
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    // Validate lease_id
    if self.lease_id.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "lease_id".to_string() ) );
    }

    if self.lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "lease_id".to_string(),
        max_length: Self::MAX_LEASE_ID_LENGTH,
      } );
    }

    // Validate request_id
    if self.request_id.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "request_id".to_string() ) );
    }

    if self.request_id.len() > Self::MAX_REQUEST_ID_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "request_id".to_string(),
        max_length: Self::MAX_REQUEST_ID_LENGTH,
      } );
    }

    // Validate tokens is positive
    if self.tokens <= 0
    {
      return Err( ValidationError::InvalidValue
      {
        field: "tokens".to_string(),
        reason: "must be positive".to_string(),
      } );
    }

    // Validate cost_microdollars is non-negative
    if self.cost_microdollars < 0
    {
      return Err( ValidationError::InvalidValue
      {
        field: "cost_microdollars".to_string(),
        reason: "cannot be negative".to_string(),
      } );
    }

    // Validate model
    if self.model.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "model".to_string() ) );
    }

    if self.model.len() > Self::MAX_MODEL_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "model".to_string(),
        max_length: Self::MAX_MODEL_LENGTH,
      } );
    }

    // Validate provider
    if self.provider.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "provider".to_string() ) );
    }

    if self.provider.len() > Self::MAX_PROVIDER_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "provider".to_string(),
        max_length: Self::MAX_PROVIDER_LENGTH,
      } );
    }

    Ok( () )
  }
}

/// Usage report response
#[ derive( Debug, Serialize ) ]
pub struct UsageReportResponse
{
  pub success: bool,
  pub budget_remaining: i64,
}

/// POST /api/budget/report
///
/// Report LLM usage cost
///
/// # Arguments
///
/// * `state` - Budget protocol state
/// * `request` - Usage report with cost and token counts
///
/// # Returns
///
/// - 200 OK if usage recorded successfully
/// - 400 Bad Request if validation fails
/// - 404 Not Found if lease doesnt exist
/// - 500 Internal Server Error if database fails
pub async fn report_usage(
  State( state ): State< BudgetState >,
  Json( request ): Json< UsageReportRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
    {
      "error": validation_error.to_string()
    } ) ) ).into_response();
  }

  // Get lease
  let lease = match state.lease_manager.get_lease( &request.lease_id ).await
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

  // Fix(issue-budget-001): Missing lease expiry validation
  //
  // Root cause: Initial implementation only checked lease existence (database fetch succeeded)
  // but not lease validity (expires_at timestamp comparison). Focused on schema validation
  // rather than business logic validation.
  //
  // Pitfall: Time-based validation gaps - implementing database fetch without subsequent state
  // validation creates gap between "resource exists" and "resource is valid for operation".
  // Applies to ANY time-bounded resource (API tokens, sessions, leases, credentials).
  // Detection: Any endpoint accepting resource ID must verify BOTH existence AND validity
  // (expiry, revocation status, enabled flag) before usage.
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

  // Check if lease has been revoked or expired
  if lease.lease_status == "revoked"
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Lease has been revoked" }) ),
    )
      .into_response();
  }

  if lease.lease_status == "expired"
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Lease expired" }) ),
    )
      .into_response();
  }

  // Fix(issue-budget-002): Missing lease budget sufficiency check (CRITICAL)
  //
  // Root cause: Implementation immediately recorded usage without verifying lease had sufficient
  // remaining budget. After fetching lease and checking expiry, code directly called
  // lease_manager.record_usage() without calculating (budget_granted - budget_spent) and
  // comparing to cost_usd. CRITICAL security bug allowing agents to exceed allocated budgets,
  // violating Protocol 005 core guarantee.
  //
  // Pitfall: Assumed enforcement - never assume "obvious" business rules are automatically
  // enforced. Budget limits, rate limits, quota constraints MUST be explicitly coded AND tested.
  // Resource consumption pattern: Any code modifying limited resource (budgets, quotas, tokens)
  // without sufficiency check is vulnerability. Pattern appears in budget systems, rate limiters,
  // token allocators, permission systems. Detection: Search for resource modification operations
  // (.record_usage(), .consume(), .allocate(), .spend()) and verify corresponding sufficiency
  // check immediately before. Check-before-modify (not check-after-modify).
  //
  // Check if lease has sufficient remaining budget
  let lease_remaining = lease.budget_granted - lease.budget_spent;
  if lease_remaining < request.cost_microdollars
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Insufficient lease budget" }) ),
    )
      .into_response();
  }

  // Record usage in lease
  if let Err( err ) = state
    .lease_manager
    .record_usage( &request.lease_id, request.cost_microdollars )
    .await
  {
    tracing::error!( "Database error recording lease usage: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to record usage" }) ),
    )
      .into_response();
  }

  // Record usage in agent budget
  if let Err( err ) = state
    .agent_budget_manager
    .record_spending( lease.agent_id, request.cost_microdollars )
    .await
  {
    tracing::error!( "Database error recording agent spending: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to update agent budget" }) ),
    )
      .into_response();
  }

  // Get updated budget
  let budget_remaining = match state
    .agent_budget_manager
    .get_budget_status( lease.agent_id )
    .await
  {
    Ok( Some( budget ) ) => budget.budget_remaining,
    _ => 0,
  };

  ( StatusCode::OK, Json( UsageReportResponse
  {
    success: true,
    budget_remaining,
  } ) )
    .into_response()
}

// ============================================================================
// Protocol 005: Budget Return Endpoint (Step 4)
// ============================================================================

/// Budget return request (Step 4: Return Unused Budget)
#[ derive( Debug, Deserialize ) ]
pub struct BudgetReturnRequest
{
  pub lease_id: String,
  /// Amount spent by client (microdollars) - from iron_cost CostController
  #[ serde( default ) ]
  pub spent_microdollars: i64,
}

impl BudgetReturnRequest
{
  /// Maximum lease_id length
  const MAX_LEASE_ID_LENGTH: usize = 100;

  /// Validate budget return request parameters
  pub fn validate( &self ) -> Result< (), ValidationError >
  {
    if self.lease_id.trim().is_empty()
    {
      return Err( ValidationError::MissingField( "lease_id".to_string() ) );
    }

    if self.lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( ValidationError::TooLong
      {
        field: "lease_id".to_string(),
        max_length: Self::MAX_LEASE_ID_LENGTH,
      } );
    }

    if self.spent_microdollars < 0
    {
      return Err( ValidationError::InvalidValue
      {
        field: "spent_microdollars".to_string(),
        reason: "cannot be negative".to_string(),
      } );
    }

    Ok( () )
  }
}

/// Budget return response
#[ derive( Debug, Serialize ) ]
pub struct BudgetReturnResponse
{
  pub success: bool,
  pub returned: i64,
}

/// POST /api/budget/return
///
/// Return unused budget when runtime shuts down
///
/// This endpoint closes the lease and credits the unused budget back to
/// the agent's available budget.
///
/// # Arguments
///
/// * `state` - Budget protocol state
/// * `request` - Budget return request with lease_id
///
/// # Returns
///
/// - 200 OK with returned amount if successful
/// - 400 Bad Request if validation fails
/// - 404 Not Found if lease doesn't exist
/// - 500 Internal Server Error if database fails
pub async fn return_budget(
  State( state ): State< BudgetState >,
  Json( request ): Json< BudgetReturnRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!(
    {
      "error": validation_error.to_string()
    } ) ) ).into_response();
  }

  // Get lease to find agent_id
  let lease = match state.lease_manager.get_lease( &request.lease_id ).await
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

  // Check if lease is already closed
  if lease.lease_status != "active"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Lease is not active" }) ),
    )
      .into_response();
  }

  // Close the lease
  if let Err( err ) = state.lease_manager.close_lease( &request.lease_id ).await
  {
    tracing::error!( "Database error closing lease: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to close lease" }) ),
    )
      .into_response();
  }

  // Calculate returned: granted - spent (capped at 0)
  let returned = ( lease.budget_granted - request.spent_microdollars ).max( 0 );

  // Fix(issue-budget-007): Restore returned budget to agent_budgets
  //
  // Root cause: return_budget only credited usage_limits but not agent_budgets.
  // The handshake reserved budget from agent_budgets via check_and_reserve_budget(),
  // but return_budget never restored the unused portion back.
  //
  // Pitfall: When implementing a reserve/return pattern, ensure BOTH reserve AND return
  // update the same state. Missing return logic causes permanent "budget leak".
  //
  // Restore the returned budget to agent_budgets
  if returned > 0
  {
    if let Err( err ) = state
      .agent_budget_manager
      .restore_reserved_budget( lease.agent_id, returned )
      .await
    {
      tracing::error!( "Database error restoring agent budget: {}", err );
      // Continue - lease is closed, log the error but don't fail the return
    }
    else
    {
      tracing::info!(
        lease_id = %request.lease_id,
        agent_id = lease.agent_id,
        returned_microdollars = %returned,
        "Budget restored to agent_budgets"
      );
    }
  }

  // Credit the returned amount back to usage_limits
  if returned > 0
  {
    // Get agent's owner_id to find the usage_limits record
    let owner_id: Option< String > = match sqlx::query_scalar(
      "SELECT owner_id FROM agents WHERE id = ?"
    )
    .bind( lease.agent_id )
    .fetch_optional( &state.db_pool )
    .await
    {
      Ok( owner ) => owner,
      Err( err ) =>
      {
        tracing::error!( "Database error fetching agent owner: {}", err );
        // Still return success since lease was closed
        None
      }
    };

    if let Some( owner_id ) = owner_id
    {
      // Credit the returned amount back to usage_limits
      // Both are now in microdollars - no conversion needed
      if let Err( err ) = sqlx::query(
        "UPDATE usage_limits SET current_cost_microdollars_this_month = current_cost_microdollars_this_month - ? WHERE user_id = ?"
      )
      .bind( returned )
      .bind( &owner_id )
      .execute( &state.db_pool )
      .await
      {
        tracing::error!( "Database error crediting usage_limits: {}", err );
        // Still return success since lease was closed
      }
      else
      {
        tracing::info!(
          lease_id = %request.lease_id,
          agent_id = lease.agent_id,
          owner_id = %owner_id,
          returned_microdollars = %returned,
          "Budget returned and credited to usage_limits"
        );
      }
    }
    else
    {
      tracing::warn!(
        lease_id = %request.lease_id,
        agent_id = lease.agent_id,
        returned_microdollars = %returned,
        "Budget returned but agent has no owner - cannot credit usage_limits"
      );
    }
  }

  // Return success response
  ( StatusCode::OK, Json( BudgetReturnResponse
  {
    success: true,
    returned,
  } ) )
    .into_response()
}
