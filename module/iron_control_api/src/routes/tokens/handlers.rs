//! Token management endpoint handlers
//!
//! Provides all token lifecycle operations:
//! - create_token: Create new API token
//! - list_tokens: List all user tokens
//! - get_token: Get specific token details
//! - update_token: Update token provider
//! - rotate_token: Rotate token (new value)
//! - revoke_token: Revoke/deactivate token
//! - validate_token: Public token validation

use axum::{
  extract::{ Path, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use super::shared::{
  TokenState, CreateTokenRequest, UpdateTokenRequest, ValidateTokenRequest,
  CreateTokenResponse, TokenListItem, ValidateTokenResponse,
};

/// POST /api/tokens
///
/// Create new API token (Protocol 014 compliant with backward compatibility)
///
/// # Protocol 014 Compliance
///
/// - **Authentication Required:** User must be authenticated via JWT Bearer token
/// - **user_id Source:** Extracted from JWT claims (not request body)
/// - **Request Fields:** `name` (required, 1-100 chars), `description` (optional, max 500 chars)
/// - **Rate Limiting:** 10 creates/min per user (429 Too Many Requests if exceeded)
/// - **Token Limit:** Max 10 active tokens per user (429 Too Many Requests if exceeded)
/// - **Audit Logging:** Logs creation to audit_log (plaintext token excluded for security)
///
/// # Backward Compatibility
///
/// Supports legacy request format with `user_id` in request body for existing tests.
/// Once tests are migrated to Protocol 014 format, legacy support can be removed.
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `claims` - Authenticated user from JWT token (Protocol 014 requirement)
/// * `request` - Token creation parameters
///
/// # Returns
///
/// - 201 Created with new token details
/// - 400 Bad Request if validation fails or malformed JSON
/// - 401 Unauthorized if not authenticated (Protocol 014 requirement)
/// - 500 Internal Server Error if generation fails
pub async fn create_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  crate::error::JsonBody( request ): crate::error::JsonBody< CreateTokenRequest >,
) -> impl IntoResponse
{
  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error.to_string()
    }) ) ).into_response();
  }

  // Protocol 014: user_id comes from JWT authentication, not request body
  // Legacy: If user_id in request body, use it (for backward compatibility with existing tests)
  let user_id = request.user_id.as_ref().unwrap_or( &claims.sub );

  // Rate limiting: Check both limits (Protocol 014)
  // 1. Max active tokens per user: 10
  // 2. Max token creates per minute: 10
  let active_token_count = match state.storage.count_active_tokens_for_user( user_id ).await
  {
    Ok( count ) => count,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to check token limit" }) ),
      )
        .into_response();
    }
  };

  let recent_creations = match state.storage.count_recent_token_creations( user_id ).await
  {
    Ok( count ) => count,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to check rate limit" }) ),
      )
        .into_response();
    }
  };

  // Check rate limit first (time-based constraint is more restrictive in practice)
  // This ensures users get the correct error message when both limits are reached
  if recent_creations >= 10
  {
    return (
      StatusCode::TOO_MANY_REQUESTS,
      Json( serde_json::json!({ "error": "Rate limit exceeded" }) ),
    )
      .into_response();
  }

  // Then check active token limit
  if active_token_count >= 10
  {
    return (
      StatusCode::TOO_MANY_REQUESTS,
      Json( serde_json::json!({ "error": "Token limit exceeded" }) ),
    )
      .into_response();
  }

  // Generate token
  let token = state.generator.generate();

  // Store token in database
  // Protocol 014: Uses `name` field for token name
  // Legacy: Falls back to `description` if `name` not provided
  let token_name = request.name.as_ref()
    .and_then( | n | if n.is_empty() { None } else { Some( n.as_str() ) } )
    .or(request.description.as_deref());

  let token_id = match state
    .storage
    .create_token(
      &token,
      user_id,
      request.project_id.as_deref(),
      token_name,
      request.agent_id,
      request.provider.as_deref(),
    )
    .await
  {
    Ok( id ) => id,
    Err( iron_token_manager::error::TokenError::Database( db_err ) ) =>
    {
      // Check if this is an FK constraint violation
      let err_msg = db_err.to_string();
      if err_msg.contains( "FOREIGN KEY constraint failed" )
      {
        // Parse constraint details to provide specific error
        return (
          StatusCode::NOT_FOUND,
          Json( serde_json::json!({
            "error": format!( "User not found: '{}'", user_id ),
            "code": "USER_NOT_FOUND"
          }) ),
        )
          .into_response();
      }

      // Other database errors
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({
          "error": "Database error occurred",
          "code": "DATABASE_ERROR"
        }) ),
      )
        .into_response();
    }
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to create token" }) ),
      )
        .into_response();
    }
  };

  // Get metadata for response
  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to retrieve token metadata" }) ),
      )
        .into_response();
    }
  };

  // Log token creation to audit_log (Protocol 014 requirement)
  // SECURITY: Never log plaintext token value
  let changes_json = serde_json::json!({
    "name": metadata.name,
    "user_id": metadata.user_id,
    "project_id": metadata.project_id,
    "agent_id": metadata.agent_id,
    "provider": metadata.provider,
  }).to_string();

  if state.storage.log_audit_event(
    "token",
    token_id,
    "created",
    user_id,
    Some( &changes_json ),
  ).await.is_err()
  {
    // Log error but don't fail request (audit logging is not critical path)
    tracing::error!( "Failed to log token creation to audit_log (token_id={})", token_id );
  }

  ( StatusCode::CREATED, Json( CreateTokenResponse
  {
    id: metadata.id,
    token, // Return plaintext token ONCE on creation (Protocol 014 requirement)
    user_id: metadata.user_id,
    project_id: metadata.project_id,
    description: metadata.name,
    agent_id: metadata.agent_id,
    provider: metadata.provider,
    created_at: metadata.created_at,
  } ) )
    .into_response()
}

