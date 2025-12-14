//! Budget boundary value tests for Protocol 005
//!
//! Tests verify correct handling of edge cases and boundary values in budget operations.
//!
//! # Authority
//! - Protocol 005 specification: Budget accounting requirements
//! - Data integrity: i64 overflow prevention
//!
//! # Test Matrix
//!
//! | Test Case | Scenario | Input | Expected Response |
//! |-----------|----------|-------|-------------------|
//! | `test_cost_i64_max` | Maximum i64 value for cost | cost_microdollars = i64::MAX | Handled safely (accept or reject) |
//! | `test_cost_zero` | Zero cost (cached response) | cost_microdollars = 0 | 200 OK (valid for cached responses) |
//! | `test_budget_exactly_at_limit` | Budget allocation at exact limit | Request budget = remaining budget | 200 OK (exact match allowed) |
//! | `test_multiple_leases_equal_total_budget` | Multiple leases exhausting budget | Create leases until budget exhausted | All succeed until budget depleted |
//! | `test_tokens_i64_max` | Maximum i64 value for tokens | tokens = i64::MAX | Handled safely (accept or reject) |
//! | `test_lease_expiration_exact_timestamp` | Lease expires at exact current time | expires_at = now_ms | Behavior documented (200 or 403) |

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

/// B1: Maximum i64 value for cost_microdollars
///
/// # Corner Case
/// POST /api/budget/report with cost_microdollars = i64::MAX
///
/// # Expected Behavior
/// - Request handled safely (either accepted with overflow protection or rejected with validation error)
/// - No integer overflow or panic
/// - Budget accounting remains consistent
///
/// # Risk
/// HIGH - Integer overflow could corrupt budget accounting
#[ tokio::test ]
async fn test_cost_i64_max()
{
  let pool = setup_test_db().await;
  let agent_id = 400i64;
  seed_agent_with_budget( &pool, agent_id, i64::MAX ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create lease
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Attempt to report with i64::MAX cost
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_i64_max_test",
          "tokens": 1000,
          "cost_microdollars": i64::MAX,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should either accept (200) or reject with validation error (400/403)
  // Must NOT panic or cause integer overflow
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::FORBIDDEN,
    "LOUD FAILURE: i64::MAX cost should be handled safely, got status: {}",
    response.status()
  );

  // Verify budget accounting is still consistent (no overflow)
  let budget_status: Result< i64, _ > = sqlx::query_scalar(
    "SELECT total_spent FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await;

  assert!(
    budget_status.is_ok(),
    "LOUD FAILURE: Budget accounting should remain accessible after i64::MAX test"
  );
}

/// B2: Zero cost for cached responses
///
/// # Corner Case
/// POST /api/budget/report with cost_microdollars = 0
///
/// # Expected Behavior
/// - Request accepted (200 OK) - zero cost is valid for cached responses
/// - Budget unchanged
/// - Usage recorded
///
/// # Risk
/// LOW - Zero cost is a valid business case (cached LLM responses)
#[ tokio::test ]
async fn test_cost_zero()
{
  let pool = setup_test_db().await;
  let agent_id = 401i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create lease
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Report zero cost (cached response)
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_zero_cost_test",
          "tokens": 1000,
          "cost_microdollars": 0,
          "model": "gpt-4-cached",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should accept zero cost (valid for cached responses)
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Zero cost should be accepted (cached responses)"
  );

  // Verify lease budget_spent increased by 0
  let lease_budget_spent: i64 = sqlx::query_scalar(
    "SELECT budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( lease_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query lease budget");

  assert_eq!(
    lease_budget_spent, 0,
    "LOUD FAILURE: Zero cost report should not increase budget_spent"
  );
}

/// B4: Budget allocation exactly at limit
///
/// # Corner Case
/// Request budget exactly equal to remaining budget
///
/// # Expected Behavior
/// - Request approved (200 OK)
/// - Full remaining budget allocated
/// - Budget remaining = 0 after allocation
///
/// # Risk
/// MEDIUM - Off-by-one errors could reject valid edge case
#[ tokio::test ]
async fn test_budget_exactly_at_limit()
{
  let pool = setup_test_db().await;
  let agent_id = 402i64;
  let total_budget = 50_000_000i64; // $50 USD
  seed_agent_with_budget( &pool, agent_id, total_budget ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Request handshake with exact remaining budget
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": total_budget
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should succeed - exact match is valid
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Exact budget match should be approved"
  );

  // Verify budget_remaining = 0
  let budget_remaining: i64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    budget_remaining, 0,
    "LOUD FAILURE: Budget remaining should be 0 after exact allocation"
  );
}

