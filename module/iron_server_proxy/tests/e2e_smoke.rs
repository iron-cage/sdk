//! End-to-end smoke tests for `iron_server_proxy`.
//!
//! Verifies the full request pipeline:
//! agent (IC Token) -> server proxy -> mock LLM provider -> response to agent.
//!
//! Uses wiremock to mock the LLM provider and in-memory `SQLite` for the database.

use core::{net::SocketAddr, time::Duration};
use std::sync::Arc;

use axum::{routing, Router};
use reqwest::Client;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use tokio::net::TcpListener;
use wiremock::{matchers, Mock, MockServer, ResponseTemplate};

use iron_cost::pricing::PricingManager;
use iron_secrets::crypto::CryptoService;
use iron_server_proxy::{proxy, AppState, AuthRateLimiter};
use iron_token_manager::ProviderKeyStorage;

/// Test master key (32 bytes) for AES-256-GCM.
const TEST_MASTER_KEY: [u8; 32] = [42u8; 32];

/// Test IC token (raw, before hashing).
const TEST_IC_TOKEN: &str = "test_ic_token_for_e2e_smoke_test_2026";

/// Test user ID.
const TEST_USER_ID: &str = "user_e2e_test";

/// Build the proxy router (mirrors `server::build_router` which is private).
fn build_test_router(state: AppState) -> Router {
  Router::new()
    .route("/health", routing::get(proxy::handle_health))
    .route("/v1/{*path}", routing::any(proxy::handle_proxy))
    .with_state(state)
}

/// Create in-memory `SQLite` pool with all migrations applied.
async fn setup_test_db() -> SqlitePool {
  let pool = SqlitePool::connect("sqlite::memory:")
    .await
    .expect("Failed to connect to in-memory SQLite");

  iron_token_manager::apply_all_migrations(&pool)
    .await
    .expect("Failed to apply migrations");

  pool
}

/// Insert a test user into the database.
async fn create_test_user(pool: &SqlitePool) {
  sqlx::query(
    "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, 'admin', 1, $5)",
  )
  .bind(TEST_USER_ID)
  .bind("e2e_admin")
  .bind(format!(
    "{:x}",
    Sha256::digest(b"e2e_test_placeholder_password")
  ))
  .bind("e2e@test.local")
  .bind(1_700_000_000_000_i64)
  .execute(pool)
  .await
  .expect("Failed to create test user");
}

/// Encrypt a test API key and insert it into `ai_provider_keys`.
/// Returns the provider key ID.
async fn create_test_provider_key(
  pool: &SqlitePool,
  crypto: &CryptoService,
  mock_base_url: &str,
) -> i64 {
  let encrypted = crypto
    .encrypt("sk-test-fake-openai-key-12345")
    .expect("Failed to encrypt test key");

  let storage = ProviderKeyStorage::new(pool.clone());
  storage
    .create_key(
      iron_token_manager::ProviderType::OpenAI,
      &encrypted.ciphertext_base64(),
      &encrypted.nonce_base64(),
      Some(mock_base_url),
      Some("E2E test key"),
      TEST_USER_ID,
    )
    .await
    .expect("Failed to create provider key")
}

/// Create a test agent with an IC token hash and assigned provider key.
/// Returns the agent ID.
async fn create_test_agent(pool: &SqlitePool, provider_key_id: i64, ic_token: &str) -> i64 {
  let token_hash = format!("{:x}", Sha256::digest(ic_token.as_bytes()));

  let result = sqlx::query(
    "INSERT INTO agents (name, providers, created_at, owner_id, provider_key_id, ic_token_hash) \
     VALUES ($1, $2, $3, $4, $5, $6)",
  )
  .bind("e2e_test_agent")
  .bind("[]")
  .bind(1_700_000_000_000_i64)
  .bind(TEST_USER_ID)
  .bind(provider_key_id)
  .bind(&token_hash)
  .execute(pool)
  .await
  .expect("Failed to create test agent");

  result.last_insert_rowid()
}

/// Build `AppState` from test components (bypasses `AppState::new` for full control).
fn build_test_state(pool: SqlitePool, crypto: Arc<CryptoService>) -> AppState {
  AppState {
    provider_key_storage: ProviderKeyStorage::new(pool.clone()),
    db_pool: pool,
    crypto_service: crypto,
    http_client: Client::builder()
      .timeout(Duration::from_secs(10))
      .build()
      .expect("Failed to build HTTP client"),
    pricing_manager: Arc::new(PricingManager::new().expect("Failed to init pricing manager")),
    auth_rate_limiter: AuthRateLimiter::new(),
  }
}

