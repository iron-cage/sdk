//! Key Fetch API endpoints
//!
//! Allows users to fetch decrypted AI provider keys using their API tokens.
//! Keys are fetched based on the project assignment of the token.

use axum::{
  extract::State,
  http::StatusCode,
  Json,
};
use serde::{ Serialize, Deserialize };
use std::sync::Arc;

use iron_token_manager::storage::TokenStorage;
use iron_token_manager::provider_key_storage::ProviderKeyStorage;
use iron_token_manager::rate_limiter::RateLimiter;
use iron_secrets::crypto::{ CryptoService, EncryptedSecret };

use crate::token_auth::{ ApiTokenAuth, ApiTokenState };

/// State for key fetch endpoints
#[ derive( Clone ) ]
pub struct KeysState
{
  /// Token storage for authentication
  pub token_storage: Arc< TokenStorage >,
  /// Provider key storage
  pub provider_storage: Arc< ProviderKeyStorage >,
  /// Crypto service for decryption
  pub crypto: Arc< CryptoService >,
  /// Rate limiter for key fetch endpoint
  pub rate_limiter: RateLimiter,
}

impl std::fmt::Debug for KeysState
{
  fn fmt( &self, f: &mut std::fmt::Formatter< '_ > ) -> std::fmt::Result
  {
    f.debug_struct( "KeysState" )
      .field( "token_storage", &"<TokenStorage>" )
      .field( "provider_storage", &"<ProviderKeyStorage>" )
      .field( "crypto", &"<CryptoService>" )
      .field( "rate_limiter", &self.rate_limiter )
      .finish()
  }
}

/// Allow ApiTokenState to be extracted from KeysState
impl axum::extract::FromRef< KeysState > for ApiTokenState
{
  fn from_ref( state: &KeysState ) -> Self
  {
    ApiTokenState
    {
      token_storage: state.token_storage.clone(),
    }
  }
}

/// Response for GET /api/keys
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct KeyResponse
{
  /// Provider type ("openai" or "anthropic")
  pub provider: String,
  /// Decrypted API key (full key, not masked)
  pub api_key: String,
  /// Optional custom base URL
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub base_url: Option< String >,
}

/// GET /api/keys
///
/// Fetch the decrypted AI provider key assigned to the token's project.
///
/// # Authentication
///
/// Requires API token authentication via `Authorization: Bearer <token>` header.
///
/// # Returns
///
/// - 200: Provider key with decrypted API key
/// - 400: Token not assigned to a project
/// - 401: Invalid or missing token
/// - 404: No provider key assigned to project
/// - 429: Rate limit exceeded
/// - 500: Decryption failed
pub async fn get_key(
  auth: ApiTokenAuth,
  State( state ): State< KeysState >,
) -> Result< Json< KeyResponse >, ( StatusCode, Json< serde_json::Value > ) >
{
  println!( "[GET /api/keys] Request started - user_id: {}, token_id: {}, project_id: {:?}",
    auth.user_id, auth.token_id, auth.project_id );

  // 0. Rate limit check
  println!( "[GET /api/keys] Checking rate limit for user_id: {}, project_id: {:?}", auth.user_id, auth.project_id );
  if !state.rate_limiter.check_rate_limit( &auth.user_id, auth.project_id.as_deref() )
  {
    println!( "[GET /api/keys] WARN: Rate limit exceeded for user_id: {}, project_id: {:?}", auth.user_id, auth.project_id );
    return Err( (
      StatusCode::TOO_MANY_REQUESTS,
      Json( serde_json::json!({ "error": "Rate limit exceeded" }) ),
    ) );
  }
  println!( "[GET /api/keys] Rate limit check passed for user_id: {}", auth.user_id );

  // 1. Enforce Protocol 005: Agent tokens CANNOT use this endpoint
  //
  // This endpoint provides direct access to decrypted provider keys without
  // budget control. Protocol 005 (Budget Control Protocol) is the ONLY
  // authorized path for agent credential access. Any token associated with
  // an agent MUST use the budget handshake flow instead.
  //
  // This enforcement ensures:
  // - All agent LLM access is budget-controlled
  // - Usage tracking is mandatory
  // - No bypass path exists for budget limits
  println!( "[GET /api/keys] Verifying token type for token_id: {}", auth.token_id );
  let pool = state.token_storage.pool();
  let agent_id: Option< i64 > = sqlx::query_scalar(
    "SELECT agent_id FROM api_tokens WHERE id = ?"
  )
  .bind( auth.token_id )
  .fetch_one( pool )
  .await
  .map_err( |e| {
    println!( "[GET /api/keys] ERROR: Failed to verify token type for token_id {}: {}", auth.token_id, e );
    tracing::error!( "Failed to verify token type for token_id {}: {}", auth.token_id, e );
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to verify token type" }) ),
    )
  } )?;