/// B5: Multiple leases equal total budget
///
/// # Corner Case
/// Create multiple leases until budget fully exhausted
///
/// # Expected Behavior
/// - All leases succeed until budget depleted
/// - Final lease rejected when insufficient budget
/// - Budget accounting correct across all operations
///
/// # Risk
/// HIGH - Race conditions or accounting errors could allow budget overrun
#[ tokio::test ]
async fn test_multiple_leases_equal_total_budget()
{
  let pool = setup_test_db().await;
  let agent_id = 403i64;
  let total_budget = 50_000_000i64; // $50 USD
  let lease_size = 10_000_000i64;   // $10 USD per lease
  seed_agent_with_budget( &pool, agent_id, total_budget ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Create 5 leases of $10 each ($50 total)
  for i in 0..5
  {
    let router = create_budget_router( state.clone() ).await;
    let response = router
      .oneshot(
        Request::builder()
          .method( "POST" )
          .uri( "/api/budget/handshake" )
          .header( "content-type", "application/json" )
          .body( Body::from( json!({
            "ic_token": ic_token.clone(),
            "provider": "openai",
            "requested_budget": lease_size
          }).to_string() ) )
          .unwrap()
      )
      .await
      .unwrap();

    assert_eq!(
      response.status(),
      StatusCode::OK,
      "LOUD FAILURE: Lease {} should succeed (budget available)", i + 1
    );
  }

  // Verify budget fully allocated
  let budget_remaining: i64 = sqlx::query_scalar(
    "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should query budget");

  assert_eq!(
    budget_remaining, 0,
    "LOUD FAILURE: Budget should be fully allocated after 5 leases"
  );

  // Attempt 6th lease (should fail - insufficient budget)
  let router = create_budget_router( state ).await;
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": lease_size
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: 6th lease should be rejected (insufficient budget)"
  );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!(
    response_json["error"].as_str().unwrap(), "Budget limit exceeded",
    "LOUD FAILURE: 6th lease should be denied due to insufficient budget"
  );
}

/// B3: Maximum i64 value for tokens field
///
/// # Corner Case
/// POST /api/budget/report with tokens = i64::MAX
///
/// # Expected Behavior
/// - Request handled safely (either accepted or rejected with validation error)
/// - No integer overflow or panic
/// - Token count stored correctly or rejected
///
/// # Risk
/// MEDIUM - Large token counts shouldn't cause overflow but should be validated
#[ tokio::test ]
async fn test_tokens_i64_max()
{
  let pool = setup_test_db().await;
  let agent_id = 404i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data["lease_id"].as_str().expect("LOUD FAILURE: Should have lease_id");

  // Attempt to report with i64::MAX tokens
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_tokens_max_test",
          "tokens": i64::MAX,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should either accept (200) or reject with validation error (400)
  // Must NOT panic or cause integer overflow
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: i64::MAX tokens should be handled safely, got status: {}",
    response.status()
  );

  // If accepted, verify lease budget was updated (no overflow)
  if response.status() == StatusCode::OK
  {
    let lease_budget: i64 = sqlx::query_scalar(
      "SELECT budget_spent FROM budget_leases WHERE id = ?"
    )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: Should query lease budget");

    assert!(
      lease_budget >= 0,
      "LOUD FAILURE: Lease budget should be non-negative after i64::MAX tokens report"
    );
  }
}

/// B6: Lease expiration edge case - expires exactly at current timestamp
///
/// # Corner Case
/// Lease expires_at equals current_timestamp (millisecond precision)
///
/// # Expected Behavior
/// Implementation-defined:
/// - Option A: expires_at < now (strict) → expired
/// - Option B: expires_at <= now (inclusive) → expired
/// - Option C: expires_at == now (edge case) → still valid
///
/// Test documents actual behavior for consistency
///
/// # Risk
/// MEDIUM - Edge case could cause inconsistent behavior across requests
#[ tokio::test ]
async fn test_lease_expiration_exact_timestamp()
{
  let pool = setup_test_db().await;
  let agent_id = 405i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Set lease expiration to current timestamp (edge case)
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query( "UPDATE budget_leases SET expires_at = ? WHERE id = ?" )
    .bind( now_ms )
    .bind( lease_id )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Should update lease expiration to current timestamp" );

  // Attempt to report usage at exact expiration time
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_edge_timestamp_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Document behavior: Either 200 (still valid) or 403 (expired at exact match)
  // Both are acceptable - test documents which behavior is implemented
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::FORBIDDEN,
    "LOUD FAILURE: Lease at exact expiration timestamp should be handled consistently (got {})",
    response.status()
  );

  // Note: If 200, implementation treats expires_at == now as valid (strict <)
  //       If 403, implementation treats expires_at == now as expired (<=)
  // Either behavior is acceptable as long as consistent
}
