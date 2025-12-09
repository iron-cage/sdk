//! Endpoint integration tests for GET /api/keys (FR-12: Key Fetch API).
//!
//! ## Test Matrix
//!
//! | Test Case | Auth State | Expected Status | Expected Behavior |
//! |-----------|------------|-----------------|-------------------|
//! | No auth header | Missing | 401 Unauthorized | Error: Missing Authorization header |
//! | Invalid Bearer format | Malformed | 401 Unauthorized | Error: Invalid Authorization header format |
//! | Wrong HTTP method (POST) | - | 405 Method Not Allowed | Rejected |
//! | Wrong HTTP method (DELETE) | - | 405 Method Not Allowed | Rejected |
//! | Response content type | - | 200 OK | application/json |
//!
//! ## Notes
//!
//! Full integration tests requiring database setup:
//! - Valid token with project and assigned key → 200 + decrypted key
//! - Valid token without project → 400 Bad Request
//! - Valid token but no key assigned → 404 Not Found
//! - Valid token but key disabled → 403 Forbidden
//! - Rate limit exceeded → 429 Too Many Requests
//!
//! These are tested via manual/E2E tests due to complex state setup requirements.

use crate::common::extract_response;
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use std::sync::Arc;
use std::time::Duration;

use iron_control_api::routes::keys::{ KeysState, get_key };
use iron_token_manager::storage::TokenStorage;
use iron_token_manager::provider_key_storage::ProviderKeyStorage;
use iron_token_manager::rate_limiter::RateLimiter;
use iron_secrets::crypto::CryptoService;

/// Test master key for cryptographic operations (32 bytes).
const TEST_MASTER_KEY: [ u8; 32 ] = [
  0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
  0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
  0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
  0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

/// Create test router with keys endpoint.
///
/// Uses in-memory databases for token and provider storage.
async fn create_test_router() -> Router
{
  // Create token storage
  let token_storage = TokenStorage::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create token storage" );

  // Create provider key storage
  let provider_storage = ProviderKeyStorage::connect( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create provider key storage" );

  // Create crypto service with test key
  let crypto = CryptoService::new( &TEST_MASTER_KEY )
    .expect( "LOUD FAILURE: Failed to create crypto service" );

  // Create rate limiter (10 requests per minute)
  let rate_limiter = RateLimiter::new( 10, Duration::from_secs( 60 ) );

  let keys_state = KeysState {
    token_storage: Arc::new( token_storage ),
    provider_storage: Arc::new( provider_storage ),
    crypto: Arc::new( crypto ),
    rate_limiter,
  };

  Router::new()
    .route( "/api/keys", get( get_key ) )
    .with_state( keys_state )
}

// =============================================================================
// Authentication Tests
// =============================================================================

/// Test missing Authorization header returns 401.
///
/// WHY: Security requirement - unauthenticated requests must be rejected.
#[ tokio::test ]
async fn test_missing_auth_header_returns_401()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Missing Authorization header must return 401 Unauthorized"
  );

  let ( _status, body ) = extract_response( response ).await;
  assert!(
    body.contains( "Missing Authorization header" ),
    "LOUD FAILURE: Error message should indicate missing header, got: {}",
    body
  );
}

/// Test invalid Bearer format returns 401.
///
/// WHY: Security requirement - malformed auth must be rejected.
#[ tokio::test ]
async fn test_invalid_bearer_format_returns_401()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( "Authorization", "Basic dXNlcjpwYXNz" ) // Basic auth instead of Bearer
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Invalid Bearer format must return 401 Unauthorized"
  );
}

/// Test empty Bearer token returns 401.
///
/// WHY: Empty token should be rejected, not crash.
#[ tokio::test ]
async fn test_empty_bearer_token_returns_401()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer " ) // Empty token
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Empty Bearer token must return 401 Unauthorized"
  );
}

/// Test invalid token returns 401.
///
/// WHY: Non-existent tokens must be rejected.
#[ tokio::test ]
async fn test_invalid_token_returns_401()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer invalid_token_that_does_not_exist" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Invalid token must return 401 Unauthorized"
  );

  let ( _status, body ) = extract_response( response ).await;
  assert!(
    body.contains( "Invalid or expired token" ),
    "LOUD FAILURE: Error message should indicate invalid token, got: {}",
    body
  );
}

// =============================================================================
// HTTP Method Tests
// =============================================================================

/// Test POST method is rejected (GET-only endpoint).
///
/// WHY: Keys should only be fetched, not created via this endpoint.
#[ tokio::test ]
async fn test_post_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer test_token" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test DELETE method is rejected.
#[ tokio::test ]
async fn test_delete_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer test_token" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: DELETE to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test PUT method is rejected.
#[ tokio::test ]
async fn test_put_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer test_token" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: PUT to GET-only endpoint must return 405 Method Not Allowed"
  );
}
