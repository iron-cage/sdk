//! Budget handshake endpoint (Protocol 005)
//!
//! IC Token → IP Token exchange with budget lease creation

use super::state::BudgetState;
use axum::
{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_token_manager::provider_key_storage::ProviderType;
use serde::{ Deserialize, Serialize };
use uuid::Uuid;

/// Budget handshake request (Step 1: Token Exchange)
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct HandshakeRequest
{
  pub ic_token: String,
  pub provider: String,
  pub provider_key_id: Option< i64 >,
  pub requested_budget: Option< i64 >,
}

impl HandshakeRequest
{
  /// Maximum IC Token length (JWT tokens can be long)
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Maximum provider name length
  const MAX_PROVIDER_LENGTH: usize = 50;

  /// Default budget lease amount (microdollars) for handshake
  const DEFAULT_HANDSHAKE_BUDGET: i64 = 10_000_000; // 10 USD

  /// Maximum budget request (microdollars) for handshake (DoS prevention)
  pub const MAX_HANDSHAKE_BUDGET: i64 = 100_000_000; // 100 USD

  /// Validate handshake request parameters
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate ic_token is not empty
    if self.ic_token.trim().is_empty()
    {
      return Err( "ic_token cannot be empty".to_string() );
    }

    // Validate ic_token length (DoS prevention)
    if self.ic_token.len() > Self::MAX_IC_TOKEN_LENGTH
    {
      return Err( format!(
        "ic_token too long (max {} characters)",
        Self::MAX_IC_TOKEN_LENGTH
      ) );
    }

    // Validate provider is not empty
    if self.provider.trim().is_empty()
    {
      return Err( "provider cannot be empty".to_string() );
    }

    // Validate provider length
    if self.provider.len() > Self::MAX_PROVIDER_LENGTH
    {
      return Err( format!(
        "provider too long (max {} characters)",
        Self::MAX_PROVIDER_LENGTH
      ) );
    }

    // Validate requested_budget if provided
    if let Some( budget ) = self.requested_budget
    {
      if budget <= 0
      {
        return Err( "requested_budget must be positive".to_string() );
      }

      if budget > Self::MAX_HANDSHAKE_BUDGET
      {
        return Err( format!(
          "requested_budget exceeds maximum ({} microdollars / ${:.2} USD)",
          Self::MAX_HANDSHAKE_BUDGET,
          Self::MAX_HANDSHAKE_BUDGET as f64 / 1_000_000.0
        ) );
      }
    }

    Ok( () )
  }
}

/// Budget handshake response
#[ derive( Debug, Serialize ) ]
pub struct HandshakeResponse
{
  pub ip_token: String,
  pub lease_id: String,
  pub budget_granted: i64,
  pub budget_remaining: i64,
  pub expires_at: Option< i64 >,
}

