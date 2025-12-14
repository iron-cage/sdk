//! Agent Provider Key endpoint (Feature 014)
//!
//! Allows agents to retrieve their assigned provider API key using IC Token authentication.
//!
//! POST /api/v1/agents/provider-key

use crate::routes::budget::BudgetState;
use axum::
{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_secrets::crypto::EncryptedSecret;
use serde::{ Deserialize, Serialize };

/// Provider key request
#[ derive( Debug, Deserialize ) ]
pub struct GetProviderKeyRequest
{
  pub ic_token: String,
}

impl GetProviderKeyRequest
{
  /// Maximum IC Token length (JWT tokens can be long)
  const MAX_IC_TOKEN_LENGTH: usize = 2000;

  /// Validate request parameters
  pub fn validate( &self ) -> Result< (), String >
  {
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

    Ok( () )
  }
}

/// Provider key response
#[ derive( Debug, Serialize ) ]
pub struct GetProviderKeyResponse
{
  /// Decrypted provider API key
  pub provider_key: String,
  /// Provider type ("openai" or "anthropic")
  pub provider: String,
  /// Optional custom base URL
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub base_url: Option< String >,
}

/// POST /api/v1/agents/provider-key
///
/// Retrieve decrypted provider API key for agent identified by IC Token.
///
/// # Flow
///
/// 1. Validate request
/// 2. Verify IC Token -> extract agent_id
/// 3. Query agents table for provider_key_id
/// 4. If NULL -> 403 NO_PROVIDER_ASSIGNED
/// 5. Get ai_provider_keys record
/// 6. Check key is enabled
/// 7. Decrypt API key using CryptoService
/// 8. Log audit entry
/// 9. Return decrypted key
///
/// # Returns
///
/// - 200 OK with provider key
/// - 400 Bad Request if validation fails (INVALID_TOKEN)
/// - 401 Unauthorized if IC Token invalid (UNAUTHORIZED)
/// - 403 Forbidden if no provider assigned (NO_PROVIDER_ASSIGNED)
/// - 404 Not Found if provider key not found (PROVIDER_NOT_FOUND)
/// - 503 Service Unavailable if crypto not configured (CRYPTO_UNAVAILABLE)
pub async fn get_provider_key(
  State( state ): State< BudgetState >,
  Json( request ): Json< GetProviderKeyRequest >,
) -> impl IntoResponse
{
  // 1. Validate request
  if let Err( validation_error ) = request.validate()
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({
        "error": validation_error,
        "code": "INVALID_TOKEN"
      }) ),
    ).into_response();
  }

  // 2. Verify IC Token
  let claims = match state.ic_token_manager.verify_token( &request.ic_token )
  {
    Ok( claims ) => claims,
    Err( _ ) =>
    {
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({
          "error": "Invalid IC Token",
          "code": "UNAUTHORIZED"
        }) ),
      ).into_response();
    }
  };

  // 3. Parse agent_id from claims (format: agent_<id>)
  let agent_id: i64 = match claims.agent_id.strip_prefix( "agent_" )
  {
    Some( id_part ) => match id_part.parse()
    {
      Ok( id ) => id,
      Err( _ ) =>
      {
        return (
          StatusCode::BAD_REQUEST,
          Json( serde_json::json!({
            "error": "Invalid agent_id format in token",
            "code": "INVALID_TOKEN"
          }) ),
        ).into_response();
      }
    },
    None =>
    {
      return (
        StatusCode::BAD_REQUEST,
        Json( serde_json::json!({
          "error": "Invalid agent_id format in token",
          "code": "INVALID_TOKEN"
        }) ),
      ).into_response();
    }
  };

  // 4. Query agent's provider_key_id
  let provider_key_id: Option< i64 > = match sqlx::query_scalar(
    "SELECT provider_key_id FROM agents WHERE id = ?"
  )
  .bind( agent_id )
  .fetch_optional( &state.db_pool )
  .await
  {
    Ok( Some( id ) ) => id,
    Ok( None ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({
          "error": "Agent not found",
          "code": "INVALID_TOKEN"
        }) ),
      ).into_response();
    }
    Err( err ) =>
    {
      tracing::error!( "Database error fetching agent provider_key_id: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({
          "error": "Database error",
          "code": "INTERNAL_ERROR"
        }) ),
      ).into_response();
    }
  };

  // 5. Check if agent has provider assigned
  let provider_key_id = match provider_key_id
  {
    Some( id ) => id,
    None =>
    {
      return (
        StatusCode::FORBIDDEN,
        Json( serde_json::json!({
          "error": "Agent has no provider assigned",
          "code": "NO_PROVIDER_ASSIGNED"
        }) ),
      ).into_response();
    }
  };

  // 6. Get provider key record (includes encrypted data)
  let key_record = match state.provider_key_storage.get_key( provider_key_id ).await
  {
    Ok( record ) => record,
    Err( err ) =>
    {
      tracing::error!( "Failed to retrieve provider key {}: {}", provider_key_id, err );
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({
          "error": "Provider key not found",
          "code": "PROVIDER_NOT_FOUND"
        }) ),
      ).into_response();
    }
  };

  // 7. Check if key is enabled
  if !key_record.metadata.is_enabled
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({
        "error": "Provider key is disabled",
        "code": "NO_PROVIDER_ASSIGNED"
      }) ),
    ).into_response();
  }

  // 8. Get crypto service
  let crypto = match &state.crypto_service
  {
    Some( c ) => c,
    None =>
    {
      tracing::error!( "CryptoService not configured" );
      return (
        StatusCode::SERVICE_UNAVAILABLE,
        Json( serde_json::json!({
          "error": "Crypto service unavailable",
          "code": "CRYPTO_UNAVAILABLE"
        }) ),
      ).into_response();
    }
  };

  // 9. Reconstruct encrypted secret from base64
  let encrypted = match EncryptedSecret::from_base64(
    &key_record.encrypted_api_key,
    &key_record.encryption_nonce,
  )
  {
    Ok( e ) => e,
    Err( err ) =>
    {
      tracing::error!( "Failed to decode encrypted key: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({
          "error": "Internal server error",
          "code": "INTERNAL_ERROR"
        }) ),
      ).into_response();
    }
  };

  // 10. Decrypt the API key
  let decrypted = match crypto.decrypt( &encrypted )
  {
    Ok( d ) => d,
    Err( err ) =>
    {
      tracing::error!( "Failed to decrypt key: {}", err );
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({
          "error": "Decryption failed",
          "code": "CRYPTO_UNAVAILABLE"
        }) ),
      ).into_response();
    }
  };

  // 11. Log audit entry (fire and forget)
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "LOUD FAILURE: Time went backwards" )
    .as_millis() as i64;

  let changes = serde_json::json!({
    "agent_id": agent_id,
    "provider_key_id": provider_key_id,
  });

  if let Err( e ) = sqlx::query(
    "INSERT INTO audit_log ( entity_type, entity_id, action, actor_user_id, changes, logged_at ) \
     VALUES ( $1, $2, $3, $4, $5, $6 )"
  )
  .bind( "provider_key" )
  .bind( provider_key_id )
  .bind( "agent_key_fetched" )
  .bind( &claims.agent_id )
  .bind( changes.to_string() )
  .bind( now_ms )
  .execute( &state.db_pool )
  .await
  {
    tracing::warn!( "Audit log insert failed: {}", e );
  }

  // 12. Update last_used_at
  if let Err( e ) = state.provider_key_storage.update_last_used( provider_key_id ).await
  {
    tracing::warn!( "Failed to update last_used_at: {}", e );
  }

  // 13. Return response
  (
    StatusCode::OK,
    Json( GetProviderKeyResponse {
      provider_key: decrypted.to_string(),
      provider: key_record.metadata.provider.to_string(),
      base_url: key_record.metadata.base_url,
    } ),
  ).into_response()
}
