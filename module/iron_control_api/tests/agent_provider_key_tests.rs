//! Integration tests for Agent Provider Key endpoint (Feature 014)
//!
//! Tests cover:
//! - Success: Agent with provider_key_id retrieves decrypted key
//! - 400 INVALID_TOKEN: Empty or malformed IC token
//! - 401 UNAUTHORIZED: Invalid IC token signature
//! - 403 NO_PROVIDER_ASSIGNED: Agent has no provider_key_id
//! - 404 INVALID_TOKEN: Agent not found
//! - 503 CRYPTO_UNAVAILABLE: CryptoService not configured
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected |
//! |-----------|----------|----------|
//! | `test_get_provider_key_success` | Valid IC token, agent has provider_key | 200 OK with provider_key |
//! | `test_get_provider_key_empty_token` | Empty ic_token | 400 INVALID_TOKEN |
//! | `test_get_provider_key_invalid_token` | Invalid JWT signature | 401 UNAUTHORIZED |
//! | `test_get_provider_key_no_provider_assigned` | Agent without provider_key_id | 403 NO_PROVIDER_ASSIGNED |
//! | `test_get_provider_key_agent_not_found` | Valid token but agent doesn't exist | 404 INVALID_TOKEN |
//! | `test_get_provider_key_crypto_unavailable` | CryptoService is None | 503 CRYPTO_UNAVAILABLE |

mod common;

use common::budget::{ setup_test_db, create_test_budget_state, create_test_budget_state_no_crypto, create_ic_token };
use axum::{
  Router,
  routing::post,
  http::{ StatusCode, Request, Method },
  body::Body,
};
use iron_control_api::routes::agent_provider_key::get_provider_key;
use iron_secrets::crypto::CryptoService;
use tower::ServiceExt;
use serde_json::json;

/// Create router for agent provider key endpoint
async fn create_provider_key_router( state: iron_control_api::routes::budget::BudgetState ) -> Router
{
  Router::new()
    .route( "/api/v1/agents/provider-key", post( get_provider_key ) )
    .with_state( state )
}

/// Seed agent with properly encrypted provider key
///
/// Creates test data that can actually be decrypted by the test CryptoService:
/// - Test user
/// - Agent with provider_key_id pointing to the key
/// - Encrypted provider key using the test crypto service
async fn seed_agent_with_encrypted_key(
  pool: &sqlx::SqlitePool,
  agent_id: i64,
  crypto: &CryptoService,
  api_key: &str,
  provider: &str,
)
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user if doesn't exist
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Encrypt the API key
  let encrypted = crypto.encrypt( api_key )
    .expect( "LOUD FAILURE: Failed to encrypt test API key" );
  let ciphertext_b64 = encrypted.ciphertext_base64();
  let nonce_b64 = encrypted.nonce_base64();

  // Insert provider key with unique ID
  let provider_key_id = agent_id * 1000;
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( provider_key_id )
  .bind( provider )
  .bind( ciphertext_b64 )
  .bind( nonce_b64 )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent with provider_key_id
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id, provider_key_id) VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ provider ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .bind( provider_key_id )
  .execute( pool )
  .await
  .unwrap();
}

/// Seed agent without provider key assignment
async fn seed_agent_without_provider_key( pool: &sqlx::SqlitePool, agent_id: i64 )
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user if doesn't exist
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent WITHOUT provider_key_id
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();
}

// ============================================================================
// Success Case
// ============================================================================

#[ tokio::test ]
async fn test_get_provider_key_success()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool.clone() ).await;

  // Seed agent with encrypted provider key
  let crypto = state.crypto_service.as_ref().unwrap();
  seed_agent_with_encrypted_key( &pool, 101, crypto, "sk-test-openai-key-12345", "openai" ).await;

  let app = create_provider_key_router( state.clone() ).await;

  // Generate IC token for agent_101
  let ic_token = create_ic_token( 101, &state.ic_token_manager );

  let request_body = json!({ "ic_token": ic_token });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK, "Should return 200 OK" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "provider_key" ].as_str().unwrap(), "sk-test-openai-key-12345" );
  assert_eq!( body[ "provider" ].as_str().unwrap(), "openai" );
}

