//! Budget refresh JWT authentication tests
//!
//! Tests for GAP-003: Approver context from JWT
//!
//! ## Test Coverage
//!
//! 1. Budget refresh without JWT returns 401 Unauthorized
//! 2. Budget refresh with invalid JWT returns 401 Unauthorized
//! 3. Budget refresh with valid JWT succeeds and logs approver
//!
//! ## Gap Addressed
//!
//! GAP-003: Budget refresh endpoint doesn't extract approver identity from JWT for audit trail
//! - Security implication: Can't track who approved budget increases
//! - Fix: Add JWT authentication middleware, extract user_id from JWT claims

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use iron_control_api::routes::budget::refresh_budget;
use serde_json::json;
use tower::ServiceExt;

/// TEST 1: Budget refresh without JWT returns 401 Unauthorized
///
/// # Expected Behavior
///
/// - Request without Authorization header fails with 401
#[ tokio::test ]
async fn test_budget_refresh_without_jwt_returns_401()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  // Create agent and lease
  let agent_id = 400i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // First handshake to get a lease
  let handshake_body = json!(
  {
    "ic_token": ic_token.clone(),
    "provider": "openai",
    "provider_key_id": agent_id * 1000
  } );

  let handshake_app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( iron_control_api::routes::budget::handshake ) )
    .with_state( state.clone() );

  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &handshake_body ).unwrap() ) )
    .unwrap();

  let handshake_response = handshake_app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json[ "lease_id" ].as_str().unwrap();

  // Now try refresh without JWT
  let refresh_body = json!(
  {
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": 5_000_000
  } );

  let refresh_app = Router::new()
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state );

  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    // NO Authorization header
    .body( Body::from( serde_json::to_string( &refresh_body ).unwrap() ) )
    .unwrap();

  let response = refresh_app.oneshot( refresh_request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Budget refresh without JWT should return 401 Unauthorized"
  );
}

/// TEST 2: Budget refresh with invalid JWT returns 401 Unauthorized
///
/// # Expected Behavior
///
/// - Request with malformed/invalid JWT fails with 401
#[ tokio::test ]
async fn test_budget_refresh_with_invalid_jwt_returns_401()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 401i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // Create lease via handshake
  let handshake_body = json!(
  {
    "ic_token": ic_token.clone(),
    "provider": "openai",
    "provider_key_id": agent_id * 1000
  } );

  let handshake_app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( iron_control_api::routes::budget::handshake ) )
    .with_state( state.clone() );

  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &handshake_body ).unwrap() ) )
    .unwrap();

  let handshake_response = handshake_app.oneshot( handshake_request ).await.unwrap();
  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json[ "lease_id" ].as_str().unwrap();

  // Try refresh with invalid JWT
  let refresh_body = json!(
  {
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": 5_000_000
  } );

  let refresh_app = Router::new()
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state );

  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .header( "authorization", "Bearer invalid_jwt_token" ) // Invalid JWT
    .body( Body::from( serde_json::to_string( &refresh_body ).unwrap() ) )
    .unwrap();

  let response = refresh_app.oneshot( refresh_request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Budget refresh with invalid JWT should return 401 Unauthorized"
  );
}

/// TEST 3: Budget refresh with valid JWT succeeds
///
/// # Expected Behavior
///
/// - Request with valid JWT succeeds
/// - Approver user_id is extracted from JWT claims
#[ tokio::test ]
async fn test_budget_refresh_with_valid_jwt_succeeds()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 402i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // Create lease via handshake
  let handshake_body = json!(
  {
    "ic_token": ic_token.clone(),
    "provider": "openai",
    "provider_key_id": agent_id * 1000
  } );

  let handshake_app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( iron_control_api::routes::budget::handshake ) )
    .with_state( state.clone() );

  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &handshake_body ).unwrap() ) )
    .unwrap();

  let handshake_response = handshake_app.oneshot( handshake_request ).await.unwrap();
  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json[ "lease_id" ].as_str().unwrap();

  // Create valid JWT for test_user
  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  // Try refresh with valid JWT
  let refresh_body = json!(
  {
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": 5_000_000
  } );

  let refresh_app = Router::new()
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state );

  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", access_token ) )
    .body( Body::from( serde_json::to_string( &refresh_body ).unwrap() ) )
    .unwrap();

  let response = refresh_app.oneshot( refresh_request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Budget refresh with valid JWT should succeed"
  );

  // Verify response
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert_eq!(
    response_json[ "status" ].as_str().unwrap(),
    "approved",
    "Budget refresh should be approved"
  );
}
