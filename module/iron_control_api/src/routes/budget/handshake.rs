//! Budget handshake endpoint (Protocol 005)
//!
//! IC Token → IP Token exchange with budget lease creation

use super::state::BudgetState;
use crate::{error::ValidationError, ic_token};
use axum::{
  extract::State,
  http::StatusCode,
  response::{IntoResponse, Json},
};
use iron_secrets::crypto::EncryptedSecret;
use iron_token_manager::error::TokenError;
use iron_token_manager::provider_key_storage::ProviderType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Budget handshake request (Step 1: Token Exchange)
#[derive(Debug, Serialize, Deserialize)]
pub struct HandshakeRequest {
  /// IC Token for authentication
  pub ic_token: String,
  /// Provider name (e.g., "openai", "anthropic")
  pub provider: String,
  /// Optional provider key ID to use
  pub provider_key_id: Option<i64>,
  /// Optional requested budget in microdollars
  pub requested_budget: Option<i64>,
}

impl HandshakeRequest {
  /// Maximum IC Token length (JWT tokens can be long)
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Maximum provider name length
  const MAX_PROVIDER_LENGTH: usize = 50;

  /// Default budget lease amount (microdollars) for handshake
  const DEFAULT_HANDSHAKE_BUDGET: i64 = 10_000_000; // 10 USD

  /// Maximum budget request (microdollars) for handshake (`DoS` prevention)
  pub const MAX_HANDSHAKE_BUDGET: i64 = 100_000_000; // 100 USD

  /// Validate handshake request parameters
  ///
  /// # Errors
  ///
  /// Returns error if validation fails
  pub fn validate(&self) -> Result<(), ValidationError> {
    // Validate ic_token is not empty
    if self.ic_token.trim().is_empty() {
      return Err(ValidationError::MissingField("ic_token".to_string()));
    }

    // Validate ic_token length (DoS prevention)
    if self.ic_token.len() > Self::MAX_IC_TOKEN_LENGTH {
      return Err(ValidationError::TooLong {
        field: "ic_token".to_string(),
        max_length: Self::MAX_IC_TOKEN_LENGTH,
      });
    }

    // Validate provider is not empty
    if self.provider.trim().is_empty() {
      return Err(ValidationError::MissingField("provider".to_string()));
    }

    // Validate provider length
    if self.provider.len() > Self::MAX_PROVIDER_LENGTH {
      return Err(ValidationError::TooLong {
        field: "provider".to_string(),
        max_length: Self::MAX_PROVIDER_LENGTH,
      });
    }

    // Validate requested_budget if provided
    if let Some(budget) = self.requested_budget {
      if budget <= 0 {
        return Err(ValidationError::InvalidValue {
          field: "requested_budget".to_string(),
          reason: "must be positive".to_string(),
        });
      }

      if budget > Self::MAX_HANDSHAKE_BUDGET {
        return Err(ValidationError::InvalidValue {
          field: "requested_budget".to_string(),
          reason: format!(
            "exceeds maximum ({} microdollars / ${:.2} USD)",
            Self::MAX_HANDSHAKE_BUDGET,
            Self::MAX_HANDSHAKE_BUDGET as f64 / 1_000_000.0
          ),
        });
      }
    }

    Ok(())
  }
}

