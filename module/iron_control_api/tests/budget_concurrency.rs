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
    .expect( "Failed to apply migrations" );
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

  BudgetState
  {
    ic_token_manager,
    ip_token_crypto,
    lease_manager,
    agent_budget_manager,
    provider_key_storage,
    db_pool: pool,
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

  manager.generate_token( &claims ).expect( "Should generate IC Token" )
}

/// Helper: Seed agent with specific budget and provider key
async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_usd: f64 )
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Insert agent
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
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
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( 1i64 )
  .bind( "openai" )
  .bind( "encrypted_test_key_base64" )
  .bind( "test_nonce_base64" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
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
  let agent_id = 1;

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
  let successful_count = results.iter()
    .filter_map( | r | r.as_ref().ok() )
    .filter_map( | response | response.as_ref().ok() )
    .filter( | response | response.status() == StatusCode::OK )
    .count();

  // All 10 requests should succeed (each gets 10 USD from 100 USD budget)
  assert_eq!(
    successful_count, 10,
    "All concurrent handshakes should succeed when sufficient budget available"
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
  let agent_id = 2;

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
  let agent_id = 3;

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
  let agent_id = 4;

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
