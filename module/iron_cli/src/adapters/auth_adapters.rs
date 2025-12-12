//! Authentication adapters for Token Manager API
//!
//! Implements login, logout, and token refresh operations.
//!
//! ## Authentication Flow
//!
//! 1. **Login:** username/password → API → access_token + refresh_token → keyring
//! 2. **Refresh:** refresh_token (keyring) → API → new access_token → keyring
//! 3. **Logout:** Clear tokens from keyring
//!
//! ## Keyring Integration
//!
//! All tokens stored securely in system keyring (no plaintext files).
//! See `super::keyring` module for storage operations.

use std::collections::HashMap;
use serde_json::{ json, Value };
use crate::handlers::auth_handlers;
use super::token::{ TokenApiClient, TokenApiConfig };
use super::keyring;

/// Format JSON response according to format parameter
fn format_response( data: &Value, format: &str ) -> Result<String, String>
{
  match format
  {
    "yaml" => serde_yaml::to_string( data ).map_err( |e| e.to_string() ),
    _ => serde_json::to_string_pretty( data ).map_err( |e| e.to_string() ),
  }
}

/// Login adapter
///
/// Authenticates user and stores tokens in keyring.
///
/// ## Parameters
///
/// - username: User credentials
/// - password: User credentials
/// - format: Output format (table|json|yaml, default: table)
///
/// ## Flow
///
/// 1. Validate params (handler)
/// 2. POST /api/v1/auth/login
/// 3. Store access_token + refresh_token in keyring
/// 4. Format success message
///
/// ## Example
///
/// ```bash
/// iron-token .auth.login username::admin password::secret123
/// ```
pub async fn login_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  auth_handlers::login_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Extract parameters
  let username = params.get( "username" ).unwrap(); // Safe: validated by handler
  let password = params.get( "password" ).unwrap(); // Safe: validated by handler

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build request body
  let body = json!({
    "username": username,
    "password": password,
  });

  // 5. Make HTTP call (no access_token needed for login)
  let response = client
    .post( "/api/v1/auth/login", body, None )
    .await
    .map_err( |e| format!( "Login failed: {}", e ) )?;

  // 6. Extract tokens from response
  let access_token = response
    .get( "access_token" )
    .and_then( |v| v.as_str() )
    .ok_or_else( || "Missing access_token in response".to_string() )?;

  let refresh_token = response
    .get( "refresh_token" )
    .and_then( |v| v.as_str() )
    .ok_or_else( || "Missing refresh_token in response".to_string() )?;

  // 7. Store tokens in keyring
  keyring::set_access_token( access_token )
    .map_err( |e| format!( "Failed to store access token: {}", e ) )?;

  keyring::set_refresh_token( refresh_token )
    .map_err( |e| format!( "Failed to store refresh token: {}", e ) )?;

  // 8. Format success output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let success_data = json!({
    "status": "success",
    "message": format!( "Login successful for user: {}", username ),
    "username": username,
  });

  format_response( &success_data, format )
}

/// Logout adapter
///
/// Clears stored tokens from keyring.
///
/// ## Parameters
///
/// - format: Output format (table|json|yaml, default: table)
///
/// ## Flow
///
/// 1. Validate params (handler)
/// 2. Clear tokens from keyring
/// 3. Format success message
///
/// ## Example
///
/// ```bash
/// iron-token .auth.logout
/// ```
pub async fn logout_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  auth_handlers::logout_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Clear tokens from keyring
  keyring::clear_tokens()
    .map_err( |e| format!( "Failed to clear tokens: {}", e ) )?;

  // 3. Format success output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let success_data = json!({
    "status": "success",
    "message": "Logout successful. Tokens cleared from keyring.",
  });

  format_response( &success_data, format )
}

/// Refresh adapter
///
/// Refreshes access token using stored refresh token.
///
/// ## Parameters
///
/// - format: Output format (table|json|yaml, default: table)
///
/// ## Flow
///
/// 1. Validate params (handler)
/// 2. Get refresh_token from keyring
/// 3. POST /api/v1/auth/refresh
/// 4. Store new access_token in keyring
/// 5. Format success message
///
/// ## Example
///
/// ```bash
/// iron-token .auth.refresh
/// ```
pub async fn refresh_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  auth_handlers::refresh_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get refresh token from keyring
  let refresh_token = keyring::get_refresh_token()
    .map_err( |e| format!( "Failed to get refresh token: {}", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build request body
  let body = json!({
    "refresh_token": refresh_token,
  });

  // 5. Make HTTP call (no access_token needed for refresh)
  let response = client
    .post( "/api/v1/auth/refresh", body, None )
    .await
    .map_err( |e| format!( "Token refresh failed: {}", e ) )?;

  // 6. Extract new access token from response
  let new_access_token = response
    .get( "access_token" )
    .and_then( |v| v.as_str() )
    .ok_or_else( || "Missing access_token in response".to_string() )?;

  // 7. Store new access token in keyring
  keyring::set_access_token( new_access_token )
    .map_err( |e| format!( "Failed to store new access token: {}", e ) )?;

  // 8. Format success output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let success_data = json!({
    "status": "success",
    "message": "Access token refreshed successfully",
  });

  format_response( &success_data, format )
}