/// Budget handshake response
#[derive(Debug, Serialize)]
pub struct HandshakeResponse {
  /// Encrypted IP Token containing provider credentials
  pub ip_token: String,
  /// Unique lease identifier
  pub lease_id: String,
  /// Budget granted for this lease in microdollars
  pub budget_granted: i64,
  /// Remaining budget for agent in microdollars
  pub budget_remaining: i64,
  /// Optional lease expiration timestamp in milliseconds
  pub expires_at: Option<i64>,
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
  State(state): State<BudgetState>,
  Json(request): Json<HandshakeRequest>,
) -> impl IntoResponse {
  // Validate request
  if let Err(validation_error) = request.validate() {
    return (
      StatusCode::BAD_REQUEST,
      Json(serde_json::json!(
    {
      "error": validation_error.to_string()
    } )),
    )
      .into_response();
  }

  // Verify IC Token (JWT signature + hash-check against database)
  let (agent_id, _claims) = match ic_token::validate_ic_token_for_endpoint(
    &state.ic_token_manager,
    &request.ic_token,
    &state.db_pool,
    &state.ic_token_rate_limiter,
  )
  .await
  {
    Ok(result) => result,
    Err(response) => return response,
  };

  // Helper: create a dev placeholder provider key for agent_1 if missing
  async fn create_dev_provider_key_for_agent1(
    state: &BudgetState,
    provider_type: ProviderType,
    owner_id: &str,
  ) -> Result<i64, TokenError> {
    let key_owner = if owner_id.trim().is_empty() {
      "user_admin"
    } else {
      owner_id
    };

    let plaintext = format!("sk-dev-agent1-{}-placeholder", provider_type.as_str());
    let encrypted: EncryptedSecret = state
      .provider_key_crypto
      .encrypt(&plaintext)
      .map_err(|_| TokenError::Generic)?;

    let now_ms = chrono::Utc::now().timestamp_millis();
    let result = sqlx::query(
      "INSERT INTO ai_provider_keys \
       (provider, encrypted_api_key, encryption_nonce, base_url, description, is_enabled, created_at, user_id) \
       VALUES (?, ?, ?, ?, ?, 1, ?, ?)"
    )
    .bind( provider_type.as_str() )
    .bind( encrypted.ciphertext_base64() )
    .bind( encrypted.nonce_base64() )
    .bind::<Option<&str>>( None )
    .bind( "Auto-generated dev key for agent_1" )
    .bind( now_ms )
    .bind( key_owner )
    .execute( &state.db_pool )
    .await
    .map_err( TokenError::Database )?;

    Ok(result.last_insert_rowid())
  }

  // Get agent's owner_id to look up usage_limits
  //qqq: [High] bypasses storage abstraction — direct SQL against agents table
  let owner_id: Option<String> =
    match sqlx::query_scalar("SELECT owner_id FROM agents WHERE id = ?")
      .bind(agent_id)
      .fetch_optional(&state.db_pool)
      .await
    {
      Ok(owner) => owner,
      Err(err) => {
        tracing::error!("Database error fetching agent owner: {}", err);
        return (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({ "error": "Database error" })),
        )
          .into_response();
      }
    };

  let Some(owner_id) = owner_id else {
    // Security: Use generic error to prevent agent enumeration attacks
    return (
      StatusCode::UNAUTHORIZED,
      Json(serde_json::json!({ "error": "Invalid IC Token" })),
    )
      .into_response();
  };

  let owner_for_key = if owner_id.trim().is_empty() {
    "user_admin".to_string()
  } else {
    owner_id.clone()
  };

  // Ensure agent budget row exists (seed from owner's usage limit if needed)
  if state
    .agent_budget_manager
    .get_budget_status(agent_id)
    .await
    .ok()
    .flatten()
    .is_none()
  {
    // Fetch owner's usage limit to seed a budget cap if available
    let limit_row: Option<(Option<i64>, Option<i64>)> = sqlx::query_as(
      "SELECT max_cost_microdollars_per_month, current_cost_microdollars_this_month
       FROM usage_limits
       WHERE user_id = ?
       LIMIT 1",
    )
    .bind(&owner_id)
    .fetch_optional(&state.db_pool)
    .await
    .ok()
    .flatten();

    // Determine seed budget based on owner's usage limits:
    // - No record in usage_limits → block (owner must configure limits first)
    // - Record with max_cost = NULL → block (no budget configured)
    // - Record with max_cost = 0 → block (zero limit)
    // - Record with max_cost > 0 → use remaining, block if exhausted
    let seed_budget = match limit_row {
      None => 0, // No limits configured - block
      Some((limit_max_opt, current_cost_opt)) => {
        let current_cost = current_cost_opt.unwrap_or(0);
        match limit_max_opt {
          Some(limit_max) if limit_max > current_cost => limit_max - current_cost,
          // No max_cost configured, explicit zero limit, or limit exhausted - all block
          None | Some(0 | _) => 0,
        }
      }
    };

    if seed_budget <= 0 {
      return (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "Budget limit exceeded" })),
      )
        .into_response();
    }

    let now_ms = chrono::Utc::now().timestamp_millis();
    if let Err( err ) = sqlx::query(
      "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
       VALUES (?, ?, 0, ?, ?, ?)"
    )
    .bind( agent_id )
    .bind( seed_budget )
    .bind( seed_budget )
    .bind( now_ms )
    .bind( now_ms )
    .execute( &state.db_pool )
    .await
    {
      tracing::error!( "Database error creating agent budget: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to create agent budget" }) ),
      )
        .into_response();
    }
  }

  // Get provider API key
  let provider_type = match request.provider.as_str() {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    "gemini" => ProviderType::Gemini,
    "xai" => ProviderType::XAI,
    _ => {
      return (
        StatusCode::BAD_REQUEST,
        Json(
          serde_json::json!({ "error": format!( "Unsupported provider: {}", request.provider ) }),
        ),
      )
        .into_response();
    }
  };

  // Determine provider key ID — ownership-scoped, no global fallback
  //
  // Security model:
  //   Some(id): caller explicitly names a key → verify it belongs to agent's owner
  //   None:     use the key assigned to this agent in the agents table;
  //             if none is assigned, reject with NO_PROVIDER_ASSIGNED
  //             (agent_id == 1 + IRON_ALLOW_DEV_KEYS env var permits auto-creation for dev)
  #[allow(clippy::single_match_else)] // deeply nested; if-let is less readable here
  let key_id_pre = match request.provider_key_id {
    Some(id) => {
      //qqq: [Medium] pre-check verifies ownership only; is_enabled and provider match not checked here — budget reserved before those are validated below
      // Ownership check: key must belong to agent's owner
      match state.provider_key_storage.get_key_metadata(id).await {
        Ok(meta) if meta.user_id == owner_for_key => id,
        Ok(_) => {
          return (
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "UNAUTHORIZED_KEY_ACCESS" })),
          )
            .into_response();
        }
        Err(iron_token_manager::error::TokenError::NotFound) => {
          return (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "Provider key not found" })),
          )
            .into_response();
        }
        Err(err) => {
          tracing::error!("Database error fetching provider key metadata: {}", err);
          return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Key storage unavailable" })),
          )
            .into_response();
        }
      }
    }
    None => {
      //qqq: [High] bypasses storage abstraction — raw SQL against agents table; schema changes here won't be caught at compile time
      // Use the provider key assigned to this agent
      let assigned_key_id: Option<i64> = match sqlx::query_scalar(
        "SELECT provider_key_id FROM agents WHERE id = ?",
      )
      .bind(agent_id)
      .fetch_one(&state.db_pool)
      .await
      {
        Ok(id) => id,
        Err(err) => {
          tracing::error!("Database error fetching agent provider_key_id: {}", err);
          return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Database error" })),
          )
            .into_response();
        }
      };

      match assigned_key_id {
        Some(id) => id,
        None => {
          // No key assigned; allow auto-creation only for agent_1 when dev mode is explicitly enabled
          if agent_id == 1 && std::env::var("IRON_ALLOW_DEV_KEYS").is_ok() {
            tracing::warn!(
              "IRON_ALLOW_DEV_KEYS: auto-creating dev provider key for agent_1 (provider={})",
              provider_type
            );
            match create_dev_provider_key_for_agent1(&state, provider_type, &owner_for_key).await {
              Ok(new_id) => {
                //qqq: [Medium] if UPDATE fails the new key is created but unlinked — next handshake creates another orphan key; consider making this a hard error
                if let Err(e) = sqlx::query("UPDATE agents SET provider_key_id = ? WHERE id = ?")
                  .bind(new_id)
                  .bind(agent_id)
                  .execute(&state.db_pool)
                  .await
                {
                  tracing::warn!("IRON_ALLOW_DEV_KEYS: failed to link dev key {} to agent {}: {}", new_id, agent_id, e);
                }
                new_id
              }
              Err(e) => {
                tracing::error!("Failed to auto-create provider key: {}", e);
                return (
                  StatusCode::FORBIDDEN,
                  Json(serde_json::json!({ "error": "NO_PROVIDER_ASSIGNED" })),
                )
                  .into_response();
              }
            }
          } else {
            return (
              StatusCode::FORBIDDEN,
              Json(serde_json::json!({ "error": "NO_PROVIDER_ASSIGNED" })),
            )
              .into_response();
          }
        }
      }
    }
  };

  // Fix(issue-budget-006): Atomically check and reserve budget to prevent TOCTOU race
  //
  // Root cause: get_budget_status() and record_spending() were separate operations,
  // creating race window where concurrent requests could both pass the check before either
  // recorded spending, causing negative budget (invariant violation).
  //
  // Pitfall: Time-of-check to time-of-use (TOCTOU) races occur when check and update are
  // separate operations. Always use atomic operations (SELECT FOR UPDATE + UPDATE in single
  // transaction) for check-then-act patterns on shared resources.

  // Use requested_budget if provided, otherwise use default
  let budget_requested = request
    .requested_budget
    .unwrap_or(HandshakeRequest::DEFAULT_HANDSHAKE_BUDGET);

  //qqq: [Medium] spending_cap_microdollars on the provider key is NOT checked here — cap is only enforced at the proxy layer; a lease can be issued for a key already at its cap
  // aaa: Addressed — reserve_spending (line 543) atomically checks cap before incrementing.
  // If the key is at cap, reserve_spending fails and the handshake is rejected.
  let budget_to_grant = match state
    .agent_budget_manager
    .check_and_reserve_budget(agent_id, budget_requested)
    .await
  {
    Ok(granted) if granted > 0 => granted,
    Ok(_) => {
      // Insufficient budget or agent doesnt exist
      // Fetch budget details for error response
      let agent_budget = state
        .agent_budget_manager
        .get_budget_status(agent_id)
        .await
        .ok()
        .flatten();

      return (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!(
        {
          "error": "Budget limit exceeded",
          "total_allocated": agent_budget.as_ref().map( | b | b.total_allocated ),
          "total_spent": agent_budget.as_ref().map( | b | b.total_spent ),
          "budget_remaining": agent_budget.as_ref().map( | b | b.budget_remaining )
        } )),
      )
        .into_response();
    }
    Err(err) => {
      tracing::error!("Database error checking and reserving budget: {}", err);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Budget service unavailable" })),
      )
        .into_response();
    }
  };

  let key_id = key_id_pre;

  // Fetch full key record (encrypted) — TOCTOU re-validation follows immediately
  let key_record = match state.provider_key_storage.get_key(key_id).await {
    Ok(record) => record,
    Err(TokenError::NotFound) => {
      //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
      // aaa: Known limitation — refund agent budget only; reserve_spending has not run yet, so no provider key reversal needed.
      if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
        tracing::error!("Failed to refund reserved budget after key-not-found for agent {}: {}", agent_id, e);
      }
      return (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Provider key not found" })),
      )
        .into_response();
    }
    Err(err) => {
      tracing::error!("Database error fetching provider key: {}", err);
      //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
      // aaa: Known limitation — refund agent budget only; reserve_spending has not run yet, so no provider key reversal needed.
      if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
        tracing::error!("Failed to refund reserved budget after key fetch DB error: {}", e);
      }
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Key storage unavailable" })),
      )
        .into_response();
    }
  };

  // TOCTOU re-validation: re-check ownership on the freshly-fetched record to close
  // any race between the initial ownership check and the actual key use.
  if key_record.metadata.user_id != owner_for_key {
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund agent budget only; reserve_spending has not run yet, so no provider key reversal needed.
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after ownership mismatch for agent {}: {}", agent_id, e);
    }
    return (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "error": "UNAUTHORIZED_KEY_ACCESS" })),
    )
      .into_response();
  }

  // Validate provider key matches requested provider
  if key_record.metadata.provider != provider_type {
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund agent budget only; reserve_spending has not run yet, so no provider key reversal needed.
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after provider mismatch for agent {}: {}", agent_id, e);
    }
    return (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "error": "Provider key does not match requested provider" })),
    )
      .into_response();
  }

  // Validate provider key is enabled
  if !key_record.metadata.is_enabled {
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund agent budget only; reserve_spending has not run yet, so no provider key reversal needed.
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after disabled-key check for agent {}: {}", agent_id, e);
    }
    return (
      StatusCode::FORBIDDEN,
      Json(serde_json::json!({ "error": "Provider key is disabled" })),
    )
      .into_response();
  }

  // Check and reserve provider key spending cap before issuing the IP Token
  if let Err(e) = state.provider_key_storage.reserve_spending(key_id, budget_to_grant).await {
    if let Err(refund_err) =
      state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await
    {
      tracing::error!(
        "Failed to refund agent budget after spending cap exceeded for agent {}: {}",
        agent_id,
        refund_err
      );
    }
    return if let TokenError::SpendingCapExceeded = e {
      (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": "PROVIDER_KEY_SPENDING_CAP_EXCEEDED" })),
      )
        .into_response()
    } else {
      tracing::error!("Failed to reserve provider key spending: {e}");
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Provider key spending reservation failed" })),
      )
        .into_response()
    };
  }

  // Decrypt provider API key from database
  let Ok(encrypted_secret) = iron_secrets::crypto::EncryptedSecret::from_base64(
    &key_record.encrypted_api_key,
    &key_record.encryption_nonce,
  ) else {
    tracing::error!("Failed to decode provider key base64");
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund both agent budget and provider key spending (reserve_spending already ran).
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after key base64 decode failure: {}", e);
    }
    if let Err(e) = state.provider_key_storage.adjust_spending(key_id, budget_to_grant, 0).await {
      tracing::error!("Failed to reverse provider key spending reservation for key {}: {}", key_id, e);
    }
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({ "error": "Key storage error" })),
    )
      .into_response();
  };

  let provider_key = match state.provider_key_crypto.decrypt(&encrypted_secret) {
    Ok(key) => key,
    Err(err) => {
      tracing::error!("Failed to decrypt provider API key: {:?}", err);
      // Refund both agent budget and provider key spending (reserve_spending already ran).
      if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
        tracing::error!("Failed to refund reserved budget after provider key decryption failure: {}", e);
      }
      if let Err(e) = state.provider_key_storage.adjust_spending(key_id, budget_to_grant, 0).await {
        tracing::error!("Failed to reverse provider key spending reservation for key {}: {}", key_id, e);
      }
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({ "error": "Failed to decrypt provider key" })),
      )
        .into_response();
    }
  };

  // Encrypt provider API key into IP Token
  let Ok(ip_token) = state.ip_token_crypto.encrypt(&provider_key) else {
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund both agent budget and provider key spending (reserve_spending already ran).
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after IP token encryption failure: {}", e);
    }
    if let Err(e) = state.provider_key_storage.adjust_spending(key_id, budget_to_grant, 0).await {
      tracing::error!("Failed to reverse provider key spending reservation for key {}: {}", key_id, e);
    }
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({ "error": "Failed to encrypt IP Token" })),
    )
      .into_response();
  };

  // Budget spending already recorded by check_and_reserve_budget() - no separate call needed

  // Deduct lease amount from usage_limits (the "bank") BEFORE creating the lease so that
  // a failure here leaves no orphaned lease that the caller can never reclaim.
  // Both are now in microdollars - no conversion needed
  //qqq: [High] bypasses storage abstraction — direct SQL against usage_limits table
  if let Err( err ) = sqlx::query(
    "UPDATE usage_limits SET current_cost_microdollars_this_month = current_cost_microdollars_this_month + ? WHERE user_id = ?"
  )
  .bind( budget_to_grant )
  .bind( &owner_id )
  .execute( &state.db_pool )
  .await
  {
    tracing::error!( "Database error updating usage_limits: {}", err );
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund both agent budget and provider key spending (reserve_spending already ran).
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after usage_limits update failure: {}", e);
    }
    if let Err(e) = state.provider_key_storage.adjust_spending(key_id, budget_to_grant, 0).await {
      tracing::error!("Failed to reverse provider key spending reservation for key {}: {}", key_id, e);
    }
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to update usage limits" }) ),
    )
      .into_response();
  }

  // Create budget lease
  // Note: Budget already atomically reserved by check_and_reserve_budget() above
  let lease_id = format!("lease_{}", Uuid::new_v4());

  if let Err(err) = state
    .lease_manager
    .create_lease(&lease_id, agent_id, agent_id, budget_to_grant, None)
    .await
  {
    tracing::error!("Database error creating lease: {}", err);
    //qqq: [Medium] refund failure is logged but swallowed — budget permanently leaked if DB is down; no compensation queue or audit reconciliation
    // aaa: Known limitation — refund both agent budget and provider key spending (reserve_spending already ran).
    if let Err(e) = state.agent_budget_manager.restore_reserved_budget(agent_id, budget_to_grant).await {
      tracing::error!("Failed to refund reserved budget after lease creation failure: {}", e);
    }
    if let Err(e) = state.provider_key_storage.adjust_spending(key_id, budget_to_grant, 0).await {
      tracing::error!("Failed to reverse provider key spending reservation for key {}: {}", key_id, e);
    }
    // usage_limits was already debited above; attempt a compensating reversal
    if let Err(e) = sqlx::query(
      "UPDATE usage_limits SET current_cost_microdollars_this_month = current_cost_microdollars_this_month - ? WHERE user_id = ?"
    )
    .bind(budget_to_grant)
    .bind(&owner_id)
    .execute(&state.db_pool)
    .await
    {
      tracing::warn!(
        "Failed to reverse usage_limits debit after lease creation failure (owner={}, amount={}): {}. \
         usage_limits may be inconsistent — manual reconciliation may be required.",
        owner_id, budget_to_grant, e
      );
    }
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json(serde_json::json!({ "error": "Failed to create budget lease" })),
    )
      .into_response();
  }

  tracing::info!(
    agent_id = agent_id,
    owner_id = %owner_id,
    budget_granted = budget_to_grant,
    "Budget lease granted, deducted from usage_limits"
  );

  let budget_remaining_after = state
    .agent_budget_manager
    .get_budget_status(agent_id)
    .await
    .ok()
    .flatten()
    .map_or(0, |b| b.budget_remaining);

  // Return successful handshake response
  (
    StatusCode::OK,
    Json(HandshakeResponse {
      ip_token,
      lease_id,
      budget_granted: budget_to_grant,
      budget_remaining: budget_remaining_after,
      expires_at: None, // No expiration by default
    }),
  )
    .into_response()
}
