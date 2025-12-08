//! Usage management adapters
//!
//! Bridge unilang CLI to usage handlers and services.
//!
//! ## Flow
//!
//! 1. Extract parameters from VerifiedCommand
//! 2. Call handler for validation (pure, sync)
//! 3. Perform async I/O via UsageService
//! 4. Format output

use super::AdapterError;
use super::services::UsageService;
use super::auth::HasParams;
use crate::handlers::usage_handlers;
use crate::formatting::Formatter;
use std::collections::HashMap;

/// Extract parameters from command
fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// Show usage adapter
pub async fn show_usage_adapter<T, S>(
  command: &T,
  usage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: UsageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = usage_handlers::show_usage_handler( &params )?;

  // Extract date range parameters
  let start_date = params.get( "start_date" ).map( |s| s.as_str() );
  let end_date = params.get( "end_date" ).map( |s| s.as_str() );

  // Perform async usage retrieval
  let records = usage_service.get_usage( start_date, end_date ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "usage retrieved".to_string() );
  output_data.insert( "record_count".to_string(), records.len().to_string() );

  if let (Some(start), Some(end)) = (start_date, end_date)
  {
    output_data.insert( "date_range".to_string(), format!( "{} to {}", start, end ) );
  }
  else if let Some(start) = start_date
  {
    output_data.insert( "date_range".to_string(), format!( "from {}", start ) );
  }
  else
  {
    output_data.insert( "date_range".to_string(), "all time".to_string() );
  }

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Usage by project adapter
pub async fn usage_by_project_adapter<T, S>(
  command: &T,
  usage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: UsageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = usage_handlers::usage_by_project_handler( &params )?;

  // Extract validated parameters
  let project_id = params.get( "project_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "project_id missing after validation".to_string() )
  })?;

  let start_date = params.get( "start_date" ).map( |s| s.as_str() );

  // Perform async usage retrieval
  let records = usage_service.get_usage_by_project( project_id, start_date ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "success".to_string() );
  output_data.insert( "project_id".to_string(), project_id.clone() );
  output_data.insert( "record_count".to_string(), records.len().to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Usage by provider adapter
pub async fn usage_by_provider_adapter<T, S>(
  command: &T,
  usage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: UsageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = usage_handlers::usage_by_provider_handler( &params )?;

  // Extract validated parameters
  let provider = params.get( "provider" ).ok_or_else( || {
    AdapterError::ExtractionError( "provider missing after validation".to_string() )
  })?;

  let aggregation = params.get( "aggregation" ).map( |s| s.as_str() );

  // Perform async usage retrieval
  let records = usage_service.get_usage_by_provider( provider, aggregation ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "success".to_string() );
  output_data.insert( "provider".to_string(), provider.clone() );
  output_data.insert( "record_count".to_string(), records.len().to_string() );

  if let Some(agg) = aggregation
  {
    output_data.insert( "aggregation".to_string(), agg.to_string() );
  }

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Export usage adapter
pub async fn export_usage_adapter<T, S>(
  command: &T,
  usage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: UsageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = usage_handlers::export_usage_handler( &params )?;

  // Extract validated parameters
  let output_path = params.get( "output" ).ok_or_else( || {
    AdapterError::ExtractionError( "output missing after validation".to_string() )
  })?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  // Perform async export
  usage_service.export_usage( output_path, format ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "exported".to_string() );
  output_data.insert( "output".to_string(), output_path.clone() );
  output_data.insert( "format".to_string(), format.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}
