//! Per-IC-key and Per-IP-key spending limit integration tests
//!
//! Verifies that `reserve_budget_with_limits` enforces both agent (IC-key) and
//! provider key (IP-key) spending caps atomically, and that `restore_budget_with_limits`
//! correctly reverses spending on both sides.
//!
//! # Test Matrix
//!
//! | Test Case | Scenario | Expected |
//! |-----------|----------|----------|
//! | `test_ic_key_cap_blocks_handshake` | Agent cap exceeded | `BlockedBy::AgentSpendingCap` |
//! | `test_ip_key_cap_blocks_handshake` | Provider key cap exceeded | `BlockedBy::ProviderKeyCap` |
//! | `test_sequential_handshakes_exhaust_ic_cap` | Multiple reserves hit IC cap | Last blocked |
//! | `test_sequential_handshakes_exhaust_ip_cap` | Multiple reserves hit IP cap | Last blocked |
//! | `test_budget_return_reverses_both_caps` | Restore reverses IC + IP spending | Spending decremented |
//! | `test_null_cap_means_unlimited` | No cap set | Unlimited spending allowed |
//! | `test_lease_stores_provider_key_id` | Lease attribution | `provider_key_id` persisted |
//! | `test_multiple_agents_share_ip_key_cap` | Shared key across agents | Cap shared |
//! | `test_agent_cap_isolated_between_agents` | Separate agents | Caps independent |
//! | `test_concurrent_handshakes_respect_cap` | Race condition | Exactly N succeed |
//! | `test_refresh_denied_when_cap_exhausted` | Cap exhausted before refresh | Blocked |
//!
//! # Authority
//! - Plan: Unified Per-IC-Key and Per-IP-Key Spending Limits (Tasks 003 + 004)

mod common;

use std::sync::Arc;

use sqlx::SqlitePool;

use iron_token_manager::{
  agent_budget::AgentBudgetManager,
  lease_manager::LeaseManager,
  provider_key_storage::ProviderKeyStorage,
  BlockedBy, SpendingCap,
};

use common::budget::setup_test_db;

/// Seed a minimal agent with budget (no provider key assignment — tests manage keys directly)
async fn seed_agent_budget(pool: &SqlitePool, agent_id: i64, budget_microdollars: i64) {
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)",
  )
  .bind("test_user")
  .bind("test_username")
  .bind("$2b$12$test_password_hash")
  .bind("test@example.com")
  .bind("admin")
  .bind(1)
  .bind(now_ms)
  .execute(pool)
  .await
  .unwrap();

  sqlx::query(
    "INSERT OR IGNORE INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, '[]', ?, ?)",
  )
  .bind(agent_id)
  .bind(format!("test_agent_{agent_id}"))
  .bind(now_ms)
  .bind("test_user")
  .execute(pool)
  .await
  .unwrap();

  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)",
  )
  .bind(agent_id)
  .bind(budget_microdollars)
  .bind(budget_microdollars)
  .bind(now_ms)
  .bind(now_ms)
  .execute(pool)
  .await
  .unwrap();
}

/// Seed a provider key with optional spending cap
async fn seed_provider_key(
  pool: &SqlitePool,
  key_id: i64,
  cap_microdollars: Option<i64>,
) {
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT OR IGNORE INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id, spending_cap_microdollars)
     VALUES (?, 'openai', 'enc_key', 'nonce', 1, ?, 'test_user', ?)",
  )
  .bind(key_id)
  .bind(now_ms)
  .bind(cap_microdollars)
  .execute(pool)
  .await
  .unwrap();
}

// ─── Test 1: IC-key (agent) spending cap blocks reservation ───────────────────

#[tokio::test]
async fn test_ic_key_cap_blocks_handshake() {
  let pool = setup_test_db().await;
  let agent_id = 200i64;
  let key_id = 200_000i64;

  seed_agent_budget(&pool, agent_id, 50_000_000).await; // $50 budget
  seed_provider_key(&pool, key_id, None).await; // No IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Set IC-key cap to $5
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(5_000_000))
    .await
    .expect("LOUD FAILURE: Should set spending cap");

  // Try to reserve $10 — exceeds $5 cap
  let result = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");

  assert_eq!(
    result.blocked_by,
    Some(BlockedBy::AgentSpendingCap),
    "LOUD FAILURE: Should be blocked by agent spending cap"
  );
  assert_eq!(result.granted, 0);
}

