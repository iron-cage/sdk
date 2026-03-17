//! E2E tests for agent transaction integrity and deterministic ordering.
//!
//! Tests cover:
//! - Creating agents with multiple provider keys in a single transaction
//! - Atomic rollback when provider key validation fails
//! - Updating agent provider keys (replace semantics)
//! - Atomic rollback on update failure
//! - Deterministic provider_key_ids ordering in list and get responses

mod common;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::{delete as delete_route, get, post, put},
  Router,
};
use common::{
  create_test_access_token, create_test_admin, create_test_user, test_state::TestAppState,
};
use iron_secrets::crypto::CryptoService;
use serde_json::json;
use sqlx::SqlitePool;
use tower::ServiceExt;

/// Helper to create test router with agents endpoints and 3 seeded provider keys.
async fn create_test_router() -> (Router, SqlitePool, String, String) {
  let app_state = TestAppState::new().await;

  let (admin_id, _) = create_test_admin(&app_state.database).await;
  let (user_id, _) = create_test_user(&app_state.database, "user@mail.com").await;

  let admin_token = create_test_access_token(
    &admin_id,
    "admin@admin.com",
    "admin",
    "test_jwt_secret_key_for_testing_12345",
  );
  let _user_token = create_test_access_token(
    &user_id,
    "user@mail.com",
    "user",
    "test_jwt_secret_key_for_testing_12345",
  );

  // Seed provider keys
  let now_ms = chrono::Utc::now().timestamp_millis();
  let provider_key_master: [u8; 32] = [42u8; 32];
  let crypto_service =
    CryptoService::new(&provider_key_master).expect("LOUD FAILURE: Should create crypto service");
  let encrypted = crypto_service
    .encrypt("sk-test-key")
    .expect("LOUD FAILURE: Should encrypt");

  // Seed 3 provider keys
  for key_id in 1..=3i64 {
    sqlx::query(
      "INSERT OR IGNORE INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id) VALUES (?, ?, ?, ?, 1, ?, ?)"
    )
    .bind(key_id)
    .bind("openai")
    .bind(encrypted.ciphertext_base64())
    .bind(encrypted.nonce_base64())
    .bind(now_ms)
    .bind(&admin_id)
    .execute(&app_state.database)
    .await
    .expect("LOUD FAILURE: Failed to seed provider key");
  }

  let router = Router::new()
    .route(
      "/api/agents",
      get(iron_control_api::routes::agents::list_agents),
    )
    .route(
      "/api/agents",
      post(iron_control_api::routes::agents::create_agent),
    )
    .route(
      "/api/agents/{id}",
      get(iron_control_api::routes::agents::get_agent),
    )
    .route(
      "/api/agents/{id}",
      put(iron_control_api::routes::agents::update_agent),
    )
    .route(
      "/api/agents/{id}",
      delete_route(iron_control_api::routes::agents::delete_agent),
    )
    .with_state(app_state.clone());

  (router, app_state.database.clone(), admin_token, _user_token)
}

/// Extract status and body string from an Axum response.
async fn extract_response(response: axum::response::Response) -> (StatusCode, String) {
  let status = response.status();
  let bytes = http_body_util::BodyExt::collect(response.into_body())
    .await
    .expect("LOUD FAILURE: Failed to read body")
    .to_bytes();
  (status, String::from_utf8(bytes.to_vec()).unwrap())
}

