//! Protocol 005 concurrency corner case tests
//!
//! Tests concurrent access scenarios for Protocol 005 (Budget Control Protocol):
//! - Multiple simultaneous handshakes (race condition prevention)
//! - Concurrent usage reports on same lease (database consistency)
//! - Concurrent report and refresh operations (consistency under contention)
//! - Multiple active leases per agent (design validation)
//!
//! # Corner Case Coverage
//!
//! Tests address the following critical gaps from gap analysis:
//! 8. Multiple simultaneous handshakes for same agent (CRITICAL - race prevention)
//! 9. Concurrent usage reports on same lease (HIGH - database consistency)
//! 10. Concurrent report and refresh on same lease (MEDIUM - complex scenario)
//! 11. Handshake while active lease exists (LOW - design clarification)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_multiple_simultaneous_handshakes` | Race condition prevention for concurrent handshakes | 10 concurrent POST /api/budget/handshake requests, 100 USD budget | All 10 succeed (10 USD each), budget fully allocated | ✅ |
//! | `test_concurrent_usage_reports_on_same_lease` | Concurrent usage reports on same lease | 2 concurrent POST /api/budget/report requests on same lease_id | Both succeed with atomic updates, no lost data | ✅ |
//! | `test_concurrent_report_and_refresh` | Concurrent report and refresh operations | Concurrent POST /api/budget/report and POST /api/budget/refresh on same lease | Both succeed, 2 leases total, no lost updates | ✅ |
//! | `test_handshake_with_existing_active_lease` | Multiple active leases per agent | POST /api/budget/handshake while active lease exists | New lease created, both leases active simultaneously | ✅ |

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use iron_control_api::
{
  ic_token::{ IcTokenClaims, IcTokenManager },
  routes::budget::{ BudgetState, handshake, report_usage, refresh_budget },
};
use iron_token_manager::lease_manager::LeaseManager;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper: Create test database with all migrations
async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect("LOUD FAILURE: Failed to apply migrations");
  pool
}

/// Helper: Create test BudgetState
async fn create_test_budget_state( pool: SqlitePool ) -> BudgetState
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];

  let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
  );
  let lease_manager = Arc::new( LeaseManager::from_pool( pool.clone() ) );
  let agent_budget_manager = Arc::new(
    iron_token_manager::agent_budget::AgentBudgetManager::from_pool( pool.clone() )
  );
  let provider_key_storage = Arc::new(
    iron_token_manager::provider_key_storage::ProviderKeyStorage::new( pool.clone() )
  );
  let jwt_secret = Arc::new( iron_control_api::jwt_auth::JwtSecret::new( "test_jwt_secret".to_string() ) );

  BudgetState
  {
    ic_token_manager,
    ip_token_crypto,
    lease_manager,
    agent_budget_manager,
    provider_key_storage,
    db_pool: pool,
    jwt_secret,
  }
}

/// Helper: Generate IC Token for test agent
fn create_ic_token( agent_id: i64, manager: &IcTokenManager ) -> String
{
  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    None,
  );

  manager.generate_token( &claims ).expect("LOUD FAILURE: Should generate IC Token")
}

/// Helper: Seed agent with specific budget and provider key
///
/// # Fix(issue-concurrency-001)
/// Root cause: Hardcoded agent_id=1 and provider_key id=1 conflicted with migration 017 seeded data
/// Pitfall: Always use unique IDs for test data; use agent_id > 100 and provider_key id = agent_id * 1000 to avoid conflicts
async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_usd: f64 )
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user if it doesn't exist (required for owner_id foreign key)
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent with owner_id
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0.0, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( budget_usd )
  .bind( budget_usd )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert into ai_provider_keys (actual table name from migration 004)
  // Use unique provider key ID based on agent_id to avoid conflicts between tests
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 )  // Unique provider key ID per test (e.g., agent 101 → key 101000)
  .bind( "openai" )
  .bind( "encrypted_test_key_base64" )
  .bind( "test_nonce_base64" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert usage_limits for test_user (required for budget validation)
  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_cents_per_month, current_cost_cents_this_month, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( 1000000i64 )  // $10,000 USD limit (in cents)
  .bind( 0i64 )        // No spending yet
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();
}

