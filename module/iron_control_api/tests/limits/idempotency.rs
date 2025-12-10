//! Idempotency tests for budget limits endpoints.
//!
//! Tests that verify DELETE operations are FULLY idempotent (repeated calls
//! return 204) and document the API contract.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Operation | Expected Result | Status |
//! |-----------|----------|-----------|----------------|--------|
//! | `test_delete_limit_twice_returns_204` | DELETE /api/limits/:id | Delete twice | Both 204 | ✅ |
//! | `test_delete_nonexistent_limit_returns_204` | DELETE /api/limits/:id | Delete non-existent | 204 No Content | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Idempotency:**
//! - ✅ DELETE is FULLY idempotent (always returns 204)
//! - ✅ Same result whether resource exists or not
//! - ⚠️ **IMPLEMENTATION INCONSISTENCY**: Token DELETE returns 404, Limits DELETE returns 204
//!
//! **Why These Tests Matter:**
//! 1. **API Contract**: Documents actual idempotency behavior
//! 2. **Client Design**: Clients can safely retry DELETE
//! 3. **Consistency**: Reveals that tokens and limits have different DELETE semantics
//!
//! **Note**: This behavior differs from token DELETE which returns 404 for
//! non-existent tokens. This is an implementation-level decision where limits
//! chose full idempotency (HTTP-compliant) while tokens chose REST semantics.

use iron_control_api::routes::limits::LimitsState;
use axum::{ Router, routing::{ post, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Create test router with limit routes.
async fn create_test_router() -> Router
{
  let limit_state = LimitsState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create limit state" );

  Router::new()
    .route( "/api/limits", post( iron_control_api::routes::limits::create_limit ) )
    .route( "/api/limits/:id", delete( iron_control_api::routes::limits::delete_limit ) )
    .with_state( limit_state )
}

/// Helper: Create a limit and return its ID.
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
  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Failed to create test limit"
  );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  body[ "id" ].as_i64().unwrap()
}

#[ tokio::test ]
async fn test_delete_limit_twice_returns_204()
{
  let router = create_test_router().await;

  // Create a limit
  let limit_id = create_limit( &router ).await;

  // WHY: First delete should succeed
  let request1 = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/limits/{}", limit_id ) )
    .body( Body::empty() )
    .unwrap();

  let response1 = router.clone().oneshot( request1 ).await.unwrap();
  assert_eq!(
    response1.status(),
    StatusCode::NO_CONTENT,
    "LOUD FAILURE: First DELETE must return 204 No Content"
  );

  // WHY: Second delete also returns 204 (FULLY IDEMPOTENT)
  // This differs from token DELETE which returns 404
  // Both behaviors are valid, but inconsistent across endpoints
  let request2 = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/limits/{}", limit_id ) )
    .body( Body::empty() )
    .unwrap();

  let response2 = router.oneshot( request2 ).await.unwrap();
  assert_eq!(
    response2.status(),
    StatusCode::NO_CONTENT,
    "LOUD FAILURE: Second DELETE returns 204 (fully idempotent behavior). Note: tokens return 404."
  );
}

#[ tokio::test ]
async fn test_delete_nonexistent_limit_returns_204()
{
  let router = create_test_router().await;

  // WHY: Deleting a limit that never existed returns 204 (fully idempotent)
  // This means DELETE always succeeds, whether resource exists or not
  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/limits/999999" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();
  assert_eq!(
    response.status(),
    StatusCode::NO_CONTENT,
    "LOUD FAILURE: DELETE of nonexistent limit returns 204 (fully idempotent). Differs from token behavior."
  );
}
