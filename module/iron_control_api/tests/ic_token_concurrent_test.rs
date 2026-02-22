//! Concurrent IC Token validation and regeneration tests
//!
//! Verifies that concurrent operations on IC tokens do not cause deadlocks,
//! data corruption, or inconsistent state.
//!
//! # Test Matrix
//!
//! | Test | Scenario | Expected |
//! |------|----------|----------|
//! | `test_concurrent_validation_during_regenerate` | Validate while regenerate runs | No deadlock; old token rejected after regenerate |
//! | `test_concurrent_double_regenerate` | Two regenerates in parallel | No deadlock; exactly one token wins |
//!
//! # Authority
//! - PR #44 review finding H5: Missing concurrent validation tests

mod common;

use iron_control_api::ic_token::{sha256_hash, IcTokenClaims, IcTokenManager};
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::time::{timeout, Duration};

async fn setup_db() -> SqlitePool {
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
        ic_token_hash TEXT,
        ic_token_created_at INTEGER
      )
    ",
  )
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Should create agents table");
  pool
}

async fn insert_agent(pool: &SqlitePool, agent_id: i64, token_hash: &str) {
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, ic_token_hash) VALUES (?, ?, '[]', 0, ?)",
  )
  .bind(agent_id)
  .bind(format!("agent_{agent_id}"))
  .bind(token_hash)
  .execute(pool)
  .await
  .expect("LOUD FAILURE: Should insert agent");
}

fn manager() -> IcTokenManager {
  IcTokenManager::new("concurrent_test_secret".to_string())
}

fn generate_token(mgr: &IcTokenManager, agent_id: i64) -> String {
  let claims = IcTokenClaims::new(
    format!("agent_{agent_id}"),
    format!("budget_{agent_id}"),
    vec!["llm:call".to_string()],
    None,
  );
  mgr
    .generate_token(&claims)
    .expect("LOUD FAILURE: Should generate token")
}

/// Simulate regenerate: generate new token, update hash inside IMMEDIATE transaction
async fn regenerate(pool: &SqlitePool, mgr: &IcTokenManager, agent_id: i64) -> String {
  let new_token = generate_token(mgr, agent_id);
  let new_hash = sha256_hash(&new_token);
  let created_at = chrono::Utc::now().timestamp();

  let mut tx = pool.begin().await.expect("LOUD FAILURE: Should begin tx");
  sqlx::query("UPDATE agents SET ic_token_hash = ?, ic_token_created_at = ? WHERE id = ?")
    .bind(&new_hash)
    .bind(created_at)
    .bind(agent_id)
    .execute(tx.as_mut())
    .await
    .expect("LOUD FAILURE: Should update hash");
  tx.commit().await.expect("LOUD FAILURE: Should commit tx");

  new_token
}

/// Validate token concurrently with a regenerate operation.
///
/// Ensures:
/// 1. No deadlocks (all tasks complete within timeout)
/// 2. After regenerate completes, old token is always rejected
/// 3. New token is always accepted
#[tokio::test]
async fn test_concurrent_validation_during_regenerate() {
  let pool = setup_db().await;
  let mgr = Arc::new(manager());
  let agent_id: i64 = 1;

  // Create agent with `Token A`
  let token_a = generate_token(&mgr, agent_id);
  let hash_a = sha256_hash(&token_a);
  insert_agent(&pool, agent_id, &hash_a).await;

  // Verify `Token A` works before concurrency test
  let (id, _) = mgr
    .validate_ic_token_runtime(&token_a, &pool)
    .await
    .expect("LOUD FAILURE: Token A should validate before regenerate");
  assert_eq!(id, agent_id);

  // Launch concurrent operations
  let pool_val = pool.clone();
  let pool_regen = pool.clone();
  let mgr_val = mgr.clone();
  let mgr_regen = mgr.clone();
  let token_a_clone = token_a.clone();

  // Task 1: Validate `Token A` 10 times in quick succession
  let validation_handle = tokio::spawn(async move {
    let mut results = Vec::new();
    for _ in 0..10 {
      let r = mgr_val
        .validate_ic_token_runtime(&token_a_clone, &pool_val)
        .await;
      results.push(r.is_ok());
    }
    results
  });

  // Task 2: Regenerate (replace `Token A` with `Token B`)
  let regenerate_handle =
    tokio::spawn(async move { regenerate(&pool_regen, &mgr_regen, agent_id).await });

  // Both must complete within 5 seconds (no deadlock)
  let (val_results, token_b) = timeout(Duration::from_secs(5), async {
    let val = validation_handle
      .await
      .expect("LOUD FAILURE: Validation task panicked");
    let tok = regenerate_handle
      .await
      .expect("LOUD FAILURE: Regenerate task panicked");
    (val, tok)
  })
  .await
  .expect("LOUD FAILURE: Concurrent operations deadlocked (5s timeout)");

  // All validations should have completed (10 results)
  assert_eq!(val_results.len(), 10, "All 10 validations should complete");

  // After regenerate: `Token A` must be rejected, `Token B` must be accepted
  let old_token_result = mgr.validate_ic_token_runtime(&token_a, &pool).await;
  assert!(
    old_token_result.is_err(),
    "Token A must be rejected after regenerate"
  );

  let new_token_result = mgr.validate_ic_token_runtime(&token_b, &pool).await;
  assert!(
    new_token_result.is_ok(),
    "Token B must be accepted after regenerate"
  );
}

/// Two regenerate operations running concurrently.
///
/// Ensures:
/// 1. No deadlocks (both complete within timeout)
/// 2. No panics from transaction conflicts
/// 3. Exactly one of the two generated tokens is active (the one whose hash was written last)
/// 4. The other token is rejected (hash mismatch)
#[tokio::test]
async fn test_concurrent_double_regenerate() {
  let pool = setup_db().await;
  let mgr = Arc::new(manager());
  let agent_id: i64 = 2;

  // Create agent with initial token
  let initial_token = generate_token(&mgr, agent_id);
  let initial_hash = sha256_hash(&initial_token);
  insert_agent(&pool, agent_id, &initial_hash).await;

  // Launch two concurrent regenerates
  let pool1 = pool.clone();
  let pool2 = pool.clone();
  let mgr1 = mgr.clone();
  let mgr2 = mgr.clone();

  let handle1 = tokio::spawn(async move { regenerate(&pool1, &mgr1, agent_id).await });

  let handle2 = tokio::spawn(async move { regenerate(&pool2, &mgr2, agent_id).await });

  // Both must complete within 5 seconds (no deadlock from IMMEDIATE transactions)
  let (token_x, token_y) = timeout(Duration::from_secs(5), async {
    let t1 = handle1.await.expect("LOUD FAILURE: Regenerate 1 panicked");
    let t2 = handle2.await.expect("LOUD FAILURE: Regenerate 2 panicked");
    (t1, t2)
  })
  .await
  .expect("LOUD FAILURE: Double regenerate deadlocked (5s timeout)");

  // Initial token must be rejected
  let initial_result = mgr.validate_ic_token_runtime(&initial_token, &pool).await;
  assert!(
    initial_result.is_err(),
    "Initial token must be rejected after double regenerate"
  );

  // Exactly one of the two new tokens should be active
  let x_valid = mgr.validate_ic_token_runtime(&token_x, &pool).await.is_ok();
  let y_valid = mgr.validate_ic_token_runtime(&token_y, &pool).await.is_ok();

  assert!(
    x_valid ^ y_valid,
    "Exactly one token should be active: x_valid={x_valid}, y_valid={y_valid}"
  );
}
