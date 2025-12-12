//! Health check adapters for Token Manager API
//!
//! Implements health and version endpoints.
//!
//! ## Authentication
//!
//! Health endpoints are public (no auth required).

use std::collections::HashMap;
use crate::handlers::health_handlers;
use super::token::{ TokenApiClient, TokenApiConfig };

/// Format JSON response according to format parameter
fn format_response( data: &serde_json::Value, format: &str ) -> Result<String, String>
{
  match format
  {
    "yaml" => serde_yaml::to_string( data ).map_err( |e| e.to_string() ),
    _ => serde_json::to_string_pretty( data ).map_err( |e| e.to_string() ),
  }
}

/// Health check adapter
///
/// Checks if Token Manager API is responding.
///
/// ## Parameters
///
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .health.check
/// ```
pub async fn health_check_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  health_handlers::health_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Create HTTP client (no auth needed for health)
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 3. Make HTTP call (no access_token needed)
  let response = client
    .get( "/api/v1/health", None, None )
    .await
    .map_err( |e| format!( "Health check failed: {}", e ) )?;

  // 4. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Version adapter
///
/// Returns CLI version information. Optionally includes API version if available.
///
/// ## Parameters
///
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .version
/// ```
pub async fn version_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  health_handlers::version_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get CLI version (always available)
  let cli_version = env!( "CARGO_PKG_VERSION" );

  // 3. Try to get API version (optional, fails gracefully)
  let api_version = {
    let config = TokenApiConfig::load();
    let client = TokenApiClient::new( config );

    client
      .get( "/api/v1/version", None, None )
      .await
      .ok()
      .and_then( |v| v.get( "version" ).and_then( |v| v.as_str() ).map( String::from ) )
  };

  // 4. Build response
  let mut version_info = serde_json::json!({
    "cli_version": cli_version,
  });

  if let Some( api_ver ) = api_version
  {
    version_info[ "api_version" ] = serde_json::json!( api_ver );
  }
  else
  {
    version_info[ "api_version" ] = serde_json::json!( "<unavailable>" );
  }

  // 5. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &version_info, format )
}
