//! Budget Control Protocol REST API endpoints
//!
//! Protocol 005: Budget Control Protocol
//!
//! Endpoints:
//! - POST /api/budget/handshake - IC Token → IP Token exchange with budget lease
//! - POST /api/budget/report - Report LLM usage cost to Control Panel
//! - POST /api/budget/refresh - Request additional budget when running low

use crate::{ ic_token::IcTokenManager, ip_token::IpTokenCrypto, jwt_auth::JwtSecret, routes::auth::AuthState };
use axum::
{
  extract::{ FromRef, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_token_manager::
{
  agent_budget::AgentBudgetManager,
  lease_manager::LeaseManager,
  provider_key_storage::{ ProviderKeyStorage, ProviderType },
};
use serde::{ Deserialize, Serialize };
use sqlx::SqlitePool;
use std::sync::Arc;
use uuid::Uuid;

/// Budget protocol shared state
#[ derive( Clone ) ]
pub struct BudgetState
{
  pub ic_token_manager: Arc< IcTokenManager >,
  pub ip_token_crypto: Arc< IpTokenCrypto >,
  pub lease_manager: Arc< LeaseManager >,
  pub agent_budget_manager: Arc< AgentBudgetManager >,
  pub provider_key_storage: Arc< ProviderKeyStorage >,
  pub db_pool: SqlitePool,
  pub jwt_secret: Arc< JwtSecret >,
}

/// Enable AuthState extraction from BudgetState
impl FromRef< BudgetState > for AuthState
{
  fn from_ref( state: &BudgetState ) -> Self
  {
    AuthState
    {
      jwt_secret: state.jwt_secret.clone(),
      db_pool: state.db_pool.clone(),
    }
  }
}

impl BudgetState
{
  /// Create new budget state
  ///
  /// # Arguments
  ///
  /// * `ic_token_secret` - Secret key for IC Token JWT signing
  /// * `ip_token_key` - 32-byte encryption key for IP Token AES-256-GCM
  /// * `jwt_secret` - Secret key for JWT access token signing/verification
  /// * `database_url` - Database connection string
  ///
  /// # Errors
  ///
  /// Returns error if database connection or crypto initialization fails
  pub async fn new(
    ic_token_secret: String,
    ip_token_key: &[ u8 ],
    jwt_secret: Arc< JwtSecret >,
    database_url: &str,
  ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let db_pool = SqlitePool::connect( database_url ).await?;
    let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
    let ip_token_crypto = Arc::new( IpTokenCrypto::new( ip_token_key )? );
    let lease_manager = Arc::new( LeaseManager::from_pool( db_pool.clone() ) );
    let agent_budget_manager = Arc::new( AgentBudgetManager::from_pool( db_pool.clone() ) );
    let provider_key_storage = Arc::new( ProviderKeyStorage::new( db_pool.clone() ) );

    Ok( Self
    {
      ic_token_manager,
      ip_token_crypto,
      lease_manager,
      agent_budget_manager,
      provider_key_storage,
      db_pool,
      jwt_secret,
    } )
  }
}

/// Budget handshake request (Step 1: Token Exchange)
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct HandshakeRequest
{
  pub ic_token: String,
  pub provider: String,
  pub provider_key_id: Option< i64 >,
}

impl HandshakeRequest
{
  /// Maximum IC Token length (JWT tokens can be long)
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Maximum provider name length
  const MAX_PROVIDER_LENGTH: usize = 50;

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

    Ok( () )
  }
}

/// Budget handshake response
#[ derive( Debug, Serialize ) ]
pub struct HandshakeResponse
{
  pub ip_token: String,
  pub lease_id: String,
  pub budget_granted: f64,
  pub budget_remaining: f64,
  pub expires_at: Option< i64 >,
}

/// Usage report request (Step 2: Cost Tracking)
#[ derive( Debug, Deserialize ) ]
pub struct UsageReportRequest
{
  pub lease_id: String,
  pub request_id: String,
  pub tokens: i64,
  pub cost_usd: f64,
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
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate lease_id
    if self.lease_id.trim().is_empty()
    {
      return Err( "lease_id cannot be empty".to_string() );
    }

    if self.lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( format!(
        "lease_id too long (max {} characters)",
        Self::MAX_LEASE_ID_LENGTH
      ) );
    }

    // Validate request_id
    if self.request_id.trim().is_empty()
    {
      return Err( "request_id cannot be empty".to_string() );
    }

    if self.request_id.len() > Self::MAX_REQUEST_ID_LENGTH
    {
      return Err( format!(
        "request_id too long (max {} characters)",
        Self::MAX_REQUEST_ID_LENGTH
      ) );
    }

    // Validate tokens is positive
    if self.tokens <= 0
    {
      return Err( "tokens must be positive".to_string() );
    }

    // Validate cost_usd is non-negative
    if self.cost_usd < 0.0
    {
      return Err( "cost_usd cannot be negative".to_string() );
    }

    // Validate model
    if self.model.trim().is_empty()
    {
      return Err( "model cannot be empty".to_string() );
    }

    if self.model.len() > Self::MAX_MODEL_LENGTH
    {
      return Err( format!(
        "model too long (max {} characters)",
        Self::MAX_MODEL_LENGTH
      ) );
    }

    // Validate provider
    if self.provider.trim().is_empty()
    {
      return Err( "provider cannot be empty".to_string() );
    }

    if self.provider.len() > Self::MAX_PROVIDER_LENGTH
    {
      return Err( format!(
        "provider too long (max {} characters)",
        Self::MAX_PROVIDER_LENGTH
      ) );
    }

    Ok( () )
  }
}

