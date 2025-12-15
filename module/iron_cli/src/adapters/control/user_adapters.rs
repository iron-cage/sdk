//! User adapter functions

use super::{ ControlApiClient, ControlApiConfig };
use crate::handlers::control::control_user_handlers;
use crate::formatting::{ TreeFmtFormatter, OutputFormat };
use std::str::FromStr;
use std::collections::HashMap;
use serde_json::json;

pub async fn list_users_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::list_users_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/users", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn create_user_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::create_user_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] User would be created (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let username = params.get( "username" ).unwrap(); // Already validated
  let email = params.get( "email" ).unwrap(); // Already validated
  let password = params.get( "password" ).unwrap(); // Already validated

  let mut body = json!({
    "username": username,
    "email": email,
    "password": password,
  });

  if let Some( role ) = params.get( "role" )
  {
    body[ "role" ] = json!( role );
  }

  let response = client
    .post( "/api/v1/users", body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn get_user_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::get_user_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/users/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn update_user_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::update_user_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] User would be updated (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated

  let mut body = json!({});

  if let Some( email ) = params.get( "email" )
  {
    body[ "email" ] = json!( email );
  }

  let path = format!( "/api/v1/users/{}", id );
  let response = client
    .put( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn delete_user_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::delete_user_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] User would be deleted (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/users/{}", id );

  let response = client
    .delete( &path )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn set_user_role_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::set_user_role_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] User role would be set (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let role = params.get( "role" ).unwrap(); // Already validated

  let body = json!({
    "role": role,
  });

  let path = format!( "/api/v1/users/{}/role", id );
  let response = client
    .put( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn reset_user_password_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::reset_password_handler( params )
    .map_err( |e| e.to_string() )?;

  let dry_run = params
    .get( "dry" )
    .and_then( |s| s.parse::< u8 >().ok() )
    .unwrap_or( 0 ) == 1;

  if dry_run
  {
    return Ok( "[DRY RUN] Password would be reset (no HTTP request made)".to_string() );
  }

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let new_password = params.get( "new_password" ).unwrap(); // Already validated

  let body = json!({
    "new_password": new_password,
  });

  let path = format!( "/api/v1/users/{}/password", id );
  let response = client
    .put( &path, body )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn get_user_permissions_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  control_user_handlers::get_user_permissions_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/users/{}/permissions", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}
