//! Budget Lease Lifecycle Tests (Batch 2)
//!
//! Tests for Protocol 005 Budget Control lease lifecycle operations:
//! - L3: Report on expired lease
//! - L4: Report on revoked lease
//! - L5: Multiple reports on same lease
//! - L6: Lease budget exhaustion mid-session
//! - L7: Lease renewal workflow
//!
//! These tests verify proper handling of lease state transitions and budget tracking.

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

/// L3: Test reporting usage on an expired lease
///
/// Verifies that expired leases reject usage reports with 403 Forbidden.
#[tokio::test]
async fn test_report_on_expired_lease()
{
  // Setup test database and app
  let pool = setup_test_db().await;
  let agent_id = 300i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Step 1: Handshake to create lease
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": 10_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Step 2: Manually expire the lease by setting expires_at to past timestamp
  let now_ms = chrono::Utc::now().timestamp_millis();
  let expired_time = now_ms - 1000; // 1 second ago

  sqlx::query( "UPDATE budget_leases SET expires_at = ? WHERE id = ?" )
    .bind( expired_time )
    .bind( lease_id )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Should update lease expiration" );

  // Step 3: Attempt to report usage on expired lease
  let report_response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_id,
          "request_id": "req_expired_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: 403 Forbidden
  assert_eq!(
    report_response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Expired lease should reject usage report"
  );

  let report_body = axum::body::to_bytes( report_response.into_body(), usize::MAX ).await.unwrap();
  let report_json: serde_json::Value = serde_json::from_slice( &report_body ).unwrap();

  assert_eq!(
    report_json["error"].as_str().unwrap(), "Lease expired",
    "LOUD FAILURE: Error message should indicate lease expired"
  );

  // Verify: Lease budget_spent unchanged (report rejected)
  let lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch lease budget_spent" );

  assert_eq!(
    lease_spent, 0,
    "LOUD FAILURE: Lease budget_spent should remain 0 (report rejected)"
  );
}

