//! Analytics adapter functions

use super::{ ControlApiClient, ControlApiConfig, format_output };
use crate::handlers::control::analytics_handlers;
use std::collections::HashMap;

pub async fn usage_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::usage_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let mut query_params = HashMap::new();

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/usage", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn spending_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::spending_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let mut query_params = HashMap::new();

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/spending", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn metrics_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::metrics_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let response = client
    .get( "/api/v1/analytics/metrics", None )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn usage_by_agent_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::usage_by_agent_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let mut query_params = HashMap::new();

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/usage/by-agent", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn usage_by_provider_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::usage_by_provider_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let mut query_params = HashMap::new();

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/usage/by-provider", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn spending_by_period_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::spending_by_period_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let period = params.get( "period" ).unwrap();

  let mut query_params = HashMap::new();
  query_params.insert( "period".to_string(), period.clone() );

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/spending/by-period", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn export_usage_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::export_usage_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let export_format = params.get( "export_format" ).unwrap();

  let mut query_params = HashMap::new();
  query_params.insert( "format".to_string(), export_format.clone() );

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/usage/export", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}

pub async fn export_spending_adapter(
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  analytics_handlers::export_spending_handler( params )
    .map_err( |e| e.to_string() )?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new( config );

  let export_format = params.get( "export_format" ).unwrap();

  let mut query_params = HashMap::new();
  query_params.insert( "format".to_string(), export_format.clone() );

  if let Some( start_date ) = params.get( "start_date" )
  {
    query_params.insert( "start_date".to_string(), start_date.clone() );
  }

  if let Some( end_date ) = params.get( "end_date" )
  {
    query_params.insert( "end_date".to_string(), end_date.clone() );
  }

  let response = client
    .get( "/api/v1/analytics/spending/export", Some( query_params ) )
    .await
    .map_err( |e| format!( "HTTP request failed: {}", e ) )?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "table" );
  format_output( &response, format )
}
