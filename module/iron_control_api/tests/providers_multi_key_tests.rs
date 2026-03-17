//! Integration tests for multi-key provider support (Task 002)
//!
//! Verifies:
//! - Two POSTs for the same provider create two distinct keys (no upsert)
//! - Cross-tenant isolation: user B cannot overwrite user A's key
//! - Quota limit: 429 after 20 keys per user per provider
//! - Handshake uses agent's assigned key when provider_key_id is omitted
//! - Handshake rejects explicit provider_key_id owned by another user (403)
//! - Handshake returns 403 when agent has no assigned key and none is provided

#![allow(missing_docs)]

mod common;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::post,
  Router,
};
use common::budget::{create_ic_token, create_test_budget_state, setup_test_db};
use common::providers::{bearer, make_providers_state, TestProvidersAppState, MASTER_KEY};
use iron_control_api::routes::{budget::handshake, providers::create_provider_key};
use iron_secrets::{crypto::CryptoService, ip_token::IpTokenCrypto};
use iron_token_manager::provider_key_storage::ProviderKeyStorage;
use serde_json::json;
use tower::ServiceExt;

// ─────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────

/// Build a router with just the POST /api/v1/providers endpoint
fn build_providers_router(state: TestProvidersAppState) -> Router {
  Router::new()
    .route("/api/v1/providers", post(create_provider_key))
    .with_state(state)
}

/// POST /api/v1/providers with the given body and bearer token
async fn post_provider(
  app: Router,
  bearer_token: &str,
  body: serde_json::Value,
) -> axum::response::Response {
  app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/v1/providers")
        .header("content-type", "application/json")
        .header("authorization", bearer_token)
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap()
}

// ─────────────────────────────────────────────────────────────────
// Provider creation tests
// ─────────────────────────────────────────────────────────────────

/// Two consecutive POSTs with the same provider → two distinct key IDs (never upsert)
#[tokio::test]
async fn test_post_providers_creates_new_key_always() {
  let pool = setup_test_db().await;

  // Seed user_a
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind("user_a")
  .bind("user_a")
  .bind("hash_a")
  .bind("user_a@example.com")
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let state = make_providers_state(&pool).await;

  let body = json!({
    "provider": "openai",
    "api_key": "sk-first-key-000000000000000000000000",
  });

  let resp1 = post_provider(build_providers_router(state.clone()), &bearer("user_a"), body.clone())
    .await;
  assert_eq!(resp1.status(), StatusCode::CREATED, "First POST must return 201");
  let bytes1 = axum::body::to_bytes(resp1.into_body(), usize::MAX).await.unwrap();
  let json1: serde_json::Value = serde_json::from_slice(&bytes1).unwrap();
  let id1 = json1["id"].as_i64().expect("Response must have id field");

  let resp2 = post_provider(build_providers_router(state.clone()), &bearer("user_a"), body.clone())
    .await;
  assert_eq!(resp2.status(), StatusCode::CREATED, "Second POST must return 201");
  let bytes2 = axum::body::to_bytes(resp2.into_body(), usize::MAX).await.unwrap();
  let json2: serde_json::Value = serde_json::from_slice(&bytes2).unwrap();
  let id2 = json2["id"].as_i64().expect("Response must have id field");

  assert_ne!(id1, id2, "Two POSTs must create two keys with distinct IDs");
}

