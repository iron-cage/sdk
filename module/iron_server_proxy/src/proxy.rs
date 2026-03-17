//! LLM proxy request handler and error types.
//!
//! Implements the core request pipeline:
//! IC Token auth -> provider key lookup -> decrypt -> forward -> cost tracking.

use core::net::SocketAddr;

use axum::{
  body::{self, Body},
  extract::{ConnectInfo, Request, State},
  http::header,
  response::{IntoResponse, Response},
  Json,
};
use secrecy::SecretBox;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{error::ProxyError, state::AppState};
use iron_llm_core::{self, ForwardRequest};
use iron_secrets::{crypto::EncryptedSecret, ip_token::ProviderKey};

/// Maximum request body size (10 MiB). Protects against OOM from oversized payloads.
const MAX_REQUEST_BODY_BYTES: usize = 10 * 1024 * 1024;

/// Health check endpoint - no authentication required.
pub async fn handle_health() -> Json<serde_json::Value> {
  Json(serde_json::json!({ "status": "ok" }))
}

/// Main LLM proxy endpoint - authenticates agent via IC Token,
/// decrypts provider key from DB, forwards request to LLM provider.
pub async fn handle_proxy(
  State(state): State<AppState>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  request: Request,
) -> Response {
  handle_proxy_inner(&state, addr, request)
    .await
    .unwrap_or_else(IntoResponse::into_response)
}

/// Agent record from database query.
#[derive(sqlx::FromRow)]
struct AgentRecord {
  #[allow(dead_code)]
  id: i64,
  #[allow(dead_code)]
  name: String,
  provider_key_id: Option<i64>,
  /// SHA-256 hash of the IC token (used for constant-time comparison).
  ic_token_hash: String,
}

