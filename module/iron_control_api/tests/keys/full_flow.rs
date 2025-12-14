//! Full flow integration tests for GET /api/keys (FR-12: Key Fetch API).
//!
//! These tests verify the complete end-to-end flow:
//! 1. Create a token with project_id
//! 2. Create and encrypt a provider key
//! 3. Assign the provider key to the project
//! 4. Fetch the key via /api/keys endpoint
//! 5. Verify the decrypted key matches the original
//!
//! ## Test Matrix
//!
//! | Test Case | Setup | Expected Result |
//! |-----------|-------|-----------------|
//! | Full happy path | Token + Key + Assignment | 200 + decrypted key |
//! | Token without project | Token (no project_id) | 400 Bad Request |
//! | No key assigned | Token + Key (no assignment) | 404 Not Found |
//! | Disabled key | Token + Disabled Key + Assignment | 403 Forbidden |
//! | Rate limit exceeded | Multiple requests | 429 Too Many Requests |

use crate::common::extract_response;
use axum::{ Router, routing::get, http::{ Request, StatusCode, header } };
use axum::body::Body;
use tower::ServiceExt;
use std::sync::Arc;
use std::time::Duration;

use iron_control_api::routes::keys::{ KeysState, get_key, KeyResponse };
use iron_token_manager::storage::TokenStorage;
use iron_token_manager::token_generator::TokenGenerator;
use iron_token_manager::provider_key_storage::{ ProviderKeyStorage, ProviderType };
use iron_token_manager::rate_limiter::RateLimiter;
use iron_secrets::crypto::CryptoService;

/// Test master key for cryptographic operations (32 bytes).
const TEST_MASTER_KEY: [ u8; 32 ] = [
  0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07,
  0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
  0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
  0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];

/// Test API key value (will be encrypted and decrypted).
const TEST_API_KEY: &str = "sk-test-openai-key-for-integration-testing";

/// Shared test state for full flow tests.
struct TestState
{
  token_storage: Arc< TokenStorage >,
  token_generator: TokenGenerator,
  provider_storage: Arc< ProviderKeyStorage >,
  crypto: Arc< CryptoService >,
  rate_limiter: RateLimiter,
}

impl TestState
{
  async fn new() -> Self
  {
    Self::with_rate_limit( 100, Duration::from_secs( 60 ) ).await
  }

