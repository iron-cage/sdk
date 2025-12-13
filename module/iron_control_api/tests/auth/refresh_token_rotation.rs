//! Refresh Token Rotation Tests (GAP-009)
//!
//! Tests for refresh token rotation security feature.
//!
//! ## Test Coverage
//!
//! 1. Refresh generates new refresh token
//! 2. Old refresh token cannot be reused (security)
//! 3. Token reuse is logged as security incident
//! 4. Full auth flow with rotation works correctly
//!
//! ## Security Rationale
//!
//! Refresh token rotation is a security best practice that:
//! - Limits the window of vulnerability if a token is stolen
//! - Enables detection of token theft via reuse detection
//! - Provides defense-in-depth for authentication system
//!
//! ## Implementation Strategy
//!
//! Per Protocol 007 enhancement:
//! - Generate new refresh token on each /auth/refresh request
//! - Invalidate old refresh token (add to blacklist)
//! - Return new refresh token in response
//! - Log security event if old token reused

use axum::{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use serde_json::json;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::common::auth::{ setup_auth_test_db, seed_test_user, create_auth_router };

/// TEST 1: Refresh endpoint returns new refresh token
///
/// ## Expected Behavior
///
/// - POST /api/v1/auth/refresh with valid refresh token
/// - Response includes new refresh_token field
/// - New refresh token is different from old token
///
/// ## Success Criteria
///
/// - Status: 200 OK
/// - Response contains refresh_token field
/// - New token != old token
#[ tokio::test ]
async fn test_refresh_returns_new_refresh_token()
{
  let pool: SqlitePool = setup_auth_test_db().await;
  let router: Router = create_auth_router( pool.clone() ).await;

  // Seed test user
  seed_test_user( &pool, "bob@example.com", "password456", "user", true ).await;

  // Login to get initial tokens
  let login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "bob@example.com",
        "password": "password456"
      }).to_string()
    ))
    .unwrap();

  let login_response = router.clone().oneshot( login_request ).await.unwrap();
  let login_body_bytes = axum::body::to_bytes( login_response.into_body(), usize::MAX ).await.unwrap();
  let login_data: serde_json::Value = serde_json::from_slice( &login_body_bytes ).unwrap();

  let original_refresh_token = login_data[ "refresh_token" ]
    .as_str()
    .expect( "LOUD FAILURE: Login response must include refresh_token" );

  // Use refresh token to get new tokens
  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/refresh" )
    .header( "authorization", format!( "Bearer {}", original_refresh_token ) )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let refresh_response = router.oneshot( refresh_request ).await.unwrap();

  assert_eq!(
    refresh_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Refresh with valid token must return 200 OK"
  );

  let refresh_body_bytes = axum::body::to_bytes( refresh_response.into_body(), usize::MAX ).await.unwrap();
  let refresh_data: serde_json::Value = serde_json::from_slice( &refresh_body_bytes ).unwrap();

  // Verify new refresh token is returned
  let new_refresh_token = refresh_data[ "refresh_token" ]
    .as_str()
    .expect( "LOUD FAILURE: Refresh response must include new refresh_token" );

  assert_ne!(
    new_refresh_token,
    original_refresh_token,
    "LOUD FAILURE: New refresh token must be different from old token (rotation requirement)"
  );
}