/// Helper: Create router for budget endpoints
async fn create_budget_router( state: BudgetState ) -> Router
{
  Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state )
}

/// Test 8: Multiple simultaneous handshakes for same agent
///
/// # Corner Case
/// 10 concurrent handshake requests, agent budget = 100 USD
///
/// # Expected Behavior
/// Only N requests succeed where N * 10 USD ≤ 100 USD (N = 10)
///
/// # Priority
/// CRITICAL - Prevents race conditions in budget allocation
#[ tokio::test ]
async fn test_multiple_simultaneous_handshakes()
{
  let pool = setup_test_db().await;
  let agent_id = 101;

  // Seed agent with exactly 100 USD budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Launch 10 concurrent handshake requests
  let mut handles = vec![];
  for _ in 0..10
  {
    let app_clone = app.clone();
    let ic_token_clone = ic_token.clone();

    let handle = tokio::spawn( async move
    {
      let request = Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from(
          json!({
            "ic_token": ic_token_clone,
            "provider": "openai"
          }).to_string()
        ))
        .unwrap();

      app_clone.oneshot( request ).await
    });

    handles.push( handle );
  }

  // Wait for all requests to complete
  let mut results = Vec::new();
  for handle in handles
  {
    results.push( handle.await );
  }

  // Count successful responses (HTTP 200)
  let mut successful_count = 0;
  let mut failed_responses = Vec::new();
  for result in &results
  {
    if let Ok( Ok( response ) ) = result
    {
      if response.status() == StatusCode::OK
      {
        successful_count += 1;
      }
      else
      {
        failed_responses.push( response.status() );
      }
    }
  }

  // Debug: Print failed response statuses if any
  if successful_count != 10
  {
    eprintln!( "Failed responses: {:?}", failed_responses );
  }

  // All 10 requests should succeed (each gets 10 USD from 100 USD budget)
  assert_eq!(
    successful_count, 10,
    "All concurrent handshakes should succeed when sufficient budget available. Failed statuses: {:?}",
    failed_responses
  );

  // Verify final budget state
  let remaining : f64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  // Budget should be fully allocated (0 remaining)
  assert_eq!( remaining, 0.0, "Budget should be fully allocated after 10x 10 USD leases" );
}

/// Test 9: Concurrent usage reports on same lease
///
/// # Corner Case
/// 2 reports submitted simultaneously, lease budget = 10 USD
///
/// # Expected Behavior
/// Both succeed if total ≤ 10, atomic updates to lease balance
///
/// # Priority
/// HIGH - Database consistency under concurrent writes
#[ tokio::test ]
async fn test_concurrent_usage_reports_on_same_lease()
{
  let pool = setup_test_db().await;
  let agent_id = 102;

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Create initial lease via handshake
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.clone().oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Launch 2 concurrent usage reports (5 USD each)
  let mut handles = vec![];
  for _ in 0..2
  {
    let app_clone = app.clone();
    let lease_id_clone = lease_id.clone();

    let handle = tokio::spawn( async move
    {
      let request = Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from(
          json!({
            "lease_id": lease_id_clone,
            "request_id": "req_test_001",
            "tokens": 1000,
            "cost_usd": 5.0,
            "model": "gpt-4",
            "provider": "openai"
          }).to_string()
        ))
        .unwrap();

      app_clone.oneshot( request ).await
    });

    handles.push( handle );
  }

  // Wait for both reports to complete
  let mut results = Vec::new();
  for handle in handles
  {
    results.push( handle.await );
  }

  // Both reports should succeed
  let successful_count = results.iter()
    .filter_map( | r | r.as_ref().ok() )
    .filter_map( | response | response.as_ref().ok() )
    .filter( | response | response.status() == StatusCode::OK )
    .count();

  assert_eq!( successful_count, 2, "Both concurrent reports should succeed" );

  // Verify lease spent is exactly 10 USD (atomic updates, no lost writes)
  let budget_spent : f64 = sqlx::query_scalar(
    "SELECT budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( &lease_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!( budget_spent, 10.0, "Lease should have exactly 10 USD spent (no lost updates)" );
}

