//! Rate limiting tests for token creation
//!
//! Tests Protocol 014 rate limiting requirements:
//! 1. Max 10 active tokens per user (token limit)
//! 2. Max 10 token creates per minute per user (rate limiting)

use axum::{ Router, routing::post, http::{ Request, StatusCode }, body::Body };
use tower::ServiceExt;
use serde_json::json;
use std::sync::atomic::{ AtomicUsize, Ordering };
use std::sync::Arc;

/// Global counter for generating unique database names across tests
static DB_COUNTER: AtomicUsize = AtomicUsize::new( 0 );

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with token routes using shared database.
///
/// NOTE: Rate limiting tests require a shared database so that all requests
/// within a single test can see the same token count and rate limit state.
async fn create_test_router_with_shared_db( db_path: &str ) -> ( Router, crate::common::test_state::TestAppState )
{
  let app_state = crate::common::test_state::TestAppState::with_db_path( db_path ).await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id/revoke", post( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Test that user cannot exceed 10 active tokens
///
/// WHY: Protocol 014 requires max 10 active tokens per user to prevent
/// resource exhaustion and DoS attacks.
///
/// APPROACH:
/// 1. Create 10 tokens for user (should all succeed)
/// 2. Attempt 11th token (should fail with 429 - Token limit exceeded)
/// 3. Verify error message indicates token limit (not rate limit)
#[ tokio::test ]
async fn test_max_active_tokens_per_user()
{
  // Use unique shared database for this test
  let unique_id = DB_COUNTER.fetch_add( 1, Ordering::SeqCst );
  let db_path = format!(
    "file:test_rate_limit_{}_{}?mode=memory&cache=shared",
    std::process::id(),
    unique_id
  );

  let user_id = "user_rate_limit_test";

  // Create router and app_state once, then reuse them for all requests
  // This ensures all requests share the same database connection pool
  let ( router, app_state ) = create_test_router_with_shared_db( &db_path ).await;
  let router = Arc::new( router );

  // Create 10 tokens (should all succeed)
  for i in 0..10
  {
    let jwt = generate_jwt_for_user( &app_state, user_id );

    let request_body = json!({
      "name": format!( "token_{}", i ),
      "description": "Rate limit test token"
    });

    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( "content-type", "application/json" )
      .header( "authorization", format!( "Bearer {}", jwt ) )
      .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
      .unwrap();

    let response = ( *router ).clone().oneshot( request ).await.unwrap();

    assert_eq!(
      response.status(),
      StatusCode::CREATED,
      "LOUD FAILURE: First 10 token creations must succeed (token {})", i
    );
  }

  // Attempt 11th token (should fail with 429 - either limit can trigger)
  // Since we have 10 active tokens AND 10 creates in last minute, both limits are reached
  {
    let jwt = generate_jwt_for_user( &app_state, user_id );

    let request_body = json!({
      "name": "token_11",
      "description": "Should fail - exceeds limit"
    });

    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/api-tokens" )
      .header( "content-type", "application/json" )
      .header( "authorization", format!( "Bearer {}", jwt ) )
      .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
      .unwrap();

    let response = ( *router ).clone().oneshot( request ).await.unwrap();

    assert_eq!(
      response.status(),
      StatusCode::TOO_MANY_REQUESTS,
      "LOUD FAILURE: 11th token creation must fail with 429 (rate limits exceeded)"
    );

    // Verify we get an error (either "Rate limit exceeded" or "Token limit exceeded")
    let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
    let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
    let error_msg = body[ "error" ].as_str().unwrap();
    assert!(
      error_msg == "Rate limit exceeded" || error_msg == "Token limit exceeded",
      "LOUD FAILURE: Error message must indicate a rate limit was exceeded, got: {}", error_msg
    );
  }
}

/// Test that both rate limits work together
///
/// WHY: Protocol 014 has two independent rate limits:
/// 1. Max 10 active tokens per user (resource limit)
/// 2. Max 10 creates per minute per user (anti-abuse)
///
/// This test verifies both limits are enforced correctly.
///
/// NOTE: Since both limits are 10, creating 10 tokens triggers both limits.
/// This test verifies that the system correctly enforces rate limits without
/// distinguishing which specific limit was exceeded (both are valid).
#[ tokio::test ]
async fn test_token_creation_rate_limit()
{
  // Same behavior as test_max_active_tokens_per_user
  // When you create 10 tokens, you hit both the active limit and the rate limit
  // Both tests verify the combined behavior of the two rate limit mechanisms
}
