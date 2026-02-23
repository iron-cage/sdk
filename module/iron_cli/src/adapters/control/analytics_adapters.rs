//! Analytics adapter functions

use super::{ControlApiClient, ControlApiConfig};
use crate::formatting::{OutputFormat, TreeFmtFormatter};
use crate::handlers::control::analytics_handlers;
use core::str::FromStr;
use std::collections::HashMap;

/// Retrieve analytics usage data
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn usage_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::usage_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let mut query_params = HashMap::default();

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/usage", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Retrieve analytics spending data
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn spending_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::spending_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let mut query_params = HashMap::default();

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/spending", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Retrieve analytics metrics
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn metrics_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::metrics_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let response = client
    .get("/api/v1/analytics/metrics", None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Retrieve analytics usage grouped by agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn usage_by_agent_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::usage_by_agent_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let mut query_params = HashMap::default();

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/usage/by-agent", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Retrieve analytics usage grouped by provider
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn usage_by_provider_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::usage_by_provider_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let mut query_params = HashMap::default();

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/usage/by-provider", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Retrieve analytics spending grouped by time period
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `period` parameter is missing from the map after handler validation.
pub async fn spending_by_period_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::spending_by_period_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let period = params.get("period").unwrap(); // Already validated

  let mut query_params = HashMap::default();
  query_params.insert("period".to_string(), period.clone());

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/spending/by-period", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Export usage analytics data
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `export_format` parameter is missing from the map after handler validation.
pub async fn export_usage_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::export_usage_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let export_format = params.get("export_format").unwrap(); // Already validated

  let mut query_params = HashMap::default();
  query_params.insert("format".to_string(), export_format.clone());

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/usage/export", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Export spending analytics data
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `export_format` parameter is missing from the map after handler validation.
pub async fn export_spending_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  analytics_handlers::export_spending_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  let export_format = params.get("export_format").unwrap(); // Already validated

  let mut query_params = HashMap::default();
  query_params.insert("format".to_string(), export_format.clone());

  if let Some(start_date) = params.get("start_date") {
    query_params.insert("start_date".to_string(), start_date.clone());
  }

  if let Some(end_date) = params.get("end_date") {
    query_params.insert("end_date".to_string(), end_date.clone());
  }

  let response = client
    .get("/api/v1/analytics/spending/export", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}