/// POST /api/budget/handshake
///
/// Budget handshake: IC Token → IP Token exchange
///
/// # Arguments
///
/// * `state` - Budget protocol state (managers, crypto, database)
/// * `request` - Handshake request with IC Token and provider selection
///
/// # Returns
///
/// - 200 OK with IP Token and lease if successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if IC Token invalid
/// - 403 Forbidden if budget exhausted
/// - 500 Internal Server Error if crypto or database fails
pub async fn handshake(
  State( state ): State< BudgetState >,
  Json( request ): Json< HandshakeRequest >,
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

  // Verify IC Token
  let claims = match state.ic_token_manager.verify_token( &request.ic_token )
  {
    Ok( claims ) => claims,
    Err( _ ) =>
    {
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "Invalid IC Token" }) ),
      )
        .into_response();
    }
  };

  // Get agent_id from IC Token claims
  let agent_id_str = &claims.agent_id;

  // Fix(authorization-bypass-handshake): Reject malformed agent_id instead of defaulting to 1
  // Root cause: Code used .unwrap_or(1) when parsing agent_id from IC Token,
  //             defaulting to agent_id=1 on parse failure. This allowed attackers to bypass
  //             authorization by sending malformed agent_id values (alphabetic, special chars,
  //             overflow, etc.), which would parse fail and default to using agent_id=1's budget.
  // Pitfall: Never use fallback values for security-critical parsing. Always reject invalid
  //          input with explicit error responses. Using .unwrap_or() for authorization data
  //          is a critical anti-pattern - silently accepts malformed input, creates authorization
  //          bypass when fallback is privileged, enables billing fraud.
  // Test coverage: See tests/handshake_malformed_agent_id_test.rs
  //
  // Parse agent_id (format: agent_<id>) to get database ID
  let agent_id : i64 = match agent_id_str.strip_prefix( "agent_" )
  {
    Some( id_part ) =>
    {
      match id_part.parse::< i64 >()
      {
        Ok( id ) if id > 0 => id,  // Valid positive ID
        Ok( _ ) =>
        {
          return (
            StatusCode::BAD_REQUEST,
            Json( serde_json::json!({ "error": "Invalid agent_id - must be positive" }) ),
          )
            .into_response();
        }
        Err( _ ) =>
        {
          return (
            StatusCode::BAD_REQUEST,
            Json( serde_json::json!({ "error": "Invalid agent_id - must be numeric" }) ),
          )
            .into_response();
        }
      }
    }
    None =>
    {
      return (
        StatusCode::BAD_REQUEST,
        Json( serde_json::json!({ "error": "Invalid agent_id format" }) ),
      )
        .into_response();
    }
  };

  // Get agent's owner_id to look up usage_limits
  let owner_id: Option< String > = match sqlx::query_scalar(
    "SELECT owner_id FROM agents WHERE id = ?"
  )
  .bind( agent_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( owner ) => owner,
    Err( err ) =>
    {
      tracing::error!( "Database error fetching agent owner: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  let owner_id = match owner_id
  {
    Some( id ) => id,
    None =>
    {
      // Security: Use generic error to prevent agent enumeration attacks
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({ "error": "Invalid IC Token" }) ),
      )
        .into_response();
    }
  };

  // Fix(issue-budget-006): Atomically check and reserve budget to prevent TOCTOU race
  //
  // Root cause: get_budget_status() and record_spending() were separate operations,
  // creating race window where concurrent requests could both pass budget check before either
  // recorded spending, causing negative budget (invariant violation).
  //
  // Pitfall: Time-of-check to time-of-use (TOCTOU) races occur when check and update are
  // separate operations. Always use atomic operations (SELECT FOR UPDATE + UPDATE in single
  // transaction) for check-then-act patterns on shared resources.

  // Use requested_budget if provided, otherwise use default
  let budget_requested = request.requested_budget.unwrap_or( HandshakeRequest::DEFAULT_HANDSHAKE_BUDGET );

  let budget_to_grant = match state
    .agent_budget_manager
    .check_and_reserve_budget( agent_id, budget_requested )
    .await
  {
    Ok( granted ) if granted > 0 => granted,
    Ok( _ ) =>
    {
      // Insufficient budget or agent doesnt exist
      // Fetch budget details for error response
      let agent_budget = state
        .agent_budget_manager
        .get_budget_status( agent_id )
        .await
        .ok()
        .flatten();

      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!(
        {
          "error": "Budget limit exceeded",
          "total_allocated": agent_budget.as_ref().map( | b | b.total_allocated ),
          "total_spent": agent_budget.as_ref().map( | b | b.total_spent ),
          "budget_remaining": agent_budget.as_ref().map( | b | b.budget_remaining )
        } ) ),
      )
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

  // Get provider API key
  let provider_type = match request.provider.as_str()
  {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    _ =>
    {
      return (
        StatusCode::BAD_REQUEST,
        Json( serde_json::json!({ "error": format!( "Unsupported provider: {}", request.provider ) }) ),
      )
        .into_response();
    }
  };

  // Get provider key ID (use provided or fetch first available for provider)
  let key_id = match request.provider_key_id
  {
    Some( id ) => id,
    None =>
    {
      // Get first available key for this provider
      match state.provider_key_storage.get_keys_by_provider( provider_type ).await
      {
        Ok( keys ) if !keys.is_empty() => keys[ 0 ],
        Ok( _ ) =>
        {
          return (
            StatusCode::NOT_FOUND,
            Json( serde_json::json!({ "error": format!( "No API keys configured for provider: {}", request.provider ) }) ),
          )
            .into_response();
        }
        Err( err ) =>
        {
          tracing::error!( "Database error fetching provider keys: {}", err );
          return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json( serde_json::json!({ "error": "Key storage unavailable" }) ),
          )
            .into_response();
        }
      }
    }
  };

  // Get provider key record (encrypted)
  let _key_record = match state.provider_key_storage.get_key( key_id ).await
  {
    Ok( record ) => record,
    Err( err ) =>
    {
      tracing::error!( "Database error fetching provider key: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Key storage unavailable" }) ),
      )
        .into_response();
    }
  };

  // Validate provider key matches requested provider
  if _key_record.metadata.provider != provider_type
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Provider key does not match requested provider" }) ),
    )
      .into_response();
  }

  // Validate provider key is enabled
  if !_key_record.metadata.is_enabled
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Provider key is disabled" }) ),
    )
      .into_response();
  }

  // Decrypt provider API key from database
  let encrypted_secret = match iron_secrets::crypto::EncryptedSecret::from_base64(
    &_key_record.encrypted_api_key,
    &_key_record.encryption_nonce,
  )
  {
    Ok( secret ) => secret,
    Err( _ ) =>
    {
      tracing::error!( "Failed to decode provider key base64" );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Key storage error" }) ),
      )
        .into_response();
    }
  };

  let provider_key = match state.provider_key_crypto.decrypt( &encrypted_secret )
  {
    Ok( key ) => key,
    Err( err ) =>
    {
      tracing::error!( "Failed to decrypt provider API key: {:?}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to decrypt provider key" }) ),
      )
        .into_response();
    }
  };

  // Encrypt provider API key into IP Token
  let ip_token = match state.ip_token_crypto.encrypt( &provider_key )
  {
    Ok( token ) => token,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to encrypt IP Token" }) ),
      )
        .into_response();
    }
  };

  // Create budget lease
  // Note: Budget already atomically reserved by check_and_reserve_budget() above
  let lease_id = format!( "lease_{}", Uuid::new_v4() );

  if let Err( err ) = state
    .lease_manager
    .create_lease( &lease_id, agent_id, agent_id, budget_to_grant, None )
    .await
  {
    tracing::error!( "Database error creating lease: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to create budget lease" }) ),
    )
      .into_response();
  }

  // Budget spending already recorded by check_and_reserve_budget() - no separate call needed

  // Deduct lease amount from usage_limits (the "bank")
  // Both are now in microdollars - no conversion needed
  if let Err( err ) = sqlx::query(
    "UPDATE usage_limits SET current_cost_microdollars_this_month = current_cost_microdollars_this_month + ? WHERE user_id = ?"
  )
  .bind( budget_to_grant )
  .bind( &owner_id )
  .execute( &state.db_pool )
  .await
  {
    tracing::error!( "Database error updating usage_limits: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to update usage limits" }) ),
    )
      .into_response();
  }

  tracing::info!(
    agent_id = agent_id,
    owner_id = %owner_id,
    budget_granted = budget_to_grant,
    "Budget lease granted, deducted from usage_limits"
  );

  // Return successful handshake response
  // budget_remaining is 0 because we grant the full remaining budget as the lease
  ( StatusCode::OK, Json( HandshakeResponse
  {
    ip_token,
    lease_id,
    budget_granted: budget_to_grant,
    budget_remaining: 0, // Full budget granted to lease
    expires_at: None, // No expiration by default
  } ) )
    .into_response()
}