/// Helper: create an agent via POST and return the parsed JSON response.
async fn create_agent(
  router: &Router,
  token: &str,
  name: &str,
  provider_key_ids: &[i64],
) -> (StatusCode, serde_json::Value) {
  let request = Request::builder()
    .method(Method::POST)
    .uri("/api/agents")
    .header("Authorization", format!("Bearer {token}"))
    .header("Content-Type", "application/json")
    .body(Body::from(
      serde_json::to_string(&json!({
        "name": name,
        "providers": ["openai"],
        "provider_key_ids": provider_key_ids,
        "initial_budget_microdollars": 1_000_000
      }))
      .unwrap(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, body) = extract_response(response).await;
  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse response as JSON: {body}"));
  (status, json)
}

// ============================================================================
// Test 1: Create agent with multiple provider keys
// ============================================================================

#[tokio::test]
async fn test_create_agent_with_multiple_keys() {
  let (router, pool, admin_token, _user_token) = create_test_router().await;

  let (status, agent) = create_agent(&router, &admin_token, "Multi-Key Agent", &[1, 2, 3]).await;

  assert_eq!(
    status,
    StatusCode::CREATED,
    "LOUD FAILURE: Should create agent with multiple keys, got {status}"
  );

  // Verify response contains all 3 provider_key_ids
  let key_ids = agent["provider_key_ids"]
    .as_array()
    .expect("LOUD FAILURE: Response should have provider_key_ids array");
  let key_ids: Vec<i64> = key_ids.iter().map(|v| v.as_i64().unwrap()).collect();
  assert_eq!(
    key_ids,
    vec![1, 2, 3],
    "LOUD FAILURE: provider_key_ids should be [1, 2, 3]"
  );

  // Verify agent_provider_keys table has 3 rows
  let agent_id = agent["id"].as_i64().expect("LOUD FAILURE: Agent should have id");
  let count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM agent_provider_keys WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query agent_provider_keys");
  assert_eq!(
    count, 3,
    "LOUD FAILURE: agent_provider_keys should have 3 rows"
  );

  // Verify agent_budgets table has a row
  let budget_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query agent_budgets");
  assert_eq!(
    budget_count, 1,
    "LOUD FAILURE: agent_budgets should have 1 row for this agent"
  );
}

// ============================================================================
// Test 2: Create agent atomic rollback on invalid provider key
// ============================================================================

#[tokio::test]
async fn test_create_agent_atomic_rollback() {
  let (router, pool, admin_token, _user_token) = create_test_router().await;

  // Count agents before the failed request
  let agents_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
    .fetch_one(&pool)
    .await
    .expect("LOUD FAILURE: Should count agents before");

  // Attempt to create agent with one valid key (1) and one non-existent key (99999)
  let request = Request::builder()
    .method(Method::POST)
    .uri("/api/agents")
    .header("Authorization", format!("Bearer {admin_token}"))
    .header("Content-Type", "application/json")
    .body(Body::from(
      serde_json::to_string(&json!({
        "name": "Should Not Exist",
        "providers": ["openai"],
        "provider_key_ids": [1, 99999],
        "initial_budget_microdollars": 1_000_000
      }))
      .unwrap(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, _body) = extract_response(response).await;

  assert_ne!(
    status,
    StatusCode::CREATED,
    "LOUD FAILURE: Should not succeed when a provider key does not exist"
  );

  // Verify NO agent was created (transaction rolled back)
  let agents_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM agents")
    .fetch_one(&pool)
    .await
    .expect("LOUD FAILURE: Should count agents after");

  assert_eq!(
    agents_before, agents_after,
    "LOUD FAILURE: Transaction should have rolled back; no new agents should exist"
  );
}

// ============================================================================
// Test 3: Update agent replaces provider keys
// ============================================================================

#[tokio::test]
async fn test_update_agent_replaces_provider_keys() {
  let (router, pool, admin_token, _user_token) = create_test_router().await;

  // Create agent with key 1
  let (status, agent) = create_agent(&router, &admin_token, "Replaceable Agent", &[1]).await;
  assert_eq!(status, StatusCode::CREATED, "LOUD FAILURE: Setup failed");
  let agent_id = agent["id"].as_i64().expect("LOUD FAILURE: Agent should have id");

  // Update agent to use keys 2 and 3
  let request = Request::builder()
    .method(Method::PUT)
    .uri(format!("/api/agents/{agent_id}"))
    .header("Authorization", format!("Bearer {admin_token}"))
    .header("Content-Type", "application/json")
    .body(Body::from(
      serde_json::to_string(&json!({
        "name": "Replaceable Agent",
        "providers": ["openai"],
        "provider_key_ids": [2, 3]
      }))
      .unwrap(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, body) = extract_response(response).await;

  assert_eq!(
    status,
    StatusCode::OK,
    "LOUD FAILURE: Update should succeed, got {status}: {body}"
  );

  let updated: serde_json::Value = serde_json::from_str(&body)
    .expect("LOUD FAILURE: Should parse update response as JSON");

  let key_ids = updated["provider_key_ids"]
    .as_array()
    .expect("LOUD FAILURE: Updated response should have provider_key_ids");
  let key_ids: Vec<i64> = key_ids.iter().map(|v| v.as_i64().unwrap()).collect();
  assert_eq!(
    key_ids,
    vec![2, 3],
    "LOUD FAILURE: provider_key_ids should be [2, 3] after update"
  );

  // Verify agent_provider_keys table has exactly 2 rows (old key 1 removed)
  let count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM agent_provider_keys WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Should query agent_provider_keys");
  assert_eq!(
    count, 2,
    "LOUD FAILURE: agent_provider_keys should have exactly 2 rows after replacement"
  );

  // Verify key 1 is no longer associated
  let old_key_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM agent_provider_keys WHERE agent_id = ? AND provider_key_id = 1",
  )
  .bind(agent_id)
  .fetch_one(&pool)
  .await
  .expect("LOUD FAILURE: Should query for old key");
  assert_eq!(
    old_key_count, 0,
    "LOUD FAILURE: Old provider key 1 should be removed"
  );
}

// ============================================================================
// Test 4: Update agent keys atomic rollback
// ============================================================================

#[tokio::test]
async fn test_update_agent_keys_atomic() {
  let (router, pool, admin_token, _user_token) = create_test_router().await;

  // Create agent with key 1
  let (status, agent) = create_agent(&router, &admin_token, "Atomic Update Agent", &[1]).await;
  assert_eq!(status, StatusCode::CREATED, "LOUD FAILURE: Setup failed");
  let agent_id = agent["id"].as_i64().expect("LOUD FAILURE: Agent should have id");

  // Attempt to update with one valid key (2) and one non-existent (99999)
  let request = Request::builder()
    .method(Method::PUT)
    .uri(format!("/api/agents/{agent_id}"))
    .header("Authorization", format!("Bearer {admin_token}"))
    .header("Content-Type", "application/json")
    .body(Body::from(
      serde_json::to_string(&json!({
        "name": "Atomic Update Agent",
        "providers": ["openai"],
        "provider_key_ids": [2, 99999]
      }))
      .unwrap(),
    ))
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, _body) = extract_response(response).await;

  assert_ne!(
    status,
    StatusCode::OK,
    "LOUD FAILURE: Update with invalid key should not succeed"
  );

  // Verify agent_provider_keys still has key 1 (transaction rolled back)
  let key_ids: Vec<(i64,)> = sqlx::query_as(
    "SELECT provider_key_id FROM agent_provider_keys WHERE agent_id = ? ORDER BY provider_key_id",
  )
  .bind(agent_id)
  .fetch_all(&pool)
  .await
  .expect("LOUD FAILURE: Should query agent_provider_keys");

  assert_eq!(
    key_ids.len(),
    1,
    "LOUD FAILURE: Should still have exactly 1 provider key after failed update"
  );
  assert_eq!(
    key_ids[0].0, 1,
    "LOUD FAILURE: Provider key 1 should still be associated after rollback"
  );
}

// ============================================================================
// Test 5: List agents includes provider_key_ids
// ============================================================================

#[tokio::test]
async fn test_list_agents_includes_provider_key_ids() {
  let (router, _pool, admin_token, _user_token) = create_test_router().await;

  // Create two agents with different provider keys
  let (status, _agent1) = create_agent(&router, &admin_token, "Agent Alpha", &[1, 2]).await;
  assert_eq!(status, StatusCode::CREATED, "LOUD FAILURE: Setup agent 1 failed");

  let (status, _agent2) = create_agent(&router, &admin_token, "Agent Beta", &[3]).await;
  assert_eq!(status, StatusCode::CREATED, "LOUD FAILURE: Setup agent 2 failed");

  // List all agents
  let request = Request::builder()
    .method(Method::GET)
    .uri("/api/agents")
    .header("Authorization", format!("Bearer {admin_token}"))
    .body(Body::empty())
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, body) = extract_response(response).await;

  assert_eq!(
    status,
    StatusCode::OK,
    "LOUD FAILURE: List agents should succeed"
  );

  let agents: Vec<serde_json::Value> = serde_json::from_str(&body)
    .expect("LOUD FAILURE: Should parse list response as JSON array");

  // Find our agents by name
  let alpha = agents
    .iter()
    .find(|a| a["name"].as_str() == Some("Agent Alpha"))
    .expect("LOUD FAILURE: Should find Agent Alpha in list");
  let beta = agents
    .iter()
    .find(|a| a["name"].as_str() == Some("Agent Beta"))
    .expect("LOUD FAILURE: Should find Agent Beta in list");

  // Verify provider_key_ids
  let alpha_keys: Vec<i64> = alpha["provider_key_ids"]
    .as_array()
    .expect("LOUD FAILURE: Agent Alpha should have provider_key_ids")
    .iter()
    .map(|v| v.as_i64().unwrap())
    .collect();
  assert_eq!(
    alpha_keys,
    vec![1, 2],
    "LOUD FAILURE: Agent Alpha should have provider_key_ids [1, 2]"
  );

  let beta_keys: Vec<i64> = beta["provider_key_ids"]
    .as_array()
    .expect("LOUD FAILURE: Agent Beta should have provider_key_ids")
    .iter()
    .map(|v| v.as_i64().unwrap())
    .collect();
  assert_eq!(
    beta_keys,
    vec![3],
    "LOUD FAILURE: Agent Beta should have provider_key_ids [3]"
  );
}

// ============================================================================
// Test 6: Get agent includes provider_key_ids
// ============================================================================

#[tokio::test]
async fn test_get_agent_includes_provider_key_ids() {
  let (router, _pool, admin_token, _user_token) = create_test_router().await;

  // Create agent with keys 1 and 2
  let (status, created) =
    create_agent(&router, &admin_token, "Detail Agent", &[1, 2]).await;
  assert_eq!(status, StatusCode::CREATED, "LOUD FAILURE: Setup failed");
  let agent_id = created["id"].as_i64().expect("LOUD FAILURE: Agent should have id");

  // GET the agent by id
  let request = Request::builder()
    .method(Method::GET)
    .uri(format!("/api/agents/{agent_id}"))
    .header("Authorization", format!("Bearer {admin_token}"))
    .body(Body::empty())
    .unwrap();

  let response = router.clone().oneshot(request).await.unwrap();
  let (status, body) = extract_response(response).await;

  assert_eq!(
    status,
    StatusCode::OK,
    "LOUD FAILURE: Get agent should succeed"
  );

  let agent: serde_json::Value = serde_json::from_str(&body)
    .expect("LOUD FAILURE: Should parse get response as JSON");

  let key_ids: Vec<i64> = agent["provider_key_ids"]
    .as_array()
    .expect("LOUD FAILURE: Get response should have provider_key_ids")
    .iter()
    .map(|v| v.as_i64().unwrap())
    .collect();
  assert_eq!(
    key_ids,
    vec![1, 2],
    "LOUD FAILURE: provider_key_ids should be [1, 2]"
  );
}
