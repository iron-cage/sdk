//! AI Provider key management REST API endpoints
//!
//! Endpoints:
//! - POST /api/providers - Create new provider key
//! - GET /api/providers - List all provider keys for user
//! - GET /api/providers/:id - Get specific provider key details
//! - PUT /api/providers/:id - Update provider key
//! - DELETE /api/providers/:id - Delete provider key
//! - POST /api/providers/:id/balance - Fetch balance from provider API
//! - POST /api/projects/:project_id/provider - Assign provider key to project

use axum::{
  extract::{ Path, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_secrets::crypto::{ CryptoService, mask_api_key };
use iron_token_manager::provider_key_storage::{ ProviderKeyStorage, ProviderType };
use serde::{ Deserialize, Serialize };
use std::sync::Arc;

/// Provider management state
#[ derive( Clone ) ]
pub struct ProvidersState
{
  pub storage: Arc< ProviderKeyStorage >,
  /// Crypto service - None if IRON_SECRETS_MASTER_KEY not set
  pub crypto: Option< Arc< CryptoService > >,
}

impl ProvidersState
{
  /// Create new providers state
  ///
  /// If IRON_SECRETS_MASTER_KEY is not set, the state will be created
  /// but crypto operations will be disabled (routes return 503).
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let storage = ProviderKeyStorage::connect( database_url ).await
      .map_err( |_| "Failed to connect to database" )?;

    // Try to initialize crypto, but don't fail if master key not set
    let crypto = match CryptoService::from_env()
    {
      Ok( c ) =>
      {
        eprintln!( "✓ AI Provider Keys feature enabled" );
        Some( Arc::new( c ) )
      }
      Err( e ) =>
      {
        eprintln!( "⚠ AI Provider Keys feature disabled: {}", e );
        eprintln!( "  Set IRON_SECRETS_MASTER_KEY environment variable to enable" );
        None
      }
    };

    Ok( Self {
      storage: Arc::new( storage ),
      crypto,
    } )
  }

  /// Check if crypto is available
  pub fn is_enabled( &self ) -> bool
  {
    self.crypto.is_some()
  }
}

/// Create provider key request
#[ derive( Debug, Deserialize ) ]
pub struct CreateProviderKeyRequest
{
  pub provider: String,
  pub api_key: String,
  pub base_url: Option< String >,
  pub description: Option< String >,
}

impl CreateProviderKeyRequest
{
  /// Maximum API key length (DoS protection)
  const MAX_API_KEY_LENGTH: usize = 500;

  /// Maximum base URL length
  const MAX_BASE_URL_LENGTH: usize = 2000;

  /// Maximum description length
  const MAX_DESCRIPTION_LENGTH: usize = 500;

  /// Validate request
  pub fn validate( &self ) -> Result< (), String >
  {
    // Validate provider type
    if self.provider != "openai" && self.provider != "anthropic"
    {
      return Err( "Invalid provider: must be 'openai' or 'anthropic'".to_string() );
    }

    // Validate API key not empty
    if self.api_key.trim().is_empty()
    {
      return Err( "api_key cannot be empty".to_string() );
    }

    // Validate API key length
    if self.api_key.len() > Self::MAX_API_KEY_LENGTH
    {
      return Err( format!( "api_key too long (max {} characters)", Self::MAX_API_KEY_LENGTH ) );
    }

    // Validate no NULL bytes
    if self.api_key.contains( '\0' )
    {
      return Err( "api_key contains invalid NULL byte".to_string() );
    }

    // Validate base_url if provided
    if let Some( ref base_url ) = self.base_url
    {
      if base_url.len() > Self::MAX_BASE_URL_LENGTH
      {
        return Err( format!( "base_url too long (max {} characters)", Self::MAX_BASE_URL_LENGTH ) );
      }
      if base_url.contains( '\0' )
      {
        return Err( "base_url contains invalid NULL byte".to_string() );
      }
    }

    // Validate description if provided
    if let Some( ref description ) = self.description
    {
      if description.len() > Self::MAX_DESCRIPTION_LENGTH
      {
        return Err( format!( "description too long (max {} characters)", Self::MAX_DESCRIPTION_LENGTH ) );
      }
      if description.contains( '\0' )
      {
        return Err( "description contains invalid NULL byte".to_string() );
      }
    }

    Ok( () )
  }
}

/// Update provider key request
#[ derive( Debug, Deserialize ) ]
pub struct UpdateProviderKeyRequest
{
  pub base_url: Option< String >,
  pub description: Option< String >,
  pub is_enabled: Option< bool >,
}

/// Provider key response (never contains plaintext API key)
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ProviderKeyResponse
{
  pub id: i64,
  pub provider: String,
  pub base_url: Option< String >,
  pub description: Option< String >,
  pub is_enabled: bool,
  pub created_at: i64,
  pub last_used_at: Option< i64 >,
  pub masked_key: String,
  /// Projects this key is assigned to
  pub assigned_projects: Vec< String >,
}

/// Assign provider to project request
#[ derive( Debug, Deserialize ) ]
pub struct AssignProviderRequest
{
  pub provider_key_id: i64,
}

/// Error response for disabled feature
fn feature_disabled_response() -> impl IntoResponse
{
  ( StatusCode::SERVICE_UNAVAILABLE, Json( serde_json::json!({
    "error": "AI Provider Keys feature is disabled. Set IRON_SECRETS_MASTER_KEY environment variable to enable."
  }) ) )
}

/// POST /api/providers
///
/// Create new AI provider key
pub async fn create_provider_key(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  crate::error::JsonBody( request ): crate::error::JsonBody< CreateProviderKeyRequest >,
) -> impl IntoResponse
{
  // Check if crypto is enabled
  let crypto = match &state.crypto
  {
    Some( c ) => c,
    None => return feature_disabled_response().into_response(),
  };

  // Validate request
  if let Err( validation_error ) = request.validate()
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Parse provider type
  let provider = match request.provider.as_str()
  {
    "openai" => ProviderType::OpenAI,
    "anthropic" => ProviderType::Anthropic,
    _ =>
    {
      return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
        "error": "Invalid provider type"
      }) ) ).into_response();
    }
  };

  // Encrypt API key
  let encrypted = match crypto.encrypt( &request.api_key )
  {
    Ok( enc ) => enc,
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to encrypt API key"
      }) ) ).into_response();
    }
  };

  // Create masked key for response
  let masked_key = mask_api_key( &request.api_key );

  // Store in database
  let key_id = match state.storage.create_key(
    provider,
    &encrypted.ciphertext_base64(),
    &encrypted.nonce_base64(),
    request.base_url.as_deref(),
    request.description.as_deref(),
    &claims.sub,
  ).await
  {
    Ok( id ) => id,
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to create provider key"
      }) ) ).into_response();
    }
  };

  // Get metadata for response
  let metadata = match state.storage.get_key_metadata( key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to retrieve provider key metadata"
      }) ) ).into_response();
    }
  };

  ( StatusCode::CREATED, Json( ProviderKeyResponse
  {
    id: metadata.id,
    provider: metadata.provider.to_string(),
    base_url: metadata.base_url,
    description: metadata.description,
    is_enabled: metadata.is_enabled,
    created_at: metadata.created_at,
    last_used_at: metadata.last_used_at,
    masked_key,
    assigned_projects: vec![], // New key has no assignments
  } ) ).into_response()
}