// ============================================================================
// Error Cases
// ============================================================================

#[ tokio::test ]
async fn test_get_provider_key_empty_token()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let app = create_provider_key_router( state ).await;

  let request_body = json!({ "ic_token": "" });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST, "Empty token should return 400" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "code" ].as_str().unwrap(), "INVALID_TOKEN" );
}

#[ tokio::test ]
async fn test_get_provider_key_invalid_token()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let app = create_provider_key_router( state ).await;

  let request_body = json!({ "ic_token": "invalid.jwt.token" });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::UNAUTHORIZED, "Invalid token should return 401" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "code" ].as_str().unwrap(), "UNAUTHORIZED" );
}

#[ tokio::test ]
async fn test_get_provider_key_no_provider_assigned()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool.clone() ).await;

  // Seed agent WITHOUT provider key
  seed_agent_without_provider_key( &pool, 102 ).await;

  let app = create_provider_key_router( state.clone() ).await;

  // Generate IC token for agent_102
  let ic_token = create_ic_token( 102, &state.ic_token_manager );

  let request_body = json!({ "ic_token": ic_token });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN, "No provider assigned should return 403" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "code" ].as_str().unwrap(), "NO_PROVIDER_ASSIGNED" );
}

#[ tokio::test ]
async fn test_get_provider_key_agent_not_found()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let app = create_provider_key_router( state.clone() ).await;

  // Generate IC token for non-existent agent
  let ic_token = create_ic_token( 99999, &state.ic_token_manager );

  let request_body = json!({ "ic_token": ic_token });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NOT_FOUND, "Non-existent agent should return 404" );
}

#[ tokio::test ]
async fn test_get_provider_key_crypto_unavailable()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state_no_crypto( pool.clone() ).await;

  // We need to seed with raw data since we don't have crypto service
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  // Insert provider key (encrypted data doesn't matter since crypto will fail first)
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( 103000i64 )
  .bind( "openai" )
  .bind( "fake_encrypted_data" )
  .bind( "fake_nonce" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  // Insert agent with provider_key_id
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id, provider_key_id) VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( 103i64 )
  .bind( "test_agent_103" )
  .bind( "[\"openai\"]" )
  .bind( now_ms )
  .bind( "test_user" )
  .bind( 103000i64 )
  .execute( &pool )
  .await
  .unwrap();

  let app = create_provider_key_router( state.clone() ).await;

  // Generate IC token for agent_103
  let ic_token = create_ic_token( 103, &state.ic_token_manager );

  let request_body = json!({ "ic_token": ic_token });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::SERVICE_UNAVAILABLE, "Missing crypto should return 503" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "code" ].as_str().unwrap(), "CRYPTO_UNAVAILABLE" );
}

#[ tokio::test ]
async fn test_get_provider_key_disabled_key()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool.clone() ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();
  let crypto = state.crypto_service.as_ref().unwrap();

  // Create test user
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  // Encrypt the API key
  let encrypted = crypto.encrypt( "sk-disabled-key" )
    .expect( "LOUD FAILURE: Failed to encrypt test API key" );
  let ciphertext_b64 = encrypted.ciphertext_base64();
  let nonce_b64 = encrypted.nonce_base64();

  // Insert DISABLED provider key
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( 104000i64 )
  .bind( "openai" )
  .bind( ciphertext_b64 )
  .bind( nonce_b64 )
  .bind( 0 ) // DISABLED
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  // Insert agent with provider_key_id
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id, provider_key_id) VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( 104i64 )
  .bind( "test_agent_104" )
  .bind( "[\"openai\"]" )
  .bind( now_ms )
  .bind( "test_user" )
  .bind( 104000i64 )
  .execute( &pool )
  .await
  .unwrap();

  let app = create_provider_key_router( state.clone() ).await;

  let ic_token = create_ic_token( 104, &state.ic_token_manager );
  let request_body = json!({ "ic_token": ic_token });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/v1/agents/provider-key" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN, "Disabled key should return 403" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let body: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( body[ "code" ].as_str().unwrap(), "NO_PROVIDER_ASSIGNED" );
}
