//! API Token authentication middleware
//!
//! Authenticates requests using API tokens (iron_xxx format) instead of JWT.
//! Used for external API access (key fetching, runtime calls).

use axum::extract::FromRef;
use std::sync::Arc;
use iron_token_manager::storage::TokenStorage;

/// Authenticated API token claims
///
/// Extracted from a valid API token via the `Authorization: Bearer <token>` header.
/// Unlike JWT authentication, this uses iron_xxx format tokens stored in the database.
#[ derive( Debug, Clone ) ]
pub struct ApiTokenAuth
{
  /// Database ID of the token
  pub token_id: i64,
  /// User ID who owns this token
  pub user_id: String,
  /// Project ID the token is assigned to (if any)
  pub project_id: Option< String >,
}

/// State required for API token authentication
#[ derive( Debug, Clone ) ]
pub struct ApiTokenState
{
  /// Token storage for verification
  pub token_storage: Arc< TokenStorage >,
}

#[ axum::async_trait ]
impl< S > axum::extract::FromRequestParts< S > for ApiTokenAuth
where
  S: Send + Sync,
  ApiTokenState: axum::extract::FromRef< S >,
{
  type Rejection = ( axum::http::StatusCode, axum::Json< serde_json::Value > );

  async fn from_request_parts(
    parts: &mut axum::http::request::Parts,
    state: &S,
  ) -> Result< Self, Self::Rejection >
  {
    // Extract API token state
    let api_token_state = ApiTokenState::from_ref( state );

    // Extract Authorization header
    let auth_header = parts
      .headers
      .get( axum::http::header::AUTHORIZATION )
      .and_then( |h| h.to_str().ok() )
      .ok_or_else( || (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": "Missing Authorization header" }) ),
      ) )?;

    // Parse "Bearer <token>" format
    let token = auth_header
      .strip_prefix( "Bearer " )
      .ok_or_else( || (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": "Invalid Authorization header format" }) ),
      ) )?;

    // Verify token and get ID
    let token_id = api_token_state
      .token_storage
      .verify_token( token )
      .await
      .map_err( |_| (
        axum::http::StatusCode::UNAUTHORIZED,
        axum::Json( serde_json::json!({ "error": "Invalid or expired token" }) ),
      ) )?;

    // Get token metadata (user_id, project_id)
    let metadata = api_token_state
      .token_storage
      .get_token_metadata( token_id )
      .await
      .map_err( |_| (
        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        axum::Json( serde_json::json!({ "error": "Failed to retrieve token metadata" }) ),
      ) )?;

    Ok( ApiTokenAuth {
      token_id,
      user_id: metadata.user_id,
      project_id: metadata.project_id,
    } )
  }
}

