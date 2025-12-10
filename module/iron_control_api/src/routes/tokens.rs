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

/// Create token request
#[ derive( Debug, Deserialize ) ]
pub struct CreateTokenRequest
{
  pub user_id: String,
  pub project_id: Option< String >,
  pub description: Option< String >,
  pub agent_id: Option< i64 >,
  pub provider: Option< String >,
}

impl CreateTokenRequest
{
  // Fix(issue-001): Prevent DoS via unlimited string validation
  // Root cause: Accepted unbounded external input without resource limits
  // Pitfall: Always validate input length before processing to prevent resource exhaustion

  /// Maximum user_id length (DoS protection). Must match database CHECK constraint.
  const MAX_USER_ID_LENGTH: usize = 500;

  /// Maximum project_id length (DoS protection). Must match database CHECK constraint.
  const MAX_PROJECT_ID_LENGTH: usize = 500;

  /// Maximum description length.
  const MAX_DESCRIPTION_LENGTH: usize = 500;

  /// Validate token creation request.
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate user_id is not empty
    if self.user_id.trim().is_empty()
    {
      return Err( "user_id cannot be empty".to_string() );
    }

    // Validate user_id length (DoS protection)
    if self.user_id.len() > Self::MAX_USER_ID_LENGTH
    {
      return Err( format!(
        "user_id too long (max {} characters)",
        Self::MAX_USER_ID_LENGTH
      ) );
    }

    // Fix(issue-002): Prevent NULL byte injection causing C string termination attacks
    // Root cause: Accepted NULL bytes in strings passed to C/FFI libraries and database drivers
    // Pitfall: Always validate against control characters when interacting with C libraries or databases using C drivers

    // Validate user_id doesnt contain NULL bytes (string termination attack prevention)
    if self.user_id.contains( '\0' )
    {
      return Err( "user_id contains invalid NULL byte".to_string() );
    }

    // Validate project_id if provided
    if let Some( ref project_id ) = self.project_id
    {
      if project_id.trim().is_empty()
      {
        return Err( "project_id cannot be empty".to_string() );
      }

      // Validate project_id length (DoS protection)
      if project_id.len() > Self::MAX_PROJECT_ID_LENGTH
      {
        return Err( format!(
          "project_id too long (max {} characters)",
          Self::MAX_PROJECT_ID_LENGTH
        ) );
      }

      // Validate project_id doesnt contain NULL bytes
      if project_id.contains( '\0' )
      {
        return Err( "project_id contains invalid NULL byte".to_string() );
      }
    }

    // Validate description if provided
    if let Some( ref description ) = self.description
    {
      // Validate description length
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
/// Create new API token
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `request` - Token creation parameters
///
/// # Returns
///
/// - 201 Created with new token details
/// - 400 Bad Request if validation fails or malformed JSON
/// - 500 Internal Server Error if generation fails
pub async fn create_token(
  State( state ): State< TokenState >,
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

  // Generate token
  let token = state.generator.generate();

  // Store token in database
  let token_id = match state
    .storage
    .create_token(
      &token,
      &request.user_id,
      request.project_id.as_deref(),
      request.description.as_deref(),
      request.agent_id,
      request.provider.as_deref(),
    )
    .await
  {
    Ok( id ) => id,
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

  ( StatusCode::CREATED, Json( CreateTokenResponse
  {
    id: metadata.id,
    token, // Return plaintext token ONCE on creation
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
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 200 OK with token details
/// - 404 Not Found if token doesn't exist
pub async fn get_token(
  State( state ): State< TokenState >,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  // TODO: Extract user_id from JWT claims and verify ownership
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
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 200 OK with updated token details
/// - 404 Not Found if token doesn't exist
pub async fn update_token(
  State( state ): State< TokenState >,
  Path( token_id ): Path< i64 >,
  crate::error::JsonBody( request ): crate::error::JsonBody< UpdateTokenRequest >,
) -> impl IntoResponse
{
  // TODO: Extract user_id from JWT claims and verify ownership
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  let token_id = match state
    .storage
    .update_token_provider(
      token_id,
      &request.provider,
    )
    .await
  {
    Ok( () ) => token_id,
    Err( _ ) =>
    {
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json( serde_json::json!({ "error": "Failed to update token" }) ),
      )
        .into_response();
    }
  };

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
/// Revoke token (mark as inactive)
///
/// # Arguments
///
/// * `state` - Token generator state
/// * `token_id` - Token ID from path
///
/// # Returns
///
/// - 204 No Content if revocation successful
/// - 404 Not Found if token doesn't exist or is already revoked
pub async fn revoke_token(
  State( state ): State< TokenState >,
  Path( token_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Deactivate token (atomic operation returns error if token doesn't exist or is already inactive)
  match state.storage.deactivate_token( token_id ).await
  {
    Ok( () ) => StatusCode::NO_CONTENT.into_response(),
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
