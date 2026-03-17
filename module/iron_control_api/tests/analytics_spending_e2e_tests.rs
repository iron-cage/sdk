//! End-to-end tests for analytics spending endpoints.
//!
//! Validates the fixes made to spending analytics:
//! - `provider_id` filter correctly maps to the `provider` column
//! - `compute_median` returns correct median values
//! - `group_by=key` returns `provider_key_id` and `alias` fields
//! - `provider_key_id` filter works correctly in by-agent queries

mod common;

use axum::{
  body::Body,
  http::{Method, Request, StatusCode},
  routing::get,
  Router,
};
use common::{create_test_access_token, create_test_admin, test_state::TestAppState};
use iron_control_api::{
  ic_token::{IcTokenManager, IcTokenRateLimiter},
  routes::analytics::AnalyticsState,
  routes::auth::AuthState,
};
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

// ============================================================================
// Combined state for analytics + auth
// ============================================================================

/// Combined application state that satisfies both `AuthenticatedUser` (needs `AuthState`)
/// and spending handlers (need `AnalyticsState`).
#[derive(Clone)]
struct TestAnalyticsAppState {
  auth: AuthState,
  analytics: AnalyticsState,
}

impl axum::extract::FromRef<TestAnalyticsAppState> for AuthState {
  fn from_ref(state: &TestAnalyticsAppState) -> Self {
    state.auth.clone()
  }
}

impl axum::extract::FromRef<TestAnalyticsAppState> for AnalyticsState {
  fn from_ref(state: &TestAnalyticsAppState) -> Self {
    state.analytics.clone()
  }
}

// ============================================================================
// Seed helpers
// ============================================================================

async fn seed_agent(pool: &SqlitePool, agent_id: i64, name: &str, owner_id: &str) {
  sqlx::query(
    "INSERT OR IGNORE INTO agents (id, name, providers, created_at, owner_id) \
     VALUES (?, ?, '[\"openai\"]', 0, ?)",
  )
  .bind(agent_id)
  .bind(name)
  .bind(owner_id)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Failed to seed agent");
}

async fn seed_analytics_event(
  pool: &SqlitePool,
  event_id: &str,
  provider: &str,
  cost_micros: i64,
  agent_id: i64,
  provider_key_id: Option<i64>,
) {
  let now = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT INTO analytics_events \
     (event_id, timestamp_ms, event_type, model, provider, input_tokens, output_tokens, cost_micros, agent_id, provider_key_id, received_at) \
     VALUES (?, ?, 'llm_request_completed', 'gpt-4', ?, 100, 50, ?, ?, ?, ?)",
  )
  .bind(event_id)
  .bind(now)
  .bind(provider)
  .bind(cost_micros)
  .bind(agent_id)
  .bind(provider_key_id)
  .bind(now)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Failed to seed analytics event");
}

async fn seed_provider_key(
  pool: &SqlitePool,
  key_id: i64,
  provider: &str,
  user_id: &str,
  alias: Option<&str>,
) {
  sqlx::query(
    "INSERT OR IGNORE INTO ai_provider_keys \
     (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id, alias) \
     VALUES (?, ?, 'enc', 'nonce', 1, 0, ?, ?)",
  )
  .bind(key_id)
  .bind(provider)
  .bind(user_id)
  .bind(alias)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Failed to seed provider key");
}

// ============================================================================
// Response extraction
// ============================================================================

async fn extract_response(response: axum::response::Response) -> (StatusCode, String) {
  let status = response.status();
  let bytes = http_body_util::BodyExt::collect(response.into_body())
    .await
    .expect("LOUD FAILURE: Failed to read response body")
    .to_bytes();
  let body =
    String::from_utf8(bytes.to_vec()).expect("LOUD FAILURE: Response body must be valid UTF-8");
  (status, body)
}

// ============================================================================
// Router and state builder
// ============================================================================