/// User B's POST must not affect user A's key count or data
#[tokio::test]
async fn test_post_providers_cross_tenant_no_overwrite() {
  let pool = setup_test_db().await;
  let now_ms = chrono::Utc::now().timestamp_millis();

  for user in ["user_p", "user_q"] {
    sqlx::query(
      "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(user)
    .bind(user)
    .bind("hash")
    .bind(format!("{user}@example.com"))
    .bind("admin")
    .bind(1)
    .bind(now_ms)
    .execute(&pool)
    .await
    .unwrap();
  }

  let state = make_providers_state(&pool).await;

  let body = json!({ "provider": "openai", "api_key": "sk-test-000000000000000000000000000" });

  // User P creates a key
  let resp_p = post_provider(build_providers_router(state.clone()), &bearer("user_p"), body.clone())
    .await;
  assert_eq!(resp_p.status(), StatusCode::CREATED);
  let bytes_p = axum::body::to_bytes(resp_p.into_body(), usize::MAX).await.unwrap();
  let json_p: serde_json::Value = serde_json::from_slice(&bytes_p).unwrap();
  let id_p = json_p["id"].as_i64().unwrap();

  // User Q creates their own key
  let resp_q = post_provider(build_providers_router(state.clone()), &bearer("user_q"), body.clone())
    .await;
  assert_eq!(resp_q.status(), StatusCode::CREATED);
  let bytes_q = axum::body::to_bytes(resp_q.into_body(), usize::MAX).await.unwrap();
  let json_q: serde_json::Value = serde_json::from_slice(&bytes_q).unwrap();
  let id_q = json_q["id"].as_i64().unwrap();

  assert_ne!(id_p, id_q, "User P and Q must have distinct key IDs");

  // Confirm user P's key still belongs to user P (not overwritten by Q)
  let p_key = state
    .providers
    .storage
    .get_key_metadata(id_p)
    .await
    .expect("Should still be able to read user_p's key");
  assert_eq!(p_key.user_id, "user_p", "user_p's key must not be overwritten by user_q");
}

/// 429 must be returned when a user already has 20 keys for the same provider
#[tokio::test]
async fn test_post_providers_quota_limit() {
  let pool = setup_test_db().await;
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind("user_quota")
  .bind("user_quota")
  .bind("hash")
  .bind("user_quota@example.com")
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let crypto = CryptoService::new(&MASTER_KEY).unwrap();
  let storage = ProviderKeyStorage::new(pool.clone());

  // Pre-seed 20 keys directly via storage (faster than 20 HTTP requests)
  for _ in 0..20 {
    storage
      .create_key(
        iron_token_manager::provider_key_storage::ProviderType::OpenAI,
        "enc",
        "nonce",
        None,
        None,
        "user_quota",
      )
      .await
      .unwrap();
  }

  let _ = crypto; // Suppress unused warning

  let state = make_providers_state(&pool).await;
  let body = json!({ "provider": "openai", "api_key": "sk-test-000000000000000000000000000" });

  let resp = post_provider(build_providers_router(state), &bearer("user_quota"), body).await;
  assert_eq!(
    resp.status(),
    StatusCode::TOO_MANY_REQUESTS,
    "21st key must be rejected with 429"
  );
}

// ─────────────────────────────────────────────────────────────────
// Handshake tests
// ─────────────────────────────────────────────────────────────────

/// Helper: create handshake router
fn build_handshake_router(state: iron_control_api::routes::budget::BudgetState) -> Router {
  Router::new()
    .route("/api/budget/handshake", post(handshake))
    .with_state(state)
}

/// Handshake without provider_key_id uses the agent's assigned key
#[tokio::test]
async fn test_handshake_uses_agent_assigned_key() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create owner user
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind("owner_h1")
  .bind("owner_h1")
  .bind("hash")
  .bind("owner_h1@example.com")
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  // Create encrypted provider key owned by owner_h1
  let crypto = state.crypto_service.as_ref().unwrap();
  let encrypted = crypto.encrypt("sk-agent-assigned-key").unwrap();
  let provider_key_id: i64 = 9001;
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(provider_key_id)
  .bind("openai")
  .bind(encrypted.ciphertext_base64())
  .bind(encrypted.nonce_base64())
  .bind(1)
  .bind(now_ms)
  .bind("owner_h1")
  .execute(&pool)
  .await
  .unwrap();

  // Create agent owned by owner_h1 with provider_key_id assigned
  let agent_id: i64 = 901;
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id, provider_key_id)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind("agent_h1")
  .bind("[\"openai\"]")
  .bind(now_ms)
  .bind("owner_h1")
  .bind(provider_key_id)
  .execute(&pool)
  .await
  .unwrap();

  // Seed agent budget
  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind(1_000_000_000_i64)
  .bind(1_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  // Seed usage limits for owner_h1
  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?)",
  )
  .bind("owner_h1")
  .bind(10_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let ic_token = create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  let app = build_handshake_router(state);
  let body = json!({ "ic_token": ic_token, "provider": "openai" }); // no provider_key_id

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(resp.status(), StatusCode::OK, "Handshake with agent-assigned key should succeed");

  // Parse the response body and verify the ip_token decrypts to the seeded provider key
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json_resp: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  let ip_token = json_resp["ip_token"]
    .as_str()
    .expect("Response must contain ip_token field");

  // Decrypt using the same ip_token_crypto key as create_test_budget_state ([0u8; 32])
  let ip_token_key: [u8; 32] = [0u8; 32];
  let client_crypto =
    IpTokenCrypto::from_slice(&ip_token_key).expect("Should create IpTokenCrypto for decryption");
  let decrypted = client_crypto.decrypt(ip_token).expect("ip_token should decrypt successfully");

  assert_eq!(
    decrypted.as_str(),
    "sk-agent-assigned-key",
    "Decrypted ip_token must equal the seeded provider API key"
  );
}

