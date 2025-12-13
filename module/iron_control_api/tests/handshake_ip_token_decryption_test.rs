//! IP Token Provider Key Decryption Test
//!
//! # Test Context
//!
//! **Feature:** Protocol 005 - Budget Control Handshake
//! **Component:** IP Token provider key decryption
//! **File:** `routes/budget/handshake.rs:347`
//!
//! ## Feature Overview
//!
//! The handshake endpoint exchanges IC Tokens for IP Tokens. An IP Token contains
//! an encrypted provider API key that agents use to make LLM API calls. The encryption
//! flow is:
//!
//! 1. Provider keys stored encrypted in database (AES-256-GCM with master key)
//! 2. Handshake decrypts provider key from database
//! 3. Handshake encrypts provider key into IP Token (AES-256-GCM with session key)
//! 4. Agent receives IP Token
//! 5. Agent decrypts IP Token to access provider key
//!
//! ## Current Implementation Status
//!
//! The handshake endpoint currently returns a placeholder IP Token:
//!
//! ```rust
//! // handshake.rs:347-349
//! // TODO: Decrypt provider API key
//! let provider_key = "sk-test_key_placeholder";
//! ```
//!
//! This test verifies that real provider key decryption is implemented.
//!
//! ## Test Strategy
//!
//! This test follows the TDD RED-GREEN-REFACTOR cycle:
//!
//! 1. **RED:** Test fails because handshake returns placeholder, not real decrypted key
//! 2. **GREEN:** Implement decryption - test passes
//! 3. **REFACTOR:** Extract decryption to dedicated module
//!
//! ## Test Requirements
//!
//! - ✅ Create encrypted provider key using iron_secrets::CryptoService
//! - ✅ Store encrypted key in database
//! - ✅ Call handshake endpoint
//! - ✅ Decrypt IP Token
//! - ✅ Verify decrypted value matches original provider key

use axum::{ body::Body, http::{ Request, StatusCode } };
use iron_secrets::crypto::CryptoService;
use serde_json::{ json, Value };
use tower::ServiceExt;

mod common;

/// Test: Handshake endpoint decrypts provider key and encrypts into IP Token
///
/// **Test Flow:**
/// 1. Encrypt real provider API key using iron_secrets::CryptoService
/// 2. Store encrypted key in database
/// 3. Call handshake endpoint
/// 4. Verify handshake succeeds (200 OK)
/// 5. Decrypt IP Token from response
/// 6. Verify decrypted value matches original provider API key
///
/// **Expected Behavior:**
/// - IP Token should contain real encrypted provider key (not placeholder)
/// - Decrypting IP Token should yield original provider key
#[tokio::test]
async fn test_handshake_decrypts_provider_key()
{
  // Setup: Create test database and state
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  // Setup: Create agent with budget
  let agent_id = 200i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  // Setup: Create real encrypted provider key
  let original_provider_key = "sk-proj-test_real_api_key_12345";

  // Create crypto service from environment
  // For tests, set IRON_SECRETS_MASTER_KEY to base64-encoded 32-byte key
  let master_key : [ u8; 32 ] = [ 42u8; 32 ]; // Test master key
  let crypto_service = CryptoService::new( &master_key )
    .expect( "LOUD FAILURE: Should create crypto service" );

  let encrypted = crypto_service.encrypt( original_provider_key )
    .expect( "LOUD FAILURE: Should encrypt provider key" );

  // Setup: Store encrypted provider key in database
  let provider_key_id = agent_id * 1000;
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR REPLACE INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( provider_key_id )
  .bind( "openai" )
  .bind( encrypted.ciphertext_base64() )
  .bind( encrypted.nonce_base64() )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .expect( "LOUD FAILURE: Should insert encrypted provider key" );

  // Setup: Create IC Token
  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );

  // Execute: Call handshake endpoint
  let app = common::budget::create_budget_router( state.clone() ).await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai",
        "provider_key_id": provider_key_id
      })
      .to_string(),
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Verify: Handshake should succeed
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Handshake should succeed with valid IC Token and encrypted provider key"
  );

  // Verify: Extract IP Token from response
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body : Value = serde_json::from_slice( &body_bytes )
    .expect( "LOUD FAILURE: Response should be valid JSON" );

  let ip_token = body[ "ip_token" ].as_str()
    .expect( "LOUD FAILURE: Response should contain ip_token field" );

  // Verify: IP Token should NOT be the placeholder
  assert!(
    !ip_token.contains( "STUB" ),
    "IP Token should not contain STUB placeholder: {}",
    ip_token
  );

  // Verify: Decrypt IP Token to recover provider key
  let decrypted_provider_key = state.ip_token_crypto.decrypt( ip_token )
    .expect( "LOUD FAILURE: Should decrypt IP Token" );

  // Verify: Decrypted provider key matches original
  assert_eq!(
    decrypted_provider_key.as_str(),
    original_provider_key,
    "Decrypted provider key should match original key"
  );
}

/// Test: Handshake fails gracefully if provider key cannot be decrypted
///
/// **Test Flow:**
/// 1. Store invalid encrypted provider key (corrupted ciphertext)
/// 2. Call handshake endpoint
/// 3. Verify handshake returns 500 Internal Server Error
///
/// **Expected Behavior:**
/// - Should not panic
/// - Should return 500 error with appropriate message
#[tokio::test]
async fn test_handshake_handles_decryption_failure()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 201i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  // Setup: Store invalid encrypted provider key (corrupted data)
  let provider_key_id = agent_id * 1000;
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR REPLACE INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( provider_key_id )
  .bind( "openai" )
  .bind( "INVALID_CORRUPTED_CIPHERTEXT" ) // Corrupted ciphertext
  .bind( "INVALID_NONCE" )                 // Corrupted nonce
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  let ic_token = common::budget::create_ic_token( agent_id, &state.ic_token_manager );
  let app = common::budget::create_budget_router( state ).await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai",
        "provider_key_id": provider_key_id
      })
      .to_string(),
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Verify: Should return 500 Internal Server Error (not panic)
  assert_eq!(
    response.status(),
    StatusCode::INTERNAL_SERVER_ERROR,
    "Handshake should return 500 when provider key decryption fails"
  );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_text = String::from_utf8( body_bytes.to_vec() ).unwrap();

  assert!(
    body_text.contains( "error" ),
    "Error response should contain error message: {}",
    body_text
  );
}
