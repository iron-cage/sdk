//! IC Token security tests for Protocol 005 Budget endpoints
//!
//! Tests verify that budget endpoints properly validate IC Token expiration
//! and reject requests with expired or invalid tokens.
//!
//! # Authority
//! - Protocol 005 specification: IC Token authentication requirement
//! - Security best practices: Token expiration enforcement
//!
//! # Test Matrix
//!
//! | Test Case | Endpoint | Token State | Expected Response |
//! |-----------|----------|-------------|-------------------|
//! | `test_handshake_expired_ic_token` | /handshake | Expired | 401 Unauthorized |
//! | `test_refresh_expired_ic_token` | /refresh | Expired | 401 Unauthorized |
//!
//! # Note
//! The /report endpoint does NOT require IC Token authentication. It uses lease_id
//! as the authentication credential. IC Token expiration testing is not applicable.

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
};
use common::budget::
{
  setup_test_db,
  create_test_budget_state,
  seed_agent_with_budget,
  create_budget_router,
};
use iron_control_api::ic_token::{ IcTokenClaims, IcTokenManager };
use serde_json::json;
use std::time::{ SystemTime, UNIX_EPOCH };
use tower::ServiceExt;

/// Helper: Create expired IC Token
///
/// Generates IC Token with expiration in the past (1 hour ago)
fn create_expired_ic_token( agent_id: i64, manager: &IcTokenManager ) -> String
{
  let now = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .expect("LOUD FAILURE: Time went backwards")
    .as_secs();

  let expired_time = now - 3600; // 1 hour ago

  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    Some( expired_time ), // Expired
  );

  manager.generate_token( &claims ).expect("LOUD FAILURE: Should generate IC Token")
}

/// E2: Handshake with expired IC Token
///
/// # Corner Case
/// POST /api/budget/handshake with IC token expired 1 hour ago
///
/// # Expected Behavior
/// - Request rejected with 401 Unauthorized
/// - Clear error message about expiration
/// - No lease created
/// - No budget reserved
///
/// # Risk
/// HIGH - Stale credentials could bypass security if not validated
///
/// # Security Impact
/// Expired tokens must be rejected to prevent:
/// - Use of stolen/leaked tokens after revocation
/// - Access with stale authorization
/// - Replay attacks with old tokens
#[ tokio::test ]
async fn test_handshake_expired_ic_token()
{
  let pool = setup_test_db().await;
  let agent_id = 300i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let expired_ic_token = create_expired_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Attempt handshake with expired IC token
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": expired_ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: 401 Unauthorized
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Expired IC Token should return 401 Unauthorized"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect("LOUD FAILURE: Response should be valid JSON");

  // Assert: Error message mentions token issue
  assert!(
    response_json.get("error").is_some(),
    "LOUD FAILURE: Response should contain error message. Response: {}",
    response_json
  );

  // Verify: No lease created
  let lease_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM budget_leases WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query lease count");

  assert_eq!(
    lease_count, 0,
    "LOUD FAILURE: No lease should be created with expired IC Token"
  );

  // Verify: Budget unchanged (no reservation)
  let budget_remaining: i64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    budget_remaining, 100_000_000,
    "LOUD FAILURE: Budget should be unchanged. Expected: $100, Actual: {}",
    budget_remaining
  );
}

/// E2b: Refresh with expired IC Token
///
/// # Corner Case
/// POST /api/budget/refresh with expired IC token
///
/// # Expected Behavior
/// - Request rejected with 401 Unauthorized
/// - No new lease created
/// - No budget reserved
///
/// # Risk
/// MEDIUM - Stale token used for budget refresh
#[ tokio::test ]
async fn test_refresh_expired_ic_token()
{
  let pool = setup_test_db().await;
  let agent_id = 302i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Create initial lease with valid token
  let valid_ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state.clone() ).await;

  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": valid_ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let current_lease_id = handshake_json["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Attempt refresh with EXPIRED IC token
  let expired_ic_token = create_expired_ic_token( agent_id, &state.ic_token_manager );
  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( json!({
          "ic_token": expired_ic_token,
          "current_lease_id": current_lease_id,
          "requested_budget": 10_000_000
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: 401 Unauthorized
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Refresh with expired IC Token should return 401 Unauthorized"
  );

  // Verify: No new lease created (only initial handshake lease)
  let lease_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM budget_leases WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query lease count");

  assert_eq!(
    lease_count, 1,
    "LOUD FAILURE: Should have only initial lease. No new lease created with expired token."
  );

  // Verify: Budget unchanged (only handshake deduction)
  let total_spent: i64 = sqlx::query_scalar(
    "SELECT total_spent FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    total_spent, 10_000_000,
    "LOUD FAILURE: total_spent should only include initial handshake ($10M)"
  );
}
