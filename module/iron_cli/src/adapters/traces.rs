//! Traces management adapters
//!
//! Bridge unilang CLI to traces handlers and services.

use super::AdapterError;
use super::services::TracesService;
use super::auth::HasParams;
use crate::handlers::traces_handlers;
use crate::formatting::TreeFmtFormatter;
use std::collections::HashMap;

fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// List traces adapter
pub async fn list_traces_adapter<T, S>(
  command: &T,
  traces_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TracesService,
{
  let params = extract_params( command );
  let _ = traces_handlers::list_traces_handler( &params )?;

  let filter = params.get( "filter" ).map( |s| s.as_str() );
  let limit = params.get( "limit" ).and_then( |s| s.parse::<u32>().ok() );

  let traces = traces_service.list_traces( filter, limit ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "traces retrieved".to_string() );
  output_data.insert( "count".to_string(), traces.len().to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Get trace adapter
pub async fn get_trace_adapter<T, S>(
  command: &T,
  traces_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TracesService,
{
  let params = extract_params( command );
  let _ = traces_handlers::get_trace_handler( &params )?;

  let trace_id = params.get( "trace_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "trace_id missing after validation".to_string() )
  })?;

  let trace = traces_service.get_trace( trace_id ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "success".to_string() );
  output_data.insert( "trace_id".to_string(), trace.id.clone() );
  output_data.insert( "request".to_string(), trace.request.clone() );
  output_data.insert( "duration_ms".to_string(), trace.duration_ms.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Export traces adapter
pub async fn export_traces_adapter<T, S>(
  command: &T,
  traces_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TracesService,
{
  let params = extract_params( command );
  let _ = traces_handlers::export_traces_handler( &params )?;

  let output_path = params.get( "output" ).ok_or_else( || {
    AdapterError::ExtractionError( "output missing after validation".to_string() )
  })?;

  let format = params.get( "format" ).map( |s| s.as_str() ).unwrap_or( "json" );

  traces_service.export_traces( output_path, format ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "exported".to_string() );
  output_data.insert( "output".to_string(), output_path.clone() );
  output_data.insert( "format".to_string(), format.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}
