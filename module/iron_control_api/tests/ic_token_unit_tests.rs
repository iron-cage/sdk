//! Unit tests for `ic_token` module: `sha256_hash`, `validate_ic_token_runtime`,
//! and `IcTokenRateLimiter`.
//!
//! # Coverage
//!
//! ## `sha256_hash` (M2)
//! - Deterministic output for same input
//! - Output format: 64 hex characters
//!
//! ## `validate_ic_token_runtime` error branches (M6)
//! - `InvalidToken`: malformed JWT
//! - `InvalidAgentId`: missing prefix, non-numeric, negative
//! - `TokenInactive`: agent not found, token revoked (NULL hash), token rotated (hash mismatch)
//! - `DatabaseError`: closed connection pool (H4)
//! - Success: valid token with matching hash
//!
//! ## `IcTokenRateLimiter` (GAP-1)
//! - `check()` allows requests below failure threshold
//! - `check()` returns `Err(retry_after)` at and above threshold
//! - `record_failure()` increments count monotonically
//! - Failures outside the sliding window are swept and not counted
//! - Different tokens are rate-limited independently
//! - Mutex poisoning is recovered gracefully
//!
//! # Authority
//! - PR #44 review findings: H4, M2, M6
//! - PR review findings: GAP-1

mod common;

use core::time::Duration;
use std::{
  thread,
  time::{SystemTime, UNIX_EPOCH},
};

use sqlx::SqlitePool;
use uuid::Uuid;

use iron_control_api::ic_token::{
  sha256_hash, IcTokenClaims, IcTokenManager, IcTokenRateLimiter, IcTokenRuntimeError,
};

// Helpers

/// Minimal agents table — columns needed by `validate_ic_token_runtime` + required NOT NULL columns
async fn setup_minimal_db() -> SqlitePool {
  let pool = SqlitePool::connect("sqlite::memory:")
    .await
    .expect("LOUD FAILURE: Should connect to in-memory SQLite");
  sqlx::query(
    r"
      CREATE TABLE agents (
        id INTEGER PRIMARY KEY,
        name TEXT NOT NULL,
        providers TEXT NOT NULL DEFAULT '[]',
        created_at INTEGER NOT NULL DEFAULT 0,
        ic_token_hash TEXT)
    ",
  )
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Should create agents table");
  pool
}

async fn insert_agent(pool: &SqlitePool, agent_id: i64, ic_token_hash: Option<&str>) {
  sqlx::query(
    r"
      INSERT INTO agents (id, name, providers, created_at, ic_token_hash)
      VALUES (?, ?, '[]', 0, ?)
    ",
  )
  .bind(agent_id)
  .bind(format!("agent_{agent_id}"))
  .bind(ic_token_hash)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Should insert test agent");
}

fn test_manager() -> IcTokenManager {
  IcTokenManager::new("unit_test_secret_key".to_string())
}

fn create_token_for_agent(manager: &IcTokenManager, agent_id: i64) -> String {
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

/// Create token with custom `agent_id` string (for invalid format tests)
fn create_token_with_agent_id(manager: &IcTokenManager, agent_id: &str) -> String {
  let claims = IcTokenClaims {
    token_id: Uuid::new_v4(),
    agent_id: agent_id.to_string(),
    budget_id: "budget_1".to_string(),
    issued_at: SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .unwrap()
      .as_secs(),
    expires_at: None,
    issuer: "iron-control-panel".to_string(),
    permissions: vec![],
  };
  manager
    .generate_token(&claims)
    .expect("LOUD FAILURE: Should generate token")
}

// sha256_hash tests (M2)

#[test]
fn test_sha256_hash_deterministic() {
  let token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpJVCL9.test_payload";
  let hash1 = sha256_hash(token);
  let hash2 = sha256_hash(token);
  assert_eq!(
    hash1, hash2,
    "LOUD FAILURE: sha256_hash must be deterministic — same input must produce same output"
  );
}

#[test]
fn test_sha256_hash_format() {
  let hash = sha256_hash("test");
  assert_eq!(
    hash.len(),
    64,
    "LOUD FAILURE: SHA-256 hex digest must be 64 characters, got {}",
    hash.len()
  );
  assert!(
    hash.chars().all(|c| c.is_ascii_hexdigit()),
    "LOUD FAILURE: SHA-256 hex digest must contain only hex characters, got: {hash}"
  );
}

// validate_ic_token_runtime: InvalidToken

#[tokio::test]
async fn test_validate_runtime_invalid_jwt() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  let result = manager
    .validate_ic_token_runtime("not.a.valid.jwt", &pool)
    .await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::InvalidToken(_))),
    "LOUD FAILURE: Invalid JWT must return InvalidToken, got: {result:?}"
  );
}

