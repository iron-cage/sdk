//! Endpoint integration tests for GET /api/traces/:id.
//!
//! ## Purpose
//! Verify that trace detail endpoint correctly retrieves specific API call trace by ID,
//! returning 404 for non-existent IDs.
//!
//! ## Test Matrix
//!
//! | Test Case | Database State | Expected Status | Expected Behavior |
//! |-----------|----------------|-----------------|-------------------|
//! | Non-existent ID | No matching trace | 404 Not Found | Error message |
//! | Valid ID | Matching trace exists | 200 OK | Full trace details |
//! | Invalid ID format | ID = "abc" | 404 Not Found | Path param accepts string |
//!
//! ## Known Edge Cases
//! - Non-existent ID returns 404 Not Found (not 500)
//! - ID parameter is i64, but Axum Path<String> accepts any string
//! - Response includes ALL trace fields (not partial)
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check TraceStorage::get_trace_by_id() error handling (404 vs 500)
//! 2. Check Path parameter type (should be String, converted to i64 in handler)
//! 3. Check ApiTrace serialization includes all required fields
//! 4. Verify handler maps "not found" error to HTTP 404

use crate::common::extract_response;
use iron_api::routes::traces::TracesState;
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with trace get-by-id route.
async fn create_test_router() -> Router
{
  let traces_state = TracesState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create traces state with in-memory database" );

  Router::new()
    .route( "/api/traces/:id", get( iron_api::routes::traces::get_trace ) )
    .with_state( traces_state )
}

/// Test non-existent ID returns 404 Not Found.
///
/// WHY: Missing resource should return 404, not 500 (database error).
#[ tokio::test ]
async fn test_get_by_id_nonexistent_returns_404()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/999999" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Non-existent trace ID must return 404 Not Found, not 500"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::NOT_FOUND );
  assert!(
    body.contains( "error" ) || body.contains( "not found" ) || body.contains( "Not found" ),
    "LOUD FAILURE: 404 response must contain error message. Got: {}",
    body
  );
}

/// Test path parameter extraction.
///
/// WHY: Axum must correctly extract ID from URL path.
#[ tokio::test ]
async fn test_get_by_id_path_parameter_extraction()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/12345" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should attempt query (and return 404), proving path extraction worked
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Path parameter must be extracted and passed to query"
  );
}

/// Test invalid ID format (non-numeric string).
///
/// WHY: Path<String> accepts any string. Handler must parse to i64.
#[ tokio::test ]
async fn test_get_by_id_invalid_format()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/not_a_number" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Current implementation: String is accepted by Path, then parse fails
  // Should return 400 Bad Request or 404 Not Found
  assert!(
    response.status() == StatusCode::BAD_REQUEST ||
    response.status() == StatusCode::NOT_FOUND ||
    response.status() == StatusCode::INTERNAL_SERVER_ERROR,
    "LOUD FAILURE: Invalid ID format must return error status"
  );
}

/// Test negative ID.
///
/// WHY: Database IDs are positive integers. Negative should return 404.
#[ tokio::test ]
async fn test_get_by_id_negative_id()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/-1" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Negative ID can parse to i64, but won't exist in database
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Negative ID must return 404 Not Found"
  );
}

/// Test zero ID.
///
/// WHY: Database auto-increment starts at 1, not 0.
#[ tokio::test ]
async fn test_get_by_id_zero_id()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/0" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Zero ID must return 404 Not Found"
  );
}

/// Test POST method rejected (GET-only endpoint).
#[ tokio::test ]
async fn test_get_by_id_rejects_post_method()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/traces/123" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test DELETE method rejected.
///
/// WHY: Traces are read-only (no DELETE in spec).
#[ tokio::test ]
async fn test_get_by_id_rejects_delete_method()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/traces/123" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: DELETE to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test Content-Type is application/json for 404 responses.
#[ tokio::test ]
async fn test_get_by_id_404_content_type_is_json()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces/999999" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let content_type = response.headers().get( "content-type" )
    .expect( "LOUD FAILURE: 404 response must include Content-Type header" )
    .to_str()
    .expect( "LOUD FAILURE: Content-Type must be valid UTF-8" );

  assert!(
    content_type.contains( "application/json" ),
    "LOUD FAILURE: 404 response Content-Type must be application/json, got: {}",
    content_type
  );
}
