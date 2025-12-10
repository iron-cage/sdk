//! Empty request body handling tests for limits endpoints.
//!
//! Tests that verify endpoints properly reject requests with empty
//! bodies, enforcing the "all-None rejected" validation rule.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Body | Expected Result | Status |
//! |-----------|----------|------|----------------|--------|
//! | `test_create_limit_with_empty_json` | POST /api/limits | {} | 422 Unprocessable Entity | ✅ |
//! | `test_update_limit_with_empty_json` | PUT /api/limits/:id | {} | 422 Unprocessable Entity | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Error Conditions:**
//! - ✅ Empty JSON on POST → 422 (all-None rejected)
//! - ✅ Empty JSON on PUT → 422 (all-None rejected)
//! - ✅ Validation ensures at least one field is set
//!
//! **Why These Tests Matter:**
//! 1. **Validation**: "all-None rejected" rule enforcement
//! 2. **API Contract**: At least one field required for limits
//! 3. **Business Logic**: Empty updates/creates make no sense

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
async fn test_create_limit_with_empty_json()
{
  let router = create_test_router().await;

  // WHY: Empty JSON has all fields as None, violating "all-None rejected" rule
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/limits" )
    .header( "content-type", "application/json" )
    .body( Body::from( "{}" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // WHY: 422 Unprocessable Entity is semantically correct for validation errors
  // (well-formed JSON but semantically invalid - all fields None)
  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Empty JSON must return 422 Unprocessable Entity (all-None rejected rule)"
  );
}

#[ tokio::test ]
async fn test_update_limit_with_empty_json()
{
  let router = create_test_router().await;

  // WHY: Empty JSON for update means no fields to update (all-None rejected)
  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/limits/1" )
    .header( "content-type", "application/json" )
    .body( Body::from( "{}" ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // WHY: 422 Unprocessable Entity is semantically correct for validation errors
  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Empty JSON update must return 422 Unprocessable Entity (all-None rejected rule)"
  );
}
