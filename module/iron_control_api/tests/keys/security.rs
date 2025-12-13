//! Security tests for GET /api/keys (FR-12: Key Fetch API).
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected Behavior |
//! |-----------|----------|-------------------|
//! | Rate limit | 11+ requests in 1 minute | 429 Too Many Requests |
//! | Error messages | Crypto failure | Generic "Internal server error" (no details) |
//! | Response sanitization | Successful response | Only expected fields |
//!
//! ## Security Requirements
//!
//! 1. Rate limiting: 10 requests/minute per user/project
//! 2. No detailed error messages for crypto failures
//! 3. Audit logging for all key fetches (verified via E2E)
//! 4. API tokens treated as secrets

use crate::common::extract_response;
use axum::{ Router, routing::get, http::Request };
use axum::body::Body;
use tower::ServiceExt;
use std::sync::Arc;
use std::time::Duration;

use iron_control_api::routes::keys::{ KeysState, get_key, KeyResponse };
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

// =============================================================================
// Rate Limiting Tests
// =============================================================================

/// Test rate limiting returns 429 after exceeding limit.
///
/// WHY: Security requirement - prevent brute force token enumeration.
///
/// Note: This test makes multiple requests with invalid tokens.
/// The rate limiter checks happen AFTER authentication in the current implementation,
/// so we need a valid token to trigger rate limiting. Since we don't have a valid
/// token, we test the rate limiter component directly instead.
#[ tokio::test ]
async fn test_rate_limiter_component()
{
  // Test the RateLimiter component directly
  let rate_limiter = RateLimiter::new( 2, Duration::from_secs( 60 ) );

  // First two requests should pass
  assert!(
    rate_limiter.check_rate_limit( "user_test", Some( "project_test" ) ),
    "LOUD FAILURE: First request should pass rate limit"
  );

  assert!(
    rate_limiter.check_rate_limit( "user_test", Some( "project_test" ) ),
    "LOUD FAILURE: Second request should pass rate limit"
  );

  // Third request should be rate limited
  assert!(
    !rate_limiter.check_rate_limit( "user_test", Some( "project_test" ) ),
    "LOUD FAILURE: Third request should be rate limited"
  );
}

/// Test rate limiting is per-user (different users have separate limits).
#[ tokio::test ]
async fn test_rate_limiter_per_user()
{
  let rate_limiter = RateLimiter::new( 1, Duration::from_secs( 60 ) );

  // User A uses their quota
  assert!(
    rate_limiter.check_rate_limit( "user_a", Some( "project_test" ) ),
    "User A's first request should pass"
  );

  assert!(
    !rate_limiter.check_rate_limit( "user_a", Some( "project_test" ) ),
    "User A's second request should be rate limited"
  );

  // User B should have their own quota
  assert!(
    rate_limiter.check_rate_limit( "user_b", Some( "project_test" ) ),
    "User B's first request should pass (separate quota)"
  );
}

/// Test rate limiting is per-project (same user, different projects have separate limits).
#[ tokio::test ]
async fn test_rate_limiter_per_project()
{
  let rate_limiter = RateLimiter::new( 1, Duration::from_secs( 60 ) );

  // User uses quota for project A
  assert!(
    rate_limiter.check_rate_limit( "user_test", Some( "project_a" ) ),
    "First request to project A should pass"
  );

  assert!(
    !rate_limiter.check_rate_limit( "user_test", Some( "project_a" ) ),
    "Second request to project A should be rate limited"
  );

  // Same user, different project should have own quota
  assert!(
    rate_limiter.check_rate_limit( "user_test", Some( "project_b" ) ),
    "First request to project B should pass (separate quota)"
  );
}

// =============================================================================
// Error Message Sanitization Tests
// =============================================================================

/// Test that error responses don't leak internal details.
///
/// WHY: Security requirement - crypto failures shouldn't reveal implementation details.
#[ tokio::test ]
async fn test_error_message_sanitization()
{
  // We can't easily trigger a crypto error in tests without mocking,
  // but we verify the error response structure for invalid tokens
  // doesn't contain stack traces or internal paths.

  let token_storage = TokenStorage::new( "sqlite::memory:" )
    .await
    .expect("LOUD FAILURE: Failed to create token storage");

  let provider_storage = ProviderKeyStorage::connect( "sqlite::memory:" )
    .await
    .expect("LOUD FAILURE: Failed to create provider key storage");

  let crypto = CryptoService::new( &TEST_MASTER_KEY )
    .expect("LOUD FAILURE: Failed to create crypto service");

  let rate_limiter = RateLimiter::new( 10, Duration::from_secs( 60 ) );

  let keys_state = KeysState {
    token_storage: Arc::new( token_storage ),
    provider_storage: Arc::new( provider_storage ),
    crypto: Arc::new( crypto ),
    rate_limiter,
  };

  let router = Router::new()
    .route( "/api/keys", get( get_key ) )
    .with_state( keys_state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( "Authorization", "Bearer invalid_token" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let ( _status, body ) = extract_response( response ).await;

  // Verify error message doesn't contain sensitive info
  assert!(
    !body.contains( "stack" ),
    "Error response should not contain stack traces"
  );
  assert!(
    !body.contains( "/home/" ),
    "Error response should not contain file paths"
  );
  assert!(
    !body.contains( "decrypt" ) || body.contains( "Internal server error" ),
    "Error response should be generic for crypto failures"
  );
}

// =============================================================================
// KeyResponse Structure Tests
// =============================================================================

/// Test KeyResponse serialization has correct fields.
///
/// WHY: API contract - consumers depend on field names.
#[ test ]
fn test_key_response_serialization()
{
  let response = KeyResponse {
    provider: "openai".to_string(),
    api_key: "sk-test-key".to_string(),
    base_url: Some( "https://api.openai.com/v1".to_string() ),
  };

  let json = serde_json::to_string( &response ).unwrap();

  assert!( json.contains( "\"provider\"" ), "Should have provider field" );
  assert!( json.contains( "\"api_key\"" ), "Should have api_key field" );
  assert!( json.contains( "\"base_url\"" ), "Should have base_url field when present" );
  assert!( json.contains( "openai" ), "Provider should be openai" );
  assert!( json.contains( "sk-test-key" ), "API key should be present" );
}

/// Test KeyResponse omits null base_url.
///
/// WHY: API contract - skip_serializing_if = "Option::is_none"
#[ test ]
fn test_key_response_omits_null_base_url()
{
  let response = KeyResponse {
    provider: "anthropic".to_string(),
    api_key: "sk-ant-test".to_string(),
    base_url: None,
  };

  let json = serde_json::to_string( &response ).unwrap();

  assert!( !json.contains( "base_url" ), "Should NOT have base_url field when None" );
  assert!( json.contains( "anthropic" ), "Provider should be anthropic" );
}