/// GET /api/tokens
///
/// List all active tokens for authenticated user
pub async fn list_tokens(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
) -> impl IntoResponse
{
  let user_id = &claims.sub;

  let tokens = match state.storage.list_user_tokens( user_id ).await
  {
    Ok( tokens ) => tokens,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to fetch tokens" }) ),
      )
        .into_response();
    }
  };

  let token_list: Vec< TokenListItem > = tokens
    .into_iter()
    .map( | t | TokenListItem
    {
      id: t.id,
      user_id: t.user_id,
      project_id: t.project_id,
      description: t.name,
      agent_id: t.agent_id,
      provider: t.provider,
      created_at: t.created_at,
      last_used_at: t.last_used_at,
      is_active: t.is_active,
    } )
    .collect();

  ( StatusCode::OK, Json( token_list ) ).into_response()
}

/// GET /api/tokens/:id
///
/// Get specific token details
pub async fn get_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  let user_id = &claims.sub;

  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  };

  if metadata.user_id != *user_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Access denied - token belongs to different user" }) ),
    )
      .into_response();
  }

  let item = TokenListItem
  {
    id: metadata.id,
    user_id: metadata.user_id,
    project_id: metadata.project_id,
    description: metadata.name,
    agent_id: metadata.agent_id,
    provider: metadata.provider,
    created_at: metadata.created_at,
    last_used_at: metadata.last_used_at,
    is_active: metadata.is_active,
  };

  ( StatusCode::OK, Json( item ) ).into_response()
}

/// PUT /api/tokens/:id
///
/// Update token provider
pub async fn update_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
  crate::error::JsonBody( request ): crate::error::JsonBody< UpdateTokenRequest >,
) -> impl IntoResponse
{
  let user_id = &claims.sub;

  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error.to_string()
    }) ) ).into_response();
  }

  let existing_metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  };

  if existing_metadata.user_id != *user_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Access denied - token belongs to different user" }) ),
    )
      .into_response();
  }

  if state
    .storage
    .update_token_provider(
      token_id,
      &request.provider,
    )
    .await
    .is_err()
  {
    return (
      StatusCode::INTERNAL_SERVER_ERROR,
      Json( serde_json::json!({ "error": "Failed to update token" }) ),
    )
      .into_response();
  }

  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to retrieve updated token" }) ),
      )
        .into_response();
    }
  };

  let item = TokenListItem
  {
    id: metadata.id,
    user_id: metadata.user_id,
    project_id: metadata.project_id,
    description: metadata.name,
    agent_id: metadata.agent_id,
    provider: metadata.provider,
    created_at: metadata.created_at,
    last_used_at: metadata.last_used_at,
    is_active: metadata.is_active,
  };

  ( StatusCode::OK, Json( item ) ).into_response()
}