/// Start the proxy server on a random port, returns the base URL.
async fn start_test_proxy(state: AppState) -> String {
  let router = build_test_router(state);
  let listener = TcpListener::bind("127.0.0.1:0")
    .await
    .expect("Failed to bind test listener");
  let addr = listener.local_addr().expect("Failed to get local addr");

  tokio::spawn(async move {
    axum::serve(
      listener,
      router.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("Test proxy server crashed");
  });

  format!("http://{addr}")
}

/// Standard mock `OpenAI` chat completions response.
fn mock_openai_response() -> Value {
  serde_json::json!({
    "id": "chatcmpl-e2e-test",
    "object": "chat.completion",
    "created": 1_700_000_000_i64,
    "model": "gpt-4",
    "choices": [{
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "Hello from mock provider!"
      },
      "finish_reason": "stop"
    }],
    "usage": {
      "prompt_tokens": 10,
      "completion_tokens": 5,
      "total_tokens": 15
    }
  })
}

/// Standard chat completion request body.
fn chat_request_body() -> Value {
  serde_json::json!({
    "model": "gpt-4",
    "messages": [{"role": "user", "content": "Hello!"}]
  })
}

/// Full happy path: agent -> proxy -> mock provider -> response.
/// Verifies: response received, API key not leaked, spending incremented.
#[tokio::test]
async fn test_e2e_chat_completion_success() {
  // Setup mock LLM provider
  let mock_server = MockServer::start().await;
  Mock::given(matchers::method("POST"))
    .and(matchers::path("/v1/chat/completions"))
    .respond_with(ResponseTemplate::new(200).set_body_json(mock_openai_response()))
    .expect(1)
    .mount(&mock_server)
    .await;

  // Setup database
  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let _agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  // Start proxy
  let state = build_test_state(pool.clone(), crypto);
  let proxy_url = start_test_proxy(state).await;

  // Send request through the proxy
  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", format!("Bearer {TEST_IC_TOKEN}"))
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 200, "Expected 200 OK");

  let body = resp
    .json::<Value>()
    .await
    .expect("Failed to parse response");

  // Verify response content
  assert_eq!(
    body["choices"][0]["message"]["content"],
    "Hello from mock provider!"
  );

  // Verify API key not leaked in response
  let body_str = serde_json::to_string(&body).expect("serialize");
  assert!(
    !body_str.contains("sk-test-fake-openai-key"),
    "API key leaked in response!"
  );

  // Verify spending was incremented
  let storage = ProviderKeyStorage::new(pool);
  let summary = storage
    .get_spending_summary(key_id)
    .await
    .expect("Failed to get spending");
  assert!(
    summary.used_microdollars > 0,
    "Spending should be incremented after successful request"
  );
}

/// Invalid IC Token returns 401 Unauthorized.
#[tokio::test]
async fn test_e2e_invalid_ic_token_returns_401() {
  let mock_server = MockServer::start().await;

  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let _agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  // Send request with wrong IC token
  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", "Bearer totally_wrong_token")
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 401, "Expected 401 Unauthorized");

  let body = resp.json::<Value>().await.expect("parse error response");
  assert_eq!(body["error"]["code"], "invalid_token");
}

/// Missing IC Token returns 401 Unauthorized.
#[tokio::test]
async fn test_e2e_missing_ic_token_returns_401() {
  let pool = setup_test_db().await;
  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  // Send request without any auth header
  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 401, "Expected 401 for missing token");
}

/// Revoked IC Token (hash removed from DB) returns 401.
#[tokio::test]
async fn test_e2e_revoked_ic_token_returns_401() {
  let mock_server = MockServer::start().await;

  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  // Revoke the IC token by clearing the hash
  sqlx::query("UPDATE agents SET ic_token_hash = NULL WHERE id = $1")
    .bind(agent_id)
    .execute(&pool)
    .await
    .expect("Failed to revoke IC token");

  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", format!("Bearer {TEST_IC_TOKEN}"))
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 401, "Revoked token should return 401");
}

/// Spending cap exceeded returns 402 Payment Required.
#[tokio::test]
async fn test_e2e_spending_cap_exceeded_returns_402() {
  let mock_server = MockServer::start().await;

  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let _agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  // Set a spending cap and exhaust it
  let storage = ProviderKeyStorage::new(pool.clone());
  storage
    .set_spending_cap(key_id, Some(1000))
    .await
    .expect("Failed to set spending cap");

  // Exhaust the cap by setting used = cap
  sqlx::query("UPDATE ai_provider_keys SET spending_used_microdollars = 1000 WHERE id = $1")
    .bind(key_id)
    .execute(&pool)
    .await
    .expect("Failed to exhaust spending");

  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", format!("Bearer {TEST_IC_TOKEN}"))
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 402, "Exhausted cap should return 402");

  let body = resp.json::<Value>().await.expect("parse error response");
  assert_eq!(body["error"]["code"], "spending_cap_exceeded");
}

/// Health endpoint returns 200 without authentication.
#[tokio::test]
async fn test_e2e_health_endpoint() {
  let pool = setup_test_db().await;
  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  let client = Client::new();
  let resp = client
    .get(format!("{proxy_url}/health"))
    .send()
    .await
    .expect("Health request failed");

  assert_eq!(resp.status(), 200);

  let body = resp.json::<Value>().await.expect("parse health response");
  assert_eq!(body["status"], "ok");
}