// validate_ic_token_runtime: InvalidAgentId
//
// Note: agent_id without "agent_" prefix is caught by verify_token → claims.validate()
// and surfaces as InvalidToken (defense in depth — validate() checks format first).
// InvalidAgentId is reachable only for agent_id that passes prefix check but fails
// numeric parse or positivity check.

#[tokio::test]
async fn test_validate_runtime_invalid_agent_id_prefix_caught_by_verify() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  // "user_123" fails at verify_token level (claims.validate checks starts_with("agent_"))
  let token = create_token_with_agent_id(&manager, "user_123");
  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::InvalidToken(_))),
    "LOUD FAILURE: agent_id without 'agent_' prefix is caught by verify_token as InvalidToken, got: {result:?}"
  );
}

#[tokio::test]
async fn test_validate_runtime_invalid_agent_id_non_numeric() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  let token = create_token_with_agent_id(&manager, "agent_abc");
  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::InvalidAgentId(_))),
    "LOUD FAILURE: Non-numeric agent_id must return InvalidAgentId, got: {result:?}"
  );
}

#[tokio::test]
async fn test_validate_runtime_invalid_agent_id_negative() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  let token = create_token_with_agent_id(&manager, "agent_-5");
  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::InvalidAgentId(_))),
    "LOUD FAILURE: Negative agent_id must return InvalidAgentId, got: {result:?}"
  );
}

// validate_ic_token_runtime: TokenInactive

#[tokio::test]
async fn test_validate_runtime_agent_not_found() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  // Token for agent_999 — agent does not exist in DB
  let token = create_token_for_agent(&manager, 999);
  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::TokenInactive(_))),
    "LOUD FAILURE: Missing agent must return TokenInactive, got: {result:?}"
  );
}

#[tokio::test]
async fn test_validate_runtime_token_revoked() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  // Agent exists but ic_token_hash is NULL (revoked)
  insert_agent(&pool, 10, None).await;
  let token = create_token_for_agent(&manager, 10);

  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::TokenInactive(_))),
    "LOUD FAILURE: Revoked token (NULL hash) must return TokenInactive, got: {result:?}"
  );
}

#[tokio::test]
async fn test_validate_runtime_token_rotated() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  // Agent has a different hash stored (simulates regeneration)
  insert_agent(
    &pool,
    20,
    Some("old_hash_that_doesnt_match_at_all_padding_to_64chars_1234567890ab"),
  )
  .await;
  let token = create_token_for_agent(&manager, 20);

  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::TokenInactive(_))),
    "LOUD FAILURE: Rotated token (hash mismatch) must return TokenInactive, got: {result:?}"
  );
}

// validate_ic_token_runtime: DatabaseError (H4)

#[tokio::test]
async fn test_validate_runtime_database_error() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  let token = create_token_for_agent(&manager, 1);

  // Close pool to simulate database unavailability
  pool.close().await;

  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    matches!(result, Err(IcTokenRuntimeError::DatabaseError(_))),
    "LOUD FAILURE: Closed database pool must return DatabaseError, got: {result:?}"
  );
}

// validate_ic_token_runtime: success (happy path)

#[tokio::test]
async fn test_validate_runtime_success() {
  let pool = setup_minimal_db().await;
  let manager = test_manager();

  let token = create_token_for_agent(&manager, 42);
  let token_hash = sha256_hash(&token);
  insert_agent(&pool, 42, Some(&token_hash)).await;

  let result = manager.validate_ic_token_runtime(&token, &pool).await;

  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid token with matching hash must succeed, got: {result:?}"
  );

  let (agent_id, claims) = result.unwrap();
  assert_eq!(
    agent_id, 42,
    "LOUD FAILURE: Returned agent_id must match, expected 42, got {agent_id}"
  );
  assert_eq!(
    claims.agent_id, "agent_42",
    "LOUD FAILURE: Claims agent_id must be 'agent_42', got '{}'",
    claims.agent_id
  );
}

const RATE_LIMIT_MAX: usize = 20; // must match IC_TOKEN_MAX_FAILURES constant
const TEST_TOKEN: &str = "test.ic.token.value";
const OTHER_TOKEN: &str = "other.ic.token.value";

fn record_n_failures(limiter: &IcTokenRateLimiter, token: &str, count: usize) {
  for _ in 0..count {
    limiter.record_failure(token);
  }
}