// ─── Test 2: IP-key (provider key) spending cap blocks reservation ────────────

#[tokio::test]
async fn test_ip_key_cap_blocks_handshake() {
  let pool = setup_test_db().await;
  let agent_id = 201i64;
  let key_id = 201_000i64;

  seed_agent_budget(&pool, agent_id, 50_000_000).await;
  seed_provider_key(&pool, key_id, Some(3_000_000)).await; // $3 IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // No IC-key cap, but IP-key cap is $3 — try to reserve $10
  let result = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");

  assert_eq!(
    result.blocked_by,
    Some(BlockedBy::ProviderKeyCap),
    "LOUD FAILURE: Should be blocked by provider key spending cap"
  );
  assert_eq!(result.granted, 0);
}

// ─── Test 3: Sequential handshakes exhaust IC-key cap ─────────────────────────

#[tokio::test]
async fn test_sequential_handshakes_exhaust_ic_cap() {
  let pool = setup_test_db().await;
  let agent_id = 202i64;
  let key_id = 202_000i64;

  seed_agent_budget(&pool, agent_id, 100_000_000).await; // $100 budget
  seed_provider_key(&pool, key_id, None).await;

  let mgr = AgentBudgetManager::from_pool(pool.clone());
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(15_000_000)) // $15 cap
    .await
    .unwrap();

  // Reserve $10 — should succeed (used: $10, cap: $15)
  let r1 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .unwrap();
  assert!(
    r1.blocked_by.is_none(),
    "LOUD FAILURE: First reservation should succeed"
  );
  assert_eq!(r1.granted, 10_000_000);

  // Reserve $10 more — should be blocked (used: $10 + $10 = $20 > $15 cap)
  let r2 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .unwrap();
  assert_eq!(
    r2.blocked_by,
    Some(BlockedBy::AgentSpendingCap),
    "LOUD FAILURE: Second reservation should be blocked by IC-key cap"
  );
}

// ─── Test 4: Sequential handshakes exhaust IP-key cap ─────────────────────────

#[tokio::test]
async fn test_sequential_handshakes_exhaust_ip_cap() {
  let pool = setup_test_db().await;
  let agent_id = 203i64;
  let key_id = 203_000i64;

  seed_agent_budget(&pool, agent_id, 100_000_000).await;
  seed_provider_key(&pool, key_id, Some(15_000_000)).await; // $15 IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Reserve $10 — should succeed
  let r1 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .unwrap();
  assert!(r1.blocked_by.is_none());

  // Reserve $10 more — should be blocked by IP-key cap
  let r2 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .unwrap();
  assert_eq!(
    r2.blocked_by,
    Some(BlockedBy::ProviderKeyCap),
    "LOUD FAILURE: Second reservation should be blocked by IP-key cap"
  );
}

// ─── Test 5: Budget return reverses both IC-key and IP-key spending ───────────

#[tokio::test]
async fn test_budget_return_reverses_both_caps() {
  let pool = setup_test_db().await;
  let agent_id = 204i64;
  let key_id = 204_000i64;

  seed_agent_budget(&pool, agent_id, 50_000_000).await;
  seed_provider_key(&pool, key_id, Some(20_000_000)).await; // $20 IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());
  let pks = ProviderKeyStorage::new(pool.clone());

  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(20_000_000)) // $20 IC-key cap
    .await
    .unwrap();

  // Reserve $15
  let r = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 15_000_000)
    .await
    .unwrap();
  assert!(r.blocked_by.is_none());
  assert_eq!(r.granted, 15_000_000);

  // Check spending before return
  let agent_summary = mgr.get_spending_summary(agent_id).await.unwrap();
  assert_eq!(agent_summary.used_microdollars, 15_000_000);

  let key_summary = pks.get_spending_summary(key_id).await.unwrap();
  assert_eq!(key_summary.used_microdollars, 15_000_000);

  // Return $10 of the $15
  mgr
    .restore_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should restore budget");

  // Verify both decremented
  let agent_summary = mgr.get_spending_summary(agent_id).await.unwrap();
  assert_eq!(
    agent_summary.used_microdollars, 5_000_000,
    "LOUD FAILURE: Agent spending should be $5 after returning $10 of $15"
  );

  let key_summary = pks.get_spending_summary(key_id).await.unwrap();
  assert_eq!(
    key_summary.used_microdollars, 5_000_000,
    "LOUD FAILURE: Provider key spending should be $5 after returning $10 of $15"
  );

  // Now reserve $15 again — should succeed (used: $5 + $15 = $20 = cap)
  let r2 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 15_000_000)
    .await
    .unwrap();
  assert!(
    r2.blocked_by.is_none(),
    "LOUD FAILURE: Should succeed after return freed up spending room"
  );
}

