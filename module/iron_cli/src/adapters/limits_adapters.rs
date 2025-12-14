//! Token limit management adapters for Token Manager API
//!
//! Implements limit querying and modification operations.
//!
//! ## Authentication
//!
//! All operations require access token from keyring.

use std::collections::HashMap;
use serde_json::json;
use crate::handlers::limits_handlers;
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

/// Show limits adapter
///
/// Displays current token limits.
///
/// ## Parameters
///
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .limits.show
/// ```
pub async fn show_limits_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  limits_handlers::list_limits_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Make HTTP call
  let response = client
    .get( "/api/v1/limits", None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get limits: {}", e ) )?;

  // 5. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Update limit adapter
///
/// Updates token limit value.
///
/// ## Parameters
///
/// - limit_type: Type of limit (daily|monthly|total)
/// - value: New limit value
/// - dry: Dry run flag (0|1)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .limits.update limit_type::daily value::1000
/// ```
pub async fn update_limit_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  limits_handlers::update_limit_handler( params )
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
      "message": "Would update token limit",
      "limit_type": params.get( "limit_type" ).unwrap(), // Already validated
      "value": params.get( "value" ).unwrap(), // Already validated
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
  let body = json!({
    "limit_type": params.get( "limit_type" ).unwrap(), // Already validated
    "value": params.get( "value" ).unwrap().parse::<i64>().expect( "value parameter validated by handler" ),
  });

  // 6. Make HTTP call
  let response = client
    .put( "/api/v1/limits", body, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to update limit: {}", e ) )?;

  // 7. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Get limit adapter
///
/// Retrieves specific limit details by ID.
///
/// ## Parameters
///
/// - limit_id: Limit ID to retrieve
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .limits.get limit_id::789
/// iron-token .limits.get limit_id::123 format::json
/// ```
pub async fn get_limit_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  limits_handlers::get_limit_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let limit_id = params.get( "limit_id" )
    .ok_or_else( || "limit_id parameter is required".to_string() )?;
  let path = format!( "/api/v1/limits/{}", limit_id );

  // 5. Make HTTP call
  let response = client
    .get( &path, None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get limit {}: {}", limit_id, e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Create limit adapter
///
/// Creates new usage limit(s).
///
/// ## Parameters
///
/// - project: Optional project ID
/// - max_tokens: Max tokens per day
/// - max_requests: Max requests per minute
/// - max_cost: Max cost per month (cents)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .limits.create max_tokens::1000000
/// iron-token .limits.create max_requests::100 max_cost::10000
/// iron-token .limits.create project::myproject max_tokens::500000
/// ```
pub async fn create_limit_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  limits_handlers::create_limit_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build request body from optional parameters
  let mut body = json!({});

  if let Some( project ) = params.get( "project" )
  {
    body["project_id"] = json!( project );
  }

  if let Some( max_tokens ) = params.get( "max_tokens" )
  {
    body["max_tokens"] = json!( max_tokens.parse::<i64>()
      .map_err( |_| "max_tokens must be a number".to_string() )? );
  }

  if let Some( max_requests ) = params.get( "max_requests" )
  {
    body["max_requests"] = json!( max_requests.parse::<i64>()
      .map_err( |_| "max_requests must be a number".to_string() )? );
  }

  if let Some( max_cost ) = params.get( "max_cost" )
  {
    body["max_cost"] = json!( max_cost.parse::<i64>()
      .map_err( |_| "max_cost must be a number".to_string() )? );
  }

  // 5. Make HTTP call
  let response = client
    .post( "/api/v1/limits", body, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to create limit: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Delete limit adapter
///
/// Deletes an existing limit.
///
/// ## Parameters
///
/// - limit_id: Limit ID to delete
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .limits.delete limit_id::789
/// iron-token .limits.delete limit_id::123 format::json
/// ```
pub async fn delete_limit_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  limits_handlers::delete_limit_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let limit_id = params.get( "limit_id" )
    .ok_or_else( || "limit_id parameter is required".to_string() )?;
  let path = format!( "/api/v1/limits/{}", limit_id );

  // 5. Make HTTP call
  let _response = client
    .delete( &path, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to delete limit {}: {}", limit_id, e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let success_data = json!({
    "status": "success",
    "message": format!( "Limit {} deleted", limit_id ),
    "limit_id": limit_id,
  });

  format_response( &success_data, format )
}