/// Usage report response
#[ derive( Debug, Serialize ) ]
pub struct UsageReportResponse
{
  pub success: bool,
  pub budget_remaining: f64,
}

/// Budget refresh request (Step 3: Request More Budget)
#[ derive( Debug, Deserialize ) ]
pub struct BudgetRefreshRequest
{
  pub ic_token: String,
  pub current_lease_id: String,
  pub requested_budget: Option< f64 >,
}

impl BudgetRefreshRequest
{
  /// Maximum IC Token length
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Maximum lease_id length
  const MAX_LEASE_ID_LENGTH: usize = 100;

  /// Maximum budget request (USD)
  const MAX_BUDGET_REQUEST: f64 = 1000.0;

  /// Default budget refresh amount (USD)
  const DEFAULT_REFRESH_BUDGET: f64 = 10.0;

  /// Validate budget refresh request parameters
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate ic_token
    if self.ic_token.trim().is_empty()
    {
      return Err( "ic_token cannot be empty".to_string() );
    }

    if self.ic_token.len() > Self::MAX_IC_TOKEN_LENGTH
    {
      return Err( format!(
        "ic_token too long (max {} characters)",
        Self::MAX_IC_TOKEN_LENGTH
      ) );
    }

    // Validate current_lease_id
    if self.current_lease_id.trim().is_empty()
    {
      return Err( "current_lease_id cannot be empty".to_string() );
    }

    if self.current_lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( format!(
        "current_lease_id too long (max {} characters)",
        Self::MAX_LEASE_ID_LENGTH
      ) );
    }

    // Validate requested_budget if provided
    if let Some( budget ) = self.requested_budget
    {
      if budget <= 0.0
      {
        return Err( "requested_budget must be positive".to_string() );
      }

      if budget > Self::MAX_BUDGET_REQUEST
      {
        return Err( format!(
          "requested_budget too large (max ${} USD)",
          Self::MAX_BUDGET_REQUEST
        ) );
      }
    }

    Ok( () )
  }

  /// Get requested budget or default
  pub fn get_requested_budget( &self ) -> f64
  {
    self.requested_budget.unwrap_or( Self::DEFAULT_REFRESH_BUDGET )
  }
}