/// Inner proxy handler with `?` error propagation.
async fn handle_proxy_inner(
  state: &AppState,
  addr: SocketAddr,
  request: Request,
) -> Result<Response, ProxyError> {
  // Step 0: Rate limit check using real client IP.
  // Behind nginx, addr.ip() is always 127.0.0.1. Use X-Real-IP set by nginx
  // (from $remote_addr, not spoofable by clients) to get the actual client IP.
  // Falls back to TCP peer IP for direct connections without a reverse proxy.
  let client_ip = request
    .headers()
    .get("x-real-ip")
    .and_then(|v| v.to_str().ok())
    .map_or_else(|| addr.ip().to_string(), ToString::to_string);
  if let Err(retry_after) = state.auth_rate_limiter.check(&client_ip) {
    return Err(ProxyError::RateLimited(retry_after));
  }

  // Step 1: Extract IC Token from Authorization header or x-api-key
  let auth_header = request
    .headers()
    .get(header::AUTHORIZATION)
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.strip_prefix("Bearer "));

  let x_api_key = request
    .headers()
    .get("x-api-key")
    .and_then(|v| v.to_str().ok());

  let ic_token = auth_header.or(x_api_key).ok_or_else(|| {
    state.auth_rate_limiter.record_failure(&client_ip);
    ProxyError::Unauthorized("Missing IC Token")
  })?;

  // Step 2: SHA-256 hash lookup with constant-time comparison.
  // Fetch candidate by hash prefix (non-secret, narrows DB scan),
  // then verify full hash via subtle::ct_eq to prevent timing attacks.
  let token_hash = format!("{:x}", Sha256::digest(ic_token.as_bytes()));
  let hash_prefix = &token_hash[..8];

  let candidates = sqlx::query_as::<_, AgentRecord>(
    "SELECT id, name, provider_key_id, ic_token_hash FROM agents WHERE ic_token_hash LIKE ?",
  )
  .bind(format!("{hash_prefix}%"))
  .fetch_all(&state.db_pool)
  .await
  .map_err(ProxyError::Database)?;

  let agent = candidates
    .into_iter()
    .find(|a| {
      a.ic_token_hash
        .as_bytes()
        .ct_eq(token_hash.as_bytes())
        .into()
    })
    .ok_or_else(|| {
      state.auth_rate_limiter.record_failure(&client_ip);
      ProxyError::Unauthorized("Invalid or revoked IC Token")
    })?;

  // Step 3: Check that agent has a provider key assigned
  let provider_key_id = agent.provider_key_id.ok_or(ProxyError::Forbidden(
    "No provider key assigned to this agent",
  ))?;

  // Step 4: Load encrypted key from DB and check if enabled
  let key_record = state
    .provider_key_storage
    .get_key(provider_key_id)
    .await
    .map_err(|e| ProxyError::Internal(format!("Failed to load provider key: {e}")))?;

  if !key_record.metadata.is_enabled {
    return Err(ProxyError::Forbidden("Provider key is disabled"));
  }

  // Step 5: Extract request metadata and read body early (needed for cost estimation)
  let method = request.method().clone();
  let path = request.uri().path().to_string();
  let query = request
    .uri()
    .query()
    .map(|q| format!("?{q}"))
    .unwrap_or_default();

  let body = body::to_bytes(request.into_body(), MAX_REQUEST_BODY_BYTES)
    .await
    .map_err(|_| ProxyError::BadRequest("Request body too large or malformed"))?
    .to_vec();

  // Step 5b: Atomic budget reservation (pre-flight spending cap enforcement).
  // Estimate max cost from request body (model + max_tokens), then atomically
  // increment spending_used. If cap exceeded, reject BEFORE forwarding to LLM.
  // This eliminates the TOCTOU race of the previous check-then-forward-then-increment pattern.
  let estimated_cost = state
    .pricing_manager
    .estimate_max_cost(&body)
    .map_or(1_000_000, |c| i64::try_from(c).unwrap_or(i64::MAX));

  if let Err(e) = state
    .provider_key_storage
    .reserve_spending(provider_key_id, estimated_cost)
    .await
  {
    return match e {
      iron_token_manager::error::TokenError::SpendingCapExceeded => {
        Err(ProxyError::SpendingCapExceeded)
      }
      other => {
        tracing::error!(key_id = provider_key_id, "Spending reservation failed: {other}");
        Err(ProxyError::Internal(format!("Spending reservation failed: {other}")))
      }
    };
  }

  // Step 6: Decrypt provider API key (AES-256-GCM, stays in memory only)
  let encrypted =
    EncryptedSecret::from_base64(&key_record.encrypted_api_key, &key_record.encryption_nonce)
      .map_err(|e| ProxyError::Internal(format!("Key decode failed: {e}")))?;

  let decrypted = state
    .crypto_service
    .decrypt(&encrypted)
    .map_err(|e| ProxyError::Internal(format!("Key decryption failed: {e}")))?;

  let provider_key = ProviderKey {
    provider: key_record.metadata.provider.as_str().to_string(),
    api_key: SecretBox::new(Box::new(iron_secrets::ip_token::ProviderApiKey::from(decrypted))),
    base_url: key_record.metadata.base_url.clone(),
  };

  // Step 7: Prepare forward request (body and metadata extracted in Step 5)
  let forward_req = ForwardRequest {
    method,
    path,
    query,
    body,
  };

  // Step 8: Forward to LLM provider via iron_llm_core
  let forward_resp = iron_llm_core::forward_request(
    &state.http_client,
    &state.pricing_manager,
    &provider_key,
    forward_req,
  )
  .await
  .map_err(|e| ProxyError::BadGateway(format!("Forward failed: {e}")))?;

  // Step 9: Adjust spending to actual cost.
  // The pre-flight reservation (Step 5b) already incremented by estimated_cost.
  // Now correct the delta: release excess if actual < estimated, or add if actual > estimated.
  // For streaming responses cost_info is None — keep the conservative estimate
  // to avoid under-counting spending (which could allow spending cap bypass).
  if let Some(cost) = &forward_resp.cost_info {
    let actual_cost = i64::try_from(cost.cost_micros).unwrap_or(i64::MAX);
    if let Err(e) = state
      .provider_key_storage
      .adjust_spending(provider_key_id, estimated_cost, actual_cost)
      .await
    {
      tracing::error!(
        key_id = provider_key_id,
        estimated = estimated_cost,
        actual = actual_cost,
        "Failed to adjust spending after forward: {e}"
      );
    }
  }

  // Step 10: Return provider response to agent (no API key leaks)
  let content_type = forward_resp
    .headers
    .get(axum::http::header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("application/json")
    .to_owned();

  let response_body = match forward_resp.body {
    iron_llm_core::ForwardBody::Buffered(bytes) => Body::from(bytes),
    iron_llm_core::ForwardBody::Streaming(provider_resp) => {
      Body::from_stream(futures_util::stream::unfold(provider_resp, |mut resp| async move {
        match resp.chunk().await {
          Ok(Some(chunk)) => Some((Ok::<_, String>(chunk), resp)),
          _ => None,
        }
      }))
    }
  };

  Response::builder()
    .status(forward_resp.status.as_u16())
    .header("content-type", content_type)
    .body(response_body)
    .map_err(|e| ProxyError::Internal(format!("Response build failed: {e}")))
}