/// L4: Test reporting usage on a revoked lease
///
/// Verifies that revoked leases immediately reject usage reports with 403 Forbidden.
#[tokio::test]
async fn test_report_on_revoked_lease()
{
  // Setup test database and app
  let pool = setup_test_db().await;
  let agent_id = 310i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Step 1: Handshake to create lease
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": 10_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Step 2: Revoke the lease
  sqlx::query( "UPDATE budget_leases SET lease_status = 'revoked' WHERE id = ?" )
    .bind( lease_id )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Should revoke lease" );

  // Step 3: Attempt to report usage on revoked lease
  let report_response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_id,
          "request_id": "req_revoked_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: 403 Forbidden
  assert_eq!(
    report_response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Revoked lease should reject usage report"
  );

  let report_body = axum::body::to_bytes( report_response.into_body(), usize::MAX ).await.unwrap();
  let report_json: serde_json::Value = serde_json::from_slice( &report_body ).unwrap();

  assert_eq!(
    report_json["error"].as_str().unwrap(), "Lease has been revoked",
    "LOUD FAILURE: Error message should indicate lease revoked"
  );

  // Verify: No budget deduction (report rejected)
  let agent_budget: i64 = sqlx::query_scalar( "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch agent budget" );

  assert_eq!(
    agent_budget, 90_000_000,
    "LOUD FAILURE: Agent budget should be $90 (only handshake deducted)"
  );

  let lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( lease_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch lease budget_spent" );

  assert_eq!(
    lease_spent, 0,
    "LOUD FAILURE: Lease budget_spent should remain 0"
  );
}

/// L5: Test multiple reports on same lease
///
/// Verifies that multiple usage reports correctly accumulate budget_spent
/// and enforce lease budget limit.
#[tokio::test]
async fn test_multiple_reports_same_lease()
{
  // Setup test database and app
  let pool = setup_test_db().await;
  let agent_id = 320i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Step 1: Handshake to create lease with $10 budget
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": 10_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap().to_string();

  // Step 2: Submit 5 reports of $2 each (total $10)
  for i in 0..5
  {
    let report_response = router
      .clone()
      .oneshot(
        Request::builder()
          .method( "POST" )
          .uri( "/api/budget/report" )
          .header( "content-type", "application/json" )
          .body( Body::from( json!(
          {
            "lease_id": lease_id,
            "request_id": format!( "req_multi_{}", i ),
            "tokens": 500,
            "cost_microdollars": 2_000_000,
            "model": "gpt-4",
            "provider": "openai"
          } ).to_string() ) )
          .unwrap()
      )
      .await
      .unwrap();

    assert_eq!(
      report_response.status(),
      StatusCode::OK,
      "LOUD FAILURE: Report {} should succeed", i
    );
  }

  // Verify: Cumulative budget tracking (all $10 spent)
  let lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( &lease_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch lease budget_spent" );

  assert_eq!(
    lease_spent, 10_000_000,
    "LOUD FAILURE: Lease should have $10 spent (5 reports × $2)"
  );

  // Step 3: 6th report should exceed lease budget ($2 more when $0 remaining)
  let response_6 = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_id,
          "request_id": "req_multi_6",
          "tokens": 500,
          "cost_microdollars": 2_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: 403 Forbidden (lease budget is hard limit)
  assert_eq!(
    response_6.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: 6th report should be rejected (insufficient lease budget)"
  );

  let response_6_body = axum::body::to_bytes( response_6.into_body(), usize::MAX ).await.unwrap();
  let response_6_json: serde_json::Value = serde_json::from_slice( &response_6_body ).unwrap();

  assert_eq!(
    response_6_json["error"].as_str().unwrap(), "Insufficient lease budget",
    "LOUD FAILURE: Error should indicate insufficient lease budget"
  );
}

/// L6: Test lease budget exhaustion mid-session
///
/// Verifies behavior when a report attempts to exceed remaining lease budget.
/// Implementation uses hard limit - reports exceeding lease budget are rejected.
#[tokio::test]
async fn test_lease_budget_exhaustion()
{
  // Setup test database and app
  let pool = setup_test_db().await;
  let agent_id = 330i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Step 1: Handshake to create lease with $10 budget
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "ic_token": ic_token,
          "provider": "openai",
          "requested_budget": 10_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap().to_string();

  // Step 2: Report $9 usage (leaving $1 remaining)
  let report_1 = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_id,
          "request_id": "req_exhaustion_1",
          "tokens": 4500,
          "cost_microdollars": 9_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( report_1.status(), StatusCode::OK );

  // Verify $9 spent, $1 remaining
  let lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( &lease_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch lease budget_spent" );

  assert_eq!( lease_spent, 9_000_000, "LOUD FAILURE: Should have $9 spent" );

  // Step 3: Report $2 usage (exceeds remaining $1)
  let report_2 = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_id,
          "request_id": "req_exhaustion_2",
          "tokens": 1000,
          "cost_microdollars": 2_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Assert: Hard limit - full rejection
  assert_eq!(
    report_2.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Report exceeding lease budget should be rejected (hard limit)"
  );

  let report_2_body = axum::body::to_bytes( report_2.into_body(), usize::MAX ).await.unwrap();
  let report_2_json: serde_json::Value = serde_json::from_slice( &report_2_body ).unwrap();

  assert_eq!(
    report_2_json["error"].as_str().unwrap(), "Insufficient lease budget",
    "LOUD FAILURE: Error should indicate insufficient lease budget"
  );

  // Verify: Budget unchanged (report rejected)
  let final_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( &lease_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch lease budget_spent" );

  assert_eq!(
    final_spent, 9_000_000,
    "LOUD FAILURE: Lease budget_spent should remain $9 (2nd report rejected)"
  );
}

/// L7: Test lease renewal workflow via refresh endpoint
///
/// Verifies that refresh endpoint creates new lease, expires old lease,
/// and correctly updates agent budget.
#[tokio::test]
async fn test_lease_renewal_workflow()
{
  // Setup test database and app
  let pool = setup_test_db().await;
  let agent_id = 340i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Create JWT access token for refresh endpoint
  let access_token = common::create_test_access_token(
    "user_123",
    "test@example.com",
    "admin",
    "test_jwt_secret"
  );

  // Step 1: Handshake to create initial lease ($10)
  let handshake_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "ic_token": ic_token.clone(),
          "provider": "openai",
          "requested_budget": 10_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let handshake_body = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &handshake_body ).unwrap();
  let lease_1 = handshake_json["lease_id"].as_str().unwrap().to_string();

  // Step 2: Spend $5 on first lease
  let report_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_1,
          "request_id": "req_renewal_1",
          "tokens": 2500,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( report_response.status(), StatusCode::OK );

  // Step 3: Refresh to new lease ($20)
  let refresh_response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .body( Body::from( json!(
        {
          "ic_token": ic_token,
          "current_lease_id": lease_1,
          "requested_budget": 20_000_000
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    refresh_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Refresh should succeed"
  );

  let refresh_body = axum::body::to_bytes( refresh_response.into_body(), usize::MAX ).await.unwrap();
  let refresh_json: serde_json::Value = serde_json::from_slice( &refresh_body ).unwrap();
  let lease_2 = refresh_json["lease_id"].as_str().unwrap().to_string();

  assert_ne!(
    lease_1, lease_2,
    "LOUD FAILURE: Refresh should create new lease_id"
  );

  // Verify: New lease active
  let new_lease_status: String = sqlx::query_scalar( "SELECT lease_status FROM budget_leases WHERE id = ?" )
    .bind( &lease_2 )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch new lease status" );

  assert_eq!(
    new_lease_status, "active",
    "LOUD FAILURE: New lease should be active"
  );

  let new_lease_granted: i64 = sqlx::query_scalar( "SELECT budget_granted FROM budget_leases WHERE id = ?" )
    .bind( &lease_2 )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch new lease budget_granted" );

  assert_eq!(
    new_lease_granted, 20_000_000,
    "LOUD FAILURE: New lease should have $20 granted"
  );

  let new_lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( &lease_2 )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch new lease budget_spent" );

  assert_eq!(
    new_lease_spent, 0,
    "LOUD FAILURE: New lease should start with $0 spent"
  );

  // Verify: Agent budget reflects both leases (committed)
  let agent_budget: i64 = sqlx::query_scalar( "SELECT total_spent FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch agent total_spent" );

  assert_eq!(
    agent_budget, 35_000_000,
    "LOUD FAILURE: Agent total_spent should be $35 ($10 lease1 + $5 usage + $20 lease2)"
  );

  let budget_remaining: i64 = sqlx::query_scalar( "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch agent budget_remaining" );

  assert_eq!(
    budget_remaining, 65_000_000,
    "LOUD FAILURE: Agent budget_remaining should be $65 ($100 - $35)"
  );

  // Verify: Old lease budget_spent preserved (for billing)
  let old_lease_spent: i64 = sqlx::query_scalar( "SELECT budget_spent FROM budget_leases WHERE id = ?" )
    .bind( &lease_1 )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should fetch old lease budget_spent" );

  assert_eq!(
    old_lease_spent, 5_000_000,
    "LOUD FAILURE: Old lease budget_spent should be preserved at $5"
  );

  // Step 4: Old lease should reject new reports
  let old_report_response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!(
        {
          "lease_id": lease_1,
          "request_id": "req_old_lease_test",
          "tokens": 500,
          "cost_microdollars": 1_000_000,
          "model": "gpt-4",
          "provider": "openai"
        } ).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!(
    old_report_response.status(),
    StatusCode::FORBIDDEN,
    "LOUD FAILURE: Old lease should reject new reports"
  );

  let old_report_body = axum::body::to_bytes( old_report_response.into_body(), usize::MAX ).await.unwrap();
  let old_report_json: serde_json::Value = serde_json::from_slice( &old_report_body ).unwrap();

  let error_msg = old_report_json["error"].as_str().unwrap();
  assert!(
    error_msg.contains( "expired" ) || error_msg.contains( "revoked" ),
    "LOUD FAILURE: Old lease should be expired or revoked. Got error: {}", error_msg
  );
}
