//! Project adapter functions

use super::{ ControlApiClient, ControlApiConfig, format_output };
use crate::handlers::control::project_handlers;
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
  format_output( &response, format )
}

pub async fn get_project_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  project_handlers::get_project_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let id = params.get( "id" ).unwrap();
  let path = format!( "/api/v1/projects/{}", id );

  let response = client
    .get( &path, None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}
