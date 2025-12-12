//! Provider adapter functions
//!
//! Bridge provider handlers with Control API HTTP client.

use super::{ ControlApiClient, ControlApiConfig, format_output };
use crate::handlers::control::provider_handlers;
use std::collections::HashMap;
use serde_json::json;

/// List all providers
pub async fn list_providers_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::list_providers_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/providers", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Create new provider
pub async fn create_provider_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::create_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Provider would be created (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let name = params.get( "name" ).unwrap();
  let api_key = params.get( "api_key" ).unwrap();

  let mut body = json!({
    "name": name,
    "api_key": api_key,
  });

  if let Some( endpoint ) = params.get( "endpoint" )
  {
    body[ "endpoint" ] = json!( endpoint );
  }

  let response = client
    .post( "/api/v1/providers", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Get provider by ID
pub async fn get_provider_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::get_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/providers/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Update provider
pub async fn update_provider_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::update_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Provider would be updated (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();

  let mut body = json!({});

  if let Some( name ) = params.get( "name" )
  {
    body[ "name" ] = json!( name );
  }

  if let Some( api_key ) = params.get( "api_key" )
  {
    body[ "api_key" ] = json!( api_key );
  }

  if let Some( endpoint ) = params.get( "endpoint" )
  {
    body[ "endpoint" ] = json!( endpoint );
  }

  let path = format!( "/api/v1/providers/{}", id );
  let response = client
    .put( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Delete provider
pub async fn delete_provider_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::delete_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Provider would be deleted (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/providers/{}", id );

  let response = client
    .delete( &path )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Assign agents to provider
pub async fn assign_agents_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::assign_agents_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Agents would be assigned (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let agent_ids = params.get( "agent_ids" ).unwrap();

  let ids: Vec< String > = agent_ids
    .split( ',' )
    .map( |s| s.trim().to_string() )
    .collect();

  let body = json!({
    "agent_ids": ids,
  });

  let path = format!( "/api/v1/providers/{}/agents", id );
  let response = client
    .post( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// List agents for provider
pub async fn list_provider_agents_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::list_provider_agents_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/providers/{}/agents", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

/// Remove agent from provider
pub async fn remove_agent_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  provider_handlers::remove_agent_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Agent would be removed (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let agent_id = params.get( "agent_id" ).unwrap();

  let path = format!( "/api/v1/providers/{}/agents/{}", id, agent_id );
  let response = client
    .delete( &path )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}
