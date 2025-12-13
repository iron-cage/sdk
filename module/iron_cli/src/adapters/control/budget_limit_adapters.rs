//! Budget limit adapter functions

use super::{ ControlApiClient, ControlApiConfig };
use crate::handlers::control::budget_limit_handlers;
use crate::formatting::{ TreeFmtFormatter, OutputFormat };
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::json;

pub async fn get_budget_limit_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_limit_handlers::get_budget_limit_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/budget/limit", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn set_budget_limit_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_limit_handlers::set_budget_limit_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Budget limit would be set (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let limit = params.get( "limit" ).unwrap(); // Already validated

  let body = json!({
    "limit": limit.parse::< i64 >().expect( "limit parameter validated by handler" ),
  });

  let response = client
    .put( "/api/v1/budget/limit", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}