/// Build the combined state and router for spending endpoint tests.
///
/// Returns the router, the database pool (for seeding), and an admin JWT token.
async fn build_spending_router() -> (Router, SqlitePool, String) {
  let app_state = TestAppState::new().await;
  let pool = app_state.database.clone();

  // Create admin user and generate JWT
  let (admin_id, _) = create_test_admin(&pool).await;
  let admin_token = create_test_access_token(
    &admin_id,
    "admin@admin.com",
    "admin",
    "test_jwt_secret_key_for_testing_12345",
  );

  let analytics = AnalyticsState {
    pool: pool.clone(),
    ic_token_manager: Arc::new(IcTokenManager::new("test_ic_secret".to_string())),
    ic_token_rate_limiter: IcTokenRateLimiter::new(),
  };

  let combined = TestAnalyticsAppState {
    auth: app_state.auth.clone(),
    analytics,
  };

  let router = Router::new()
    .route(
      "/api/v1/analytics/spending/total",
      get(iron_control_api::routes::analytics::get_spending_total),
    )
    .route(
      "/api/v1/analytics/spending/by-agent",
      get(iron_control_api::routes::analytics::get_spending_by_agent),
    )
    .route(
      "/api/v1/analytics/spending/by-provider",
      get(iron_control_api::routes::analytics::get_spending_by_provider),
    )
    .route(
      "/api/v1/analytics/spending/avg-per-request",
      get(iron_control_api::routes::analytics::get_spending_avg),
    )
    .with_state(combined);

  (router, pool, admin_token)
}

// ============================================================================
// Test 1: Spending total filters by provider
// ============================================================================

/// Validates that `provider_id` query parameter correctly filters spending
/// by the `provider` column (not a non-existent `provider_id` column).
#[tokio::test]
async fn test_spending_total_filters_by_provider() {
  let (router, pool, token) = build_spending_router().await;
  let admin_id = "user_admin_test";

  // Seed an agent owned by the admin
  seed_agent(&pool, 1, "test-agent", admin_id).await;

  // Seed events: 2 openai (1000 + 2000 = 3000 micros), 1 anthropic (5000 micros)
  seed_analytics_event(&pool, "evt_oa_1", "openai", 1000, 1, None).await;
  seed_analytics_event(&pool, "evt_oa_2", "openai", 2000, 1, None).await;
  seed_analytics_event(&pool, "evt_an_1", "anthropic", 5000, 1, None).await;

  // Query with provider_id=openai
  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/spending/total?period=all-time&provider_id=openai")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let (status, body) = extract_response(response).await;
  assert_eq!(status, StatusCode::OK, "Expected 200 OK, body: {body}");

  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse JSON: {body}"));

  // 3000 micros = 0.003 USD
  let total_spend = json["total_spend"].as_f64().expect("total_spend should be a number");
  assert!(
    (total_spend - 0.003).abs() < 1e-9,
    "Expected total_spend = 0.003 (openai only), got {total_spend}"
  );

  // Verify the provider_id filter is echoed back
  assert_eq!(
    json["filters"]["provider_id"].as_str(),
    Some("openai"),
    "Filter should echo back provider_id=openai"
  );
}

// ============================================================================
// Test 2: Spending avg includes median
// ============================================================================

/// Validates that `median_cost_per_request` is returned and correctly computed.
/// With costs [100, 200, 300, 400, 500], median = 300 micros = 0.0003 USD.
#[tokio::test]
async fn test_spending_avg_includes_median() {
  let (router, pool, token) = build_spending_router().await;
  let admin_id = "user_admin_test";

  seed_agent(&pool, 10, "median-agent", admin_id).await;

  // Seed 5 events with known costs
  for (i, cost) in [100i64, 200, 300, 400, 500].iter().enumerate() {
    seed_analytics_event(
      &pool,
      &format!("evt_med_{i}"),
      "openai",
      *cost,
      10,
      None,
    )
    .await;
  }

  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/spending/avg-per-request?period=all-time")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let (status, body) = extract_response(response).await;
  assert_eq!(status, StatusCode::OK, "Expected 200 OK, body: {body}");

  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse JSON: {body}"));

  // Verify median_cost_per_request exists and is correct
  let median = json["median_cost_per_request"]
    .as_f64()
    .expect("median_cost_per_request should be a number");

  // Median of [100, 200, 300, 400, 500] = 300 micros = 0.0003 USD
  assert!(
    (median - 0.0003).abs() < 1e-9,
    "Expected median = 0.0003 (300 micros), got {median}"
  );

  // Verify other stats are present
  assert_eq!(json["total_requests"].as_i64(), Some(5));
  assert!(json["min_cost_per_request"].as_f64().is_some());
  assert!(json["max_cost_per_request"].as_f64().is_some());
}

