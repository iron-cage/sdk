//! Health management adapters
//!
//! Bridge unilang CLI to health handlers and services.

use super::auth::HasParams;
use super::services::HealthService;
use super::AdapterError;
use crate::formatting::TreeFmtFormatter;
use crate::handlers::health_handlers;
use std::collections::HashMap;

fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// Health check adapter
///
/// # Errors
///
/// Returns [`AdapterError`] if the handler validation or health service call fails.
pub async fn health_adapter<T, S>(
  command: &T,
  health_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: HealthService,
{
  let params = extract_params(command);
  let _ = health_handlers::health_handler(&params)?;

  let health = health_service.get_health().await?;

  let mut output_data = HashMap::new();
  output_data.insert("status".to_string(), "health check".to_string());
  output_data.insert("health".to_string(), health.status.clone());

  let output = formatter.format_single(&output_data);

  Ok(output)
}

/// Version adapter
///
/// # Errors
///
/// Returns [`AdapterError`] if the handler validation or version service call fails.
pub async fn version_adapter<T, S>(
  command: &T,
  health_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: HealthService,
{
  let params = extract_params(command);
  let _ = health_handlers::version_handler(&params)?;

  let version = health_service.get_version().await?;

  let mut output_data = HashMap::new();
  output_data.insert("status".to_string(), "version retrieved".to_string());
  output_data.insert("version".to_string(), version);

  let output = formatter.format_single(&output_data);

  Ok(output)
}
