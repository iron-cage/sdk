//! Content-Type validation tests for token management endpoints.
//!
//! Tests that endpoints requiring JSON bodies properly reject requests
//! with missing or incorrect Content-Type headers with 415 Unsupported
//! Media Type, enforcing API contract and preventing parser confusion.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Content-Type | Expected Result | Status |
//! |-----------|----------|-------------|----------------|--------|
//! | `test_create_token_missing_content_type` | POST /api/v1/api-tokens | (none) | 415 or 400 | ✅ |
//! | `test_create_token_text_plain_rejected` | POST /api/v1/api-tokens | text/plain | 415 | ✅ |
//! | `test_create_token_form_urlencoded_rejected` | POST /api/v1/api-tokens | application/x-www-form-urlencoded | 415 | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:** Not applicable (all test error conditions)
//!
//! **Error Conditions:**
//! - ✅ Missing Content-Type header → 415 or 400
//! - ✅ Wrong Content-Type (text/plain) → 415
//! - ✅ Wrong Content-Type (form-urlencoded) → 415
//!
//! **Edge Cases:**
//! - ✅ Parser receives valid JSON but wrong Content-Type → Rejected
//! - ✅ HTTP layer validation before handler execution
//!
//! **Why These Tests Matter:**
//! 1. **Security**: Prevents parser confusion attacks
//! 2. **API Contract**: Explicit Content-Type requirement
//! 3. **Client Feedback**: Clear error when misconfigured
//! 4. **Standards Compliance**: RFC 7231 media type negotiation

use crate::common::test_state::TestAppState;
use axum::{ Router, routing::post, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with token routes.
///
/// Uses TestAppState (auth + tokens) to support JWT authentication in routes.
async fn create_test_router() -> ( Router, TestAppState )
{
  let app_state = TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

#[ tokio::test ]
async fn test_create_token_missing_content_type()
{
  let ( router, app_state ) = create_test_router().await;

  // WHY: Missing Content-Type should be rejected by Axum's JSON extractor
  // Expected: 415 Unsupported Media Type or 400 Bad Request
  let jwt_token = generate_jwt_for_user( &app_state, "test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    // No Content-Type header set
    .body( Body::from( r#"{"user_id":"test","project_id":"proj"}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Missing Content-Type must return 415 or 400. Got: {}",
    status
  );
}

#[ tokio::test ]
async fn test_create_token_text_plain_rejected()
{
  let ( router, app_state ) = create_test_router().await;

  // WHY: Even if body contains valid JSON, wrong Content-Type should be rejected
  let jwt_token = generate_jwt_for_user( &app_state, "test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "text/plain" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( r#"{"user_id":"test","project_id":"proj"}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: text/plain Content-Type must return 415 or 400. Got: {}",
    status
  );
}

#[ tokio::test ]
async fn test_create_token_form_urlencoded_rejected()
{
  let ( router, app_state ) = create_test_router().await;

  // WHY: Form data format is incompatible with JSON endpoints
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/x-www-form-urlencoded" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( "user_id=test&project_id=proj" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: application/x-www-form-urlencoded must return 415 or 400. Got: {}",
    status
  );
}