/// GET /api/providers
///
/// List all provider keys for authenticated user
pub async fn list_provider_keys(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
) -> impl IntoResponse
{
  let keys = match state.storage.list_keys( &claims.sub ).await
  {
    Ok( keys ) => keys,
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to fetch provider keys"
      }) ) ).into_response();
    }
  };

  // For each key, fetch assigned projects and build response
  let mut responses: Vec< ProviderKeyResponse > = Vec::with_capacity( keys.len() );

  for meta in keys
  {
    // Fetch projects assigned to this key
    let assigned_projects = state.storage
      .get_key_projects( meta.id )
      .await
      .unwrap_or_default();

    responses.push( ProviderKeyResponse
    {
      id: meta.id,
      provider: meta.provider.to_string(),
      base_url: meta.base_url,
      description: meta.description,
      is_enabled: meta.is_enabled,
      created_at: meta.created_at,
      last_used_at: meta.last_used_at,
      masked_key: "***".to_string(), // Cannot unmask without decrypting
      assigned_projects,
    } );
  }

  ( StatusCode::OK, Json( responses ) ).into_response()
}

/// GET /api/providers/:id
///
/// Get specific provider key details
pub async fn get_provider_key(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( key_id ): Path< i64 >,
) -> impl IntoResponse
{
  let metadata = match state.storage.get_key_metadata( key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Provider key not found"
      }) ) ).into_response();
    }
  };

  // Verify ownership
  if metadata.user_id != claims.sub
  {
    return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
      "error": "Provider key not found"
    }) ) ).into_response();
  }

  // Fetch assigned projects
  let assigned_projects = state.storage
    .get_key_projects( key_id )
    .await
    .unwrap_or_default();

  ( StatusCode::OK, Json( ProviderKeyResponse
  {
    id: metadata.id,
    provider: metadata.provider.to_string(),
    base_url: metadata.base_url,
    description: metadata.description,
    is_enabled: metadata.is_enabled,
    created_at: metadata.created_at,
    last_used_at: metadata.last_used_at,
    masked_key: "***".to_string(),
    assigned_projects,
  } ) ).into_response()
}