// ─── Test 6: NULL cap means unlimited ─────────────────────────────────────────

#[tokio::test]
async fn test_null_cap_means_unlimited() {
  let pool = setup_test_db().await;
  let agent_id = 205i64;
  let key_id = 205_000i64;

  seed_agent_budget(&pool, agent_id, 1_000_000_000).await; // $1000 budget
  seed_provider_key(&pool, key_id, None).await; // No IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());
  // No IC-key cap set (default is Unlimited)

  // Reserve large amounts — should succeed
  for i in 0..5 {
    let r = mgr
      .reserve_budget_with_limits(agent_id, Some(key_id), 100_000_000) // $100 each
      .await
      .unwrap();
    assert!(
      r.blocked_by.is_none(),
      "LOUD FAILURE: Reservation {i} should succeed with unlimited caps"
    );
  }

  let summary = mgr.get_spending_summary(agent_id).await.unwrap();
  assert_eq!(
    summary.used_microdollars, 500_000_000,
    "LOUD FAILURE: All 5 × $100 reservations should be recorded in spending_used"
  );
}

// ─── Test 7: Lease stores provider_key_id ─────────────────────────────────────

#[tokio::test]
async fn test_lease_stores_provider_key_id() {
  let pool = setup_test_db().await;
  let agent_id = 206i64;
  let key_id = 206_000i64;

  seed_agent_budget(&pool, agent_id, 50_000_000).await;
  seed_provider_key(&pool, key_id, None).await;

  let lm = LeaseManager::from_pool(pool.clone());
  let lease_id = "lease_test_pkey_attr";

  lm.create_lease(lease_id, agent_id, agent_id, 10_000_000, None, Some(key_id))
    .await
    .expect("LOUD FAILURE: Should create lease");

  let lease = lm
    .get_lease(lease_id)
    .await
    .expect("LOUD FAILURE: Should fetch lease")
    .expect("LOUD FAILURE: Lease should exist");

  assert_eq!(
    lease.provider_key_id,
    Some(key_id),
    "LOUD FAILURE: Lease should store provider_key_id"
  );
}

// ─── Test 8: Multiple agents share IP-key cap ─────────────────────────────────

#[tokio::test]
async fn test_multiple_agents_share_ip_key_cap() {
  let pool = setup_test_db().await;
  let agent_a = 207i64;
  let agent_b = 208i64;
  let shared_key = 207_000i64;

  seed_agent_budget(&pool, agent_a, 100_000_000).await;
  seed_agent_budget(&pool, agent_b, 100_000_000).await;
  seed_provider_key(&pool, shared_key, Some(20_000_000)).await; // $20 shared IP cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Agent A reserves $12
  let r1 = mgr
    .reserve_budget_with_limits(agent_a, Some(shared_key), 12_000_000)
    .await
    .unwrap();
  assert!(r1.blocked_by.is_none());

  // Agent B tries $12 — should be blocked (shared key used: $12 + $12 = $24 > $20)
  let r2 = mgr
    .reserve_budget_with_limits(agent_b, Some(shared_key), 12_000_000)
    .await
    .unwrap();
  assert_eq!(
    r2.blocked_by,
    Some(BlockedBy::ProviderKeyCap),
    "LOUD FAILURE: Agent B should be blocked by shared IP-key cap"
  );

  // Agent B reserves $8 — should succeed (shared key used: $12 + $8 = $20 = cap)
  let r3 = mgr
    .reserve_budget_with_limits(agent_b, Some(shared_key), 8_000_000)
    .await
    .unwrap();
  assert!(
    r3.blocked_by.is_none(),
    "LOUD FAILURE: Agent B should succeed with $8 (exactly at cap)"
  );
}

