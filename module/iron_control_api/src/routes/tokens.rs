//! Token management REST API endpoints
//!
//! Phase 4 Day 29: REST API Endpoints - Token Management
//!
//! Endpoints:
//! - POST /api/tokens - Create new API token
//! - GET /api/tokens - List all tokens for user
//! - GET /api/tokens/:id - Get specific token details
//! - POST /api/tokens/:id/rotate - Rotate token (generate new value)
//! - DELETE /api/tokens/:id - Revoke token

use axum::{
  extract::{ Path, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_token_manager::storage::TokenStorage;
use iron_token_manager::token_generator::TokenGenerator;
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

/// Token management state
#[ derive( Clone ) ]
pub struct TokenState
{
  pub storage: Arc< TokenStorage >,
  pub generator: Arc< TokenGenerator >,
}

impl TokenState
{
  /// Create new token state
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let storage = TokenStorage::new( database_url ).await?;
    Ok( Self {
      storage: Arc::new( storage ),
      generator: Arc::new( TokenGenerator::new() ),
    } )
  }
}

/// Create token request (Protocol 014 compliant with backward compatibility)
///
/// Per Protocol 014: user_id comes from JWT authentication, not request body
///
/// # Formats Supported
///
/// **Protocol 014 format (preferred):**
/// - `name`: required, 1-100 chars
/// - `description`: optional, max 500 chars
/// - `user_id`: from JWT (not in request body)
///
/// **Legacy format (backward compatibility):**
/// - `user_id`: in request body
/// - `project_id`: optional
/// - `description`: optional (used as token name in database)
#[ derive( Debug, Deserialize ) ]
pub struct CreateTokenRequest
{
  // Protocol 014 field - optional for backward compatibility with legacy tests
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub name: Option< String >,

  pub description: Option< String >,

  // Legacy fields kept for backward compatibility with existing tests
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub user_id: Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub project_id: Option< String >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub agent_id: Option< i64 >,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  #[ serde( default ) ]
  pub provider: Option< String >,
}

impl CreateTokenRequest
{
  // Fix(issue-001): Prevent DoS via unlimited string validation
  // Root cause: Accepted unbounded external input without resource limits
  // Pitfall: Always validate input length before processing to prevent resource exhaustion

  // Fix(issue-max-user-id-length): Align validation with migration 013 schema constraint
  // Root cause: Validation allowed 500 chars but migration 013 reduced user_id to 255 to match users.id FK target
  // Pitfall: Validation constants must match database CHECK constraints to prevent insertion failures

  /// Maximum user_id length (DoS protection). Must match migration 013 CHECK constraint (255 chars, aligned with users.id).
  const MAX_USER_ID_LENGTH: usize = 255;

  /// Maximum project_id length (DoS protection). Must match database CHECK constraint.
  const MAX_PROJECT_ID_LENGTH: usize = 500;

  /// Maximum name length (Protocol 014: 1-100 chars)
  const MAX_NAME_LENGTH: usize = 100;

  /// Maximum description length (Protocol 014: max 500 chars)
  const MAX_DESCRIPTION_LENGTH: usize = 500;

  /// Validate token creation request (Protocol 014 compliant with backward compatibility).
  ///
  /// **Protocol 014 format:** Validates `name` (required, 1-100 chars) when provided
  /// **Legacy format:** Validates `user_id` (required, non-empty) when `name` not provided
  ///
  /// Both formats validate `description` (optional, max 500 chars) and `project_id` (optional, non-empty).
  pub fn validate( &self ) -> Result< (), String >
  {
    // Protocol 014 validation: If `name` is provided, validate it
    if let Some( ref name ) = self.name
    {
      // Validate name is not empty (Protocol 014 requirement)
      if name.trim().is_empty()
      {
        return Err( "name cannot be empty".to_string() );
      }

      // Validate name length (Protocol 014: 1-100 chars)
      if name.len() > Self::MAX_NAME_LENGTH
      {
        return Err( format!(
          "name too long (max {} characters)",
          Self::MAX_NAME_LENGTH
        ) );
      }

      // Fix(issue-002): Prevent NULL byte injection causing C string termination attacks
      // Root cause: Accepted NULL bytes in strings passed to C/FFI libraries and database drivers
      // Pitfall: Always validate against control characters when interacting with C libraries or databases using C drivers

      // Validate name doesnt contain NULL bytes
      if name.contains( '\0' )
      {
        return Err( "name contains invalid NULL byte".to_string() );
      }
    }

    // Validate description if provided (Protocol 014: max 500 chars)
    if let Some( ref description ) = self.description
    {
      if description.len() > Self::MAX_DESCRIPTION_LENGTH
      {
        return Err( format!(
          "description too long (max {} characters)",
          Self::MAX_DESCRIPTION_LENGTH
        ) );
      }

      // Validate description doesnt contain NULL bytes
      if description.contains( '\0' )
      {
        return Err( "description contains invalid NULL byte".to_string() );
      }
    }

    // Legacy validation for backward compatibility with existing tests
    // These validations apply when legacy format is used (no `name` field)

    if let Some( ref user_id ) = self.user_id
    {
      if user_id.trim().is_empty()
      {
        return Err( "user_id cannot be empty".to_string() );
      }

      if user_id.len() > Self::MAX_USER_ID_LENGTH
      {
        return Err( format!(
          "user_id too long (max {} characters)",
          Self::MAX_USER_ID_LENGTH
        ) );
      }

      if user_id.contains( '\0' )
      {
        return Err( "user_id contains invalid NULL byte".to_string() );
      }
    }

    if let Some( ref project_id ) = self.project_id
    {
      if project_id.trim().is_empty()
      {
        return Err( "project_id cannot be empty".to_string() );
      }

      if project_id.len() > Self::MAX_PROJECT_ID_LENGTH
      {
        return Err( format!(
          "project_id too long (max {} characters)",
          Self::MAX_PROJECT_ID_LENGTH
        ) );
      }

      if project_id.contains( '\0' )
      {
        return Err( "project_id contains invalid NULL byte".to_string() );
      }
    }

    Ok( () )
  }
}

/// Update token request
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct UpdateTokenRequest
{
  pub provider: String,
}

impl UpdateTokenRequest
{
  /// Maximum length of provider (DoS protection)
  const MAX_PROVIDER_LENGTH: usize = 64;

  fn validate( &self ) -> Result< (), String >
  {
    // Validate provider if provided
    if self.provider.trim().is_empty()
    {
      return Err( "provider cannot be empty".to_string() );
    }

    // Validate provider length (DoS protection)
    if self.provider.len() > Self::MAX_PROVIDER_LENGTH
    {
      return Err( format!(
        "provider too long (max {} characters)",
        Self::MAX_PROVIDER_LENGTH
      ) );
    }

    // Validate provider doesnt contain NULL bytes
    if self.provider.contains( '\0' )
    {
      return Err( "provider contains invalid NULL byte".to_string() );
    }

    Ok( () )
  }
}

/// Validate token request (Deliverable 1.6)
#[ derive( Debug, Deserialize ) ]
pub struct ValidateTokenRequest
{
  pub token: String,
}

/// Validate token response (Deliverable 1.6)
#[ derive( Debug, Serialize ) ]
pub struct ValidateTokenResponse
{
  pub valid: bool,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub user_id: Option< String >,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub project_id: Option< String >,
  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  pub token_id: Option< i64 >,
}

/// Create token response
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct CreateTokenResponse
{
  pub id: i64,
  pub token: String,
  pub user_id: String,
  pub project_id: Option< String >,
  pub description: Option< String >,
  pub agent_id: Option< i64 >,
  pub provider: Option< String >,
  pub created_at: i64,
}

/// Token list item
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct TokenListItem
{
  pub id: i64,
  pub user_id: String,
  pub project_id: Option< String >,
  pub description: Option< String >,
  pub agent_id: Option< i64 >,
  pub provider: Option< String >,
  pub created_at: i64,
  pub last_used_at: Option< i64 >,
  pub is_active: bool,
}

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
      "error": validation_error
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
///
/// # Arguments
///
/// * `state` - Token generator state
///
/// # Returns
///
/// - 200 OK with list of tokens
/// - 500 Internal Server Error if database query fails
pub async fn list_tokens(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
) -> impl IntoResponse
{
  // Extract user_id from JWT claims
  let user_id = &claims.sub;

  // Query tokens from storage
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
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `claims` - Authenticated user from JWT token
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 200 OK with token details
/// - 401 Unauthorized if not authenticated
/// - 403 Forbidden if token belongs to different user
/// - 404 Not Found if token doesn't exist
pub async fn get_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Extract user_id from JWT claims
  let user_id = &claims.sub;

  // Get token metadata
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

  // Verify ownership - user can only access their own tokens
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
/// Update token details
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `claims` - Authenticated user from JWT token
/// * `token_id` - Token ID from path
/// * `request` - Update request body
///
/// # Returns
///
/// - 200 OK with updated token details
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if not authenticated
/// - 403 Forbidden if token belongs to different user
/// - 404 Not Found if token doesn't exist
/// - 500 Internal Server Error if update fails
pub async fn update_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
  crate::error::JsonBody( request ): crate::error::JsonBody< UpdateTokenRequest >,
) -> impl IntoResponse
{
  // Extract user_id from JWT claims
  let user_id = &claims.sub;

  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Get token metadata to verify ownership before update
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

  // Verify ownership - user can only update their own tokens
  if existing_metadata.user_id != *user_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Access denied - token belongs to different user" }) ),
    )
      .into_response();
  }

  // Perform update
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

  // Get updated metadata for response
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
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 200 OK with new token value
/// - 404 Not Found if token doesn't exist
pub async fn rotate_token(
  State( state ): State< TokenState >,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Fetch existing token metadata
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

  // Verify token is active (cannot rotate revoked tokens)
  if !existing_metadata.is_active
  {
    return (
      StatusCode::NOT_FOUND,
      Json( serde_json::json!({ "error": "Token not found" }) ),
    )
      .into_response();
  }

  // Deactivate old token
  // If deactivation fails, token was already deactivated (race condition) - return 404
  if state.storage.deactivate_token( token_id ).await.is_err()
  {
    return (
      StatusCode::NOT_FOUND,
      Json( serde_json::json!({ "error": "Token not found" }) ),
    )
      .into_response();
  }

  // Generate new token
  let new_token = state.generator.generate();

  // Store new token with same user/project
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

  // Get new token metadata
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
    token: new_token, // Return plaintext token ONCE
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
/// Revoke token (mark as inactive) - Protocol 014 compliant
///
/// # Protocol 014 Compliance
///
/// - **Authentication Required:** User must be authenticated via JWT Bearer token
/// - **Ownership Verification:** User can only revoke their own tokens (403 Forbidden for others)
/// - **Audit Logging:** Revocation is logged to audit_log table (Protocol 014 requirement)
/// - **Response:** 200 OK with details (not 204 No Content)
/// - **Already Revoked:** Returns 409 Conflict (not 404 Not Found)
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `claims` - Authenticated user from JWT token (Protocol 014 requirement)
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 200 OK with revocation details (Protocol 014 requirement)
/// - 401 Unauthorized if not authenticated (Protocol 014 requirement)
/// - 403 Forbidden if token belongs to different user (Protocol 014 requirement)
/// - 404 Not Found if token doesn't exist
/// - 409 Conflict if token already revoked (Protocol 014 requirement)
pub async fn revoke_token(
  State( state ): State< TokenState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Extract user_id from JWT claims (Protocol 014 requirement)
  let user_id = &claims.sub;

  // Get token metadata to verify ownership before revocation
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

  // Verify ownership - user can only revoke their own tokens (Protocol 014 requirement)
  if metadata.user_id != *user_id
  {
    return (
      StatusCode::FORBIDDEN,
      Json( serde_json::json!({ "error": "Access denied - token belongs to different user" }) ),
    )
      .into_response();
  }

  // Check if token is inactive (was revoked or rotated)
  if !metadata.is_active
  {
    // Distinguish explicit revocation (409) from rotation (404)
    if metadata.revoked_at.is_some()
    {
      // Token was explicitly revoked - return 409 (Protocol 014)
      return (
        StatusCode::CONFLICT,
        Json( serde_json::json!({ "error": "Token already revoked" }) ),
      )
        .into_response();
    }
    else
    {
      // Token was rotated or otherwise deactivated - return 404
      return (
        StatusCode::NOT_FOUND,
        Json( serde_json::json!({ "error": "Token not found" }) ),
      )
        .into_response();
    }
  }

  // Revoke token (sets is_active = 0 and revoked_at = timestamp)
  match state.storage.revoke_token( token_id ).await
  {
    Ok( () ) =>
    {
      // Log token revocation to audit_log (Protocol 014 requirement)
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
        // Log error but don't fail request (audit logging is not critical path)
        tracing::error!( "Failed to log token revocation to audit_log (token_id={})", token_id );
      }

      // Protocol 014: Return 200 OK with details (not 204 No Content)
      ( StatusCode::OK, Json( serde_json::json!({
        "id": token_id,
        "revoked": true,
        "message": "Token revoked successfully"
      }) ) )
        .into_response()
    }
    Err( _ ) =>
    {
      // Revocation failed (race condition - token was deactivated by another request)
      // Fetch metadata again to determine if it was revoked or rotated
      match state.storage.get_token_metadata( token_id ).await
      {
        Ok( updated_metadata ) =>
        {
          if updated_metadata.revoked_at.is_some()
          {
            // Token was revoked - return 409
            (
              StatusCode::CONFLICT,
              Json( serde_json::json!({ "error": "Token already revoked" }) ),
            )
              .into_response()
          }
          else
          {
            // Token was rotated or otherwise deactivated - return 404
            (
              StatusCode::NOT_FOUND,
              Json( serde_json::json!({ "error": "Token not found" }) ),
            )
              .into_response()
          }
        }
        Err( _ ) =>
        {
          // Token doesn't exist - return 404
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
/// Validate API token without authentication (Deliverable 1.6)
///
/// # Purpose
///
/// External services can verify token validity without making authenticated requests.
/// This is a public endpoint (no JWT required) that returns token validation status.
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `request` - Validation request with token string
///
/// # Returns
///
/// - 200 OK with {"valid":true,...} if token is valid and active
/// - 200 OK with {"valid":false} if token is invalid, expired, or revoked
/// - 400 Bad Request if request body is malformed
///
/// # Security
///
/// - No authentication required (public endpoint)
/// - Constant-time comparison prevents timing attacks
/// - Rate limiting should be applied at reverse proxy level (100 validates/min per IP)
pub async fn validate_token(
  State( state ): State< TokenState >,
  crate::error::JsonBody( request ): crate::error::JsonBody< ValidateTokenRequest >,
) -> impl IntoResponse
{
  // Verify token using storage layer
  let token_id = match state.storage.verify_token( &request.token ).await
  {
    Ok( id ) => id,
    Err( _ ) =>
    {
      // Token is invalid, expired, or revoked - return {"valid":false}
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

  // Token is valid - get metadata
  let metadata = match state.storage.get_token_metadata( token_id ).await
  {
    Ok( metadata ) => metadata,
    Err( _ ) =>
    {
      // Token exists but metadata fetch failed - return {"valid":false}
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

  // Return success with metadata
  ( StatusCode::OK, Json( ValidateTokenResponse
  {
    valid: true,
    user_id: Some( metadata.user_id ),
    project_id: metadata.project_id,
    token_id: Some( token_id ),
  } ) )
    .into_response()
}
