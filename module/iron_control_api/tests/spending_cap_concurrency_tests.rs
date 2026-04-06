//! Spending Cap Concurrency Tests
//!
//! Replacement coverage for deleted `budget_concurrency_diagnosis.rs`.
//! Validates that the atomic CASE WHEN UPDATE in `reserve_spending` is correct
//! under concurrent access -- no overspending is possible.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Expected |
//! |-----------|----------|----------|
//! | `test_concurrent_reserve_spending_respects_cap` | 10 tasks each reserve 1 USD against 5 USD cap | Exactly 5 succeed, 5 fail |
//! | `test_concurrent_reserve_spending_no_cap` | 10 tasks reserve without cap | All 10 succeed |

mod common;

use std::sync::Arc;

use iron_token_manager::provider_key_storage::ProviderKeyStorage;
use iron_token_manager::ProviderType;

/// Create a shared `ProviderKeyStorage` backed by a file-based `SQLite` database.
///
/// File-based is required for concurrency tests because in-memory databases
/// are not shared across connections in the default `SQLite` pool.
async fn create_shared_provider_storage() -> (Arc<ProviderKeyStorage>, tempfile::TempDir) {
  let tmp_dir = tempfile::tempdir().expect("LOUD FAILURE: Failed to create temp directory");
  let db_path = tmp_dir.path().join("test_spending.db");
  let db_url = format!("sqlite:{}?mode=rwc", db_path.display());

  let pool = sqlx::sqlite::SqlitePoolOptions::new()
    .max_connections(10)
    .connect(&db_url)
    .await
    .expect("LOUD FAILURE: Failed to create SQLite pool for concurrency test");

  // Enable WAL mode for better concurrent write performance
  sqlx::query("PRAGMA journal_mode=WAL")
    .execute(&pool)
    .await
    .expect("LOUD FAILURE: Failed to set WAL mode");

  iron_token_manager::migrations::apply_all_migrations(&pool)
    .await
    .expect("LOUD FAILURE: Failed to apply migrations");

  // Seed a test user (FK constraint on ai_provider_keys.user_id)
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)",
  )
  .bind("user_concurrency")
  .bind("concurrency_user")
  .bind("test_hash")
  .bind("concurrency@example.com")
  .bind("manager")
  .bind(1)
  .bind(now_ms)
  .execute(&pool)
  .await
  .expect("LOUD FAILURE: Failed to seed test user");

  let storage = Arc::new(ProviderKeyStorage::new(pool));
  (storage, tmp_dir)
}

/// Spawn N concurrent `reserve_spending` tasks and return (`success_count`, `failure_count`).
async fn run_concurrent_reserves(
  storage: &Arc<ProviderKeyStorage>,
  key_id: i64,
  per_request_amount: i64,
  num_tasks: usize,
) -> (usize, usize) {
  let mut handles = Vec::with_capacity(num_tasks);

  for _ in 0..num_tasks {
    let s = Arc::clone(storage);
    let handle = tokio::spawn(async move { s.reserve_spending(key_id, per_request_amount).await });
    handles.push(handle);
  }

  let mut successes = 0usize;
  let mut failures = 0usize;

  for handle in handles {
    match handle.await.expect("LOUD FAILURE: Task panicked") {
      Ok(()) => successes += 1,
      Err(_) => failures += 1,
    }
  }

  (successes, failures)
}

/// 10 concurrent tasks each try to reserve `1_000_000` microdollars (1 USD)
/// against a key with a `5_000_000` microdollar (5 USD) spending cap.
///
/// Exactly 5 must succeed (`floor(5_000_000 / 1_000_000) = 5`) and 5 must fail
/// with `SpendingCapExceeded`. The final used amount must equal exactly the cap.
#[tokio::test]
async fn test_concurrent_reserve_spending_respects_cap() {
  let (storage, _tmp_dir) = create_shared_provider_storage().await;

  let key_id = storage
    .create_key(
      ProviderType::OpenAI,
      "encrypted_key",
      "nonce",
      None,
      Some("concurrency test key"),
      "user_concurrency",
    )
    .await
    .expect("LOUD FAILURE: Failed to create provider key");

  // Set spending cap to 5 USD (5_000_000 microdollars)
  let cap_microdollars = 5_000_000i64;
  storage
    .set_spending_cap(key_id, Some(cap_microdollars))
    .await
    .expect("LOUD FAILURE: Failed to set spending cap");

  let per_request = 1_000_000i64; // 1 USD
  let num_tasks = 10usize;
  let expected_successes = usize::try_from(cap_microdollars / per_request).expect("cap/request fits usize");

  let (successes, failures) = run_concurrent_reserves(&storage, key_id, per_request, num_tasks).await;

  assert_eq!(
    successes, expected_successes,
    "LOUD FAILURE: Exactly {expected_successes} reserves should succeed under cap {cap_microdollars} \
     with per-request {per_request}. Got {successes} successes."
  );
  assert_eq!(
    failures,
    num_tasks - expected_successes,
    "LOUD FAILURE: Remaining tasks should fail with SpendingCapExceeded"
  );

  // Verify final spending equals exactly the cap (no overspend, no underspend)
  let summary = storage
    .get_spending_summary(key_id)
    .await
    .expect("LOUD FAILURE: Failed to get spending summary");

  assert_eq!(
    summary.used_microdollars, cap_microdollars,
    "LOUD FAILURE: Final spending must equal exactly the cap (no overspending). \
     Got {}, expected {}",
    summary.used_microdollars, cap_microdollars
  );
}

/// When no spending cap is set, all concurrent reservations must succeed.
#[tokio::test]
async fn test_concurrent_reserve_spending_no_cap() {
  let (storage, _tmp_dir) = create_shared_provider_storage().await;

  let key_id = storage
    .create_key(
      ProviderType::Anthropic,
      "encrypted_key",
      "nonce",
      None,
      Some("no-cap concurrency test"),
      "user_concurrency",
    )
    .await
    .expect("LOUD FAILURE: Failed to create provider key");

  // No cap set -- all should succeed
  let per_request = 1_000_000i64;
  let num_tasks = 10usize;

  let (successes, failures) = run_concurrent_reserves(&storage, key_id, per_request, num_tasks).await;

  assert_eq!(
    successes, num_tasks,
    "LOUD FAILURE: All reserves should succeed when no spending cap is set"
  );
  assert_eq!(failures, 0, "LOUD FAILURE: No failures expected without cap");

  let summary = storage
    .get_spending_summary(key_id)
    .await
    .expect("LOUD FAILURE: Failed to get spending summary");

  let expected_total = per_request * i64::try_from(num_tasks).expect("num_tasks fits i64");
  assert_eq!(
    summary.used_microdollars, expected_total,
    "LOUD FAILURE: Total spending must equal sum of all reservations"
  );
}