/// Budget refresh response (approved)
#[ derive( Debug, Serialize ) ]
pub struct BudgetRefreshResponse
{
  pub status: String,
  pub budget_granted: Option< f64 >,
  pub budget_remaining: f64,
  pub lease_id: Option< String >,
  pub reason: Option< String >,
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

  // Parse agent_id (format: agent_<id>) to get database ID
  // For now, we'll extract the numeric part after "agent_"
  let agent_id : i64 = match agent_id_str.strip_prefix( "agent_" )
  {
    Some( id_part ) =>
    {
      // Try parsing as i64 directly, or use placeholder if not numeric
      // In production, you'd look up agent by string ID
      id_part.parse::< i64 >().unwrap_or( 1 )
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
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Agent not found" }) ),
      )
        .into_response();
    }
  };

  // Get usage limit for this user (budget from usage_limits table)
  let usage_limit: Option< ( Option< i64 >, i64 ) > = match sqlx::query_as(
    "SELECT max_cost_cents_per_month, current_cost_cents_this_month FROM usage_limits WHERE user_id = ?"
  )
  .bind( &owner_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( limit ) => limit,
    Err( err ) =>
    {
      tracing::error!( "Database error fetching usage limit: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Database error" }) ),
      )
        .into_response();
    }
  };

  // Calculate budget from usage_limits (convert cents to USD)
  let ( limit_usd, spent_usd, remaining_usd ) = match usage_limit
  {
    Some( ( Some( limit_cents ), spent_cents ) ) =>
    {
      let limit = limit_cents as f64 / 100.0;
      let spent = spent_cents as f64 / 100.0;
      let remaining = ( limit - spent ).max( 0.0 );
      ( limit, spent, remaining )
    }
    Some( ( None, _ ) ) =>
    {
      // No cost limit set - allow unlimited
      ( f64::MAX, 0.0, f64::MAX )
    }
    None =>
    {
      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!({ "error": "No budget limit configured for user" }) ),
      )
        .into_response();
    }
  };

  // Grant the full remaining budget as the lease
  let budget_to_grant = remaining_usd;
  if budget_to_grant <= 0.0
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!(
      {
        "error": "Budget limit exceeded",
        "limit_usd": limit_usd,
        "spent_usd": spent_usd,
        "remaining_usd": remaining_usd
      } ) ),
    )
      .into_response();
  }

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

  // TODO: Decrypt provider API key
  // For now, use a placeholder - in production this would decrypt _key_record.encrypted_api_key
  let provider_key = "sk-test_key_placeholder";

  // Encrypt provider API key into IP Token
  let ip_token = match state.ip_token_crypto.encrypt( provider_key )
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

  // Update agent budget (deduct granted amount from remaining)
  if let Err( err ) = state
    .agent_budget_manager
    .record_spending( agent_id, budget_to_grant )
    .await
  {
    tracing::error!( "Database error updating agent budget: {}", err );
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to update agent budget" }) ),
    )
      .into_response();
  }

  // Deduct lease amount from usage_limits (the "bank")
  let granted_cents = ( budget_to_grant * 100.0 ).round() as i64;
  if let Err( err ) = sqlx::query(
    "UPDATE usage_limits SET current_cost_cents_this_month = current_cost_cents_this_month + ? WHERE user_id = ?"
  )
  .bind( granted_cents )
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
    budget_remaining: 0.0, // Full budget granted to lease
    expires_at: None, // No expiration by default
  } ) )
    .into_response()
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
      "error": validation_error
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
  if lease_remaining < request.cost_usd
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
    .record_usage( &request.lease_id, request.cost_usd )
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
    .record_spending( lease.agent_id, request.cost_usd )
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
    _ => 0.0,
  };

  ( StatusCode::OK, Json( UsageReportResponse
  {
    success: true,
    budget_remaining,
  } ) )
    .into_response()
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
/// - 404 Not Found if lease doesnt exist
/// - 500 Internal Server Error if database fails
pub async fn refresh_budget(
  State( state ): State< BudgetState >,
  Json( request ): Json< BudgetRefreshRequest >,
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

  // Check if agent has sufficient remaining budget
  if agent_budget.budget_remaining < requested_budget
  {
    // Deny request
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

  // Approve request - create new lease
  let new_lease_id = format!( "lease_{}", Uuid::new_v4() );

  if let Err( err ) = state
    .lease_manager
    .create_lease(
      &new_lease_id,
      lease.agent_id,
      lease.budget_id,
      requested_budget,
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

  // Update agent budget (deduct granted amount from remaining)
  // Note: The budget was already deducted during handshake, so we dont double-deduct here
  // Instead, we just return the current status

  ( StatusCode::OK, Json( BudgetRefreshResponse
  {
    status: "approved".to_string(),
    budget_granted: Some( requested_budget ),
    budget_remaining: agent_budget.budget_remaining - requested_budget,
    lease_id: Some( new_lease_id ),
    reason: None,
  } ) )
    .into_response()
}

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

  /// Maximum budget in USD (1 trillion USD = safe limit before i64 overflow in microdollars)
  ///
  /// Rationale: Microdollar conversion multiplies by 1,000,000, so we need USD values
  /// that won't overflow i64::MAX (9,223,372,036,854,775,807) when converted.
  /// 1 trillion USD × 1 million microdollars/USD = 10^18 microdollars (safe margin).
  const MAX_BUDGET_USD: f64 = 1_000_000_000_000.0;  // 1 trillion USD

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
    Ok( Some( budget ) ) => ( budget.budget_remaining * 1_000_000.0 ) as i64,
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
  // TODO: Get approver_id from authenticated user context instead of using placeholder
  let approver_id = "system-admin";
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

// ============================================================================
// Protocol 005: Budget Return Endpoint
// ============================================================================

/// Budget return request (Step 4: Return Unused Budget)
#[ derive( Debug, Deserialize ) ]
pub struct BudgetReturnRequest
{
  pub lease_id: String,
  /// Amount spent by client (USD) - from iron_cost CostController
  #[ serde( default ) ]
  pub spent_usd: f64,
}

impl BudgetReturnRequest
{
  /// Maximum lease_id length
  const MAX_LEASE_ID_LENGTH: usize = 100;

  /// Validate budget return request parameters
  pub fn validate( &self ) -> Result< (), String >
  {
    if self.lease_id.trim().is_empty()
    {
      return Err( "lease_id cannot be empty".to_string() );
    }

    if self.lease_id.len() > Self::MAX_LEASE_ID_LENGTH
    {
      return Err( format!( "lease_id too long (max {} characters)", Self::MAX_LEASE_ID_LENGTH ) );
    }

    if self.spent_usd < 0.0
    {
      return Err( "spent_usd cannot be negative".to_string() );
    }

    Ok( () )
  }
}

/// Budget return response
#[ derive( Debug, Serialize ) ]
pub struct BudgetReturnResponse
{
  pub success: bool,
  pub returned: f64,
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
      "error": validation_error
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
  let returned = ( lease.budget_granted - request.spent_usd ).max( 0.0 );

  // Credit the returned amount back to usage_limits
  if returned > 0.0
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
      let returned_cents = ( returned * 100.0 ).round() as i64;
      if let Err( err ) = sqlx::query(
        "UPDATE usage_limits SET current_cost_cents_this_month = current_cost_cents_this_month - ? WHERE user_id = ?"
      )
      .bind( returned_cents )
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
          returned_usd = %returned,
          returned_cents = %returned_cents,
          "Budget returned and credited to usage_limits"
        );
      }
    }
    else
    {
      tracing::warn!(
        lease_id = %request.lease_id,
        agent_id = lease.agent_id,
        returned_usd = %returned,
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
