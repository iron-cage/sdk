//! Agent adapter functions
//!
//! Bridge agent handlers with Control API HTTP client.
//! Async functions that validate params, make HTTP calls, and format output.

use super::{ControlApiClient, ControlApiConfig};
use crate::formatting::{OutputFormat, TreeFmtFormatter};
use crate::handlers::control::agent_handlers;
use core::str::FromStr;
use serde_json::json;
use std::collections::HashMap;

/// List all agents
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
pub async fn list_agents_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::list_agents_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Build query parameters
  let mut query_params = HashMap::new();

  if let Some(page) = params.get("page") {
    query_params.insert("page".to_string(), page.clone());
  }

  if let Some(limit) = params.get("limit") {
    query_params.insert("limit".to_string(), limit.clone());
  }

  // Make HTTP GET request
  let response = client
    .get("/api/v1/agents", Some(query_params))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output based on format parameter
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Create new agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if validated parameters (`name`, `providers`, `provider_key_id`, `budget`)
/// are missing from the map after handler validation, or if their values cannot be parsed.
pub async fn create_agent_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::create_agent_handler(params).map_err(|e| e.to_string())?;

  // Check dry run mode
  let dry_run = params
    .get("dry")
    .and_then(|s| s.parse::<u8>().ok())
    .unwrap_or(0)
    == 1;

  if dry_run {
    return Ok("[DRY RUN] Agent would be created (no HTTP request made)".to_string());
  }

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Build request body
  let name = params.get("name").unwrap(); // Already validated
  let providers_str = params.get("providers").unwrap(); // Already validated
  let provider_key_id = params.get("provider_key_id").unwrap(); // Already validated
  let budget = params.get("budget").unwrap(); // Already validated

  // Parse providers as comma-separated list
  let providers: Vec<String> = providers_str
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();

  let body = json!({
    "name": name,
    "providers": providers,
    "provider_key_id": provider_key_id.parse::< i64 >().expect( "provider_key_id validated by handler" ),
    "initial_budget_microdollars": budget.parse::< i64 >().expect( "Budget parameter validated by handler" ),
  });

  // Make HTTP POST request
  let response = client
    .post("/api/v1/agents", body)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Get agent by ID
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn get_agent_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::get_agent_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP GET request
  let path = format!("/api/v1/agents/{id}");
  let response = client
    .get(&path, None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Update agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing, or if `budget` cannot be parsed as `i64`.
pub async fn update_agent_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::update_agent_handler(params).map_err(|e| e.to_string())?;

  // Check dry run mode
  let dry_run = params
    .get("dry")
    .and_then(|s| s.parse::<u8>().ok())
    .unwrap_or(0)
    == 1;

  if dry_run {
    return Ok("[DRY RUN] Agent would be updated (no HTTP request made)".to_string());
  }

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Build request body with optional fields
  let mut body = json!({});

  if let Some(name) = params.get("name") {
    body["name"] = json!(name);
  }

  if let Some(budget) = params.get("budget") {
    body["budget"] = json!(budget
      .parse::<i64>()
      .expect("Budget parameter validated by handler"));
  }

  // Make HTTP PUT request
  let path = format!("/api/v1/agents/{id}");
  let response = client
    .put(&path, body)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Delete agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn delete_agent_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::delete_agent_handler(params).map_err(|e| e.to_string())?;

  // Check dry run mode
  let dry_run = params
    .get("dry")
    .and_then(|s| s.parse::<u8>().ok())
    .unwrap_or(0)
    == 1;

  if dry_run {
    return Ok("[DRY RUN] Agent would be deleted (no HTTP request made)".to_string());
  }

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP DELETE request
  let path = format!("/api/v1/agents/{id}");
  let response = client
    .delete(&path)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Assign providers to agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if validated `id` or `provider_ids` parameters are missing after handler validation.
pub async fn assign_providers_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::assign_providers_handler(params).map_err(|e| e.to_string())?;

  // Check dry run mode
  let dry_run = params
    .get("dry")
    .and_then(|s| s.parse::<u8>().ok())
    .unwrap_or(0)
    == 1;

  if dry_run {
    return Ok("[DRY RUN] Providers would be assigned (no HTTP request made)".to_string());
  }

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get parameters
  let id = params.get("id").unwrap(); // Already validated
  let provider_ids = params.get("provider_ids").unwrap(); // Already validated

  // Parse provider IDs
  let ids: Vec<String> = provider_ids
    .split(',')
    .map(|s| s.trim().to_string())
    .collect();

  // Build request body
  let body = json!({
    "provider_ids": ids,
  });

  // Make HTTP POST request
  let path = format!("/api/v1/agents/{id}/providers");
  let response = client
    .post(&path, body)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// List providers for agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn list_agent_providers_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::list_agent_providers_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP GET request
  let path = format!("/api/v1/agents/{id}/providers");
  let response = client
    .get(&path, None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Remove provider from agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if validated `id` or `provider_id` parameters are missing after handler validation.
pub async fn remove_provider_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::remove_provider_handler(params).map_err(|e| e.to_string())?;

  // Check dry run mode
  let dry_run = params
    .get("dry")
    .and_then(|s| s.parse::<u8>().ok())
    .unwrap_or(0)
    == 1;

  if dry_run {
    return Ok("[DRY RUN] Provider would be removed (no HTTP request made)".to_string());
  }

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get parameters
  let id = params.get("id").unwrap(); // Already validated
  let provider_id = params.get("provider_id").unwrap(); // Already validated

  // Make HTTP DELETE request
  let path = format!("/api/v1/agents/{id}/providers/{provider_id}");
  let response = client
    .delete(&path)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Generate IC token for agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn generate_ic_token_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::generate_ic_token_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP POST request
  let path = format!("/api/v1/agents/{id}/ic-token");
  let response = client
    .post(&path, json!({}))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Get IC token status for agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn get_ic_token_status_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::get_ic_token_status_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP GET request
  let path = format!("/api/v1/agents/{id}/ic-token");
  let response = client
    .get(&path, None)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Regenerate IC token for agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn regenerate_ic_token_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::regenerate_ic_token_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP POST request
  let path = format!("/api/v1/agents/{id}/ic-token/regenerate");
  let response = client
    .post(&path, json!({}))
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}

/// Revoke IC token for agent
///
/// # Errors
///
/// Returns `Err(String)` if handler validation or the HTTP request fails.
///
/// # Panics
///
/// Panics if the validated `id` parameter is missing from the map after handler validation.
pub async fn revoke_ic_token_adapter<S: ::core::hash::BuildHasher>(
  params: &HashMap<String, String, S>,
) -> Result<String, String> {
  // Validate parameters using handler
  agent_handlers::revoke_ic_token_handler(params).map_err(|e| e.to_string())?;

  // Create HTTP client
  let config = ControlApiConfig::load();
  let client = ControlApiClient::new(config);

  // Get agent ID
  let id = params.get("id").unwrap(); // Already validated

  // Make HTTP DELETE request
  let path = format!("/api/v1/agents/{id}/ic-token");
  let response = client
    .delete(&path)
    .await
    .map_err(|e| format!("HTTP request failed: {e}"))?;

  // Format output
  let format = params.get("format").map_or("table", String::as_str);

  let output_format = OutputFormat::from_str(format).unwrap_or_default();
  let formatter = TreeFmtFormatter::new(output_format);
  formatter.format_value(&response)
}