/// Test 10: Concurrent report and refresh on same lease
///
/// # Corner Case
/// Usage report running while refresh executes on same lease
///
/// # Expected Behavior
/// No lost updates, consistent final state (operations atomic)
///
/// # Priority
/// MEDIUM - Complex concurrent scenario
#[ tokio::test ]
async fn test_concurrent_report_and_refresh()
{
  let pool = setup_test_db().await;
  let agent_id = 103;

  // Seed agent with sufficient budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Create initial lease
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.clone().oneshot( handshake_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Launch concurrent report and refresh
  let app_report = app.clone();
  let lease_id_report = lease_id.clone();
  let report_handle = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/budget/report" )
      .header( "content-type", "application/json" )
      .body( Body::from(
        json!({
          "lease_id": lease_id_report,
          "request_id": "req_test_concurrent",
          "tokens": 500,
          "cost_usd": 3.0,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string()
      ))
      .unwrap();

    app_report.oneshot( request ).await
  });

  let app_refresh = app.clone();
  let ic_token_refresh = ic_token.clone();
  let refresh_handle = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/budget/refresh" )
      .header( "content-type", "application/json" )
      .body( Body::from(
        json!({
          "ic_token": ic_token_refresh,
          "current_lease_id": lease_id,
          "requested_budget": 10.0
        }).to_string()
      ))
      .unwrap();

    app_refresh.oneshot( request ).await
  });

  // Wait for both operations
  let ( report_result, refresh_result ) = tokio::join!( report_handle, refresh_handle );

  // Both operations should succeed
  assert!( report_result.unwrap().unwrap().status() == StatusCode::OK );
  assert!( refresh_result.unwrap().unwrap().status() == StatusCode::OK );

  // Verify database consistency (no lost updates)
  let lease_count : i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM budget_leases WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  // Should have 2 leases total (original + refreshed)
  assert_eq!( lease_count, 2, "Should have original lease + refreshed lease" );
}

/// Test 11: Handshake while active lease exists
///
/// # Corner Case
/// Agent already has active lease, requests another handshake
///
/// # Expected Behavior
/// New lease created, both leases active simultaneously (design clarification)
///
/// # Priority
/// LOW - Design validation
#[ tokio::test ]
async fn test_handshake_with_existing_active_lease()
{
  let pool = setup_test_db().await;
  let agent_id = 104;

  // Seed agent with sufficient budget for multiple leases
  seed_agent_with_budget( &pool, agent_id, 50.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Create first lease
  let first_handshake = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let first_response = app.clone().oneshot( first_handshake ).await.unwrap();
  assert_eq!( first_response.status(), StatusCode::OK );

  // Create second lease while first is still active
  let second_handshake = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let second_response = app.clone().oneshot( second_handshake ).await.unwrap();

  // Second handshake should succeed
  assert_eq!(
    second_response.status(), StatusCode::OK,
    "Should allow multiple active leases per agent"
  );

  // Verify both leases exist and are active
  let active_lease_count : i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM budget_leases WHERE agent_id = ? AND lease_status = 'active'"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!(
    active_lease_count, 2,
    "Agent should have 2 active leases simultaneously"
  );
}

