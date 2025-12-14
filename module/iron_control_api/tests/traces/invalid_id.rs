//! Tests for invalid ID parameter handling in traces endpoints
//!
//! Verifies that non-numeric IDs return JSON error responses per FR-5 specification.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_get_trace_with_non_numeric_id_returns_json_error` | Get trace with non-numeric ID | GET /api/traces/abc with auth | 400 Bad Request, JSON error | ✅ |

use axum::{ http::StatusCode, body::Body, Router };
use tower::ServiceExt;
use http_body_util::BodyExt;
use crate::common::test_state::TestTracesAppState;

/// Generate JWT token for test user
fn generate_jwt_for_user( app_state: &TestTracesAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Build minimal test app with traces routes for testing
async fn build_test_app() -> ( Router, TestTracesAppState )
{
  use axum::routing::get;

  let app_state = TestTracesAppState::new().await;

  let router = Router::new()
    .route( "/api/traces", get( iron_control_api::routes::traces::list_traces ) )
    .route( "/api/traces/:id", get( iron_control_api::routes::traces::get_trace ) )
    .with_state( app_state.clone() );

  ( router, app_state )
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
  let ( app, app_state ) = build_test_app().await;
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  // Test: GET /api/traces/abc (non-numeric ID)
  let request = axum::http::Request::builder()
    .uri( "/api/traces/abc" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
    .expect("LOUD FAILURE: Error response should be valid JSON, got plain text");

  // Verify JSON has error field
  assert!( json.get( "error" ).is_some(),
    "Error response should have 'error' field per FR-5" );

  // Error message should be descriptive
  let error_msg = json[ "error" ].as_str().unwrap();
  assert!( !error_msg.is_empty(), "Error message should not be empty" );
}