  async fn with_rate_limit( requests: u32, period: Duration ) -> Self
  {
    let token_storage = TokenStorage::new( "sqlite::memory:" )
      .await
      .expect( "LOUD FAILURE: Failed to create token storage" );

    let provider_storage = ProviderKeyStorage::connect( "sqlite::memory:" )
      .await
      .expect( "LOUD FAILURE: Failed to create provider key storage" );

    let crypto = CryptoService::new( &TEST_MASTER_KEY )
      .expect( "LOUD FAILURE: Failed to create crypto service" );

    let rate_limiter = RateLimiter::new( requests, period );

    // Create test users to satisfy foreign key constraints (migration 013)
    let pool = token_storage.pool();
    let now = std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .expect("LOUD FAILURE: Time went backwards")
      .as_secs() as i64;

    // Create test_user (used in most tests)
    sqlx::query(
      "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( "test_user" )
    .bind( "test_user" )
    .bind( "test@example.com" )
    .bind( "hash" )
    .bind( "user" )
    .bind( 1 )
    .bind( now )
    .execute( pool )
    .await
    .expect( "LOUD FAILURE: Failed to create test_user" );

    // Create user_a (used in different_projects test)
    sqlx::query(
      "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( "user_a" )
    .bind( "user_a" )
    .bind( "user_a@example.com" )
    .bind( "hash" )
    .bind( "user" )
    .bind( 1 )
    .bind( now )
    .execute( pool )
    .await
    .expect( "LOUD FAILURE: Failed to create user_a" );

    // Create user_b (used in different_projects test)
    sqlx::query(
      "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( "user_b" )
    .bind( "user_b" )
    .bind( "user_b@example.com" )
    .bind( "hash" )
    .bind( "user" )
    .bind( 1 )
    .bind( now )
    .execute( pool )
    .await
    .expect( "LOUD FAILURE: Failed to create user_b" );

    Self {
      token_storage: Arc::new( token_storage ),
      token_generator: TokenGenerator::new(),
      provider_storage: Arc::new( provider_storage ),
      crypto: Arc::new( crypto ),
      rate_limiter,
    }
  }

  fn keys_state( &self ) -> KeysState
  {
    KeysState {
      token_storage: self.token_storage.clone(),
      provider_storage: self.provider_storage.clone(),
      crypto: self.crypto.clone(),
      rate_limiter: self.rate_limiter.clone(),
    }
  }

  fn router( &self ) -> Router
  {
    Router::new()
      .route( "/api/keys", get( get_key ) )
      .with_state( self.keys_state() )
  }

  /// Create a token and return its plaintext value.
  async fn create_token( &self, user_id: &str, project_id: Option< &str > ) -> String
  {
    // Generate a plaintext token
    let plaintext_token = self.token_generator.generate();

    // Store it (will be hashed internally)
    let _token_id = self.token_storage
      .create_token( &plaintext_token, user_id, project_id, Some( "test token" ), None,  None)
      .await
      .expect( "LOUD FAILURE: Failed to create token" );

    // Return plaintext for use in Authorization header
    plaintext_token
  }

  /// Create an encrypted provider key and return its ID.
  async fn create_provider_key( &self, api_key: &str ) -> i64
  {
    // Encrypt the API key
    let encrypted = self.crypto.encrypt( api_key )
      .expect( "LOUD FAILURE: Failed to encrypt API key" );

    // Store in database
    let key_id = self.provider_storage
      .create_key(
        ProviderType::OpenAI,
        &encrypted.ciphertext_base64(),
        &encrypted.nonce_base64(),
        None, // base_url
        Some( "Test provider key" ),
        "test_user",
      )
      .await
      .expect( "LOUD FAILURE: Failed to create provider key" );

    key_id
  }

  /// Assign a provider key to a project.
  async fn assign_key_to_project( &self, project_id: &str, key_id: i64 )
  {
    self.provider_storage
      .assign_to_project( key_id, project_id )
      .await
      .expect( "LOUD FAILURE: Failed to assign key to project" );
  }

  /// Disable a provider key.
  async fn disable_key( &self, key_id: i64 )
  {
    self.provider_storage
      .set_enabled( key_id, false )
      .await
      .expect( "LOUD FAILURE: Failed to disable key" );
  }
}

// =============================================================================
// Full Flow Tests
// =============================================================================

/// Test complete encrypt → store → fetch → decrypt flow.
///
/// WHY: This is the critical security path - must work end-to-end.
#[ tokio::test ]
async fn test_full_flow_encrypt_store_fetch_decrypt()
{
  let state = TestState::new().await;

  // 1. Create token with project_id
  let project_id = "test-project-123";
  let token = state.create_token( "test_user", Some( project_id ) ).await;

  // 2. Create encrypted provider key
  let key_id = state.create_provider_key( TEST_API_KEY ).await;

  // 3. Assign key to project
  state.assign_key_to_project( project_id, key_id ).await;

  // 4. Fetch via /api/keys endpoint
  let router = state.router();
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // 5. Verify response
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Full flow should return 200 OK"
  );

  let ( _status, body ) = extract_response( response ).await;
  let key_response: KeyResponse = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response should be valid KeyResponse JSON" );

  // 6. Verify decrypted key matches original
  assert_eq!(
    key_response.api_key, TEST_API_KEY,
    "LOUD FAILURE: Decrypted key must match original"
  );
  assert_eq!(
    key_response.provider, "openai",
    "LOUD FAILURE: Provider should be openai"
  );
}

/// Test token without project_id returns 400 Bad Request.
///
/// WHY: Project ID is required to look up the assigned key.
#[ tokio::test ]
async fn test_token_without_project_returns_400()
{
  let state = TestState::new().await;

  // Create token WITHOUT project_id
  let token = state.create_token( "test_user", None ).await;

  let router = state.router();
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Token without project_id must return 400 Bad Request"
  );

  let ( _status, body ) = extract_response( response ).await;
  assert!(
    body.contains( "Token not assigned to a project" ),
    "LOUD FAILURE: Error message should indicate missing project, got: {}",
    body
  );
}

/// Test project with no key assigned returns 404 Not Found.
///
/// WHY: Must clearly indicate when no key is configured.
#[ tokio::test ]
async fn test_no_key_assigned_returns_404()
{
  let state = TestState::new().await;

  // Create token with project_id but DON'T assign any key
  let project_id = "project-without-key";
  let token = state.create_token( "test_user", Some( project_id ) ).await;

  let router = state.router();
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Project with no key assigned must return 404 Not Found"
  );

  let ( _status, body ) = extract_response( response ).await;
  assert!(
    body.contains( "No provider key assigned" ),
    "LOUD FAILURE: Error message should indicate no key assigned, got: {}",
    body
  );
}