// ============================================================================
// Test 3: Spending avg median with provider filter
// ============================================================================

/// Validates that `compute_median` correctly uses the `provider` column filter.
/// openai costs: [100, 300, 500] -> median = 300 micros = 0.0003 USD.
/// anthropic costs: [200, 400] -> should be excluded.
#[tokio::test]
async fn test_spending_avg_median_with_provider_filter() {
  let (router, pool, token) = build_spending_router().await;
  let admin_id = "user_admin_test";

  seed_agent(&pool, 20, "provider-filter-agent", admin_id).await;

  // openai events
  seed_analytics_event(&pool, "evt_pf_oa_1", "openai", 100, 20, None).await;
  seed_analytics_event(&pool, "evt_pf_oa_2", "openai", 300, 20, None).await;
  seed_analytics_event(&pool, "evt_pf_oa_3", "openai", 500, 20, None).await;

  // anthropic events (should be excluded by filter)
  seed_analytics_event(&pool, "evt_pf_an_1", "anthropic", 200, 20, None).await;
  seed_analytics_event(&pool, "evt_pf_an_2", "anthropic", 400, 20, None).await;

  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/spending/avg-per-request?period=all-time&provider_id=openai")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let (status, body) = extract_response(response).await;
  assert_eq!(status, StatusCode::OK, "Expected 200 OK, body: {body}");

  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse JSON: {body}"));

  // Median of [100, 300, 500] = 300 micros = 0.0003 USD
  let median = json["median_cost_per_request"]
    .as_f64()
    .expect("median_cost_per_request should be a number");
  assert!(
    (median - 0.0003).abs() < 1e-9,
    "Expected median = 0.0003 (openai-only median of [100,300,500]), got {median}"
  );

  // Only 3 openai requests should be counted
  assert_eq!(
    json["total_requests"].as_i64(),
    Some(3),
    "Should count only openai requests"
  );
}

// ============================================================================
// Test 4: Spending by provider grouped by key
// ============================================================================

/// Validates that `group_by=key` returns `provider_key_id` and `alias` fields
/// in the by-provider response.
#[tokio::test]
async fn test_spending_by_provider_group_by_key() {
  let (router, pool, token) = build_spending_router().await;
  let admin_id = "user_admin_test";

  seed_agent(&pool, 30, "key-group-agent", admin_id).await;

  // Seed provider keys with aliases
  seed_provider_key(&pool, 100, "openai", admin_id, Some("prod-key")).await;
  seed_provider_key(&pool, 101, "openai", admin_id, Some("dev-key")).await;

  // Events using different keys
  seed_analytics_event(&pool, "evt_kg_1", "openai", 1000, 30, Some(100)).await;
  seed_analytics_event(&pool, "evt_kg_2", "openai", 2000, 30, Some(100)).await;
  seed_analytics_event(&pool, "evt_kg_3", "openai", 3000, 30, Some(101)).await;

  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/spending/by-provider?period=all-time&group_by=key")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let (status, body) = extract_response(response).await;
  assert_eq!(status, StatusCode::OK, "Expected 200 OK, body: {body}");

  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse JSON: {body}"));

  let data = json["data"]
    .as_array()
    .expect("data should be an array");

  // Should have 2 groups (one per key)
  assert_eq!(data.len(), 2, "Should have 2 provider key groups, got {}", data.len());

  // Verify provider_key_id and alias fields are present
  for entry in data {
    assert!(
      entry.get("provider_key_id").is_some(),
      "Each entry should have provider_key_id field"
    );
    assert!(
      entry.get("alias").is_some(),
      "Each entry should have alias field"
    );
  }

  // Find the entry for key 100 (prod-key) and verify spending
  let prod_entry = data
    .iter()
    .find(|e| e["provider_key_id"].as_i64() == Some(100))
    .expect("Should have entry for provider_key_id=100");
  assert_eq!(
    prod_entry["alias"].as_str(),
    Some("prod-key"),
    "Key 100 should have alias 'prod-key'"
  );
  // 1000 + 2000 = 3000 micros = 0.003 USD
  let prod_spending = prod_entry["spending"].as_f64().unwrap();
  assert!(
    (prod_spending - 0.003).abs() < 1e-9,
    "prod-key spending should be 0.003, got {prod_spending}"
  );

  // Find the entry for key 101 (dev-key) and verify spending
  let dev_entry = data
    .iter()
    .find(|e| e["provider_key_id"].as_i64() == Some(101))
    .expect("Should have entry for provider_key_id=101");
  assert_eq!(
    dev_entry["alias"].as_str(),
    Some("dev-key"),
    "Key 101 should have alias 'dev-key'"
  );
  // 3000 micros = 0.003 USD
  let dev_spending = dev_entry["spending"].as_f64().unwrap();
  assert!(
    (dev_spending - 0.003).abs() < 1e-9,
    "dev-key spending should be 0.003, got {dev_spending}"
  );
}

