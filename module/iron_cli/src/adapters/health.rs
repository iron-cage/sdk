//! Health management adapters
//!
//! Bridge unilang CLI to health handlers and services.

use super::AdapterError;
use super::services::HealthService;
use super::auth::HasParams;
use crate::handlers::health_handlers;
use crate::formatting::TreeFmtFormatter;
use std::collections::HashMap;

fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// Health check adapter
pub async fn health_adapter<T, S>(
  command: &T,
  health_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: HealthService,
{
  let params = extract_params( command );
  let _ = health_handlers::health_handler( &params )?;

  let health = health_service.get_health().await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "health check".to_string() );
  output_data.insert( "health".to_string(), health.status.clone() );
  output_data.insert( "version".to_string(), health.version.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Version adapter
pub async fn version_adapter<T, S>(
  command: &T,
  health_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: HealthService,
{
  let params = extract_params( command );
  let _ = health_handlers::version_handler( &params )?;

  let version = health_service.get_version().await?;

  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "version retrieved".to_string() );
  output_data.insert( "version".to_string(), version );

  let output = formatter.format_single( &output_data );

  Ok( output )
}