/// Test disabled key returns 403 Forbidden.
///
/// WHY: Disabled keys should not be fetchable.
#[ tokio::test ]
async fn test_disabled_key_returns_403()
{
  println!( "\n=== TEST START: test_disabled_key_returns_403 ===" );

  let state = TestState::new().await;
  println!( "✓ TestState initialized" );

  // Create token with project_id
  let project_id = "project-with-disabled-key";
  println!( "→ Creating token for user='test_user', project_id='{}'", project_id );
  let token = state.create_token( "test_user", Some( project_id ) ).await;
  println!( "✓ Token created: {}", &token[..20.min(token.len())] );

  // Create and assign key
  println!( "→ Creating provider key with TEST_API_KEY" );
  let key_id = state.create_provider_key( TEST_API_KEY ).await;
  println!( "✓ Provider key created with ID: {}", key_id );

  println!( "→ Assigning key {} to project '{}'", key_id, project_id );
  state.assign_key_to_project( project_id, key_id ).await;
  println!( "✓ Key assigned to project" );

  // Disable the key
  println!( "→ Disabling key with ID: {}", key_id );
  state.disable_key( key_id ).await;
  println!( "✓ Key disabled successfully" );

  println!( "→ Building GET /api/keys request with Authorization header" );
  let router = state.router();
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
    .body( Body::empty() )
    .unwrap();
  println!( "✓ Request built successfully" );

  println!( "→ Sending request to router" );
  let response = router.oneshot( request ).await.unwrap();
  let status = response.status();
  println!( "✓ Response received with status: {}", status );

  println!( "→ Verifying status code is 403 FORBIDDEN" );
  assert_eq!(
    status,
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Disabled key must return 403 Forbidden, got: {}", status
  );
  println!( "✓ Status code assertion passed" );

  println!( "→ Extracting response body" );
  let ( _status, body ) = extract_response( response ).await;
  println!( "✓ Response body extracted: {}", body );

  println!( "→ Verifying error message contains 'disabled'" );
  assert!(
    body.contains( "disabled" ),
    "LOUD FAILURE: Error message should indicate key is disabled, got: {}",
    body
  );
  println!( "✓ Error message assertion passed" );

  println!( "=== TEST PASSED: test_disabled_key_returns_403 ===\n" );
}

/// Test rate limiting returns 429 after exceeding limit.
///
/// WHY: Must prevent brute force attacks.
#[ tokio::test ]
async fn test_rate_limit_exceeded_returns_429()
{
  // Create state with very low rate limit: 2 requests per minute
  let state = TestState::with_rate_limit( 2, Duration::from_secs( 60 ) ).await;

  // Create token with project_id
  let project_id = "rate-limit-test-project";
  let token = state.create_token( "test_user", Some( project_id ) ).await;

  // Create and assign key
  let key_id = state.create_provider_key( TEST_API_KEY ).await;
  state.assign_key_to_project( project_id, key_id ).await;

  // First 2 requests should succeed
  for i in 0..2
  {
    let router = state.router();
    let request = Request::builder()
      .method( "GET" )
      .uri( "/api/keys" )
      .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
      .body( Body::empty() )
      .unwrap();

    let response = router.oneshot( request ).await.unwrap();

    assert_eq!(
      response.status(),
      StatusCode::OK,
      "LOUD FAILURE: Request {} should succeed (within rate limit)",
      i + 1
    );
  }

  // Third request should be rate limited
  let router = state.router();
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::TOO_MANY_REQUESTS,
    "LOUD FAILURE: Request 3 should be rate limited (429)"
  );

  let ( _status, body ) = extract_response( response ).await;
  assert!(
    body.contains( "Rate limit exceeded" ),
    "LOUD FAILURE: Error message should indicate rate limit, got: {}",
    body
  );
}

/// Test different projects have separate keys.
///
/// WHY: Key isolation between projects is a security requirement.
#[ tokio::test ]
async fn test_different_projects_have_separate_keys()
{
  let state = TestState::new().await;

  // Create two projects with different keys
  let project_a = "project-alpha";
  let project_b = "project-beta";

  let token_a = state.create_token( "user_a", Some( project_a ) ).await;
  let token_b = state.create_token( "user_b", Some( project_b ) ).await;

  let key_a = "sk-openai-key-for-project-alpha";
  let key_b = "sk-openai-key-for-project-beta";

  let key_id_a = state.create_provider_key( key_a ).await;
  let key_id_b = state.create_provider_key( key_b ).await;

  state.assign_key_to_project( project_a, key_id_a ).await;
  state.assign_key_to_project( project_b, key_id_b ).await;

  println!( "→ Building GET /api/keys request with Authorization header" );

  // Fetch key for project A
  let router_a = state.router();
  let request_a = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token_a ) )
    .body( Body::empty() )
    .unwrap();

  println!( "→ Sending request to router" );

  let response_a = router_a.oneshot( request_a ).await.unwrap();

  println!( "→ Extracting response body" );

  let ( _, body_a ) = extract_response( response_a ).await;
  println!( "✓ Response body extracted: {}", body_a );

  let key_response_a: KeyResponse = serde_json::from_str( &body_a ).unwrap();

  // Fetch key for project B
  let router_b = state.router();
  let request_b = Request::builder()
    .method( "GET" )
    .uri( "/api/keys" )
    .header( header::AUTHORIZATION, format!( "Bearer {}", token_b ) )
    .body( Body::empty() )
    .unwrap();

  let response_b = router_b.oneshot( request_b ).await.unwrap();
  let ( _, body_b ) = extract_response( response_b ).await;
  let key_response_b: KeyResponse = serde_json::from_str( &body_b ).unwrap();

  // Verify isolation
  assert_eq!( key_response_a.api_key, key_a, "Project A should get key A" );
  assert_eq!( key_response_b.api_key, key_b, "Project B should get key B" );
  assert_ne!(
    key_response_a.api_key, key_response_b.api_key,
    "LOUD FAILURE: Projects must have different keys"
  );
}
