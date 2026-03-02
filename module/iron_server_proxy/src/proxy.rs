//! LLM proxy request handler and error types.
//!
//! Implements the core request pipeline:
//! IC Token auth -> provider key lookup -> decrypt -> forward -> cost tracking.

use axum::{
  body::{self, Body},
  extract::{Request, State},
  http::header,
  response::{IntoResponse, Response},
  Json,
};
use secrecy::SecretBox;
use sha2::{Digest, Sha256};

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
pub async fn handle_proxy(State(state): State<AppState>, request: Request) -> Response {
  handle_proxy_inner(&state, request)
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
}

/// Inner proxy handler with `?` error propagation.
async fn handle_proxy_inner(state: &AppState, request: Request) -> Result<Response, ProxyError> {
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

  let ic_token = auth_header
    .or(x_api_key)
    .ok_or(ProxyError::Unauthorized("Missing IC Token"))?;

  // Step 2: SHA-256 hash lookup in agents table
  let token_hash = format!("{:x}", Sha256::digest(ic_token.as_bytes()));

  let agent = sqlx::query_as::<_, AgentRecord>(
    "SELECT id, name, provider_key_id FROM agents WHERE ic_token_hash = ?",
  )
  .bind(&token_hash)
  .fetch_optional(&state.db_pool)
  .await
  .map_err(ProxyError::Database)?
  .ok_or(ProxyError::Unauthorized("Invalid or revoked IC Token"))?;

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

  // Step 5: Check spending cap before making the request
  let cap_ok = state
    .provider_key_storage
    .check_spending_cap(provider_key_id)
    .await
    .map_err(|e| ProxyError::Internal(format!("Failed to check spending cap: {e}")))?;

  if !cap_ok {
    return Err(ProxyError::SpendingCapExceeded);
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
    api_key: SecretBox::new(Box::new(decrypted.to_string())),
    base_url: key_record.metadata.base_url.clone(),
  };

  // Step 7: Prepare forward request from incoming HTTP request
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

  // Step 9: Increment spending counter (best-effort, don't fail the response)
  if let Some(ref cost_info) = forward_resp.cost_info {
    let amount = i64::try_from(cost_info.cost_micros).unwrap_or(i64::MAX);
    if let Err(e) = state
      .provider_key_storage
      .increment_spending(provider_key_id, amount)
      .await
    {
      tracing::warn!(
        key_id = provider_key_id,
        cost_micros = cost_info.cost_micros,
        "Failed to increment spending: {e}"
      );
    }
  }

  // Step 10: Return provider response to agent (no API key leaks)
  Ok(
    Response::builder()
      .status(forward_resp.status.as_u16())
      .header("content-type", "application/json")
      .body(Body::from(forward_resp.body))
      .expect("Failed to build response"),
  )
}