/// Explicit provider_key_id owned by another user must return 403 UNAUTHORIZED_KEY_ACCESS
#[tokio::test]
async fn test_handshake_rejects_cross_tenant_explicit_key() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create two users: owner_h2 owns the agent; owner_other owns the key
  for (uid, email) in [("owner_h2", "owner_h2@example.com"), ("owner_other", "other@example.com")] {
    sqlx::query(
      "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uid)
    .bind(uid)
    .bind("hash")
    .bind(email)
    .bind("admin")
    .bind(1)
    .bind(now_ms)
    .execute(&pool)
    .await
    .unwrap();
  }

  // Key owned by owner_other (not agent's owner)
  let other_key_id: i64 = 9002;
  let crypto = state.crypto_service.as_ref().unwrap();
  let encrypted = crypto.encrypt("sk-other-user-key").unwrap();
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(other_key_id)
  .bind("openai")
  .bind(encrypted.ciphertext_base64())
  .bind(encrypted.nonce_base64())
  .bind(1)
  .bind(now_ms)
  .bind("owner_other")
  .execute(&pool)
  .await
  .unwrap();

  // Agent owned by owner_h2 (different from key owner)
  let agent_id: i64 = 902;
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind("agent_h2")
  .bind("[\"openai\"]")
  .bind(now_ms)
  .bind("owner_h2")
  .execute(&pool)
  .await
  .unwrap();

  // Seed agent budget
  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind(1_000_000_000_i64)
  .bind(1_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?)",
  )
  .bind("owner_h2")
  .bind(10_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let ic_token = create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  let app = build_handshake_router(state);
  // Explicitly pass other user's key ID
  let body = json!({
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": other_key_id,
  });

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Cross-tenant key access must be rejected with 403"
  );
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json["error"].as_str().unwrap(),
    "UNAUTHORIZED_KEY_ACCESS",
    "Error code must be UNAUTHORIZED_KEY_ACCESS"
  );
}

/// Handshake with no assigned key and no explicit key → 403 NO_PROVIDER_ASSIGNED
#[tokio::test]
async fn test_handshake_no_assigned_key_returns_403() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind("owner_h3")
  .bind("owner_h3")
  .bind("hash")
  .bind("owner_h3@example.com")
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  // Agent with NO provider_key_id
  let agent_id: i64 = 903;
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind("agent_h3")
  .bind("[\"openai\"]")
  .bind(now_ms)
  .bind("owner_h3")
  .execute(&pool)
  .await
  .unwrap();

  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind(1_000_000_000_i64)
  .bind(1_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?)",
  )
  .bind("owner_h3")
  .bind(10_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let ic_token = create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  let app = build_handshake_router(state);
  let body = json!({ "ic_token": ic_token, "provider": "openai" }); // no provider_key_id

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "Agent without assigned key must get 403"
  );
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json["error"].as_str().unwrap(),
    "NO_PROVIDER_ASSIGNED",
    "Error code must be NO_PROVIDER_ASSIGNED"
  );
}

// ─────────────────────────────────────────────────────────────────
// Step 6 — Handshake: dev path + TOCTOU re-check
// ─────────────────────────────────────────────────────────────────

use common::DEV_KEY_ENV_LOCK;

/// Seed user and update agent_1's owner_id.
///
/// Migration 018 seeds agent_1 with `owner_id = NULL`. This helper inserts a user
/// and sets that user as agent_1's owner so the handshake can proceed.
async fn seed_agent1_owner(pool: &sqlx::SqlitePool, user_id: &str) {
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(user_id)
  .bind(user_id)
  .bind("hash")
  .bind(format!("{user_id}@example.com"))
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(pool)
  .await
  .unwrap();

  sqlx::query("UPDATE agents SET owner_id = ? WHERE id = 1")
    .bind(user_id)
    .execute(pool)
    .await
    .unwrap();
}

/// agent_id == 1 without IRON_ALLOW_DEV_KEYS → 403 NO_PROVIDER_ASSIGNED
#[tokio::test]
async fn handshake_dev_key_creation_requires_flag() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  seed_agent1_owner(&pool, "owner_dev1").await;

  // agent_1 must have no provider_key_id assigned
  sqlx::query("UPDATE agents SET provider_key_id = NULL WHERE id = 1")
    .execute(&pool)
    .await
    .unwrap();

  let ic_token = create_ic_token(&pool, 1, &state.ic_token_manager).await;

  let app = build_handshake_router(state);
  let body = json!({ "ic_token": ic_token, "provider": "openai" });

  // Hold the env-var lock for the duration of the handshake call to prevent the
  // concurrent `works_with_flag` test from setting the var mid-flight.
  let resp = {
    let _guard = DEV_KEY_ENV_LOCK.lock().await;
    std::env::remove_var("IRON_ALLOW_DEV_KEYS");
    app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/budget/handshake")
          .header("content-type", "application/json")
          .body(Body::from(serde_json::to_string(&body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap()
  };

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "agent_1 without IRON_ALLOW_DEV_KEYS must get 403"
  );
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json["error"].as_str().unwrap(),
    "NO_PROVIDER_ASSIGNED",
    "Error code must be NO_PROVIDER_ASSIGNED"
  );
}

