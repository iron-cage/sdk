//! Direct tests for `validate_ic_token_for_endpoint`
//!
//! Tests each code path of the orchestrator function that combines
//! rate limiting, JWT validation, database hash-check, and timing padding.
//!
//! # Coverage (GAP-2)
//!
//! | Scenario                         | Expected status | Failure recorded? |
//! |----------------------------------|-----------------|-------------------|
//! | Rate limit exceeded              | 429             | (already recorded)|
//! | Invalid JWT                      | 401             | yes               |
//! | Token revoked (NULL hash)        | 401             | yes               |
//! | Database error                   | 500             | no                |
//! | Valid token                      | `Ok(agent_id)`  | no                |
//!
//! Database errors must NOT increment the rate limiter — the failure is on
//! the server side, not the client's fault. Counting them would allow a DoS
//! attack via error amplification.
//!
//! # Authority
//! - PR review finding: GAP-2

mod common;

use sqlx::SqlitePool;

use iron_control_api::ic_token::{
  sha256_hash, validate_ic_token_for_endpoint, IcTokenClaims, IcTokenManager, IcTokenRateLimiter,
};

async fn setup_db() -> SqlitePool {
  let pool = SqlitePool::connect("sqlite::memory:")
    .await
    .expect("LOUD FAILURE: Should connect to in-memory SQLite");
  sqlx::query(
    "CREATE TABLE agents (
      id INTEGER PRIMARY KEY,
      name TEXT NOT NULL,
      providers TEXT NOT NULL DEFAULT '[]',
      created_at INTEGER NOT NULL DEFAULT 0,
      ic_token_hash TEXT
    )",
  )
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Should create agents table");
  pool
}

fn test_manager() -> IcTokenManager {
  IcTokenManager::new("endpoint_test_secret".to_string())
}

fn create_valid_token(manager: &IcTokenManager, agent_id: i64) -> String {
  let claims = IcTokenClaims::new(
    format!("agent_{agent_id}"),
    format!("budget_{agent_id}"),
    vec!["llm:call".to_string()],
    None,
  );
  manager
    .generate_token(&claims)
    .expect("LOUD FAILURE: Should generate token")
}

async fn insert_agent_with_hash(pool: &SqlitePool, agent_id: i64, hash: Option<&str>) {
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, ic_token_hash) VALUES (?, ?, '[]', 0, ?)",
  )
  .bind(agent_id)
  .bind(format!("agent_{agent_id}"))
  .bind(hash)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Should insert agent");
}

#[tokio::test]
async fn test_endpoint_returns_429_when_rate_limited() {
  let pool = setup_db().await;
  let manager = test_manager();
  let rate_limiter = IcTokenRateLimiter::new();
  let token = create_valid_token(&manager, 1);

  // Pre-fill the rate limiter to the threshold (20 failures).
  for _ in 0..20 {
    rate_limiter.record_failure(&token);
  }

  let result = validate_ic_token_for_endpoint(&manager, &token, &pool, &rate_limiter).await;

  let response = result.expect_err("LOUD FAILURE: rate-limited request must return Err response");
  assert_eq!(
    response.status(),
    axum::http::StatusCode::TOO_MANY_REQUESTS,
    "LOUD FAILURE: rate-limited request must return 429"
  );
}

#[tokio::test]
async fn test_endpoint_returns_401_for_invalid_jwt_and_records_failure() {
  let pool = setup_db().await;
  let manager = test_manager();
  let rate_limiter = IcTokenRateLimiter::new();
  let bad_token = "not.a.valid.jwt";

  let result = validate_ic_token_for_endpoint(&manager, bad_token, &pool, &rate_limiter).await;

  let response = result.expect_err("LOUD FAILURE: invalid JWT must return Err response");
  assert_eq!(
    response.status(),
    axum::http::StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: invalid JWT must return 401"
  );

  // Auth failures must be recorded so repeated attempts trigger 429.
  // After 19 more failures the limiter must block.
  for _ in 0..19 {
    rate_limiter.record_failure(bad_token);
  }
  assert!(
    rate_limiter.check(bad_token).is_err(),
    "LOUD FAILURE: failure must have been recorded — 20 total should trigger rate limit"
  );
}

#[tokio::test]
async fn test_endpoint_returns_401_for_revoked_token_and_records_failure() {
  let pool = setup_db().await;
  let manager = test_manager();
  let rate_limiter = IcTokenRateLimiter::new();
  let token = create_valid_token(&manager, 5);

  // Agent exists but ic_token_hash is NULL (revoked).
  insert_agent_with_hash(&pool, 5, None).await;

  let result = validate_ic_token_for_endpoint(&manager, &token, &pool, &rate_limiter).await;

  let response = result.expect_err("LOUD FAILURE: revoked token must return Err response");
  assert_eq!(
    response.status(),
    axum::http::StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: revoked token must return 401"
  );

  // Failure must have been recorded.
  for _ in 0..19 {
    rate_limiter.record_failure(&token);
  }
  assert!(
    rate_limiter.check(&token).is_err(),
    "LOUD FAILURE: failure must have been recorded — 20 total should trigger rate limit"
  );
}

#[tokio::test]
async fn test_endpoint_returns_500_for_db_error_and_does_not_record_failure() {
  let pool = setup_db().await;
  let manager = test_manager();
  let rate_limiter = IcTokenRateLimiter::new();
  let token = create_valid_token(&manager, 10);

  // Close pool to simulate database unavailability.
  pool.close().await;

  let result = validate_ic_token_for_endpoint(&manager, &token, &pool, &rate_limiter).await;

  let response = result.expect_err("LOUD FAILURE: DB error must return Err response");
  assert_eq!(
    response.status(),
    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
    "LOUD FAILURE: DB error must return 500, not 401"
  );

  // Database errors must NOT be counted as auth failures.
  // After 19 genuine failures the limiter must still allow the token
  // (demonstrating the DB error did not consume a failure slot).
  for _ in 0..19 {
    rate_limiter.record_failure(&token);
  }
  assert!(
    rate_limiter.check(&token).is_ok(),
    "LOUD FAILURE: DB error must not increment the failure counter — \
     only 19 explicit failures recorded, limiter should still allow"
  );
}

#[tokio::test]
async fn test_endpoint_returns_ok_for_valid_token() {
  let pool = setup_db().await;
  let manager = test_manager();
  let rate_limiter = IcTokenRateLimiter::new();
  let token = create_valid_token(&manager, 42);
  let token_hash = sha256_hash(&token);

  insert_agent_with_hash(&pool, 42, Some(&token_hash)).await;

  let result = validate_ic_token_for_endpoint(&manager, &token, &pool, &rate_limiter).await;

  let (agent_id, claims) = result.expect("LOUD FAILURE: valid token must return Ok");
  assert_eq!(
    agent_id, 42,
    "LOUD FAILURE: returned agent_id must be 42, got {agent_id}"
  );
  assert_eq!(
    claims.agent_id, "agent_42",
    "LOUD FAILURE: returned claims.agent_id must be 'agent_42', got '{}'",
    claims.agent_id
  );
}
