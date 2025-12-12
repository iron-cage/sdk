//! Request trace adapters for Token Manager API
//!
//! Implements trace querying and analysis operations.
//!
//! ## Authentication
//!
//! All operations require access token from keyring.

use std::collections::HashMap;
use serde_json::json;
use crate::handlers::traces_handlers;
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

/// List traces adapter
///
/// Lists request traces with optional filtering.
///
/// ## Parameters
///
/// - page: Page number (optional)
/// - limit: Results per page (optional)
/// - start_date: Start date filter (YYYY-MM-DD, optional)
/// - end_date: End date filter (YYYY-MM-DD, optional)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .traces.list
/// iron-token .traces.list limit::20 page::1
/// ```
pub async fn list_traces_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  traces_handlers::list_traces_handler( params )
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
    .get( "/api/v1/traces", Some( query_params ), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to list traces: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Get trace adapter
///
/// Retrieves details for specific request trace.
///
/// ## Parameters
///
/// - id: Trace ID (UUID)
/// - format: Output format (table|json|yaml)
///
/// ## Example
///
/// ```bash
/// iron-token .traces.get id::abc123-def456
/// ```
pub async fn get_trace_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  traces_handlers::get_trace_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Build path
  let id = params.get( "id" ).unwrap(); // Safe: validated by handler
  let path = format!( "/api/v1/traces/{}", id );

  // 5. Make HTTP call
  let response = client
    .get( &path, None, Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to get trace: {}", e ) )?;

  // 6. Format output
  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  format_response( &response, format )
}

/// Export traces adapter
///
/// Exports trace data to file or stdout.
///
/// ## Parameters
///
/// - export_format: Export format (json|csv)
/// - output: Output file path (stdout if not provided)
/// - format: Display format for confirmation message
///
/// ## Example
///
/// ```bash
/// iron-token .traces.export
/// iron-token .traces.export export_format::json output::traces.json
/// iron-token .traces.export export_format::csv output::traces.csv
/// ```
pub async fn export_traces_adapter(
  params: &HashMap<String, String>,
) -> Result<String, String>
{
  // 1. Validate with handler
  traces_handlers::export_traces_handler( params )
    .map_err( |e| e.to_string() )?;

  // 2. Get access token from keyring
  let access_token = keyring::get_access_token()
    .map_err( |e| format!( "Not authenticated: {}. Please run .auth.login first.", e ) )?;

  // 3. Create HTTP client
  let config = TokenApiConfig::load();
  let client = TokenApiClient::new( config );

  // 4. Fetch all traces (up to 1000)
  let mut query_params = HashMap::new();
  query_params.insert( "limit".to_string(), "1000".to_string() );

  let response = client
    .get( "/api/v1/traces", Some( query_params ), Some( &access_token ) )
    .await
    .map_err( |e| format!( "Failed to fetch traces: {}", e ) )?;

  // 5. Handle export format
  let export_format = params.get( "export_format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  let export_data = match export_format
  {
    "json" => serde_json::to_string_pretty( &response )
      .map_err( |e| format!( "Failed to serialize JSON: {}", e ) )?,
    "csv" =>
    {
      // For CSV, convert JSON array to CSV format
      // This is a simplified implementation - full CSV support would need proper CSV library
      return Err( "CSV export not yet implemented".to_string() );
    }
    _ => return Err( format!( "Unsupported export format: {}", export_format ) ),
  };

  // 6. Write to file or stdout
  if let Some( output_file ) = params.get( "output" )
  {
    std::fs::write( output_file, &export_data )
      .map_err( |e| format!( "Failed to write file: {}", e ) )?;

    // Return success message
    let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

    let success_data = json!({
      "status": "success",
      "message": format!( "Traces exported to {}", output_file ),
      "file": output_file,
      "format": export_format,
    });

    format_response( &success_data, format )
  }
  else
  {
    // Output to stdout
    Ok( export_data )
  }
}
