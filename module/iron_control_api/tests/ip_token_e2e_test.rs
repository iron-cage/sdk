//! End-to-end IP Token encryption tests
//!
//! Verifies the full encrypt/decrypt flow across security boundaries:
//! - Server encrypts provider key with `IP_TOKEN_KEY` → HTTP response
//! - Client decrypts with the same `IP_TOKEN_KEY` → recovers original key
//!
//! ## Flows tested
//!
//! | Flow | Endpoint | Server action | Client action |
//! |------|----------|---------------|---------------|
//! | 1 | POST /api/budget/handshake | encrypt `ip_token` | decrypt `ip_token` |
//! | 2 | POST /api/v1/agents/provider-key | encrypt `ip_token` | decrypt `ip_token` |
//!
//! ## Security invariant
//!
//! The plaintext provider key NEVER appears in the HTTP response body.
//! Only the encrypted `AES256:{IV}:{ciphertext}:{tag}` string is transmitted.

mod common;

use axum::{
  body::Body,
  http::{Request, StatusCode},
  routing::post,
  Router,
};
use common::budget;
use iron_secrets::ip_token::IpTokenCrypto;
use serde_json::{json, Value};
use tower::ServiceExt;

/// The shared IP Token key used by both server and client in tests.
/// In production this comes from the `IP_TOKEN_KEY` environment variable.
const TEST_IP_TOKEN_KEY: [u8; 32] = [0u8; 32];

/// The master key used for at-rest encryption in the database.
const TEST_MASTER_KEY: [u8; 32] = [42u8; 32];

async fn parse_body(response: axum::http::Response<Body>) -> Value {
  let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
    .await
    .unwrap();
  serde_json::from_slice(&body_bytes).expect("Response should be valid JSON")
}

async fn seed_encrypted_provider_key(pool: &sqlx::SqlitePool, agent_id: i64, api_key: &str) {
  budget::seed_agent_with_budget(pool, agent_id, 100_000_000).await;

  let crypto = iron_secrets::crypto::CryptoService::new(&TEST_MASTER_KEY).unwrap();
  let encrypted = crypto.encrypt(api_key).unwrap();

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
      .execute( pool )
      .await
      .unwrap();

  // Link agent to its provider key (required for /api/v1/agents/provider-key endpoint)
  sqlx::query("UPDATE agents SET provider_key_id = ? WHERE id = ?")
    .bind(provider_key_id)
    .bind(agent_id)
    .execute(pool)
    .await
    .unwrap();
}

#[allow(clippy::similar_names)] // ic_token and ip_token are distinct domain concepts
#[tokio::test]
async fn e2e_handshake_server_encrypts_client_decrypts() {
  // --- Server side ---
  let pool = budget::setup_test_db().await;
  let state = budget::create_test_budget_state(pool.clone());

  let agent_id = 300i64;
  let original_key = "sk-proj-real-openai-key-e2e-test";
  seed_encrypted_provider_key(&pool, agent_id, original_key).await;

  let ic_token = budget::create_ic_token(agent_id, &state.ic_token_manager);
  let app = budget::create_budget_router(state);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body: Value = parse_body(response).await;
  let ip_token = body["ip_token"]
    .as_str()
    .expect("Response must contain ip_token");

  // Verify: plaintext key is NOT in the response
  let raw_body = serde_json::to_string(&body).unwrap();
  assert!(
    !raw_body.contains(original_key),
    "Plaintext provider key must NOT appear in HTTP response"
  );

  // --- Client side (separate IpTokenCrypto instance, same key) ---
  let client_crypto =
    IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).expect("Client should create IpTokenCrypto");

  let decrypted = client_crypto
    .decrypt(ip_token)
    .expect("Client should decrypt IP Token");

  assert_eq!(decrypted.as_str(), original_key);
}