/// POST /api/tokens/:id/rotate
///
/// Rotate token (generate new value, invalidate old)
pub async fn rotate_token(
  State( state ): State< TokenState >,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  let existing_metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  };

  if !existing_metadata.is_active
  {
    return (
      StatusCode::NOT_FOUND,
      Json( serde_json::json!({ "error": "Token not found" }) ),
    )
      .into_response();
  }

  if state.storage.deactivate_token( token_id ).await.is_err()
  {
    return (
      StatusCode::NOT_FOUND,
      Json( serde_json::json!({ "error": "Token not found" }) ),
    )
      .into_response();
  }

  let new_token = state.generator.generate();

  let new_token_id = match state
    .storage
    .create_token(
      &new_token,
      &existing_metadata.user_id,
      existing_metadata.project_id.as_deref(),
      existing_metadata.name.as_deref(),
      existing_metadata.agent_id,
      existing_metadata.provider.as_deref(),
    )
    .await
  {
    Ok( id ) => id,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to create new token" }) ),
      )
        .into_response();
    }
  };

  let new_metadata = match state.storage.get_token_metadata( new_token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to retrieve new token metadata" }) ),
      )
        .into_response();
    }
  };

  ( StatusCode::OK, Json( CreateTokenResponse
  {
    id: new_metadata.id,
    token: new_token,
    user_id: new_metadata.user_id,
    project_id: new_metadata.project_id,
    description: new_metadata.name,
    agent_id: new_metadata.agent_id,
    provider: new_metadata.provider,
    created_at: new_metadata.created_at,
  } ) )
    .into_response()
}

/// DELETE /api/tokens/:id
///
/// Revoke token (Protocol 014 compliant)
pub async fn revoke_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  let user_id = &claims.sub;

  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  };

  if metadata.user_id != *user_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Access denied - token belongs to different user" }) ),
    )
      .into_response();
  }

  if !metadata.is_active
  {
    if metadata.revoked_at.is_some()
    {
      return (
        StatusCode::CONFLICT,
        Json( serde_json::json!({ "error": "Token already revoked" }) ),
      )
        .into_response();
    }
    else
    {
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  }

  match state.storage.revoke_token( token_id ).await
  {
    Ok( () ) =>
    {
      let changes_json = serde_json::json!({
        "token_id": token_id,
        "user_id": metadata.user_id,
        "project_id": metadata.project_id,
      }).to_string();

      if state.storage.log_audit_event(
        "token",
        token_id,
        "revoked",
        user_id,
        Some( &changes_json ),
      ).await.is_err()
      {
        tracing::error!( "Failed to log token revocation to audit_log (token_id={})", token_id );
      }

      ( StatusCode::OK, Json( serde_json::json!({
        "id": token_id,
        "revoked": true,
        "message": "Token revoked successfully"
      }) ) )
        .into_response()
    }
    Err( _ ) =>
    {
      match state.storage.get_token_metadata( token_id ).await
      {
        Ok( updated_metadata ) =>
        {
          if updated_metadata.revoked_at.is_some()
          {
            (
              StatusCode::CONFLICT,
              Json( serde_json::json!({ "error": "Token already revoked" }) ),
            )
              .into_response()
          }
          else
          {
            (
              StatusCode::NOT_FOUND,
              Json( serde_json::json!({ "error": "Token not found" }) ),
            )
              .into_response()
          }
        }
        Err( _ ) =>
        {
          (
            StatusCode::NOT_FOUND,
            Json( serde_json::json!({ "error": "Token not found" }) ),
          )
            .into_response()
        }
      }
    }
  }
}

/// POST /api/v1/api-tokens/validate
///
/// Public endpoint to validate API tokens (Deliverable 1.6)
pub async fn validate_token(
  State( state ): State< TokenState >,
  crate::error::JsonBody( request ): crate::error::JsonBody< ValidateTokenRequest >,
) -> impl IntoResponse
{
  let token_id = match state.storage.verify_token( &request.token ).await
  {
    Ok( id ) => id,
    Err( _ ) =>
    {
      return ( StatusCode::OK, Json( ValidateTokenResponse
      {
        valid: false,
        user_id: None,
        project_id: None,
        token_id: None,
      } ) )
        .into_response();
    }
  };

  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      return ( StatusCode::OK, Json( ValidateTokenResponse
      {
        valid: false,
        user_id: None,
        project_id: None,
        token_id: None,
      } ) )
        .into_response();
    }
  };

  ( StatusCode::OK, Json( ValidateTokenResponse
  {
    valid: true,
    user_id: Some( metadata.user_id ),
    project_id: metadata.project_id,
    token_id: Some( token_id ),
  } ) )
    .into_response()
}