/// agent_id == 1 with IRON_ALLOW_DEV_KEYS set → 200 and valid IC token response
#[tokio::test]
async fn handshake_dev_key_creation_works_with_flag() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  seed_agent1_owner(&pool, "owner_dev2").await;

  // agent_1 must have no provider_key_id so the dev-creation path is triggered
  sqlx::query("UPDATE agents SET provider_key_id = NULL WHERE id = 1")
    .execute(&pool)
    .await
    .unwrap();

  let ic_token = create_ic_token(&pool, 1, &state.ic_token_manager).await;

  let app = build_handshake_router(state);
  let body = json!({ "ic_token": ic_token, "provider": "openai" });

  let resp = {
    let _guard = DEV_KEY_ENV_LOCK.lock().await;
    std::env::set_var("IRON_ALLOW_DEV_KEYS", "1");
    let r = app
      .oneshot(
        Request::builder()
          .method(Method::POST)
          .uri("/api/budget/handshake")
          .header("content-type", "application/json")
          .body(Body::from(serde_json::to_string(&body).unwrap()))
          .unwrap(),
      )
      .await
      .unwrap();
    std::env::remove_var("IRON_ALLOW_DEV_KEYS");
    r
  };

  assert_eq!(
    resp.status(),
    StatusCode::OK,
    "agent_1 with IRON_ALLOW_DEV_KEYS must get 200"
  );
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert!(
    json["ip_token"].is_string() && !json["ip_token"].as_str().unwrap().is_empty(),
    "Response must contain a non-empty ip_token"
  );
}

