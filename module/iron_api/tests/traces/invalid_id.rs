//! Tests for invalid ID parameter handling in traces endpoints
//!
//! Verifies that non-numeric IDs return JSON error responses per FR-5 specification.

use axum::{ http::StatusCode, body::Body, Router };
use tower::ServiceExt;
use http_body_util::BodyExt;

/// Build minimal test app with traces routes for testing
async fn build_test_app() -> Router
{
  use axum::routing::get;

  let traces_state = iron_api::routes::traces::TracesState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to initialize traces state" );

  Router::new()
    .route( "/api/traces", get( iron_api::routes::traces::list_traces ) )
    .route( "/api/traces/:id", get( iron_api::routes::traces::get_trace ) )
    .with_state( traces_state )
}

/// Reproduces Issue #2: Non-JSON error response for invalid path parameters (traces endpoint)
///
/// **Bug:** When non-numeric ID is provided to GET /api/traces/:id, Axum returns
/// plain text error: "Invalid URL: Cannot parse \"abc\" to a `i64`" instead of JSON
///
/// **Spec:** FR-5 requires all error responses to be JSON format
///
/// **Root Cause:** Same as limits endpoint - Axum's default path parameter rejection
///
/// **Pitfall:** All endpoints with numeric path parameters need consistent JSON error handling
#[ tokio::test ]
async fn test_get_trace_with_non_numeric_id_returns_json_error()
{
  let app = build_test_app().await;

  // Test: GET /api/traces/abc (non-numeric ID)
  let request = axum::http::Request::builder()
    .uri( "/api/traces/abc" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return 400 Bad Request
  assert_eq!( response.status(), StatusCode::BAD_REQUEST,
    "Non-numeric trace ID should return 400 Bad Request" );

  // Response should be JSON (not plain text)
  let content_type = response.headers()
    .get( "content-type" )
    .and_then( |v| v.to_str().ok() )
    .unwrap_or( "" );

  assert!( content_type.contains( "application/json" ),
    "Error response should have Content-Type: application/json, got: {}", content_type );

  // Response body should be valid JSON
  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice( &body )
    .expect( "Error response should be valid JSON, got plain text" );

  // Verify JSON has error field
  assert!( json.get( "error" ).is_some(),
    "Error response should have 'error' field per FR-5" );

  // Error message should be descriptive
  let error_msg = json[ "error" ].as_str().unwrap();
  assert!( !error_msg.is_empty(), "Error message should not be empty" );
}