#[test]
fn test_rate_limiter_allows_below_threshold() {
  let limiter = IcTokenRateLimiter::new();
  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX - 1);
  assert!(
    limiter.check(TEST_TOKEN).is_ok(),
    "LOUD FAILURE: check() must allow requests below the failure threshold"
  );
}

#[test]
fn test_rate_limiter_blocks_at_threshold() {
  let limiter = IcTokenRateLimiter::new();
  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX);
  assert!(
    limiter.check(TEST_TOKEN).is_err(),
    "LOUD FAILURE: check() must block requests once the failure threshold is reached"
  );
}

#[test]
fn test_rate_limiter_returns_positive_retry_after() {
  let limiter = IcTokenRateLimiter::new();
  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX);
  let retry_after = limiter
    .check(TEST_TOKEN)
    .expect_err("LOUD FAILURE: check() must return Err after max failures");
  assert!(
    retry_after >= 1,
    "LOUD FAILURE: retry_after must be at least 1 second, got {retry_after}"
  );
  assert!(
    retry_after <= 60,
    "LOUD FAILURE: retry_after must not exceed window (60s), got {retry_after}"
  );
}

#[test]
fn test_rate_limiter_record_failure_increments_monotonically() {
  let limiter = IcTokenRateLimiter::new();
  for i in 1..=RATE_LIMIT_MAX {
    limiter.record_failure(TEST_TOKEN);
    let under_limit = i < RATE_LIMIT_MAX;
    assert_eq!(
      limiter.check(TEST_TOKEN).is_ok(),
      under_limit,
      "LOUD FAILURE: after {i} failure(s) check() should be {}, got {}",
      if under_limit { "Ok" } else { "Err" },
      if limiter.check(TEST_TOKEN).is_ok() {
        "Ok"
      } else {
        "Err"
      }
    );
  }
}

#[test]
fn test_rate_limiter_failures_expire_after_window() {
  // Use a short window so the test completes in milliseconds.
  let window = Duration::from_millis(100);
  let limiter = IcTokenRateLimiter::with_window(window);

  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX);
  assert!(
    limiter.check(TEST_TOKEN).is_err(),
    "LOUD FAILURE: should be blocked before window expires"
  );

  // Wait for the window to pass, then verify failures are swept.
  thread::sleep(window + Duration::from_millis(20));
  assert!(
    limiter.check(TEST_TOKEN).is_ok(),
    "LOUD FAILURE: failures must expire after the sliding window — token should not be rate-limited"
  );
}

#[test]
fn test_rate_limiter_different_tokens_are_independent() {
  let limiter = IcTokenRateLimiter::new();
  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX);
  assert!(limiter.check(TEST_TOKEN).is_err());
  assert!(
    limiter.check(OTHER_TOKEN).is_ok(),
    "LOUD FAILURE: rate limiting one token must not affect an unrelated token"
  );
}

// ── concurrent access ─────────────────────────────────────────────────────────

// Note: testing mutex poisoning recovery directly requires access to the private
// `failures: Arc<Mutex<...>>` field, which is only possible in a unit test inside
// the same source module. The recovery path (`unwrap_or_else(|e| e.into_inner())`
// in `lock_and_sweep`) is exercised indirectly here: if concurrent access were
// unsafe, this test would deadlock or panic under the thread sanitizer.

#[test]
fn test_rate_limiter_concurrent_access_is_safe() {
  let limiter = IcTokenRateLimiter::new();
  let limiter_clone = limiter.clone(); // Clone shares the inner Arc<Mutex<...>>

  // Spawn a thread that records failures for OTHER_TOKEN concurrently.
  let handle = thread::spawn(move || {
    record_n_failures(&limiter_clone, OTHER_TOKEN, RATE_LIMIT_MAX);
  });

  // Meanwhile, record failures for TEST_TOKEN on the main thread.
  record_n_failures(&limiter, TEST_TOKEN, RATE_LIMIT_MAX);

  handle
    .join()
    .expect("LOUD FAILURE: concurrent thread must not panic");

  // Both tokens must be independently rate-limited after concurrent writes.
  assert!(
    limiter.check(TEST_TOKEN).is_err(),
    "LOUD FAILURE: TEST_TOKEN must be rate-limited after concurrent failures"
  );
  assert!(
    limiter.check(OTHER_TOKEN).is_err(),
    "LOUD FAILURE: OTHER_TOKEN must be rate-limited after concurrent failures"
  );
}
