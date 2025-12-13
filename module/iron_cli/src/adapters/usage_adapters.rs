//! Usage tracking adapters for Token Manager API
//!
//! Implements usage querying and export operations.
//!
//! ## Authentication
//!
//! All operations require access token from keyring.

use std::collections::HashMap;
use serde_json::json;
use crate::handlers::usage_handlers;
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

/// Show usage adapter
///
/// Displays usage statistics with optional date range filtering.
///
/// ## Parameters
///
/// - start_date: Optional start date (YYYY-MM-DD)
/// - end_date: Optional end date (YYYY-MM-DD)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .usage.show
/// iron-token .usage.show start_date::2025-01-01 end_date::2025-01-31
/// ```
pub async fn show_usage_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  usage_handlers::show_usage_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build query parameters
  let mut query_params = HashMap::new();

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  // 5. Make HTTP call
  let response = client
    .get( "/api/v1/usage", Some( query_params ), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get usage: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Export usage adapter
///
/// Exports usage statistics to file (CSV/JSON).
///
/// ## Parameters
///
/// - output_file: File path for export
/// - export_format: Export format (csv|json)
/// - start_date: Optional start date (YYYY-MM-DD)
/// - end_date: Optional end date (YYYY-MM-DD)
/// - format: Output format for confirmation message (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .usage.export output_file::usage.csv export_format::csv
/// ```
pub async fn export_usage_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  usage_handlers::export_usage_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build query parameters
  let mut query_params = HashMap::new();

  query_params.insert(
    "format".to_string(),
    params.get( "export_format" ).unwrap().clone(), // Already validated
  );

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  // 5. Make HTTP call
  let response = client
    .get( "/api/v1/usage/export", Some( query_params ), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to export usage: {}", e ) )?;

  // 6. Write to file
  let output_file = params.get( "output_file" ).unwrap(); // Already validated
  let export_data = serde_json::to_string_pretty( &response )
    .map_err( |e| format!( "Failed to serialize data: {}", e ) )?;

  std::fs::write( output_file, export_data )
    .map_err( |e| format!( "Failed to write file: {}", e ) )?;

  // 7. Format success message
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let success_data = json!({
    "status": "success",
    "message": format!( "Usage data exported to {}", output_file ),
    "file": output_file,
  });

  format_response( &success_data, format )
}

/// Get usage by project adapter
///
/// Displays usage statistics for a specific project.
///
/// ## Parameters
///
/// - project_id: Project ID to query
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .usage.by_project project_id::myproject
/// iron-token .usage.by_project project_id::prod format::json
/// ```
pub async fn get_usage_by_project_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  usage_handlers::usage_by_project_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let project_id = params.get( "project_id" )
    .ok_or_else( || "project_id parameter is required".to_string() )?;
  let path = format!( "/api/v1/usage/by-project/{}", project_id );

  // 5. Make HTTP call
  let response = client
    .get( &path, None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get usage for project {}: {}", project_id, e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Get usage by provider adapter
///
/// Displays usage statistics for a specific provider.
///
/// ## Parameters
///
/// - provider: Provider name (e.g., openai, anthropic)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .usage.by_provider provider::openai
/// iron-token .usage.by_provider provider::anthropic format::json
/// ```
pub async fn get_usage_by_provider_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  usage_handlers::usage_by_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let provider = params.get( "provider" )
    .ok_or_else( || "provider parameter is required".to_string() )?;
  let path = format!( "/api/v1/usage/by-provider/{}", provider );

  // 5. Make HTTP call
  let response = client
    .get( &path, None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get usage for provider {}: {}", provider, e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}