// ============================================================================
// Test 5: Spending by agent with provider_key_id filter
// ============================================================================

/// Validates that the `provider_key_id` filter correctly limits results
/// in the by-agent endpoint to events matching the given key.
#[tokio::test]
async fn test_spending_by_agent_with_provider_key_filter() {
  let (router, pool, token) = build_spending_router().await;
  let admin_id = "user_admin_test";

  seed_agent(&pool, 40, "key-filter-agent", admin_id).await;

  // Seed provider keys
  seed_provider_key(&pool, 200, "openai", admin_id, Some("key-a")).await;
  seed_provider_key(&pool, 201, "openai", admin_id, Some("key-b")).await;

  // Events: key-a gets 2 events (1000 + 2000 = 3000 micros), key-b gets 1 event (5000 micros)
  seed_analytics_event(&pool, "evt_kf_1", "openai", 1000, 40, Some(200)).await;
  seed_analytics_event(&pool, "evt_kf_2", "openai", 2000, 40, Some(200)).await;
  seed_analytics_event(&pool, "evt_kf_3", "openai", 5000, 40, Some(201)).await;

  // Filter by provider_key_id=200 (key-a)
  let response = router
    .oneshot(
      Request::builder()
        .method(Method::GET)
        .uri("/api/v1/analytics/spending/by-agent?period=all-time&provider_key_id=200")
        .header("Authorization", format!("Bearer {token}"))
        .body(Body::empty())
        .unwrap(),
    )
    .await
    .unwrap();

  let (status, body) = extract_response(response).await;
  assert_eq!(status, StatusCode::OK, "Expected 200 OK, body: {body}");

  let json: serde_json::Value = serde_json::from_str(&body)
    .unwrap_or_else(|_| panic!("LOUD FAILURE: Failed to parse JSON: {body}"));

  let data = json["data"]
    .as_array()
    .expect("data should be an array");

  // Should have exactly 1 agent entry
  assert_eq!(data.len(), 1, "Should have 1 agent, got {}", data.len());

  let agent = &data[0];
  assert_eq!(agent["agent_id"].as_i64(), Some(40));

  // Only key-a events counted: 1000 + 2000 = 3000 micros = 0.003 USD
  let spending = agent["spending"].as_f64().unwrap();
  assert!(
    (spending - 0.003).abs() < 1e-9,
    "Spending should be 0.003 (key-a only), got {spending}"
  );

  // Only 2 requests from key-a
  assert_eq!(
    agent["request_count"].as_i64(),
    Some(2),
    "Should count only key-a requests"
  );
}
