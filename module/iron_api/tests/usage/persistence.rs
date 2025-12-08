//! Usage data persistence tests for FR-8.
//!
//! ## Purpose
//! Verify that usage data persists across UsageTracker restarts when using
//! file-based SQLite databases (not in-memory).
//!
//! ## Why This Test Exists
//! During manual testing (2025-12-07), usage aggregation returned zeros after
//! server restart. Investigation revealed this was a test artifact (manual
//! sqlite3 insertion without proper commit), NOT a bug. This test provides
//! permanent regression coverage to ensure usage data persists correctly in
//! production scenarios.
//!
//! ## Root Cause (Issue-003: Migration CASCADE DELETE Data Loss)
//!
//! Migration 002 (`002_add_length_constraints.sql`) was NOT idempotent. It executed
//! `DROP TABLE IF EXISTS api_tokens;` on EVERY run, which triggered CASCADE DELETE
//! on all `token_usage` records due to the foreign key constraint:
//!
//! ```sql
//! CREATE TABLE token_usage (
//!   token_id INTEGER NOT NULL,
//!   FOREIGN KEY(token_id) REFERENCES api_tokens(id) ON DELETE CASCADE
//! );
//! ```
//!
//! **Execution flow causing data loss:**
//! 1. Cycle 1: `TokenState::new()` runs migrations, creates token, records 150 tokens
//! 2. Cycle 2: `TokenState::new()` runs migrations AGAIN:
//!    - Migration 002 drops `api_tokens` table
//!    - CASCADE DELETE removes all `token_usage` records (150 tokens lost!)
//!    - Creates new empty `api_tokens` table
//!    - Creates new token, records 300 tokens
//!    - Aggregation shows only 300 tokens instead of 450 (150 + 300)
//!
//! **Technical detail:** Both `TokenStorage::new()` and `UsageTracker::new()` run
//! migrations on initialization. In production, each server restart creates new
//! instances, running migrations again.
//!
//! ## Why Not Caught
//!
//! 1. **No multi-restart test coverage:** Existing tests only created single instances
//!    or didn't verify cumulative data persistence across multiple initialization cycles
//! 2. **Migration ran silently:** `DROP TABLE IF EXISTS` succeeded without error,
//!    masking the destructive operation
//! 3. **First-run success:** Migration worked correctly on first run (empty database),
//!    only failed on subsequent runs (existing data)
//! 4. **Foreign key cascade hidden:** The connection between `api_tokens` drop and
//!    `token_usage` deletion wasn't obvious without schema analysis
//! 5. **Test isolation:** Unit tests used fresh databases per test, never triggering
//!    the multi-initialization scenario
//!
//! ## Fix Applied
//!
//! Made migration 002 idempotent using guard table pattern:
//!
//! **SQL changes (`002_add_length_constraints.sql`):**
//! - Added guard table `_migration_002_completed` created at end of migration
//! - Migration now creates this table to mark successful completion
//!
//! **Rust changes (`storage.rs`):**
//! ```rust
//! // Check if migration already applied
//! let migration_002_completed: i64 = sqlx::query_scalar(
//!   "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='_migration_002_completed'"
//! )
//! .fetch_one(&pool)
//! .await
//! .map_err(|_| crate::error::TokenError)?;
//!
//! // Only run migration if guard table doesn't exist
//! if migration_002_completed == 0 {
//!   let migration_002 = include_str!("../migrations/002_add_length_constraints.sql");
//!   sqlx::raw_sql(migration_002).execute(&pool).await.map_err(|_| crate::error::TokenError)?;
//! }
//! ```
//!
//! This ensures migration 002 runs exactly once per database, not once per initialization.
//!
//! ## Prevention
//!
//! 1. **All migrations must be idempotent:** Check if already applied before executing
//! 2. **Use guard tables:** Create `_migration_NNN_completed` marker tables
//! 3. **Test multi-restart scenarios:** Always test data persistence across multiple
//!    initialization cycles (like `test_usage_persists_across_multiple_restarts`)
//! 4. **Avoid DROP TABLE in migrations:** Prefer `CREATE TABLE IF NOT EXISTS` or
//!    check for existence before dropping
//! 5. **Document CASCADE constraints:** Explicitly note which deletes trigger cascades
//! 6. **Migration framework consideration:** For production, use proper migration
//!    tracking (like SQLx migrations or Diesel) instead of raw SQL
//!
//! ## Pitfall
//!
//! **Idempotency is NOT automatic for SQL migrations.** The pattern `CREATE TABLE IF NOT EXISTS`
//! is idempotent, but `DROP TABLE IF EXISTS` is NOT - it succeeds every time, destroying
//! data. When migrations modify existing tables (ALTER TABLE equivalent via drop/copy/rename),
//! you MUST add explicit guards to prevent re-execution. Always ask: "What happens if this
//! migration runs twice?" If the answer is data loss, add a guard table.
//!
//! ## Test Strategy
//! 1. Create UsageTracker with file-based database
//! 2. Record usage via proper API (`UsageTracker.record_usage_with_cost()`)
//! 3. Drop tracker (simulates server stop)
//! 4. Create new tracker with same database (simulates server restart)
//! 5. Verify all usage data persisted
//!
//! ## Coverage
//! - File-based SQLite persistence (production scenario)
//! - Cross-token aggregation persistence
//! - Cross-provider breakdown persistence
//! - Cost tracking persistence
//! - Request counting persistence
//!
//! ## Known Edge Cases
//! - In-memory databases (`sqlite::memory:`) do NOT persist (expected)
//! - Manual sqlite3 inserts may not commit properly (test artifact, not production issue)
//! - Production code uses `UsageTracker.record_usage()` which commits automatically

