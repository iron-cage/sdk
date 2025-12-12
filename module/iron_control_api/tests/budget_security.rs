//! Protocol 005 security corner case tests
//!
//! Tests security-critical scenarios for Protocol 005 (Budget Control Protocol):
//! - SQL injection protection in provider names
//! - Authorization enforcement (IC Token ownership validation)
//! - IP Token replay attack prevention
//!
//! # Corner Case Coverage
//!
//! Tests address the following critical security gaps from gap analysis:
//! 12. SQL injection in provider name (CRITICAL - security)
//! 13. IC Token from different agent / authorization (CRITICAL - authorization)
//! 14. IP Token replay attack (MEDIUM - cryptographic property)

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use iron_control_api::
{
  ic_token::{ IcTokenClaims, IcTokenManager },
  routes::budget::{ BudgetState, handshake, refresh_budget },
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
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state )
}

/// Test 12: SQL injection protection in provider name
///
/// # Corner Case
/// Malicious provider name attempting SQL injection: `"openai'; DROP TABLE agents; --"`
///
/// # Expected Behavior
/// Parameterized queries prevent injection, return validation error instead of executing SQL
///
/// # Priority
/// CRITICAL - Security vulnerability prevention
#[ tokio::test ]
async fn test_sql_injection_in_provider_name()
{
  let pool = setup_test_db().await;
  let agent_id = 1;

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Attempt SQL injection via provider field
  let malicious_provider = "openai'; DROP TABLE agents; --";

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": malicious_provider
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return validation error (400) not execute SQL
  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
    "SQL injection should be prevented, got status: {}", response.status()
  );

  // Verify agents table still exists (injection failed)
  let agent_count : i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM agents" )
    .fetch_one( &pool )
    .await
    .expect( "agents table should still exist" );

  assert_eq!( agent_count, 1, "agents table should be intact (SQL injection prevented)" );
}

/// Test 13: Authorization enforcement - IC Token from different agent
///
/// # Corner Case
/// IC Token contains agent_id=123, but refresh request is for lease owned by agent_id=456
///
/// # Expected Behavior
/// HTTP 403 Forbidden "Unauthorized - lease belongs to different agent"
///
/// # Priority
/// CRITICAL - Authorization bypass prevention
#[ tokio::test ]
async fn test_ic_token_authorization_enforcement()
{
  let pool = setup_test_db().await;
  let agent_1 = 1;
  let agent_2 = 2;

  // Seed both agents with budgets
  seed_agent_with_budget( &pool, agent_1, 100.0 ).await;

  // Manually add agent 2 (different from agent 1)
  let now_ms = chrono::Utc::now().timestamp_millis();
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)"
  )
  .bind( agent_2 )
  .bind( "test_agent_2" )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0.0, ?, ?, ?)"
  )
  .bind( agent_2 )
  .bind( 100.0 )
  .bind( 100.0 )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_test_budget_state( pool.clone() ).await;

  // Create IC Token for agent 1
  let ic_token_agent_1 = create_ic_token( agent_1, &state.ic_token_manager );

  // Create lease for agent 1
  let app = create_budget_router( state.clone() ).await;
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token_agent_1.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Attempt to refresh agent 1's lease using agent 2's IC Token (authorization violation)
  let ic_token_agent_2 = create_ic_token( agent_2, &state.ic_token_manager );

  let app2 = create_budget_router( state ).await;
  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token_agent_2,
        "current_lease_id": lease_id
      }).to_string()
    ))
    .unwrap();

  let refresh_response = app2.oneshot( refresh_request ).await.unwrap();

  // Should return 403 Forbidden (authorization violation)
  assert_eq!(
    refresh_response.status(), StatusCode::FORBIDDEN,
    "Should reject refresh from different agent's IC Token"
  );
}

/// Test 14: IP Token replay attack prevention
///
/// # Corner Case
/// Same IP Token used multiple times (replay attack)
///
/// # Expected Behavior
/// Each handshake produces unique IP Token (different nonce), preventing replay
///
/// # Priority
/// MEDIUM - Cryptographic security property
#[ tokio::test ]
async fn test_ip_token_replay_prevention()
{
  let pool = setup_test_db().await;
  let agent_id = 1;

  // Seed agent with sufficient budget for multiple handshakes
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Perform first handshake
  let app1 = create_budget_router( state.clone() ).await;
  let request1 = Request::builder()
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

  let response1 = app1.oneshot( request1 ).await.unwrap();
  assert_eq!( response1.status(), StatusCode::OK );

  let body1 = axum::body::to_bytes( response1.into_body(), usize::MAX ).await.unwrap();
  let data1 : serde_json::Value = serde_json::from_slice( &body1 ).unwrap();
  let ip_token_1 = data1[ "ip_token" ].as_str().unwrap();

  // Perform second handshake with same IC Token
  let app2 = create_budget_router( state ).await;
  let request2 = Request::builder()
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

  let response2 = app2.oneshot( request2 ).await.unwrap();
  assert_eq!( response2.status(), StatusCode::OK );

  let body2 = axum::body::to_bytes( response2.into_body(), usize::MAX ).await.unwrap();
  let data2 : serde_json::Value = serde_json::from_slice( &body2 ).unwrap();
  let ip_token_2 = data2[ "ip_token" ].as_str().unwrap();

  // IP Tokens should be different (unique nonce per encryption)
  assert_ne!(
    ip_token_1, ip_token_2,
    "Each handshake should produce unique IP Token (prevents replay attacks)"
  );
}
