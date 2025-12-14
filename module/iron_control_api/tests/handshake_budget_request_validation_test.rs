//! Handshake budget request validation tests
//!
//! Tests for GAP-002: Maximum budget request validation
//!
//! ## Test Coverage
//!
//! 1. Handshake with explicit budget request (within limits)
//! 2. Handshake with budget request exceeding maximum
//! 3. Handshake with zero budget request
//! 4. Handshake with negative budget request
//! 5. Handshake without budget request (uses default)
//!
//! ## Gap Addressed
//!
//! GAP-002: Handshake endpoint doesn't enforce maximum budget request limits
//! - Security implication: Client could request unlimited budgets
//! - Fix: Add maximum budget validation to HandshakeRequest

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use iron_control_api::routes::budget::handshake;
use serde_json::json;
use tower::ServiceExt;

/// TEST 1: Handshake with explicit budget request (within limits)
///
/// # Expected Behavior
///
/// - Request with budget <= MAX_HANDSHAKE_BUDGET succeeds
/// - Granted budget equals requested budget (if available)
#[ tokio::test ]
async fn test_handshake_with_valid_budget_request()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  // Create agent with sufficient budget
  let agent_id = 300i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await; // $100

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // Request $5 (5,000,000 microdollars) - within limits
  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": agent_id * 1000,
    "requested_budget": 5_000_000 // $5
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Handshake with valid budget request should succeed"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  let granted = response_json[ "budget_granted" ].as_i64().unwrap();
  assert_eq!(
    granted, 5_000_000,
    "Should grant requested budget of $5, got {} microdollars",
    granted
  );
}

/// TEST 2: Handshake with budget request exceeding maximum
///
/// # Expected Behavior
///
/// - Request with budget > MAX_HANDSHAKE_BUDGET fails with 400 Bad Request
/// - Error message indicates budget request too large
#[ tokio::test ]
async fn test_handshake_with_excessive_budget_request()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 301i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 1_000_000_000 ).await; // $1000

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // Request $101 (101,000,000 microdollars) - exceeds MAX_HANDSHAKE_BUDGET ($100)
  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": agent_id * 1000,
    "requested_budget": 101_000_000 // $101 (exceeds maximum)
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Handshake with excessive budget request should fail with 400"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_text = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    response_text.contains( "budget" ) || response_text.contains( "exceeds" ),
    "Error message should mention budget/exceeds, got: {}",
    response_text
  );
}

/// TEST 3: Handshake with zero budget request
///
/// # Expected Behavior
///
/// - Request with budget = 0 fails with 400 Bad Request
#[ tokio::test ]
async fn test_handshake_with_zero_budget_request()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 302i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": agent_id * 1000,
    "requested_budget": 0
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Handshake with zero budget request should fail with 400"
  );
}

/// TEST 4: Handshake with negative budget request
///
/// # Expected Behavior
///
/// - Request with budget < 0 fails with 400 Bad Request
#[ tokio::test ]
async fn test_handshake_with_negative_budget_request()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 303i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": agent_id * 1000,
    "requested_budget": -5_000_000 // Negative
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Handshake with negative budget request should fail with 400"
  );
}

/// TEST 5: Handshake without budget request (uses default)
///
/// # Expected Behavior
///
/// - Request without requested_budget field succeeds
/// - Granted budget equals DEFAULT_HANDSHAKE_BUDGET ($10)
#[ tokio::test ]
async fn test_handshake_without_budget_request_uses_default()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 304i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // No requested_budget field
  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": agent_id * 1000
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Handshake without budget request should succeed with default"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  let granted = response_json[ "budget_granted" ].as_i64().unwrap();
  assert_eq!(
    granted, 10_000_000,
    "Should grant default budget of $10, got {} microdollars",
    granted
  );
}