use iron_api::routes::usage::UsageState;
use iron_api::routes::tokens::TokenState;
use iron_token_manager::token_generator::TokenGenerator;

/// Test usage data persists across UsageState restart (simulates server restart).
///
/// WHY: Critical for production - usage data must persist for billing/analytics.
/// If this fails: CRITICAL BUG - usage data loss on server restart.
#[ tokio::test ]
async fn test_usage_persists_across_restart()
{
  // Create temporary file database
  let temp_db = format!( "/tmp/iron_test_usage_persistence_{}.db", std::process::id() );
  let db_url = format!( "sqlite://{}?mode=rwc", temp_db );

  // Clean up any existing test database
  let _ = std::fs::remove_file( &temp_db );

  let expected_tokens: i64;
  let expected_requests: i64;
  let expected_cost: i64;

  // Phase 1: Create usage data (simulates normal server operation)
  {
    let usage_state = UsageState::new( &db_url )
      .await
      .expect( "LOUD FAILURE: Failed to create usage state" );

    let token_state = TokenState::new( &db_url )
      .await
      .expect( "LOUD FAILURE: Failed to create token state" );

    // Create token
    let generator = TokenGenerator::new();
    let token = generator.generate();

    let token_id = token_state.storage.create_token(
      &token,
      "persistence-test-user",
      Some( "persistence-proj" ),
      Some( "Persistence test token" )
    )
    .await
    .expect( "LOUD FAILURE: Failed to create token" );

    // Record usage 1: OpenAI
    usage_state.tracker.record_usage_with_cost(
      token_id,
      "openai",
      "gpt-4",
      500,   // input_tokens
      300,   // output_tokens
      800,   // total_tokens
      120,   // cost_cents
    )
    .await
    .expect( "LOUD FAILURE: Failed to record usage 1" );

    // Record usage 2: Anthropic (different provider)
    usage_state.tracker.record_usage_with_cost(
      token_id,
      "anthropic",
      "claude-sonnet-4-5",
      1000,  // input_tokens
      500,   // output_tokens
      1500,  // total_tokens
      200,   // cost_cents
    )
    .await
    .expect( "LOUD FAILURE: Failed to record usage 2" );

    // Verify data recorded before restart
    let aggregate = usage_state.tracker.get_all_aggregate_usage()
      .await
      .expect( "LOUD FAILURE: Failed to get aggregate usage before restart" );

    assert_eq!(
      aggregate.total_tokens, 2300,
      "LOUD FAILURE: Before restart should have 800 + 1500 = 2300 tokens"
    );
    assert_eq!(
      aggregate.total_requests, 2,
      "LOUD FAILURE: Before restart should have 2 requests"
    );
    assert_eq!(
      aggregate.total_cost_cents, 320,
      "LOUD FAILURE: Before restart should have 120 + 200 = 320 cost_cents"
    );

    expected_tokens = aggregate.total_tokens;
    expected_requests = aggregate.total_requests;
    expected_cost = aggregate.total_cost_cents;
  }
  // usage_state and token_state dropped here (simulates server shutdown)

  // Phase 2: Restart (create new UsageState, simulates server restart)
  {
    let usage_state = UsageState::new( &db_url )
      .await
      .expect( "LOUD FAILURE: Failed to create usage state after restart" );

    // Verify all usage data persisted
    let aggregate = usage_state.tracker.get_all_aggregate_usage()
      .await
      .expect( "LOUD FAILURE: Failed to get aggregate usage after restart" );

    assert_eq!(
      aggregate.total_tokens, expected_tokens,
      "LOUD FAILURE: Usage tokens should persist across restart"
    );
    assert_eq!(
      aggregate.total_requests, expected_requests,
      "LOUD FAILURE: Request count should persist across restart"
    );
    assert_eq!(
      aggregate.total_cost_cents, expected_cost,
      "LOUD FAILURE: Cost should persist across restart"
    );

    // Verify provider breakdown persisted
    let provider_breakdown = usage_state.tracker.get_usage_by_provider_all()
      .await
      .expect( "LOUD FAILURE: Failed to get provider breakdown after restart" );

    assert_eq!(
      provider_breakdown.len(), 2,
      "LOUD FAILURE: Should have 2 providers (openai, anthropic) after restart"
    );

    // Find OpenAI provider
    let openai = provider_breakdown.iter()
      .find( |( provider, _ )| provider == "openai" )
      .expect( "LOUD FAILURE: OpenAI provider should exist after restart" );
    assert_eq!(
      openai.1.total_tokens, 800,
      "LOUD FAILURE: OpenAI tokens should persist"
    );

    // Find Anthropic provider
    let anthropic = provider_breakdown.iter()
      .find( |( provider, _ )| provider == "anthropic" )
      .expect( "LOUD FAILURE: Anthropic provider should exist after restart" );
    assert_eq!(
      anthropic.1.total_tokens, 1500,
      "LOUD FAILURE: Anthropic tokens should persist"
    );
  }

  // Clean up test database
  let _ = std::fs::remove_file( &temp_db );
}