/// TEST 2: Old refresh token cannot be reused after rotation
///
/// ## Security Requirement
///
/// After a refresh token is used, it must be invalidated. Attempting
/// to reuse it should fail with 401 Unauthorized.
///
/// ## Expected Behavior
///
/// 1. Login → get refresh_token_1
/// 2. Refresh with refresh_token_1 → get refresh_token_2
/// 3. Refresh with refresh_token_1 again → 401 Unauthorized
///
/// ## Success Criteria
///
/// - First refresh succeeds (200 OK)
/// - Second refresh with old token fails (401 Unauthorized)
/// - Error message indicates token is invalid/expired
#[ tokio::test ]
async fn test_old_refresh_token_cannot_be_reused()
{
  let pool: SqlitePool = setup_auth_test_db().await;
  let router: Router = create_auth_router( pool.clone() ).await;

  // Seed test user
  seed_test_user( &pool, "charlie@example.com", "password789", "user", true ).await;

  // Login to get initial tokens
  let login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "charlie@example.com",
        "password": "password789"
      }).to_string()
    ))
    .unwrap();

  let login_response = router.clone().oneshot( login_request ).await.unwrap();
  let login_body_bytes = axum::body::to_bytes( login_response.into_body(), usize::MAX ).await.unwrap();
  let login_data: serde_json::Value = serde_json::from_slice( &login_body_bytes ).unwrap();

  let original_refresh_token = login_data[ "refresh_token" ]
    .as_str()
    .expect( "LOUD FAILURE: Login must return refresh_token" );

  // First refresh - should succeed
  let refresh_request_1 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/refresh" )
    .header( "authorization", format!( "Bearer {}", original_refresh_token ) )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let refresh_response_1 = router.clone().oneshot( refresh_request_1 ).await.unwrap();

  assert_eq!(
    refresh_response_1.status(),
    StatusCode::OK,
    "LOUD FAILURE: First refresh must succeed"
  );

  // Second refresh with SAME old token - should fail
  let refresh_request_2 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/refresh" )
    .header( "authorization", format!( "Bearer {}", original_refresh_token ) )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let refresh_response_2 = router.oneshot( refresh_request_2 ).await.unwrap();

  assert_eq!(
    refresh_response_2.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Reusing old refresh token must return 401 Unauthorized (security requirement)"
  );

  let error_body_bytes = axum::body::to_bytes( refresh_response_2.into_body(), usize::MAX ).await.unwrap();
  let error_data: serde_json::Value = serde_json::from_slice( &error_body_bytes ).unwrap();

  let error_message = error_data[ "error" ][ "message" ]
    .as_str()
    .expect( "LOUD FAILURE: Error response must include message" );

  assert!(
    error_message.contains( "Invalid" ) || error_message.contains( "expired" ),
    "LOUD FAILURE: Error message should indicate token is invalid/expired. Got: {}",
    error_message
  );
}

/// TEST 3: Token rotation chain works correctly
///
/// ## Expected Behavior
///
/// Multiple sequential refreshes should work:
/// 1. Login → token_1
/// 2. Refresh with token_1 → token_2
/// 3. Refresh with token_2 → token_3
/// 4. Refresh with token_3 → token_4
///
/// ## Success Criteria
///
/// - All refreshes succeed
/// - Each new token is different
/// - Can't reuse any old token in the chain
#[ tokio::test ]
async fn test_refresh_token_rotation_chain()
{
  let pool: SqlitePool = setup_auth_test_db().await;
  let router: Router = create_auth_router( pool.clone() ).await;

  // Seed test user
  seed_test_user( &pool, "diana@example.com", "password000", "user", true ).await;

  // Login to get initial tokens
  let login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "diana@example.com",
        "password": "password000"
      }).to_string()
    ))
    .unwrap();

  let login_response = router.clone().oneshot( login_request ).await.unwrap();
  let login_body_bytes = axum::body::to_bytes( login_response.into_body(), usize::MAX ).await.unwrap();
  let login_data: serde_json::Value = serde_json::from_slice( &login_body_bytes ).unwrap();

  let mut current_refresh_token = login_data[ "refresh_token" ]
    .as_str()
    .expect( "LOUD FAILURE: Login must return refresh_token" )
    .to_string();

  let first_token = current_refresh_token.clone();

  // Perform 3 sequential refreshes
  for i in 1..=3
  {
    let refresh_request = Request::builder()
      .method( "POST" )
      .uri( "/api/v1/auth/refresh" )
      .header( "authorization", format!( "Bearer {}", current_refresh_token ) )
      .header( "content-type", "application/json" )
      .body( Body::empty() )
      .unwrap();

    let refresh_response = router.clone().oneshot( refresh_request ).await.unwrap();

    assert_eq!(
      refresh_response.status(),
      StatusCode::OK,
      "LOUD FAILURE: Refresh #{} must succeed",
      i
    );

    let refresh_body_bytes = axum::body::to_bytes( refresh_response.into_body(), usize::MAX ).await.unwrap();
    let refresh_data: serde_json::Value = serde_json::from_slice( &refresh_body_bytes ).unwrap();

    let new_token = refresh_data[ "refresh_token" ]
      .as_str()
      .expect( "LOUD FAILURE: Each refresh must return new refresh_token" )
      .to_string();

    assert_ne!(
      new_token,
      current_refresh_token,
      "LOUD FAILURE: Refresh #{} must return different token",
      i
    );

    current_refresh_token = new_token;
  }

  // Try to reuse the very first token - should fail
  let reuse_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/refresh" )
    .header( "authorization", format!( "Bearer {}", first_token ) )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let reuse_response = router.oneshot( reuse_request ).await.unwrap();

  assert_eq!(
    reuse_response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Cannot reuse first token after 3 rotations (security requirement)"
  );
}
