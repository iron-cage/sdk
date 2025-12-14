//! Budget accounting invariant validation tests
//!
//! Tests verify the critical budget accounting invariant is maintained across
//! all Protocol 005 budget operations:
//!
//! **Invariant**: `total_allocated = total_spent + budget_remaining`
//!
//! This invariant MUST hold after every budget operation (handshake, report, refresh).
//! Violation indicates budget accounting corruption.
//!
//! # Authority
//! test_organization.rulebook.md § Budget Accounting Correctness
//!
//! ## Test Matrix
//!
//! | Test Case | Operation | Verification | Risk |
//! |-----------|-----------|--------------|------|
//! | `test_budget_invariant_after_handshake` | POST /api/budget/handshake | Invariant maintained after lease creation | HIGH |
//! | `test_budget_invariant_after_report` | POST /api/budget/report | Invariant maintained after usage reporting | HIGH |
//! | `test_budget_invariant_after_refresh` | POST /api/budget/refresh | Invariant maintained after budget addition | HIGH |

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
use sqlx::Row;
use tower::ServiceExt;

// xxx: Manual Test Gap #20: Budget invariant after handshake - DEFERRED
//
// Test implementation COMPLETE. Test currently failing due to incorrect API assumption.
//
// Current behavior: budget_remaining does NOT decrease after handshake (lease creation)
// Expected behavior (per test): budget_remaining should decrease by budget_granted
//
// Issue: Protocol 005 budget accounting model needs clarification:
// - Does lease creation decrease budget_remaining immediately?
// - Or does budget_remaining only decrease when usage is reported?
// - How does the budget invariant work with lease allocations?
//
// Defer until budget accounting model is clarified and test expectations updated.
// When clarified, remove #[ignore] annotation and update test assertions.
//
/// Manual Test Gap #20: Budget invariant after handshake
///
/// # Corner Case
/// POST /api/budget/handshake succeeds → verify total_allocated = total_spent + budget_remaining
///
/// # Expected Behavior
/// After handshake:
/// - Lease created with budget_granted
/// - Agent budget_remaining decreased by budget_granted
/// - Invariant maintained: total_allocated = total_spent + budget_remaining
///
/// # Risk
/// HIGH - Budget accounting corruption
#[ tokio::test ]
async fn test_budget_invariant_after_handshake()
{
  let pool = setup_test_db().await;
  let agent_id = 130i64;
  let initial_budget = 100_000_000i64;  // $100 USD
  seed_agent_with_budget( &pool, agent_id, initial_budget ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Perform handshake
  let request_body = json!({
    "ic_token": ic_token,
    "provider": "openai",
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(), StatusCode::OK,
    "LOUD FAILURE: Handshake should succeed"
  );

  // Verify budget invariant: total_allocated = total_spent + budget_remaining
  let budget = sqlx::query( "SELECT total_allocated, total_spent, budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should fetch agent budget");

  let total_allocated : i64 = budget.get( "total_allocated" );
  let total_spent : i64 = budget.get( "total_spent" );
  let budget_remaining : i64 = budget.get( "budget_remaining" );

  assert_eq!(
    total_allocated,
    total_spent + budget_remaining,
    "LOUD FAILURE: Budget invariant violated after handshake. total_allocated={}, total_spent={}, budget_remaining={}",
    total_allocated, total_spent, budget_remaining
  );

  // Verify budget_remaining decreased (lease was created)
  assert!(
    budget_remaining < initial_budget,
    "LOUD FAILURE: budget_remaining should decrease after handshake. Initial: {}, Current: {}",
    initial_budget, budget_remaining
  );
}

// xxx: Manual Test Gap #22: Budget invariant after report - DEFERRED
//
// Test implementation COMPLETE. Test currently failing due to incorrect API assumption.
//
// Current behavior: budget_remaining is 98M after create_lease() with 10M budget_granted
// Expected behavior (per test): budget_remaining should be 90M (initial_budget - budget_granted)
//
// Issue: Test assumes create_lease() decreases agent budget_remaining, but it doesn't.
// This is the same accounting model question as Test #20:
// - Does lease creation decrease budget_remaining immediately?
// - Or does budget_remaining only decrease when usage is reported?
//
// Defer until budget accounting model is clarified and test setup updated.
// When clarified, remove #[ignore] annotation and update test assertions.
//
/// Manual Test Gap #22: Budget invariant after report
///
/// # Corner Case
/// POST /api/budget/report succeeds → verify total_allocated = total_spent + budget_remaining
///
/// # Expected Behavior
/// After usage report:
/// - Lease budget_spent increased by cost_microdollars
/// - Agent total_spent increased by cost_microdollars
/// - Agent budget_remaining unchanged (spent from lease, not from remaining)
/// - Invariant maintained: total_allocated = total_spent + budget_remaining
///
/// # Risk
/// HIGH - Budget accounting corruption
#[ tokio::test ]
async fn test_budget_invariant_after_report()
{
  let pool = setup_test_db().await;
  let agent_id = 131i64;
  let initial_budget = 100_000_000i64;  // $100 USD
  seed_agent_with_budget( &pool, agent_id, initial_budget ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state.clone() ).await;

  // Create lease through handshake (reserves budget properly)
  let budget_granted = 10_000_000i64;  // $10 USD
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({ "ic_token": ic_token, "provider": "openai" }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK, "LOUD FAILURE: Handshake should succeed" );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Report usage
  let cost_microdollars = 2_000_000i64;  // $2 USD
  let request_body = json!({
    "lease_id": lease_id,
    "request_id": "req_invariant_test",
    "tokens": 1000,
    "cost_microdollars": cost_microdollars,
    "model": "gpt-4",
    "provider": "openai",
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(), StatusCode::OK,
    "LOUD FAILURE: Report usage should succeed"
  );

  // Verify budget invariant: total_allocated = total_spent + budget_remaining
  let budget = sqlx::query( "SELECT total_allocated, total_spent, budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should fetch agent budget");

  let total_allocated : i64 = budget.get( "total_allocated" );
  let total_spent : i64 = budget.get( "total_spent" );
  let budget_remaining : i64 = budget.get( "budget_remaining" );

  assert_eq!(
    total_allocated,
    total_spent + budget_remaining,
    "LOUD FAILURE: Budget invariant violated after report. total_allocated={}, total_spent={}, budget_remaining={}",
    total_allocated, total_spent, budget_remaining
  );

  // Verify total_spent = budget_granted + cost_microdollars
  // Note: total_spent tracks "committed budget" (reservations + actual usage), not just actual consumption
  assert_eq!(
    total_spent,
    budget_granted + cost_microdollars,
    "LOUD FAILURE: total_spent should equal budget_granted + cost_microdollars. Expected: {}, Actual: {}",
    budget_granted + cost_microdollars, total_spent
  );

  // Verify budget_remaining decreased by budget_granted + cost_microdollars
  assert_eq!(
    budget_remaining,
    initial_budget - budget_granted - cost_microdollars,
    "LOUD FAILURE: budget_remaining incorrect. Expected: {}, Actual: {}",
    initial_budget - budget_granted - cost_microdollars, budget_remaining
  );
}

/// Manual Test Gap #30: Budget invariant after refresh
///
/// # Corner Case
/// POST /api/budget/refresh succeeds → verify total_allocated = total_spent + budget_remaining
///
/// # Expected Behavior
/// After budget refresh:
/// - New lease created with requested_budget
/// - Old lease expired
/// - Agent total_spent increased by requested_budget (commitment tracking)
/// - Agent budget_remaining decreased by requested_budget
/// - Invariant maintained: total_allocated = total_spent + budget_remaining
///
/// # Risk
/// HIGH - Budget accounting corruption
#[ tokio::test ]
async fn test_budget_invariant_after_refresh()
{
  let pool = setup_test_db().await;
  let agent_id = 126i64;
  let initial_budget = 100_000_000i64;  // $100 USD
  seed_agent_with_budget( &pool, agent_id, initial_budget ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state.clone() ).await;

  // Create initial lease through handshake
  let initial_grant = 10_000_000i64;  // $10 USD
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({ "ic_token": ic_token.clone(), "provider": "openai" }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK, "LOUD FAILURE: Handshake should succeed" );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let current_lease_id = handshake_json["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Refresh budget (request additional lease)
  let requested_budget = 50_000_000i64;  // $50 USD
  let access_token = common::create_test_access_token( "test_user", "test@example.com", "admin", "test_jwt_secret" );

  let request_body = json!({
    "ic_token": ic_token,
    "current_lease_id": current_lease_id,
    "requested_budget": requested_budget,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(), StatusCode::OK,
    "LOUD FAILURE: Budget refresh should succeed"
  );

  // Verify budget invariant: total_allocated = total_spent + budget_remaining
  let budget = sqlx::query( "SELECT total_allocated, total_spent, budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should fetch agent budget");

  let total_allocated : i64 = budget.get( "total_allocated" );
  let total_spent : i64 = budget.get( "total_spent" );
  let budget_remaining : i64 = budget.get( "budget_remaining" );

  assert_eq!(
    total_allocated,
    total_spent + budget_remaining,
    "LOUD FAILURE: Budget invariant violated after refresh. total_allocated={}, total_spent={}, budget_remaining={}",
    total_allocated, total_spent, budget_remaining
  );

  // Verify total_spent = initial_grant + requested_budget (both leases committed)
  assert_eq!(
    total_spent,
    initial_grant + requested_budget,
    "LOUD FAILURE: total_spent should equal initial_grant + requested_budget. Expected: {}, Actual: {}",
    initial_grant + requested_budget, total_spent
  );

  // Verify budget_remaining decreased by both leases
  assert_eq!(
    budget_remaining,
    initial_budget - initial_grant - requested_budget,
    "LOUD FAILURE: budget_remaining incorrect. Expected: {}, Actual: {}",
    initial_budget - initial_grant - requested_budget, budget_remaining
  );

  // Verify total_allocated unchanged (no new budget added, just reallocated)
  assert_eq!(
    total_allocated,
    initial_budget,
    "LOUD FAILURE: total_allocated should remain unchanged. Expected: {}, Actual: {}",
    initial_budget, total_allocated
  );
}
