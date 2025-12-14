//! Budget request adapter functions

use super::{ ControlApiClient, ControlApiConfig };
use crate::handlers::control::budget_request_handlers;
use crate::formatting::{ TreeFmtFormatter, OutputFormat };
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::json;

pub async fn list_budget_requests_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::list_budget_requests_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let mut query_params = HashMap::new();

  if let Some( status ) = params.get( "status" )
  {
    query_params.insert( "status".to_string(), status.clone() );
  }

  let response = client
    .get( "/api/v1/budget/requests", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn create_budget_request_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::create_budget_request_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Budget request would be created (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let agent_id = params.get( "agent_id" ).unwrap(); // Already validated
  let amount = params.get( "amount" ).unwrap(); // Already validated
  let reason = params.get( "reason" ).unwrap(); // Already validated

  let body = json!({
    "agent_id": agent_id,
    "amount": amount.parse::< i64 >().expect( "amount parameter validated by handler" ),
    "reason": reason,
  });

  let response = client
    .post( "/api/v1/budget/requests", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn get_budget_request_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::get_budget_request_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/budget/requests/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn approve_budget_request_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::approve_budget_request_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Budget request would be approved (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/budget/requests/{}/approve", id );

  let response = client
    .post( &path, json!({}) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn reject_budget_request_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::reject_budget_request_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Budget request would be rejected (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let reason = params.get( "reason" ).unwrap(); // Already validated

  let body = json!({
    "reason": reason,
  });

  let path = format!( "/api/v1/budget/requests/{}/reject", id );

  let response = client
    .post( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn cancel_budget_request_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  budget_request_handlers::cancel_budget_request_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Budget request would be cancelled (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/budget/requests/{}/cancel", id );

  let response = client
    .post( &path, json!({}) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}