/// Test multiple restart cycles preserve data integrity.
///
/// WHY: Verify data persists across multiple server restarts, not just one.
#[ tokio::test ]
async fn test_usage_persists_across_multiple_restarts()
{
  let temp_db = format!( "/tmp/iron_test_multi_restart_{}.db", std::process::id() );
  let db_url = format!( "sqlite://{}?mode=rwc", temp_db );

  let _ = std::fs::remove_file( &temp_db );

  // Restart cycle 1: Create initial data
  {
    let usage_state = UsageState::new( &db_url ).await.expect( "Failed cycle 1 create" );
    let token_state = TokenState::new( &db_url ).await.expect( "Failed token create" );

    let generator = TokenGenerator::new();
    let token = generator.generate();
    let token_id = token_state.storage.create_token(
      &token,
      "multi-restart-user",
      None,
      None
    )
    .await
    .expect( "Failed to create token" );

    usage_state.tracker.record_usage_with_cost(
      token_id,
      "openai",
      "gpt-4",
      100, 50, 150, 20
    )
    .await
    .expect( "Failed record 1" );
  }

  // Restart cycle 2: Add more data
  {
    let usage_state = UsageState::new( &db_url ).await.expect( "Failed cycle 2 create" );
    let token_state = TokenState::new( &db_url ).await.expect( "Failed token state 2" );

    let generator = TokenGenerator::new();
    let token = generator.generate();
    let token_id = token_state.storage.create_token(
      &token,
      "multi-restart-user-2",
      None,
      None
    )
    .await
    .expect( "Failed to create token 2" );

    usage_state.tracker.record_usage_with_cost(
      token_id,
      "anthropic",
      "claude",
      200, 100, 300, 40
    )
    .await
    .expect( "Failed record 2" );

    // Verify cumulative data
    let aggregate = usage_state.tracker.get_all_aggregate_usage().await.unwrap();
    assert_eq!( aggregate.total_tokens, 450, "Should have 150 + 300 after cycle 2" );
  }

  // Restart cycle 3: Verify all data still persists
  {
    let usage_state = UsageState::new( &db_url ).await.expect( "Failed cycle 3 create" );

    let aggregate = usage_state.tracker.get_all_aggregate_usage().await.unwrap();
    assert_eq!(
      aggregate.total_tokens, 450,
      "LOUD FAILURE: All data should persist across 3 restart cycles"
    );
    assert_eq!(
      aggregate.total_requests, 2,
      "LOUD FAILURE: Both requests should persist"
    );
    assert_eq!(
      aggregate.total_cost_cents, 60,
      "LOUD FAILURE: Total cost should persist"
    );
  }

  let _ = std::fs::remove_file( &temp_db );
}
