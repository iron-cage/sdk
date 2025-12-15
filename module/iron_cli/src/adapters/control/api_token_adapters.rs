//! API token adapter functions

use super::{ ControlApiClient, ControlApiConfig };
use crate::handlers::control::api_token_handlers;
use crate::formatting::{ TreeFmtFormatter, OutputFormat };
use std::str::FromStr;
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
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
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

  let name = params.get( "name" ).unwrap(); // Already validated

  let mut body = json!({
    "name": name,
  });

  if let Some( expires_in ) = params.get( "expires_in" )
  {
    body[ "expires_in" ] = json!( expires_in.parse::< i64 >().expect( "expires_in parameter validated by handler" ) );
  }

  let response = client
    .post( "/api/v1/tokens", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn get_api_token_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  api_token_handlers::get_api_token_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/tokens/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
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

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/tokens/{}/revoke", id );

  let response = client
    .post( &path, json!({}) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}
