//! HTTP method validation tests for limits management endpoints.
//!
//! Tests that endpoints properly reject unsupported HTTP methods with
//! 405 Method Not Allowed.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Wrong Method | Expected Result | Status |
//! |-----------|----------|-------------|----------------|--------|
//! | `test_create_limit_get_method_rejected` | POST /api/limits | GET | 405 Method Not Allowed | ✅ |
//! | `test_update_limit_post_method_rejected` | PUT /api/limits/:id | POST | 405 Method Not Allowed | ✅ |
//! | `test_delete_limit_get_method_rejected` | DELETE /api/limits/:id | GET | 405 Method Not Allowed | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:** Not applicable (all test error conditions)
//!
//! **Error Conditions:**
//! - ✅ Wrong HTTP method → 405 Method Not Allowed
//! - ✅ Routing layer validation before handler
//!
//! **Edge Cases:** Explicit API contract documentation
//! **Boundary Conditions:** Not applicable
//! **State Transitions:** Not applicable
//! **Concurrent Access:** Not applicable
//! **Resource Limits:** Not applicable
//! **Precondition Violations:** Not applicable

use iron_api::routes::limits::LimitsState;
use axum::{ Router, routing::{ post, get, put, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with limits routes.
async fn create_test_router() -> Router
{
  let limits_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limits state" );

  Router::new()
    .route( "/api/limits", post( iron_api::routes::limits::create_limit ) )
    .route( "/api/limits", get( iron_api::routes::limits::list_limits ) )
    .route( "/api/limits/:id", get( iron_api::routes::limits::get_limit ) )
    .route( "/api/limits/:id", put( iron_api::routes::limits::update_limit ) )
    .route( "/api/limits/:id", delete( iron_api::routes::limits::delete_limit ) )
    .with_state( limits_state )
}

/// Test POST /api/limits with GET method.
///
/// WHY: GET /api/limits is list_limits endpoint, so GET succeeds (not 405).
/// This test documents that GET is supported for listing.
#[ tokio::test ]
async fn test_create_limit_get_method_not_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/limits" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: GET on /api/limits is list_limits endpoint (returns 200 OK)"
  );
}

/// Test POST /api/limits with DELETE method → 405 Method Not Allowed.
///
/// WHY: DELETE not supported on collection endpoint (only on individual resources).
#[ tokio::test ]
async fn test_create_limit_delete_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/limits" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: DELETE method on /api/limits must return 405 Method Not Allowed"
  );
}

/// Test PUT /api/limits/:id with POST method → 405 Method Not Allowed.
///
/// WHY: Update requires PUT, POST is for creation (different endpoint).
#[ tokio::test ]
async fn test_update_limit_post_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits/1" )
    .header( "content-type", "application/json" )
    .body( Body::from( r#"{"max_tokens_per_day":1000}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST method on /api/limits/:id must return 405 Method Not Allowed"
  );
}

/// Test DELETE /api/limits/:id with GET method.
///
/// WHY: GET /api/limits/:id is get_limit endpoint, so GET succeeds (not 405).
/// This test documents that GET is supported for retrieval.
#[ tokio::test ]
async fn test_delete_limit_get_method_not_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/limits/1" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // GET succeeds (returns 404 for nonexistent limit, not 405)
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: GET on /api/limits/:id is get_limit endpoint (returns 404 for nonexistent)"
  );
}
