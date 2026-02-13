//! Unit tests for `ic_token` module: `sha256_hash` and `validate_ic_token_runtime`
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
//! # Authority
//! - PR #44 review findings: H4, M2, M6

mod common;

use iron_control_api::ic_token::{sha256_hash, IcTokenClaims, IcTokenManager, IcTokenRuntimeError};
use sqlx::SqlitePool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

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
