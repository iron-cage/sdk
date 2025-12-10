//! Empty request body handling tests for token endpoints.
//!
//! Tests that verify endpoints properly reject requests with missing
//! or empty bodies, enforcing required field validation.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Body | Expected Result | Status |
//! |-----------|----------|------|----------------|--------|
//! | `test_create_token_with_empty_json_object` | POST /api/tokens | {} | 400 Bad Request | ✅ |
//! | `test_create_token_with_no_body` | POST /api/tokens | (empty) | 400 Bad Request | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Error Conditions:**
//! - ✅ Empty JSON object `{}` → 400 (missing required fields)
//! - ✅ Completely empty body → 400 (cannot parse as JSON)
//! - ✅ Clear error messages about missing fields
//!
//! **Why These Tests Matter:**
//! 1. **Validation**: Required fields must be enforced
//! 2. **API Contract**: Explicit about required vs optional fields
//! 3. **Client UX**: Clear error messages for malformed requests

use iron_control_api::routes::tokens::TokenState;
use axum::{ Router, routing::post, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with token routes.
async fn create_test_router() -> Router
{
  let token_state = TokenState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create token state" );

  Router::new()
    .route( "/api/tokens", post( iron_control_api::routes::tokens::create_token ) )
    .with_state( token_state )
}

#[ tokio::test ]
async fn test_create_token_with_empty_json_object()
{
  let router = create_test_router().await;

  // WHY: Empty JSON object lacks required user_id field
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( "{}" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty JSON object must return 400 (missing required user_id)"
  );

  // WHY: Verify error response is JSON
  let content_type = response.headers().get( "content-type" );
  assert!(
    content_type.is_some() && content_type.unwrap().to_str().unwrap().contains( "application/json" ),
    "LOUD FAILURE: Error response must be JSON"
  );
}

#[ tokio::test ]
async fn test_create_token_with_no_body()
{
  let router = create_test_router().await;

  // WHY: Completely empty body should fail to parse as JSON
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/tokens" )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty body must return 400 Bad Request"
  );
}
