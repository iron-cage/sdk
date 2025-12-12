//! Call tracing REST API endpoints
//!
//! Phase 4 Day 29: REST API Endpoints - Call Tracing
//!
//! Endpoints:
//! - GET /api/traces - Query API call traces
//! - GET /api/traces/:id - Get specific trace details

use axum::{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use crate::error::JsonPath;
use iron_token_manager::trace_storage::TraceStorage;
use serde::{ Serialize, Deserialize };
use std::sync::Arc;

/// Trace query state
#[ derive( Clone ) ]
pub struct TracesState
{
  pub storage: Arc< TraceStorage >,
}

impl TracesState
{
  /// Create new traces state
  ///
  /// # Errors
  ///
  /// Returns error if database connection fails
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let storage = TraceStorage::new( database_url ).await?;
    Ok( Self { storage: Arc::new( storage ) } )
  }
}

/// API call trace
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct ApiTrace
{
  pub id: i64,
  pub token_id: i64,
  pub provider: String,
  pub model: String,
  pub endpoint: String,
  pub status_code: i32,
  pub latency_ms: i64,
  pub input_tokens: i64,
  pub output_tokens: i64,
  pub cost_cents: i64,
  pub timestamp: i64,
}

/// GET /api/traces
///
/// Query API call traces
///
/// # Arguments
///
/// * `_user` - Authenticated user (enforces JWT authentication)
/// * `state` - Traces state
///
/// # Returns
///
/// - 200 OK with list of traces
/// - 401 Unauthorized if authentication fails
/// - 500 Internal Server Error if database query fails
pub async fn list_traces(
  _user: crate::jwt_auth::AuthenticatedUser,
  State( state ): State< TracesState >,
) -> impl IntoResponse
{
  // Query all traces
  let traces = match state.storage.get_all_traces().await
  {
    Ok( traces ) => traces,
    Err( e ) => {
      tracing::error!( "Failed to list traces: {:?}", e );
      return ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "Database query failed"
      }) ) ).into_response();
    }
  };

  // Map to response type
  let response: Vec< ApiTrace > = traces.into_iter().map( |trace| {
    ApiTrace {
      id: trace.id,
      token_id: trace.token_id,
      provider: trace.provider,
      model: trace.model,
      endpoint: trace.endpoint,
      status_code: trace.response_status,
      latency_ms: trace.duration_ms,
      input_tokens: trace.input_tokens,
      output_tokens: trace.output_tokens,
      cost_cents: trace.cost_cents,
      timestamp: trace.traced_at,
    }
  } ).collect();

  ( StatusCode::OK, Json( response ) ).into_response()
}

/// GET /api/traces/:id
///
/// Get specific trace details
///
/// # Arguments
///
/// * `_user` - Authenticated user (enforces JWT authentication)
/// * `state` - Traces state
/// * `trace_id` - Trace identifier
///
/// # Returns
///
/// - 200 OK with trace details
/// - 401 Unauthorized if authentication fails
/// - 404 Not Found if trace doesn't exist
/// - 500 Internal Server Error if database query fails
pub async fn get_trace(
  _user: crate::jwt_auth::AuthenticatedUser,
  State( state ): State< TracesState >,
  JsonPath( trace_id ): JsonPath< i64 >,
) -> impl IntoResponse
{
  // Query trace by ID
  let trace = match state.storage.get_trace_by_id( trace_id ).await
  {
    Ok( trace ) => trace,
    Err( e ) => {
      tracing::error!( "Failed to get trace {}: {:?}", trace_id, e );
      return ( StatusCode::NOT_FOUND, Json( serde_json::json!({
        "error": "Trace not found"
      }) ) ).into_response();
    }
  };

  let response = ApiTrace
  {
    id: trace.id,
    token_id: trace.token_id,
    provider: trace.provider,
    model: trace.model,
    endpoint: trace.endpoint,
    status_code: trace.response_status,
    latency_ms: trace.duration_ms,
    input_tokens: trace.input_tokens,
    output_tokens: trace.output_tokens,
    cost_cents: trace.cost_cents,
    timestamp: trace.traced_at,
  };

  ( StatusCode::OK, Json( response ) ).into_response()
}
