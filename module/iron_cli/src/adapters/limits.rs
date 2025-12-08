//! Limits management adapters
//!
//! Bridge unilang CLI to limits handlers and services.

use super::AdapterError;
use super::services::LimitsService;
use super::auth::HasParams;
use crate::handlers::limits_handlers;
use crate::formatting::Formatter;
use std::collections::HashMap;

fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// List limits adapter
pub async fn list_limits_adapter<T, S>(
  command: &T,
  limits_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: LimitsService,
{
  let params = extract_params( command );
  let _ = limits_handlers::list_limits_handler( &params )?;

  let limits = limits_service.list_limits().await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "limits retrieved".to_string() );
  output_data.insert( "count".to_string(), limits.len().to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Get limit adapter
pub async fn get_limit_adapter<T, S>(
  command: &T,
  limits_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: LimitsService,
{
  let params = extract_params( command );
  let _ = limits_handlers::get_limit_handler( &params )?;

  let limit_id = params.get( "limit_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "limit_id missing after validation".to_string() )
  })?;

  let limit = limits_service.get_limit( limit_id ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "success".to_string() );
  output_data.insert( "limit_id".to_string(), limit.id.clone() );
  output_data.insert( "resource_type".to_string(), limit.resource_type.clone() );
  output_data.insert( "limit_value".to_string(), limit.limit_value.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Create limit adapter
pub async fn create_limit_adapter<T, S>(
  command: &T,
  limits_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: LimitsService,
{
  let params = extract_params( command );
  let _ = limits_handlers::create_limit_handler( &params )?;

  let resource_type = params.get( "resource_type" ).ok_or_else( || {
    AdapterError::ExtractionError( "resource_type missing after validation".to_string() )
  })?;

  let limit_value_str = params.get( "limit_value" ).ok_or_else( || {
    AdapterError::ExtractionError( "limit_value missing after validation".to_string() )
  })?;

  let limit_value: u64 = limit_value_str.parse().map_err( |_| {
    AdapterError::ExtractionError( "limit_value must be a valid integer".to_string() )
  })?;

  let limit = limits_service.create_limit( resource_type, limit_value ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "created".to_string() );
  output_data.insert( "limit_id".to_string(), limit.id.clone() );
  output_data.insert( "resource_type".to_string(), limit.resource_type.clone() );
  output_data.insert( "limit_value".to_string(), limit.limit_value.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Update limit adapter
pub async fn update_limit_adapter<T, S>(
  command: &T,
  limits_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: LimitsService,
{
  let params = extract_params( command );
  let _ = limits_handlers::update_limit_handler( &params )?;

  let limit_id = params.get( "limit_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "limit_id missing after validation".to_string() )
  })?;

  let limit_value_str = params.get( "limit_value" ).ok_or_else( || {
    AdapterError::ExtractionError( "limit_value missing after validation".to_string() )
  })?;

  let new_value: u64 = limit_value_str.parse().map_err( |_| {
    AdapterError::ExtractionError( "limit_value must be a valid integer".to_string() )
  })?;

  let limit = limits_service.update_limit( limit_id, new_value ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "updated".to_string() );
  output_data.insert( "limit_id".to_string(), limit.id.clone() );
  output_data.insert( "limit_value".to_string(), limit.limit_value.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Delete limit adapter
pub async fn delete_limit_adapter<T, S>(
  command: &T,
  limits_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: LimitsService,
{
  let params = extract_params( command );
  let _ = limits_handlers::delete_limit_handler( &params )?;

  let limit_id = params.get( "limit_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "limit_id missing after validation".to_string() )
  })?;

  limits_service.delete_limit( limit_id ).await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "deleted".to_string() );
  output_data.insert( "limit_id".to_string(), limit_id.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}