/// TOCTOU re-check: agent owned by user_b has a key that belongs to user_a.
///
/// The initial ownership check is skipped (no explicit provider_key_id in request).
/// After budget reservation, the handler re-validates the fetched key's owner against
/// the agent's owner. The mismatch must return 403 UNAUTHORIZED_KEY_ACCESS.
#[tokio::test]
async fn handshake_toctou_recheck_fails_for_wrong_owner() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create two users
  for (uid, email) in [
    ("owner_toctou_a", "toctou_a@example.com"),
    ("owner_toctou_b", "toctou_b@example.com"),
  ] {
    sqlx::query(
      "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
       VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind(uid)
    .bind(uid)
    .bind("hash")
    .bind(email)
    .bind("admin")
    .bind(1)
    .bind(now_ms)
    .execute(&pool)
    .await
    .unwrap();
  }

  // Create an AI provider key owned by user_a (proper encryption required for decryption path)
  let crypto = state.crypto_service.as_ref().unwrap();
  let encrypted = crypto.encrypt("sk-user-a-toctou-key").unwrap();
  let key_id: i64 = sqlx::query(
    "INSERT INTO ai_provider_keys \
     (provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id) \
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind("openai")
  .bind(encrypted.ciphertext_base64())
  .bind(encrypted.nonce_base64())
  .bind(1)
  .bind(now_ms)
  .bind("owner_toctou_a")
  .execute(&pool)
  .await
  .unwrap()
  .last_insert_rowid();

  // Agent is owned by user_b but has user_a's key assigned — the inconsistency
  // that the TOCTOU re-check is designed to catch
  let agent_id: i64 = 9199;
  sqlx::query(
    "INSERT INTO agents \
     (id, name, providers, created_at, owner_id, provider_key_id) \
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind("agent_toctou")
  .bind("[\"openai\"]")
  .bind(now_ms)
  .bind("owner_toctou_b")
  .bind(key_id)
  .execute(&pool)
  .await
  .unwrap();

  // Budget for agent
  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets \
     (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at) \
     VALUES (?, ?, 0, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind(1_000_000_000_i64)
  .bind(1_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  // Usage limits for user_b (agent owner)
  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits \
     (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, \
      created_at, updated_at) \
     VALUES (?, ?, 0, ?, ?)",
  )
  .bind("owner_toctou_b")
  .bind(10_000_000_000_i64)
  .bind(now_ms)
  .bind(now_ms)
  .execute(&pool)
  .await
  .unwrap();

  let ic_token = create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  // Save a reference to the pool before state is consumed by the router builder
  let db_pool = state.db_pool.clone();
  let app = build_handshake_router(state);
  // No explicit provider_key_id — the handler will use the agent's assigned key,
  // then the TOCTOU re-check will find that key.user_id (user_a) ≠ owner_for_key (user_b)
  let body = json!({ "ic_token": ic_token, "provider": "openai" });

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    resp.status(),
    StatusCode::FORBIDDEN,
    "TOCTOU re-check must return 403 when key owner ≠ agent owner"
  );
  let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
  let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
  assert_eq!(
    json["error"].as_str().unwrap(),
    "UNAUTHORIZED_KEY_ACCESS",
    "TOCTOU re-check error code must be UNAUTHORIZED_KEY_ACCESS"
  );

  // Verify budget was NOT consumed (reservation should not have been made, or was refunded)
  let initial_budget: i64 = 1_000_000_000;
  let budget: Option<(i64,)> = sqlx::query_as(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?",
  )
  .bind(agent_id)
  .fetch_optional(&db_pool)
  .await
  .unwrap();
  assert_eq!(
    budget.map(|b| b.0),
    Some(initial_budget),
    "Budget should be unchanged after rejected handshake"
  );
}

// ─────────────────────────────────────────────────────────────────
// Key deletion cascade tests
// ─────────────────────────────────────────────────────────────────

/// Handshake must fail gracefully after the agent's assigned provider key is deleted.
///
/// Simulates the sequence:
///   1. Agent is assigned a provider key
///   2. The key is hard-deleted from `ai_provider_keys` (ON DELETE SET NULL fires)
///   3. Agent's `provider_key_id` is set to NULL (simulating the cascade)
///   4. Handshake is called — must return 403 NO_PROVIDER_ASSIGNED (or 404)
///   5. Budget must remain unchanged
#[tokio::test]
async fn handshake_fails_gracefully_after_assigned_key_deleted() {
  let pool = setup_test_db().await;
  let state = create_test_budget_state(pool.clone()).await;

  let agent_id: i64 = 905;

  // Seed user + agent with a provider key and budget via the common helper
  common::budget::seed_agent_with_budget(&pool, agent_id, 1_000_000_000).await;

  // Capture the provider_key_id that was seeded (agent_id * 1000 per seed_agent_with_budget)
  let provider_key_id = agent_id * 1000;

  // Delete the provider key — simulates what delete_provider_key endpoint does.
  // First NULL out agents.provider_key_id (ON DELETE SET NULL behaviour), then delete the key.
  sqlx::query("UPDATE agents SET provider_key_id = NULL WHERE id = ?")
    .bind(agent_id)
    .execute(&pool)
    .await
    .unwrap();

  sqlx::query("DELETE FROM ai_provider_keys WHERE id = ?")
    .bind(provider_key_id)
    .execute(&pool)
    .await
    .unwrap();

  // Create IC token for the agent
  let ic_token = create_ic_token(&pool, agent_id, &state.ic_token_manager).await;

  // Save pool reference before state is consumed by router builder
  let db_pool = state.db_pool.clone();

  let app = build_handshake_router(state);
  let body = json!({ "ic_token": ic_token, "provider": "openai" }); // no provider_key_id

  let resp = app
    .oneshot(
      Request::builder()
        .method(Method::POST)
        .uri("/api/budget/handshake")
        .header("content-type", "application/json")
        .body(Body::from(serde_json::to_string(&body).unwrap()))
        .unwrap(),
    )
    .await
    .unwrap();

  // Must return 403 NO_PROVIDER_ASSIGNED or 404 (key not found); must NOT be 200
  let status = resp.status();
  assert!(
    status == StatusCode::FORBIDDEN || status == StatusCode::NOT_FOUND,
    "Handshake after key deletion must return 403 or 404, got {status}"
  );

  if status == StatusCode::FORBIDDEN {
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json_resp: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
      json_resp["error"].as_str().unwrap(),
      "NO_PROVIDER_ASSIGNED",
      "Error code must be NO_PROVIDER_ASSIGNED when key is deleted"
    );
  }

  // Budget must not have been consumed
  let initial_budget: i64 = 1_000_000_000;
  let budget: Option<(i64,)> =
    sqlx::query_as("SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_optional(&db_pool)
      .await
      .unwrap();
  assert_eq!(
    budget.map(|b| b.0),
    Some(initial_budget),
    "Budget must be unchanged after handshake with deleted key"
  );
}