// ─── Test 9: Agent cap isolated between agents ────────────────────────────────

#[tokio::test]
async fn test_agent_cap_isolated_between_agents() {
  let pool = setup_test_db().await;
  let agent_a = 209i64;
  let agent_b = 210i64;
  let key_a = 209_000i64;
  let key_b = 210_000i64;

  seed_agent_budget(&pool, agent_a, 100_000_000).await;
  seed_agent_budget(&pool, agent_b, 100_000_000).await;
  seed_provider_key(&pool, key_a, None).await;
  seed_provider_key(&pool, key_b, None).await;

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Set IC cap on agent A only
  mgr
    .set_spending_cap(agent_a, SpendingCap::Limited(5_000_000))
    .await
    .unwrap();

  // Agent A blocked by its $5 cap
  let r1 = mgr
    .reserve_budget_with_limits(agent_a, Some(key_a), 10_000_000)
    .await
    .unwrap();
  assert_eq!(r1.blocked_by, Some(BlockedBy::AgentSpendingCap));

  // Agent B unaffected — no cap set
  let r2 = mgr
    .reserve_budget_with_limits(agent_b, Some(key_b), 10_000_000)
    .await
    .unwrap();
  assert!(
    r2.blocked_by.is_none(),
    "LOUD FAILURE: Agent B should not be affected by Agent A's cap"
  );
}

// ─── Test 10: Concurrent handshakes respect cap ───────────────────────────────

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_handshakes_respect_cap() {
  let pool = setup_test_db().await;
  let agent_id = 211i64;
  let key_id = 211_000i64;

  seed_agent_budget(&pool, agent_id, 1_000_000_000).await; // $1000 budget (plenty)
  seed_provider_key(&pool, key_id, None).await;

  let mgr = Arc::new(AgentBudgetManager::from_pool(pool.clone()));

  // $50 IC-key cap, $10 per reservation → exactly 5 should succeed
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(50_000_000))
    .await
    .unwrap();

  let mut handles = Vec::new();
  for _ in 0..10 {
    let mgr = mgr.clone();
    handles.push(tokio::spawn(async move {
      mgr
        .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
        .await
        .unwrap()
    }));
  }

  let mut succeeded = 0;
  let mut blocked = 0;
  for handle in handles {
    let result = handle.await.unwrap();
    if result.blocked_by.is_none() {
      succeeded += 1;
    } else {
      blocked += 1;
    }
  }

  assert_eq!(
    succeeded, 5,
    "LOUD FAILURE: Exactly 5 reservations should succeed ($50 cap / $10 each). Got {succeeded} succeeded, {blocked} blocked"
  );
  assert_eq!(blocked, 5);

  // Verify total spending equals exactly cap (5 × $10 = $50)
  let summary = mgr.get_spending_summary(agent_id).await.unwrap();
  assert_eq!(
    summary.used_microdollars, 50_000_000,
    "LOUD FAILURE: Total spending should equal exactly the cap"
  );
}

// ─── Test 11: Insufficient budget blocks before caps checked ──────────────────

#[tokio::test]
async fn test_insufficient_budget_blocks() {
  let pool = setup_test_db().await;
  let agent_id = 212i64;
  let key_id = 212_000i64;

  seed_agent_budget(&pool, agent_id, 5_000_000).await; // Only $5 budget
  seed_provider_key(&pool, key_id, None).await;

  let mgr = AgentBudgetManager::from_pool(pool.clone());
  // No caps — pure budget exhaustion

  let r = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000) // Request $10
    .await
    .unwrap();

  // Should get partial grant of $5 (the remaining budget)
  assert!(
    r.blocked_by.is_none(),
    "LOUD FAILURE: Should get partial grant, not be blocked"
  );
  assert_eq!(
    r.granted, 5_000_000,
    "LOUD FAILURE: Should grant remaining $5 as partial"
  );

  // Second reservation should be blocked — budget exhausted
  let r2 = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 1_000_000)
    .await
    .unwrap();
  assert_eq!(
    r2.blocked_by,
    Some(BlockedBy::InsufficientBudget),
    "LOUD FAILURE: Should be blocked by insufficient budget"
  );
}

