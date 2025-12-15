//! Project adapter functions

use super::{ ControlApiClient, ControlApiConfig };
use crate::handlers::control::project_handlers;
use crate::formatting::{ TreeFmtFormatter, OutputFormat };
use std::str::FromStr;
use std::collections::HashMap;

pub async fn list_projects_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  project_handlers::list_projects_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/projects", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}

pub async fn get_project_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  project_handlers::get_project_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap(); // Already validated
  let path = format!( "/api/v1/projects/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  let output_format = OutputFormat::from_str( format ).unwrap_or_default();
  let formatter = TreeFmtFormatter::new( output_format );
  formatter.format_value( &response )
}
