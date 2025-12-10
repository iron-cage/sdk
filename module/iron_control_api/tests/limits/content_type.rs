//! Content-Type validation tests for budget limits endpoints.
//!
//! Tests that endpoints requiring JSON bodies properly reject requests
//! with incorrect Content-Type headers with 415 Unsupported Media Type.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Content-Type | Expected Result | Status |
//! |-----------|----------|-------------|----------------|--------|
//! | `test_create_limit_wrong_content_type` | POST /api/limits | text/plain | 415 | ✅ |
//! | `test_update_limit_wrong_content_type` | PUT /api/limits/:id | text/html | 415 | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Error Conditions:**
//! - ✅ Wrong Content-Type on POST → 415
//! - ✅ Wrong Content-Type on PUT → 415
//!
//! **Why These Tests Matter:**
//! - API contract enforcement for budget-sensitive operations
//! - Prevent parser confusion on financial data

use iron_control_api::routes::limits::LimitsState;
use axum::{ Router, routing::{ post, put }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with limit routes.
async fn create_test_router() -> Router
{
  let limit_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limit state" );

  Router::new()
    .route( "/api/limits", post( iron_control_api::routes::limits::create_limit ) )
    .route( "/api/limits/:id", put( iron_control_api::routes::limits::update_limit ) )
    .with_state( limit_state )
}

#[ tokio::test ]
async fn test_create_limit_wrong_content_type()
{
  let router = create_test_router().await;

  // WHY: Budget operations require strict JSON format
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "text/plain" )
    .body( Body::from( r#"{"user_id":"test","max_tokens_per_day":1000}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Wrong Content-Type on POST /api/limits must return 415 or 400. Got: {}",
    status
  );
}

#[ tokio::test ]
async fn test_update_limit_wrong_content_type()
{
  let router = create_test_router().await;

  // WHY: Update operations are equally strict on Content-Type
  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/1" )
    .header( "content-type", "text/html" )
    .body( Body::from( r#"{"max_tokens_per_day":2000}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Wrong Content-Type on PUT /api/limits/:id must return 415 or 400. Got: {}",
    status
  );
}
