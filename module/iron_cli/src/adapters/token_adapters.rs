//! Token management adapters for Token Manager API
//!
//! Implements IC token CRUD operations (generate, list, get, rotate, revoke).
//!
//! ## Authentication
//!
//! All operations require access token from keyring.
//! On 401 response, user should run `.auth.refresh` to get new access token.

use std::collections::HashMap;
use serde_json::json;
use crate::handlers::token_handlers;
use super::token::{ TokenApiClient, TokenApiConfig };
use super::keyring;

/// Format JSON response according to format parameter
fn format_response( data: &serde_json::Value, format: &str ) -> Result<String, String>
{
  match format
  {
    "yaml" => serde_yaml::to_string( data ).map_err( |e| e.to_string() ),
    _ => serde_json::to_string_pretty( data ).map_err( |e| e.to_string() ),
  }
}

/// Generate token adapter
///
/// Creates new IC token with specified name and scope.
///
/// ## Parameters
///
/// - name: Token name
/// - scope: Token scope (action:resource format)
/// - expires_in: Optional expiration (days)
/// - dry: Dry run flag (0|1)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .tokens.generate name::my-token scope::read:tokens
/// ```
pub async fn generate_token_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  token_handlers::generate_token_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Check dry_run
  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::<u8>().ok() )
    .unwrap_or( 0 )
    == 1;

  if dry_run
  {
    let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

    let dry_data = json!({
      "status": "dry_run",
      "message": "Would generate new IC token",
      "name": params.get( "name" ).unwrap(), // Already validated
      "scope": params.get( "scope" ).unwrap(), // Already validated
    });

    return format_response( &dry_data, format );
  }

  // 3. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 4. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 5. Build request body
  let mut body = json!({
    "name": params.get( "name" ).unwrap(), // Already validated
    "scope": params.get( "scope" ).unwrap(), // Already validated
  });

  if let Some( expires_in ) = params.get( "expires_in" )
  {
    body["expires_in_days"] = json!( expires_in );
  }

  // 6. Make HTTP call
  let response = client
    .post( "/api/v1/tokens", body, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to generate token: {}", e ) )?;

  // 7. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// List tokens adapter
///
/// Lists all IC tokens with optional filtering.
///
/// ## Parameters
///
/// - page: Page number (optional)
/// - limit: Results per page (optional)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .tokens.list
/// iron-token .tokens.list limit::10 page::2
/// ```
pub async fn list_tokens_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  token_handlers::list_tokens_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build query parameters
  let mut query_params = HashMap::new();

  if let Some( page ) = params.get( "page" )
  {
    query_params.insert( "page".to_string(), page.clone() );
  }

  if let Some( limit ) = params.get( "limit" )
  {
    query_params.insert( "limit".to_string(), limit.clone() );
  }

  // 5. Make HTTP call
  let response = client
    .get( "/api/v1/tokens", Some( query_params ), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to list tokens: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Get token adapter
///
/// Retrieves details for specific IC token.
///
/// ## Parameters
///
/// - id: Token ID (UUID)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .tokens.get id::abc123-def456
/// ```
pub async fn get_token_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  token_handlers::get_token_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/tokens/{}", id );

  // 5. Make HTTP call
  let response = client
    .get( &path, None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get token: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Rotate token adapter
///
/// Rotates IC token (creates new value, invalidates old).
///
/// ## Parameters
///
/// - id: Token ID (UUID)
/// - dry: Dry run flag (0|1)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .tokens.rotate id::abc123-def456
/// ```
pub async fn rotate_token_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  token_handlers::rotate_token_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Check dry_run
  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::<u8>().ok() )
    .unwrap_or( 0 )
    == 1;

  if dry_run
  {
    let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

    let dry_data = json!({
      "status": "dry_run",
      "message": "Would rotate IC token",
      "id": params.get( "id" ).unwrap(), // Already validated
    });

    return format_response( &dry_data, format );
  }

  // 3. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 4. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 5. Build path
  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/tokens/{}/rotate", id );

  // 6. Make HTTP call
  let response = client
    .post( &path, json!({}), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to rotate token: {}", e ) )?;

  // 7. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Revoke token adapter
///
/// Revokes IC token (permanently disables).
///
/// ## Parameters
///
/// - id: Token ID (UUID)
/// - dry: Dry run flag (0|1)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .tokens.revoke id::abc123-def456
/// ```
pub async fn revoke_token_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  token_handlers::revoke_token_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Check dry_run
  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::<u8>().ok() )
    .unwrap_or( 0 )
    == 1;

  if dry_run
  {
    let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

    let dry_data = json!({
      "status": "dry_run",
      "message": "Would revoke IC token",
      "id": params.get( "id" ).unwrap(), // Already validated
    });

    return format_response( &dry_data, format );
  }

  // 3. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 4. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 5. Build path
  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/tokens/{}", id );

  // 6. Make HTTP call
  let response = client
    .delete( &path, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to revoke token: {}", e ) )?;

  // 7. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}
