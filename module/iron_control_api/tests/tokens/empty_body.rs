//! Empty request body handling tests for token endpoints.
//!
//! Tests that verify endpoints properly handle requests with empty bodies.
//!
//! ## Test Matrix (Protocol 014)
//!
//! | Test Case | Endpoint | Body | Expected Result | Status |
//! |-----------|----------|------|----------------|--------|
//! | `test_create_token_with_empty_json_object` | POST /api/v1/api-tokens | {} | 201 Created | ✅ |
//! | `test_create_token_with_no_body` | POST /api/v1/api-tokens | (empty) | 400 Bad Request | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Protocol 014 Changes:**
//! - ✅ Empty JSON object `{}` → 201 Created (user_id from JWT, all fields optional)
//! - ✅ Completely empty body → 400 (cannot parse as JSON)
//!
//! **Why These Tests Matter:**
//! 1. **Protocol 014**: user_id extracted from JWT, not required in request
//! 2. **API Contract**: All request fields are optional (name, project_id, description)
//! 3. **Client UX**: Minimal request `{}` with JWT auth is valid

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
async fn test_create_token_with_empty_json_object()
{
  let ( router, app_state ) = create_test_router().await;

  // WHY: Protocol 014 - Empty JSON object is valid (user_id from JWT, all fields optional)
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( "{}" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Empty JSON object with JWT auth must return 201 Created (Protocol 014)"
  );

  // WHY: Verify response is JSON with token
  let content_type = response.headers().get( "content-type" );
  assert!(
    content_type.is_some() && content_type.unwrap().to_str().unwrap().contains( "application/json" ),
    "LOUD FAILURE: Success response must be JSON"
  );
}

#[ tokio::test ]
async fn test_create_token_with_no_body()
{
  let ( router, app_state ) = create_test_router().await;

  // WHY: Completely empty body should fail to parse as JSON
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty body must return 400 Bad Request"
  );
}