/// Streaming responses must preserve the pre-flight spending reservation.
/// Regression test: previously `cost_info: None` on streaming caused
/// `adjust_spending(estimated, 0)`, releasing the full reservation and
/// tracking the request as free — allowing spending cap bypass over time.
#[tokio::test]
async fn test_e2e_streaming_preserves_spending_reservation() {
  // Setup mock LLM provider returning SSE streaming response
  let mock_server = MockServer::start().await;
  let sse_body = "\
    data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"Hi\"},\"finish_reason\":null}]}\n\n\
    data: {\"id\":\"chatcmpl-stream\",\"object\":\"chat.completion.chunk\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"!\"},\"finish_reason\":\"stop\"}]}\n\n\
    data: [DONE]\n\n";
  Mock::given(matchers::method("POST"))
    .and(matchers::path("/v1/chat/completions"))
    .respond_with(
      ResponseTemplate::new(200)
        .insert_header("content-type", "text/event-stream")
        .set_body_string(sse_body),
    )
    .expect(1)
    .mount(&mock_server)
    .await;

  // Setup database
  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let _agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  // Verify spending starts at zero
  let storage = ProviderKeyStorage::new(pool.clone());
  let before = storage
    .get_spending_summary(key_id)
    .await
    .expect("spending summary");
  assert_eq!(before.used_microdollars, 0, "Spending should start at 0");

  // Start proxy and send streaming request
  let state = build_test_state(pool.clone(), crypto);
  let proxy_url = start_test_proxy(state).await;

  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", format!("Bearer {TEST_IC_TOKEN}"))
    .json(&serde_json::json!({
      "model": "gpt-4",
      "messages": [{"role": "user", "content": "Hello!"}],
      "stream": true
    }))
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 200, "Expected 200 OK");

  // Consume the streaming response fully
  let _body = resp.bytes().await.expect("Failed to read streaming body");

  // Allow a small window for the proxy to finish its post-response work
  tokio::time::sleep(Duration::from_millis(50)).await;

  // Verify spending was NOT released back to zero
  let after = storage
    .get_spending_summary(key_id)
    .await
    .expect("spending summary");
  assert!(
    after.used_microdollars > 0,
    "Streaming request spending must be preserved (got {}), \
     not released to zero — otherwise spending cap can be bypassed",
    after.used_microdollars
  );
}

/// IC Token via x-api-key header also works.
#[tokio::test]
async fn test_e2e_x_api_key_header_auth() {
  let mock_server = MockServer::start().await;
  Mock::given(matchers::method("POST"))
    .and(matchers::path("/v1/chat/completions"))
    .respond_with(ResponseTemplate::new(200).set_body_json(mock_openai_response()))
    .expect(1)
    .mount(&mock_server)
    .await;

  let pool = setup_test_db().await;
  create_test_user(&pool).await;

  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let key_id = create_test_provider_key(&pool, &crypto, &mock_server.uri()).await;
  let _agent_id = create_test_agent(&pool, key_id, TEST_IC_TOKEN).await;

  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  // Use x-api-key instead of Authorization header
  let client = Client::new();
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("x-api-key", TEST_IC_TOKEN)
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 200, "x-api-key auth should work");
}

/// Repeated auth failures from the same IP trigger 429 rate limiting.
#[tokio::test]
async fn test_e2e_rate_limit_after_repeated_auth_failures() {
  let pool = setup_test_db().await;
  let crypto = Arc::new(CryptoService::new(&TEST_MASTER_KEY).expect("crypto init"));
  let state = build_test_state(pool, crypto);
  let proxy_url = start_test_proxy(state).await;

  let client = Client::new();

  // Send 20 requests with invalid tokens (the rate limit threshold)
  for i in 0..20 {
    let resp = client
      .post(format!("{proxy_url}/v1/chat/completions"))
      .header("Authorization", format!("Bearer bad_token_{i}"))
      .json(&chat_request_body())
      .send()
      .await
      .expect("Request failed");
    assert_eq!(resp.status(), 401, "Request {i} should return 401");
  }

  // The 21st request should be rate-limited (429)
  let resp = client
    .post(format!("{proxy_url}/v1/chat/completions"))
    .header("Authorization", "Bearer one_more_bad_token")
    .json(&chat_request_body())
    .send()
    .await
    .expect("Request failed");

  assert_eq!(resp.status(), 429, "Should return 429 after 20 failures");

  let retry_after = resp
    .headers()
    .get("retry-after")
    .expect("Missing Retry-After header");
  let retry_secs: u64 = retry_after
    .to_str()
    .expect("valid string")
    .parse()
    .expect("valid number");
  assert!(retry_secs > 0, "Retry-After must be positive");
  assert!(retry_secs <= 61, "Retry-After must be within window");

  let body = resp.json::<Value>().await.expect("parse error response");
  assert_eq!(body["error"]["code"], "rate_limited");
}