#[allow(clippy::similar_names)] // ic_token and ip_token are distinct domain concepts
#[tokio::test]
async fn e2e_provider_key_server_encrypts_client_decrypts() {
  // --- Server side ---
  let pool = budget::setup_test_db().await;
  let state = budget::create_test_budget_state(pool.clone());

  let agent_id = 301i64;
  let original_key = "sk-ant-api03-real-anthropic-key-e2e";
  seed_encrypted_provider_key(&pool, agent_id, original_key).await;

  let ic_token = budget::create_ic_token(agent_id, &state.ic_token_manager);

  let app = Router::new()
    .route(
      "/api/v1/agents/provider-key",
      post(iron_control_api::routes::agent_provider_key::get_provider_key),
    )
    .with_state(state);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/agents/provider-key")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "ic_token": ic_token }).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(response.status(), StatusCode::OK);

  let body: Value = parse_body(response).await;
  let ip_token = body["ip_token"]
    .as_str()
    .expect("Response must contain ip_token");
  let provider = body["provider"]
    .as_str()
    .expect("Response must contain provider");

  assert_eq!(provider, "openai");

  // Verify: plaintext key is NOT in the response
  let raw_body = serde_json::to_string(&body).unwrap();
  assert!(
    !raw_body.contains(original_key),
    "Plaintext provider key must NOT appear in HTTP response"
  );

  // --- Client side ---
  let client_crypto =
    IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).expect("Client should create IpTokenCrypto");

  let decrypted = client_crypto
    .decrypt(ip_token)
    .expect("Client should decrypt IP Token");

  assert_eq!(decrypted.as_str(), original_key);
}

#[allow(clippy::similar_names)] // ic_token and ip_token are distinct domain concepts
#[tokio::test]
async fn e2e_corrupted_ip_token_client_handles_gracefully() {
  // Get a real IP Token from server
  let pool = budget::setup_test_db().await;
  let state = budget::create_test_budget_state(pool.clone());

  let agent_id = 302i64;
  seed_encrypted_provider_key(&pool, agent_id, "sk-test-key").await;

  let ic_token = budget::create_ic_token(agent_id, &state.ic_token_manager);
  let app = budget::create_budget_router(state);

  let response = app
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();

  let body: Value = parse_body(response).await;
  let ip_token = body["ip_token"].as_str().unwrap();

  // Corrupt the IP Token (simulate network tampering)
  let corrupted = format!("{ip_token}CORRUPTED");

  // Client should fail gracefully, not panic
  let client_crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();
  let result = client_crypto.decrypt(&corrupted);
  assert!(
    result.is_err(),
    "Corrupted IP Token should return error, not panic"
  );

  // Wrong key should also fail gracefully
  let wrong_key_crypto = IpTokenCrypto::new(&[0xFFu8; 32]).unwrap();
  let result = wrong_key_crypto.decrypt(ip_token);
  assert!(
    result.is_err(),
    "Wrong IP Token key should return error, not panic"
  );
}

#[tokio::test]
async fn e2e_same_key_decrypts_both_flows() {
  let pool = budget::setup_test_db().await;
  let state = budget::create_test_budget_state(pool.clone());

  let agent_id = 303i64;
  let original_key = "sk-proj-shared-key-both-flows";
  seed_encrypted_provider_key(&pool, agent_id, original_key).await;

  let ic_token = budget::create_ic_token(agent_id, &state.ic_token_manager);

  // Flow 1: Handshake
  let app1 = budget::create_budget_router(state.clone());
  let resp1 = app1
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(
          json!({
            "ic_token": ic_token,
            "provider": "openai",
            "provider_key_id": agent_id * 1000
          })
          .to_string(),
        ))
        .unwrap(),
    )
    .await
    .unwrap();
  let body1: Value = parse_body(resp1).await;

  // Flow 2: Provider Key
  let ic_token2 = budget::create_ic_token(agent_id, &state.ic_token_manager);
  let app2 = Router::new()
    .route(
      "/api/v1/agents/provider-key",
      post(iron_control_api::routes::agent_provider_key::get_provider_key),
    )
    .with_state(state);
  let resp2 = app2
    .oneshot(
      Request::builder()
        .method("POST")
        .uri("/api/v1/agents/provider-key")
        .header("content-type", "application/json")
        .body(Body::from(json!({ "ic_token": ic_token2 }).to_string()))
        .unwrap(),
    )
    .await
    .unwrap();
  let body2: Value = parse_body(resp2).await;

  // One client crypto instance decrypts both
  let client_crypto = IpTokenCrypto::new(&TEST_IP_TOKEN_KEY).unwrap();

  let decrypted1 = client_crypto
    .decrypt(body1["ip_token"].as_str().unwrap())
    .unwrap();
  let decrypted2 = client_crypto
    .decrypt(body2["ip_token"].as_str().unwrap())
    .unwrap();

  assert_eq!(decrypted1.as_str(), original_key);
  assert_eq!(decrypted2.as_str(), original_key);

  // But the encrypted tokens should be different (different nonce each time)
  assert_ne!(
    body1["ip_token"].as_str().unwrap(),
    body2["ip_token"].as_str().unwrap(),
    "Each encryption should produce a unique token (different nonce)"
  );
}