  if agent_id.is_some()
  {
    println!( "[GET /api/keys] WARN: Attempted to use agent token (token_id: {}, agent_id: {:?}) - Protocol 005 enforcement triggered", auth.token_id, agent_id );
    return Err( (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({
        "error": "Agent tokens cannot use this endpoint",
        "details": "Agent credentials must be obtained through Protocol 005 (Budget Control). Use POST /api/budget/handshake with your IC Token.",
        "protocol": "005"
      }) ),
    ) );
  }
  println!( "[GET /api/keys] Token type verification passed - not an agent token (token_id: {})", auth.token_id );

  // 2. Require project_id
  let project_id = auth.project_id.ok_or_else( || {
    println!( "[GET /api/keys] WARN: Token not assigned to a project - token_id: {}, user_id: {}", auth.token_id, auth.user_id );
    (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({ "error": "Token not assigned to a project" }) ),
    )
  } )?;
  println!( "[GET /api/keys] Project ID verified: {}", project_id );

  // 2. Get provider key ID assigned to project
  println!( "[GET /api/keys] Fetching provider key assignment for project_id: {}", project_id );
  let provider_key_id = state.provider_storage
    .get_project_key( &project_id )
    .await
    .map_err( |e| {
      println!( "[GET /api/keys] ERROR: Failed to query project key assignment for project {}: {}", project_id, e );
      tracing::error!( "Failed to query project key assignment for project {}: {}", project_id, e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to query project key assignment" }) ),
      )
    } )?
    .ok_or_else( || {
      println!( "[GET /api/keys] WARN: No provider key assigned to project_id: {}", project_id );
      (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "No provider key assigned to project" }) ),
      )
    } )?;
  println!( "[GET /api/keys] Provider key assignment found - project_id: {}, provider_key_id: {}", project_id, provider_key_id );

  // 3. Get full key record (includes encrypted data)
  println!( "[GET /api/keys] Retrieving full provider key record - provider_key_id: {}", provider_key_id );
  let key_record = state.provider_storage
    .get_key( provider_key_id )
    .await
    .map_err( |e| {
      println!( "[GET /api/keys] ERROR: Failed to retrieve provider key {}: {}", provider_key_id, e );
      tracing::error!( "Failed to retrieve provider key {}: {}", provider_key_id, e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to retrieve provider key" }) ),
      )
    } )?;
  println!( "[GET /api/keys] Provider key record retrieved - provider_key_id: {}, provider: {}, is_enabled: {}",
    provider_key_id, key_record.metadata.provider, key_record.metadata.is_enabled );

  // 4. Check if key is enabled
  if !key_record.metadata.is_enabled
  {
    println!( "[GET /api/keys] WARN: Attempted to use disabled provider key - provider_key_id: {}, project_id: {}, user_id: {}",
      provider_key_id, project_id, auth.user_id );
    return Err( (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Provider key is disabled" }) ),
    ) );
  }
  println!( "[GET /api/keys] Provider key is enabled and ready for use - provider_key_id: {}", provider_key_id );

  // 5. Reconstruct encrypted secret from base64
  println!( "[GET /api/keys] Decoding encrypted secret from base64 - provider_key_id: {}", provider_key_id );
  let encrypted = EncryptedSecret::from_base64(
    &key_record.encrypted_api_key,
    &key_record.encryption_nonce,
  )
  .map_err( |e| {
    println!( "[GET /api/keys] ERROR: Failed to decode encrypted key for provider_key_id {}: {}", provider_key_id, e );
    tracing::error!( "Failed to decode encrypted key: {}", e );
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Internal server error" }) ),
    )
  } )?;
  println!( "[GET /api/keys] Encrypted secret decoded successfully - provider_key_id: {}", provider_key_id );

  // 6. Decrypt the API key
  println!( "[GET /api/keys] Decrypting API key - provider_key_id: {}, provider: {}", provider_key_id, key_record.metadata.provider );
  let decrypted = state.crypto
    .decrypt( &encrypted )
    .map_err( |e| {
      println!( "[GET /api/keys] ERROR: Failed to decrypt key for provider_key_id {}: {}", provider_key_id, e );
      tracing::error!( "Failed to decrypt key: {}", e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Internal server error" }) ),
      )
    } )?;
  println!( "[GET /api/keys] API key decrypted successfully - provider_key_id: {}, provider: {}", provider_key_id, key_record.metadata.provider );

  // 7. Log to audit_log
  println!( "[GET /api/keys] Creating audit log entry - provider_key_id: {}, user_id: {}, project_id: {}",
    provider_key_id, auth.user_id, project_id );
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "LOUD FAILURE: Time went backwards" )
    .as_millis() as i64;

  let changes = serde_json::json!({
    "token_id": auth.token_id,
    "project_id": project_id,
  });

  // Insert audit log entry (fire and forget - don't fail request if logging fails)
  let pool = state.token_storage.pool();
  if let Err( e ) = sqlx::query(
    "INSERT INTO audit_log ( entity_type, entity_id, action, actor_user_id, changes, logged_at ) \
     VALUES ( $1, $2, $3, $4, $5, $6 )"
  )
  .bind( "provider_key" )
  .bind( provider_key_id )
  .bind( "key_fetched" )
  .bind( &auth.user_id )
  .bind( changes.to_string() )
  .bind( now_ms )
  .execute( pool )
  .await
  {
    println!( "[GET /api/keys] WARN: Audit log insert failed for provider_key_id {}, user_id {}: {}", provider_key_id, auth.user_id, e );
    tracing::warn!( "Audit log insert failed: {}", e );
  }
  println!( "[GET /api/keys] Audit log entry created - action: key_fetched, provider_key_id: {}, user_id: {}", provider_key_id, auth.user_id );

  // 8. Update last_used_at for the provider key
  println!( "[GET /api/keys] Updating last_used_at timestamp - provider_key_id: {}", provider_key_id );
  if let Err( e ) = state.provider_storage.update_last_used( provider_key_id ).await
  {
    println!( "[GET /api/keys] WARN: Failed to update last_used_at for provider_key_id {}: {}", provider_key_id, e );
    tracing::warn!( "Failed to update last_used_at: {}", e );
  }
  println!( "[GET /api/keys] Last_used_at updated successfully - provider_key_id: {}", provider_key_id );

  // 9. Return decrypted key
  println!( "[GET /api/keys] Request completed successfully - user_id: {}, project_id: {}, provider: {}, provider_key_id: {}",
    auth.user_id, project_id, key_record.metadata.provider, provider_key_id );
  Ok( Json( KeyResponse {
    provider: key_record.metadata.provider.to_string(),
    api_key: decrypted.to_string(),
    base_url: key_record.metadata.base_url,
  } ) )
}