/// Test: TOCTOU race condition - 2 concurrent handshakes with insufficient budget
///
/// # Corner Case
/// 2 concurrent handshake requests, agent budget = exactly $10.00
/// Each request wants DEFAULT_HANDSHAKE_BUDGET ($10.00)
///
/// # Expected Behavior
/// Exactly 1 request succeeds, exactly 1 fails with 403 Budget Exceeded
/// This tests the Time-of-Check-to-Time-of-Use (TOCTOU) race condition
///
/// # Root Cause (if this test fails)
/// Handshake function checks budget_remaining BEFORE creating lease and recording spending.
/// Without transactional protection of the entire check-grant-record sequence:
/// - Thread A: checks budget_remaining = $10, passes
/// - Thread B: checks budget_remaining = $10, passes  (before Thread A commits)
/// - Thread A: grants $10 lease, records -$10 spending
/// - Thread B: grants $10 lease, records -$10 spending
/// - Result: budget_remaining = -$10 (NEGATIVE, violates invariant)
///
/// # Why Not Caught (if this test didn't exist)
/// Existing test `test_multiple_simultaneous_handshakes` uses sufficient budget ($100 for 10x$10),
/// so all requests succeed. Race condition only manifests at budget boundary.
///
/// # Fix Applied (if race exists)
/// Must wrap check-grant-record sequence in a database transaction with proper locking.
/// Options:
/// 1. Explicit transaction around entire handshake logic
/// 2. SELECT FOR UPDATE on agent_budgets table during check
/// 3. Optimistic locking with version field
///
/// # Prevention
/// Always test boundary conditions for concurrent operations:
/// - Exact resource match (budget = 2 requests, not budget >> requests)
/// - Resource contention scenarios
/// - Add "insufficient budget" variants for all concurrency tests
///
/// # Pitfall
/// SQLite's database-level locking might HIDE this race in simple tests.
/// Need to test with high concurrency (20+ threads) or add artificial delays
/// to expose race window. Consider testing with PostgreSQL for finer-grained locking.
#[ tokio::test ]
async fn test_toctou_race_insufficient_budget()
{
  let pool = setup_test_db().await;
  let agent_id = 105;

  // Seed agent with EXACTLY $10.00 (enough for only 1 handshake)
  seed_agent_with_budget( &pool, agent_id, 10.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Launch 2 concurrent handshake requests (each wants $10.00)
  let mut handles = vec![];
  for _ in 0..2
  {
    let app_clone = app.clone();
    let ic_token_clone = ic_token.clone();

    let handle = tokio::spawn( async move
    {
      let request = Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from(
          json!({
            "ic_token": ic_token_clone,
            "provider": "openai"
          }).to_string()
        ))
        .unwrap();

      app_clone.oneshot( request ).await
    });

    handles.push( handle );
  }

  // Wait for all requests to complete
  let mut results = Vec::new();
  for handle in handles
  {
    results.push( handle.await );
  }

  // Count successful (200 OK) and failed (403 Forbidden) responses
  let mut success_count = 0;
  let mut forbidden_count = 0;
  let mut other_statuses = Vec::new();

  for result in &results
  {
    if let Ok( Ok( response ) ) = result
    {
      match response.status()
      {
        StatusCode::OK => success_count += 1,
        StatusCode::FORBIDDEN => forbidden_count += 1,
        status => other_statuses.push( status ),
      }
    }
  }

  // Debug: Print actual distribution if test fails
  if success_count != 1 || forbidden_count != 1
  {
    eprintln!( "TOCTOU Race Detected!" );
    eprintln!( "Success: {}, Forbidden: {}, Other: {:?}", success_count, forbidden_count, other_statuses );
  }

  // CRITICAL: Exactly 1 should succeed, exactly 1 should fail
  assert_eq!(
    success_count, 1,
    "TOCTOU race violation: Expected exactly 1 successful handshake with insufficient budget. Got {success_count} successes."
  );

  assert_eq!(
    forbidden_count, 1,
    "Expected exactly 1 forbidden (403) response. Got {forbidden_count} forbidden."
  );

  // Verify final budget state - should be EXACTLY 0 (not negative)
  let remaining : f64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!(
    remaining >= 0.0,
    "Budget remaining must not be negative (invariant violation). Got: {remaining}"
  );

  assert_eq!(
    remaining, 0.0,
    "Budget should be fully allocated after 1x $10 lease from $10 budget. Got: {remaining}"
  );
}
