//! Idempotency tests for token management endpoints.
//!
//! Tests that verify token creation is intentionally NON-idempotent
//! (same data produces different tokens) for security, and that
//! DELETE operations have consistent idempotency semantics.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Operation | Expected Result | Status |
//! |-----------|----------|-----------|----------------|--------|
//! | `test_create_token_same_data_produces_different_tokens` | POST /api/v1/api-tokens | Create twice with same data | Two different tokens | ✅ |
//! | `test_revoke_nonexistent_token_returns_404` | DELETE /api/v1/api-tokens/:id | Delete non-existent | 404 Not Found | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Non-Idempotency (by design):**
//! - ✅ POST /api/v1/api-tokens is NOT idempotent
//! - ✅ Same user_id + project_id → Different token each time
//! - ✅ Security: Prevents token prediction
//!
//! **Idempotency:**
//! - ✅ DELETE of non-existent token → 404 (consistent)
//!
//! **Why These Tests Matter:**
//! 1. **Security**: Token uniqueness prevents prediction attacks
//! 2. **API Contract**: Clients expect new token on each POST
//! 3. **RESTful Semantics**: POST creates new resource, not idempotent
//!
//! **Note**: Token revocation idempotency (delete twice → 404) is tested
//! in `state_transitions.rs::test_revoke_already_revoked_token`.

use iron_control_api::routes::tokens::CreateTokenResponse;
use axum::{ Router, routing::{ post, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with token routes.
async fn create_test_router() -> ( Router, crate::common::test_state::TestAppState )
{
  // Create test application state with auth + token support
  let app_state = crate::common::test_state::TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

#[ tokio::test ]
async fn test_create_token_same_data_produces_different_tokens()
{
  let ( router, _app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "test_user",
    "project_id": "test_project",
  });

  // WHY: Create first token
  let request1 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response1 = router.clone().oneshot( request1 ).await.unwrap();
  assert_eq!(
    response1.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: First token creation must succeed"
  );

  let body1_bytes = axum::body::to_bytes( response1.into_body(), usize::MAX ).await.unwrap();
  let body1: CreateTokenResponse = serde_json::from_slice( &body1_bytes ).unwrap();

  // WHY: Create second token with SAME data
  let request2 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response2 = router.oneshot( request2 ).await.unwrap();
  assert_eq!(
    response2.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Second token creation must succeed"
  );

  let body2_bytes = axum::body::to_bytes( response2.into_body(), usize::MAX ).await.unwrap();
  let body2: CreateTokenResponse = serde_json::from_slice( &body2_bytes ).unwrap();

  // WHY: Tokens MUST be different (security requirement)
  assert_ne!(
    body1.id,
    body2.id,
    "LOUD FAILURE: Token IDs must be different"
  );

  assert_ne!(
    body1.token,
    body2.token,
    "LOUD FAILURE: Token values must be different for security (no predictability)"
  );

  // WHY: Both tokens should be for same user/project but with unique values
  // This is intentional NON-idempotency for security
}

#[ tokio::test ]
async fn test_revoke_nonexistent_token_returns_404()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for a test user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  // WHY: Revoking a token that never existed should return 404
  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/v1/api-tokens/999999" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: DELETE of nonexistent token must return 404 Not Found"
  );
}
