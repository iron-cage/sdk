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
  // 0. Rate limit check
  if !state.rate_limiter.check_rate_limit( &auth.user_id, auth.project_id.as_deref() )
  {
    return Err( (
      StatusCode::TOO_MANY_REQUESTS,
      Json( serde_json::json!({ "error": "Rate limit exceeded" }) ),
    ) );
  }

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
  let pool = state.token_storage.pool();
  let agent_id: Option< i64 > = sqlx::query_scalar(
    "SELECT agent_id FROM api_tokens WHERE id = ?"
  )
  .bind( auth.token_id )
  .fetch_one( pool )
  .await
  .map_err( |_| (
    StatusCode::INTERNAL_SERVER_ERROR,
    Json( serde_json::json!({ "error": "Failed to verify token type" }) ),
  ) )?;

  if agent_id.is_some()
  {
    return Err( (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({
        "error": "Agent tokens cannot use this endpoint",
        "details": "Agent credentials must be obtained through Protocol 005 (Budget Control). Use POST /api/budget/handshake with your IC Token.",
        "protocol": "005"
      }) ),
    ) );
  }

  // 2. Require project_id
  let project_id = auth.project_id.ok_or_else( || (
    StatusCode::BAD_REQUEST,
    Json( serde_json::json!({ "error": "Token not assigned to a project" }) ),
  ) )?;

  // 2. Get provider key ID assigned to project
  let provider_key_id = state.provider_storage
    .get_project_key( &project_id )
    .await
    .map_err( |_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to query project key assignment" }) ),
    ) )?
    .ok_or_else( || (
      StatusCode::NOT_FOUND,
      Json( serde_json::json!({ "error": "No provider key assigned to project" }) ),
    ) )?;

  // 3. Get full key record (includes encrypted data)
  let key_record = state.provider_storage
    .get_key( provider_key_id )
    .await
    .map_err( |_| (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to retrieve provider key" }) ),
    ) )?;

  // 4. Check if key is enabled
  if !key_record.metadata.is_enabled
  {
    return Err( (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Provider key is disabled" }) ),
    ) );
  }

  // 5. Reconstruct encrypted secret from base64
  let encrypted = EncryptedSecret::from_base64(
    &key_record.encrypted_api_key,
    &key_record.encryption_nonce,
  )
  .map_err( |e| {
    tracing::error!( "Failed to decode encrypted key: {}", e );
    (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Internal server error" }) ),
    )
  } )?;

  // 6. Decrypt the API key
  let decrypted = state.crypto
    .decrypt( &encrypted )
    .map_err( |e| {
      tracing::error!( "Failed to decrypt key: {}", e );
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Internal server error" }) ),
      )
    } )?;

  // 7. Log to audit_log
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
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
    tracing::warn!( "Audit log insert failed: {}", e );
  }

  // 8. Update last_used_at for the provider key
  if let Err( e ) = state.provider_storage.update_last_used( provider_key_id ).await
  {
    tracing::warn!( "Failed to update last_used_at: {}", e );
  }

  // 9. Return decrypted key
  Ok( Json( KeyResponse {
    provider: key_record.metadata.provider.to_string(),
    api_key: decrypted.to_string(),
    base_url: key_record.metadata.base_url,
  } ) )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn key_response_serializes_correctly()
  {
    let response = KeyResponse {
      provider: "openai".to_string(),
      api_key: "sk-proj-xxx".to_string(),
      base_url: None,
    };

    let json = serde_json::to_string( &response ).unwrap();
    assert!( json.contains( "openai" ) );
    assert!( json.contains( "sk-proj-xxx" ) );
    assert!( !json.contains( "base_url" ) ); // Skipped when None
  }

  #[ test ]
  fn key_response_with_base_url()
  {
    let response = KeyResponse {
      provider: "anthropic".to_string(),
      api_key: "sk-ant-xxx".to_string(),
      base_url: Some( "https://custom.api.com".to_string() ),
    };

    let json = serde_json::to_string( &response ).unwrap();
    assert!( json.contains( "base_url" ) );
    assert!( json.contains( "https://custom.api.com" ) );
  }
}