/// PUT /api/providers/:id
///
/// Update provider key (description, base_url, is_enabled)
pub async fn update_provider_key(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( key_id ): Path< i64 >,
  crate::error::JsonBody( request ): crate::error::JsonBody< UpdateProviderKeyRequest >,
) -> impl IntoResponse
{
  // Verify ownership
  let metadata = match state.storage.get_key_metadata( key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Provider key not found"
      }) ) ).into_response();
    }
  };

  if metadata.user_id != claims.sub
  {
    return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
      "error": "Provider key not found"
    }) ) ).into_response();
  }

  // Update fields if provided
  if let Some( ref description ) = request.description
  {
    if state.storage.update_description( key_id, Some( description ) ).await.is_err()
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to update description"
      }) ) ).into_response();
    }
  }

  if let Some( ref base_url ) = request.base_url
  {
    let url = if base_url.is_empty() { None } else { Some( base_url.as_str() ) };
    if state.storage.update_base_url( key_id, url ).await.is_err()
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to update base_url"
      }) ) ).into_response();
    }
  }

  if let Some( is_enabled ) = request.is_enabled
  {
    if state.storage.set_enabled( key_id, is_enabled ).await.is_err()
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to update is_enabled"
      }) ) ).into_response();
    }
  }

  // Get updated metadata
  let updated = match state.storage.get_key_metadata( key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to retrieve updated metadata"
      }) ) ).into_response();
    }
  };

  // Fetch assigned projects
  let assigned_projects = state.storage
    .get_key_projects( key_id )
    .await
    .unwrap_or_default();

  ( StatusCode::OK, Json( ProviderKeyResponse
  {
    id: updated.id,
    provider: updated.provider.to_string(),
    base_url: updated.base_url,
    description: updated.description,
    is_enabled: updated.is_enabled,
    created_at: updated.created_at,
    last_used_at: updated.last_used_at,
    masked_key: "***".to_string(),
    assigned_projects,
  } ) ).into_response()
}

/// DELETE /api/providers/:id
///
/// Delete provider key
pub async fn delete_provider_key(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( key_id ): Path< i64 >,
) -> impl IntoResponse
{
  // Verify ownership
  let metadata = match state.storage.get_key_metadata( key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Provider key not found"
      }) ) ).into_response();
    }
  };

  if metadata.user_id != claims.sub
  {
    return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
      "error": "Provider key not found"
    }) ) ).into_response();
  }

  // Delete
  match state.storage.delete_key( key_id ).await
  {
    Ok( () ) => StatusCode::NO_CONTENT.into_response(),
    Err( _ ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to delete provider key"
      }) ) ).into_response()
    }
  }
}

/// POST /api/projects/:project_id/provider
///
/// Assign provider key to project
pub async fn assign_provider_to_project(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( claims ): crate::jwt_auth::AuthenticatedUser,
  Path( project_id ): Path< String >,
  crate::error::JsonBody( request ): crate::error::JsonBody< AssignProviderRequest >,
) -> impl IntoResponse
{
  // Verify key ownership
  let metadata = match state.storage.get_key_metadata( request.provider_key_id ).await
  {
    Ok( meta ) => meta,
    Err( _ ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Provider key not found"
      }) ) ).into_response();
    }
  };

  if metadata.user_id != claims.sub
  {
    return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
      "error": "Provider key not found"
    }) ) ).into_response();
  }

  // Assign to project
  match state.storage.assign_to_project( request.provider_key_id, &project_id ).await
  {
    Ok( () ) => StatusCode::OK.into_response(),
    Err( _ ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to assign provider key to project"
      }) ) ).into_response()
    }
  }
}

/// DELETE /api/projects/:project_id/provider
///
/// Unassign provider key from project
pub async fn unassign_provider_from_project(
  State( state ): State< ProvidersState >,
  crate::jwt_auth::AuthenticatedUser( _claims ): crate::jwt_auth::AuthenticatedUser,
  Path( project_id ): Path< String >,
) -> impl IntoResponse
{
  // Get the current assignment to verify it exists
  let provider_key_id = match state.storage.get_project_key( &project_id ).await
  {
    Ok( Some( id ) ) => id,
    Ok( None ) =>
    {
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "No provider key assigned to this project"
      }) ) ).into_response();
    }
    Err( _ ) =>
    {
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to query project assignment"
      }) ) ).into_response();
    }
  };

  // Unassign from project
  match state.storage.unassign_from_project( provider_key_id, &project_id ).await
  {
    Ok( () ) => StatusCode::NO_CONTENT.into_response(),
    Err( _ ) =>
    {
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Failed to unassign provider key from project"
      }) ) ).into_response()
    }
  }
}
