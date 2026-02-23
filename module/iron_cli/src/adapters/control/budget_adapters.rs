//! Budget adapter functions

use super::{ControlApiClient, ControlApiConfig};
use crate::formatting::{OutputFormat, TreeFmtFormatter};
use crate::handlers::control::budget_handlers;
use core::str::FromStr;
use std::collections::HashMap;

/// Get budget status across agents (analytics)
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn budget_status_adapter<S: ::core::hash::BuildHasher + Default>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  budget_handlers::budget_status_handler(params).map_err(|e| e.to_string())?;

  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Build query parameters
  let mut query_params = HashMap::default();

  if let Some(agent_id) = params.get("agent_id") {
    query_params.insert("agent_id".to_string(), agent_id.clone());
  }

  if let Some(threshold) = params.get("threshold") {
    query_params.insert("threshold".to_string(), threshold.clone());
  }

  if let Some(status) = params.get("status") {
    query_params.insert("status".to_string(), status.clone());
  }

  if let Some(page) = params.get("page") {
    query_params.insert("page".to_string(), page.clone());
  }

  if let Some(per_page) = params.get("per_page") {
    query_params.insert("per_page".to_string(), per_page.clone());
  }

  let query = if query_params.is_empty() {
    None
  } else {
    Some(query_params)
  };

  let response = client
    .get("/api/v1/analytics/budget/status", query)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  let format = params.get("format").map_or("table", String::as_str);
  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}
