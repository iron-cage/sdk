//! API token adapter functions

use super::{ ControlApiClient, ControlApiConfig, format_output };
use crate::handlers::control::api_token_handlers;
use std::collections::HashMap;
use serde_json::json;

pub async fn list_api_tokens_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  api_token_handlers::list_api_tokens_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/tokens", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn create_api_token_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  api_token_handlers::create_api_token_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] API token would be created (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let name = params.get( "name" ).unwrap();

  let mut body = json!({
    "name": name,
  });

  if let Some( expires_in ) = params.get( "expires_in" )
  {
    body[ "expires_in" ] = json!( expires_in.parse::< i64 >().unwrap() );
  }

  let response = client
    .post( "/api/v1/tokens", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn get_api_token_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  api_token_handlers::get_api_token_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/tokens/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn revoke_api_token_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  api_token_handlers::revoke_api_token_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] API token would be revoked (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/tokens/{}/revoke", id );

  let response = client
    .post( &path, json!({}) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}
