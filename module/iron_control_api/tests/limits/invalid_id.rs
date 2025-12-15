//! Tests for invalid ID parameter handling in limits endpoints
//!
//! Verifies that non-numeric IDs return JSON error responses per FR-5 specification.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_get_limit_with_non_numeric_id_returns_json_error` | Get limit with non-numeric ID | GET /api/limits/abc | 400 Bad Request, JSON error with deserialize message | ✅ |
//! | `test_update_limit_with_non_numeric_id_returns_json_error` | Update limit with non-numeric ID | PUT /api/limits/xyz | 400 Bad Request, JSON error with deserialize message | ✅ |
//! | `test_delete_limit_with_non_numeric_id_returns_json_error` | Delete limit with non-numeric ID | DELETE /api/limits/foo | 400 Bad Request, JSON error with deserialize message | ✅ |

use axum::{ http::StatusCode, body::Body, Router };
use tower::ServiceExt;
use http_body_util::BodyExt;

/// Build minimal test app with limits routes for testing
async fn build_test_app() -> Router
{
  use axum::routing::{ get, post };

  let limits_state = iron_control_api::routes::limits::LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to initialize limits state" );

  Router::new()
    .route( "/api/limits", get( iron_control_api::routes::limits::list_limits ) )
    .route( "/api/limits", post( iron_control_api::routes::limits::create_limit ) )
    .route( "/api/limits/:id", get( iron_control_api::routes::limits::get_limit ) )
    .route( "/api/limits/:id", axum::routing::put( iron_control_api::routes::limits::update_limit ) )
    .route( "/api/limits/:id", axum::routing::delete( iron_control_api::routes::limits::delete_limit ) )
    .with_state( limits_state )
}

/// Reproduces Issue #2: Non-JSON error response for invalid path parameters
///
/// **Bug:** When non-numeric ID is provided to GET /api/limits/:id, Axum returns
/// plain text error: "Invalid URL: Cannot parse \"abc\" to a `i64`" instead of JSON
///
/// **Spec:** FR-5 requires all error responses to be JSON format with structure:
/// `{"error": "...", "code": "...", "details": "..."}`
///
/// **Impact:**
/// - Violates API contract (FR-5)
/// - Frontend expecting JSON will fail to parse error
/// - Inconsistent error handling across endpoints
///
/// **Root Cause:**
/// Axum's default path parameter rejection handler returns plain text errors.
/// Need custom rejection handler to convert to JSON format.
///
/// **Pitfall:**
/// Axum's built-in extractors (Path, Query, etc.) have default rejection responses
/// that don't match custom error formats. Always implement custom rejection handling
/// for consistent API error responses.
#[ tokio::test ]
async fn test_get_limit_with_non_numeric_id_returns_json_error()
{
  let app = build_test_app().await;

  // Test: GET /api/limits/abc (non-numeric ID)
  let request = axum::http::Request::builder()
    .uri( "/api/limits/abc" )
    .body( Body::empty() )
    .unwrap();

  let response = app.clone().oneshot( request ).await.unwrap();

  // Should return 400 Bad Request
  assert_eq!( response.status(), StatusCode::BAD_REQUEST,
    "Non-numeric ID should return 400 Bad Request" );

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

/// Verifies UPDATE endpoint with non-numeric ID returns JSON error
#[ tokio::test ]
async fn test_update_limit_with_non_numeric_id_returns_json_error()
{
  let app = build_test_app().await;

  let request = axum::http::Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/xyz" )
    .header( "content-type", "application/json" )
    .body( Body::from( r#"{"user_id":"user_1","max_cost_per_month_microdollars":1000}"# ) )
    .unwrap();

  let response = app.clone().oneshot( request ).await.unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST );

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice( &body )
    .expect("LOUD FAILURE: Error response should be valid JSON");

  assert!( json.get( "error" ).is_some() );
}

/// Verifies DELETE endpoint with non-numeric ID returns JSON error
#[ tokio::test ]
async fn test_delete_limit_with_non_numeric_id_returns_json_error()
{
  let app = build_test_app().await;

  let request = axum::http::Request::builder()
    .method( "DELETE" )
    .uri( "/api/limits/invalid" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST );

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: serde_json::Value = serde_json::from_slice( &body )
    .expect("LOUD FAILURE: Error response should be valid JSON");

  assert!( json.get( "error" ).is_some() );
}
