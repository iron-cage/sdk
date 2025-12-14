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

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
};
use common::budget::
{
  setup_test_db,
  create_test_budget_state,
  create_ic_token,
  seed_agent_with_budget,
  create_budget_router,
};
use serde_json::json;
use tower::ServiceExt;

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
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

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
  let remaining : i64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  // Budget should be fully allocated (0 remaining)
  assert_eq!( remaining, 0, "Budget should be fully allocated after 10x 10 USD leases" );
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
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

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
            "cost_microdollars": 5_000_000,
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
  let budget_spent : i64 = sqlx::query_scalar(
    "SELECT budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( &lease_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!( budget_spent, 10_000_000, "Lease should have exactly 10 USD spent (no lost updates)" );
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
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

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
          "cost_microdollars": 3_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string()
      ))
      .unwrap();

    app_report.oneshot( request ).await
  });

  let app_refresh = app.clone();
  let ic_token_refresh = ic_token.clone();

  // Create JWT token for authenticated request (GAP-003: JWT authentication required)
  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  let refresh_handle = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( "/api/budget/refresh" )
      .header( "content-type", "application/json" )
      .header( "authorization", format!( "Bearer {}", access_token ) )
      .body( Body::from(
        json!({
          "ic_token": ic_token_refresh,
          "current_lease_id": lease_id,
          "requested_budget": 10_000_000
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
  seed_agent_with_budget( &pool, agent_id, 50_000_000 ).await;

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
/// # Fix Applied (issue-budget-006)
/// Created `AgentBudgetManager::check_and_reserve_budget()` method that atomically
/// checks and reserves budget using conditional UPDATE with WHERE clause and CASE
/// expression to calculate partial grants: `min(requested, budget_remaining)`.
/// SQLite's row-level write lock ensures only one UPDATE succeeds when multiple
/// transactions compete for insufficient budget. This is the "optimistic concurrency
/// control" pattern. Replaced separate `get_budget_status()` + `record_spending()`
/// calls in handshake with single `check_and_reserve_budget()` call.
///
/// **Critical discovery**: Under high concurrency (10+ simultaneous requests), SQLite
/// returns "database is deadlocked" errors (not just "locked" or "busy"). Added retry
/// logic with exponential backoff (50 retries, max 256ms delay) that detects all three
/// error types: "database is locked", "database is busy", and "deadlock".
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
// test_kind: bug_reproducer(issue-budget-006)
#[ tokio::test ]
async fn test_toctou_race_insufficient_budget()
{
  let pool = setup_test_db().await;
  let agent_id = 105;

  // Seed agent with EXACTLY $10.00 (enough for only 1 handshake)
  seed_agent_with_budget( &pool, agent_id, 10_000_000 ).await;

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
  let remaining : i64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!(
    remaining >= 0,
    "Budget remaining must not be negative (invariant violation). Got: {remaining}"
  );

  assert_eq!(
    remaining, 0,
    "Budget should be fully allocated after 1x $10 lease from $10 budget. Got: {remaining}"
  );
}

// xxx: Manual Test Gap #29: Concurrent refreshes - DEFERRED
//
// Issue: Agent-level refresh endpoint (agent_id + additional_budget) may not exist
// or may use different API than lease-level refresh (ic_token + current_lease_id).
// Need to investigate correct refresh API before implementing concurrent test.
//
// Original test attempted to test:
// - Multiple simultaneous POST /api/budget/refresh with agent_id + additional_budget
// - Expected all refreshes to succeed (additive operation)
// - Verify final_budget = initial_budget + (additional_budget × request_count)
// - Verify budget invariant maintained
//
// See also: test_concurrent_report_and_refresh which uses lease-level refresh API
