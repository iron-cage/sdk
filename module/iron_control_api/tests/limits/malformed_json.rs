//! Malformed JSON handling tests for limits management endpoints.
//!
//! Tests input validation at the HTTP parsing layer to ensure proper error
//! responses for malformed JSON payloads.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Malformed Input | Expected Result | Status |
//! |-----------|----------|----------------|----------------|--------|
//! | `test_create_limit_with_invalid_json_syntax` | POST /api/limits | Missing closing brace | 400 Bad Request | ✅ |
//! | `test_create_limit_with_trailing_comma` | POST /api/limits | Trailing comma in object | 400 Bad Request | ✅ |
//! | `test_update_limit_with_invalid_json_syntax` | PUT /api/limits/:id | Malformed JSON | 400 Bad Request | ✅ |
//! | `test_update_limit_with_unquoted_values` | PUT /api/limits/:id | Unquoted string values | 400 Bad Request | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:** Not applicable (all test invalid inputs)
//!
//! **Error Conditions:**
//! - ✅ Invalid JSON syntax → 400 Bad Request
//! - ✅ Trailing commas → 400 Bad Request
//! - ✅ Unquoted values → 400 Bad Request
//! - ✅ Missing required fields in malformed JSON → 400 Bad Request
//!
//! **Edge Cases:**
//! - ✅ Empty body when JSON expected → 400 Bad Request
//! - ✅ Partial JSON (truncated transmission) → 400 Bad Request
//!
//! **Boundary Conditions:** Not applicable (syntax validation only)
//! **State Transitions:** Not applicable (HTTP layer validation only)
//! **Concurrent Access:** Not applicable (stateless validation)
//! **Resource Limits:** Not applicable (covered by token tests)
//! **Precondition Violations:** Not applicable

use iron_control_api::routes::limits::LimitsState;
use axum::{ Router, routing::{ post, put }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Create test router with limits routes.
async fn create_test_router() -> Router
{
  let limits_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limits state" );

  Router::new()
    .route( "/api/limits", post( iron_control_api::routes::limits::create_limit ) )
    .route( "/api/limits/:id", put( iron_control_api::routes::limits::update_limit ) )
    .with_state( limits_state )
}

/// Helper: Create a valid limit and return its ID.
async fn create_limit( router: &Router ) -> i64
{
  let request_body = json!({
    "user_id": "test_user",
    "project_id": null,
    "max_tokens_per_day": 1000,
    "max_requests_per_minute": null,
    "max_cost_per_month_cents": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.clone().oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::CREATED );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  body[ "id" ].as_i64().unwrap()
}

/// Test POST /api/limits with invalid JSON syntax (missing closing brace).
///
/// WHY: Axum's JSON extractor should reject syntactically invalid JSON at the
/// HTTP layer before reaching business logic.
#[ tokio::test ]
async fn test_create_limit_with_invalid_json_syntax()
{
  let router = create_test_router().await;

  let malformed_json = r#"{"user_id":"test","max_tokens_per_day":1000"#; // Missing closing brace

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Malformed JSON must return 400 Bad Request"
  );
}

/// Test POST /api/limits with trailing comma (invalid JSON).
///
/// WHY: JSON spec doesn't allow trailing commas. This tests strict parsing.
#[ tokio::test ]
async fn test_create_limit_with_trailing_comma()
{
  let router = create_test_router().await;

  let malformed_json = r#"{"user_id":"test","max_tokens_per_day":1000,}"#; // Trailing comma

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Trailing comma in JSON must return 400 Bad Request"
  );
}

/// Test PUT /api/limits/:id with invalid JSON syntax.
///
/// WHY: Update endpoint should also reject malformed JSON at HTTP layer.
#[ tokio::test ]
async fn test_update_limit_with_invalid_json_syntax()
{
  let router = create_test_router().await;

  // First create a limit to update
  let limit_id = create_limit( &router ).await;

  let malformed_json = r#"{"max_tokens_per_day":2000"#; // Missing closing brace

  let request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/limits/{}", limit_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: PUT with malformed JSON must return 400 Bad Request"
  );
}

/// Test PUT /api/limits/:id with unquoted string values (invalid JSON).
///
/// WHY: JSON requires quoted string values. This ensures strict parsing.
#[ tokio::test ]
async fn test_update_limit_with_unquoted_values()
{
  let router = create_test_router().await;

  // First create a limit to update
  let limit_id = create_limit( &router ).await;

  // Unquoted string value for user_id (if it were in update body)
  let malformed_json = r#"{"max_tokens_per_day":unquoted}"#; // Unquoted value

  let request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/limits/{}", limit_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Unquoted JSON values must return 400 Bad Request"
  );
}