// ─── Test 12: Refresh denied when cap exhausted ───────────────────────────────

#[tokio::test]
async fn test_refresh_denied_when_cap_exhausted() {
  let pool = setup_test_db().await;
  let agent_id = 213i64;
  let key_id = 213_000i64;

  seed_agent_budget(&pool, agent_id, 100_000_000).await; // $100 budget (plenty)
  seed_provider_key(&pool, key_id, None).await;

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Set IC-key cap to $10
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(10_000_000))
    .await
    .expect("LOUD FAILURE: Should set spending cap");

  // Exhaust the cap — reserve $10 (fills it exactly)
  let initial = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");
  assert!(
    initial.blocked_by.is_none(),
    "LOUD FAILURE: Initial reservation should succeed"
  );
  assert_eq!(initial.granted, 10_000_000);

  // Simulate a refresh: attempt another reservation — cap is now exhausted
  let refresh_attempt = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");

  assert_eq!(
    refresh_attempt.blocked_by,
    Some(BlockedBy::AgentSpendingCap),
    "LOUD FAILURE: Refresh should be blocked by exhausted agent spending cap"
  );
  assert_eq!(
    refresh_attempt.granted, 0,
    "LOUD FAILURE: No budget should be granted when cap is exhausted"
  );

  // Verify spending summary reflects the cap is fully used
  let summary = mgr.get_spending_summary(agent_id).await.unwrap();
  assert_eq!(
    summary.used_microdollars, 10_000_000,
    "LOUD FAILURE: Spending used should equal the cap after exhaustion"
  );
}

// ─── Test 13: Both caps exceeded — IP-key cap checked first ──────────────────

#[tokio::test]
async fn test_both_caps_exceeded_provider_key_wins() {
  let pool = setup_test_db().await;
  let agent_id = 214i64;
  let key_id = 214_000i64;

  seed_agent_budget(&pool, agent_id, 100_000_000).await; // $100 budget (plenty)
  seed_provider_key(&pool, key_id, Some(3_000_000)).await; // $3 IP-key cap

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Set IC-key cap to $5 (also exceeded by $10 request)
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(5_000_000))
    .await
    .expect("LOUD FAILURE: Should set spending cap");

  // Try to reserve $10 — both caps would be exceeded
  let result = mgr
    .reserve_budget_with_limits(agent_id, Some(key_id), 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");

  // IP-key cap is checked first via explicit early-return before the combined UPDATE,
  // so ProviderKeyCap wins even when both are exceeded.
  assert_eq!(
    result.blocked_by,
    Some(BlockedBy::ProviderKeyCap),
    "LOUD FAILURE: IP-key cap is checked first and should be reported as the block reason"
  );
  assert_eq!(result.granted, 0);
}

// ─── Test 14: No provider key still enforces agent cap ────────────────────────

#[tokio::test]
async fn test_no_provider_key_still_enforces_agent_cap() {
  let pool = setup_test_db().await;
  let agent_id = 215i64;

  seed_agent_budget(&pool, agent_id, 100_000_000).await; // $100 budget (plenty)

  let mgr = AgentBudgetManager::from_pool(pool.clone());

  // Set IC-key cap to $5
  mgr
    .set_spending_cap(agent_id, SpendingCap::Limited(5_000_000))
    .await
    .expect("LOUD FAILURE: Should set spending cap");

  // Reserve $10 with no provider key — should be blocked by agent cap
  let result = mgr
    .reserve_budget_with_limits(agent_id, None, 10_000_000)
    .await
    .expect("LOUD FAILURE: Should return result, not DB error");

  assert_eq!(
    result.blocked_by,
    Some(BlockedBy::AgentSpendingCap),
    "LOUD FAILURE: Agent cap should be enforced even when no provider key is provided"
  );
  assert_eq!(result.granted, 0);
}
