//! Usage analytics REST API endpoints
//!
//! Phase 4 Day 29: REST API Endpoints - Usage Analytics
//!
//! Endpoints:
//! - GET /api/usage/aggregate - Aggregate usage statistics
//! - GET /api/usage/by-project/:project_id - Usage by specific project
//! - GET /api/usage/by-provider/:provider - Usage by specific provider
//!
//! # Path Parameter Validation
//!
//! Path parameters (`:project_id`, `:provider`) are validated before database queries to:
//! 1. **Prevent DoS attacks** via excessively long strings (project_id max 1000, provider max 100 chars)
//! 2. **Enforce consistency** with request body validation (e.g., CreateTokenRequest.project_id)
//! 3. **Provide explicit errors** (400 BAD_REQUEST) instead of generic database failures (500)
//! 4. **Defense in depth** - don't rely solely on URL length limits (vary by server: 2KB-8KB)
//!
//! See tests/usage/path_validation.rs for comprehensive test coverage and rationale.

use axum::{
  extract::{ Path, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use iron_token_manager::usage_tracker::UsageTracker;
use serde::{ Serialize, Deserialize };
use std::sync::Arc;

/// Maximum project_id length for DoS prevention
const MAX_PROJECT_ID_LENGTH: usize = 1000;

/// Maximum provider name length for DoS prevention
const MAX_PROVIDER_LENGTH: usize = 100;

/// Validate project_id path parameter
///
/// # Errors
///
/// Returns error if:
/// - project_id is whitespace-only
/// - project_id exceeds 1000 characters
fn validate_project_id( project_id: &str ) -> Result< (), String >
{
  // Validate not whitespace-only
  if project_id.trim().is_empty()
  {
    return Err( "project_id cannot be empty or whitespace-only".to_string() );
  }

  // Validate length (DoS prevention)
  if project_id.len() > MAX_PROJECT_ID_LENGTH
  {
    return Err( format!(
      "project_id too long (max {} characters)",
      MAX_PROJECT_ID_LENGTH
    ) );
  }

  Ok( () )
}

/// Validate provider path parameter
///
/// # Errors
///
/// Returns error if:
/// - provider is whitespace-only
/// - provider exceeds 100 characters
fn validate_provider( provider: &str ) -> Result< (), String >
{
  // Validate not whitespace-only
  if provider.trim().is_empty()
  {
    return Err( "provider cannot be empty or whitespace-only".to_string() );
  }

  // Validate length (DoS prevention)
  if provider.len() > MAX_PROVIDER_LENGTH
  {
    return Err( format!(
      "provider too long (max {} characters)",
      MAX_PROVIDER_LENGTH
    ) );
  }

  Ok( () )
}

/// Usage analytics state
#[ derive( Clone ) ]
pub struct UsageState
{
  pub tracker: Arc< UsageTracker >,
}

impl UsageState
{
  /// Create new usage state
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let tracker = UsageTracker::new( database_url ).await?;
    Ok( Self { tracker: Arc::new( tracker ) } )
  }
}

/// Aggregate usage response
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct AggregateUsageResponse
{
  pub total_tokens: i64,
  pub total_requests: i64,
  pub total_cost_cents: i64,
  pub providers: Vec< ProviderStats >,
}

/// Provider statistics
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ProviderStats
{
  pub provider: String,
  pub tokens: i64,
  pub requests: i64,
  pub cost_cents: i64,
}

/// GET /api/usage/aggregate
///
/// Get aggregate usage statistics across all tokens
///
/// # Arguments
///
/// * `state` - Usage tracker state
///
/// # Returns
///
/// - 200 OK with aggregate statistics
/// - 500 Internal Server Error if query fails
pub async fn get_aggregate_usage( State( state ): State< UsageState > ) -> impl IntoResponse
{
  // Get aggregate totals
  let aggregate = match state.tracker.get_all_aggregate_usage().await
  {
    Ok( agg ) => agg,
    Err( e ) => {
      tracing::error!( "Failed to query aggregate usage: {}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  // Get provider breakdown
  let provider_data = match state.tracker.get_usage_by_provider_all().await
  {
    Ok( data ) => data,
    Err( e ) => {
      tracing::error!( "Failed to query provider usage: {}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  let providers: Vec< ProviderStats > = provider_data.into_iter().map( |( provider, usage )| {
    ProviderStats {
      provider,
      tokens: usage.total_tokens,
      requests: usage.total_requests,
      cost_cents: usage.total_cost_cents,
    }
  } ).collect();

  let response = AggregateUsageResponse
  {
    total_tokens: aggregate.total_tokens,
    total_requests: aggregate.total_requests,
    total_cost_cents: aggregate.total_cost_cents,
    providers,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// GET /api/usage/by-project/:project_id
///
/// Get usage statistics for specific project
///
/// # Arguments
///
/// * `state` - Usage tracker state
/// * `project_id` - Project identifier
///
/// # Returns
///
/// - 200 OK with project usage stats
/// - 404 Not Found if project has no usage
pub async fn get_usage_by_project(
  State( state ): State< UsageState >,
  Path( project_id ): Path< String >,
) -> impl IntoResponse
{
  // Validate path parameter
  if let Err( validation_error ) = validate_project_id( &project_id )
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Query usage for the specific project
  let usage = match state.tracker.get_usage_by_project( &project_id ).await
  {
    Ok( usage ) => usage,
    Err( e ) => {
      tracing::error!( "Failed to query usage for project {}: {}", project_id, e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  // Query provider breakdown for the project
  let provider_data = match state.tracker.get_usage_by_provider_for_project( &project_id ).await
  {
    Ok( data ) => data,
    Err( e ) => {
      tracing::error!( "Failed to query provider usage for project {}: {}", project_id, e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  let providers: Vec< ProviderStats > = provider_data.into_iter().map( |( provider, usage )| {
    ProviderStats {
      provider,
      tokens: usage.total_tokens,
      requests: usage.total_requests,
      cost_cents: usage.total_cost_cents,
    }
  } ).collect();

  let response = AggregateUsageResponse
  {
    total_tokens: usage.total_tokens,
    total_requests: usage.total_requests,
    total_cost_cents: usage.total_cost_cents,
    providers,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// GET /api/usage/by-provider/:provider
///
/// Get usage statistics for specific provider
///
/// # Arguments
///
/// * `state` - Usage tracker state
/// * `provider` - Provider name (openai, anthropic, google)
///
/// # Returns
///
/// - 200 OK with provider usage stats
/// - 404 Not Found if provider has no usage
pub async fn get_usage_by_provider(
  State( state ): State< UsageState >,
  Path( provider ): Path< String >,
) -> impl IntoResponse
{
  // Validate path parameter
  if let Err( validation_error ) = validate_provider( &provider )
  {
    return ( StatusCode::BAD_REQUEST, Json( serde_json::json!({
      "error": validation_error
    }) ) ).into_response();
  }

  // Query usage for the specific provider
  let usage = match state.tracker.get_all_usage_for_provider( &provider ).await
  {
    Ok( usage ) => usage,
    Err( e ) => {
      tracing::error!( "Failed to query usage for provider {}: {}", provider, e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  let response = ProviderStats
  {
    provider,
    tokens: usage.total_tokens,
    requests: usage.total_requests,
    cost_cents: usage.total_cost_cents,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}
