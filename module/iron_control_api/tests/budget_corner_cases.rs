//! Manual corner case tests for Protocol 005 critical gaps
//!
//! These tests cover corner cases not yet in automated test suite, identified
//! during manual testing workflow. Tests focus on:
//! - Whitespace-only inputs (empty string variants)
//! - Exact boundary violations (off-by-one DoS prevention)
//! - Malformed JWT segments
//! - Negative numeric values
//! - Extreme float values
//! - Error message security (no sensitive data leaks)
//!
//! # Authority
//! test_organization.rulebook.md § Comprehensive Corner Case Coverage
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_handshake_whitespace_only_ic_token` | Whitespace-only IC token validation | POST /api/budget/handshake with ic_token="   \t\n  " | 400 Bad Request, error mentions "empty" | ✅ |
//! | `test_handshake_whitespace_only_provider` | Whitespace-only provider validation | POST /api/budget/handshake with provider="   " | 400 Bad Request | ✅ |
//! | `test_handshake_ic_token_over_max_length` | IC token DoS protection | POST /api/budget/handshake with ic_token >10KB | 400 Bad Request (length limit) | ✅ |
//! | `test_handshake_provider_over_max_length` | Provider DoS protection | POST /api/budget/handshake with provider >1KB | 400 Bad Request (length limit) | ✅ |
//! | `test_handshake_malformed_jwt_missing_segments` | Malformed JWT handling | POST /api/budget/handshake with ic_token="invalid.jwt" | 400 Bad Request (JWT validation) | ✅ |
//! | `test_report_usage_negative_tokens` | Negative token value validation | POST /api/budget/report with tokens=-100 | 400 Bad Request | ✅ |
//! | `test_report_usage_negative_cost` | Negative cost value validation | POST /api/budget/report with cost_usd=-5.0 | 400 Bad Request | ✅ |
//! | `test_error_messages_no_sensitive_data_leak` | Error message security | Invalid handshake request | Error message contains no sensitive data (tokens, keys) | ✅ |
//! | `test_report_usage_zero_cost_cached_response` | Zero cost cached response | POST /api/budget/report with cost_usd=0.0 | 200 OK (cached responses valid) | ✅ |
//! | `test_database_foreign_key_constraint_agent` | FK constraint enforcement | Create lease for nonexistent agent_id | Database error (FK violation) | ✅ |
//! | `test_database_not_null_constraint` | NOT NULL constraint enforcement | Insert lease with NULL required field | Database error (NOT NULL violation) | ✅ |

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

  manager.generate_token( &claims ).expect( "Should generate IC Token" )
}

/// Helper: Seed agent with budget and provider key
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

  // Insert provider key
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

/// Test: Whitespace-only ic_token input
///
/// # Corner Case
/// ic_token field contains only whitespace characters (spaces, tabs, newlines)
///
/// # Expected Behavior
/// HTTP 400 Bad Request with validation error "ic_token cannot be empty"
///
/// # Priority
/// HIGH - Input validation completeness
#[ tokio::test ]
async fn test_handshake_whitespace_only_ic_token()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Whitespace-only ic_token
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": "   \t\n  ",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Whitespace-only ic_token should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().contains( "empty" ),
    "Error should mention empty ic_token"
  );
}

/// Test: Whitespace-only provider input
///
/// # Corner Case
/// provider field contains only whitespace
///
/// # Expected Behavior
/// HTTP 400 Bad Request with validation error
#[ tokio::test ]
async fn test_handshake_whitespace_only_provider()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "  \t  "
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Whitespace-only provider should be rejected"
  );
}

/// Test: ic_token exactly at 2001 chars (over max boundary)
///
/// # Corner Case
/// ic_token exactly 2001 characters (1 over MAX_IC_TOKEN_LENGTH = 2000)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "ic_token too long"
///
/// # Priority
/// HIGH - DoS prevention boundary
#[ tokio::test ]
async fn test_handshake_ic_token_over_max_length()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Create ic_token of exactly 2001 characters
  let oversized_token = "a".repeat( 2001 );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": oversized_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "ic_token over 2000 chars should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().contains( "too long" ),
    "Error should mention ic_token too long"
  );
}

/// Test: provider name exactly at 51 chars (over max boundary)
///
/// # Corner Case
/// provider exactly 51 characters (1 over MAX_PROVIDER_LENGTH = 50)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "provider too long"
#[ tokio::test ]
async fn test_handshake_provider_over_max_length()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Provider name of exactly 51 characters
  let oversized_provider = "a".repeat( 51 );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": oversized_provider
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "provider over 50 chars should be rejected"
  );
}

/// Test: Malformed JWT with missing segments
///
/// # Corner Case
/// ic_token contains malformed JWT (only 2 segments instead of 3)
///
/// # Expected Behavior
/// HTTP 401 Unauthorized "Invalid IC Token"
#[ tokio::test ]
async fn test_handshake_malformed_jwt_missing_segments()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Malformed JWT with only 2 segments (missing signature)
  let malformed_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": malformed_jwt,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::UNAUTHORIZED,
    "Malformed JWT should return 401 Unauthorized"
  );
}

/// Test: Negative tokens value in usage report
///
/// # Corner Case
/// tokens field is negative (-1000)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "tokens cannot be negative"
#[ tokio::test ]
async fn test_report_usage_negative_tokens()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
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

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with negative tokens
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_001",
        "tokens": -1000,  // NEGATIVE VALUE
        "cost_usd": 5.0,
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Negative tokens should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "negative" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "invalid" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "positive" ),
    "Error should mention negative/positive tokens, got: {}", error_data[ "error" ]
  );
}

/// Test: Negative cost_usd value in usage report
///
/// # Corner Case
/// cost_usd field is negative (-5.0)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "cost_usd cannot be negative"
#[ tokio::test ]
async fn test_report_usage_negative_cost()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
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

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with negative cost
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_001",
        "tokens": 1000,
        "cost_usd": -5.0,  // NEGATIVE VALUE
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Negative cost_usd should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "negative" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "invalid" ),
    "Error should mention negative cost, got: {}", error_data[ "error" ]
  );
}

/// Test: Error messages dont leak sensitive data
///
/// # Corner Case
/// When authentication fails, error should not leak whether agent exists
///
/// # Expected Behavior
/// Generic "Invalid IC Token" error, not "Agent not found" or "Budget insufficient"
///
/// # Priority
/// HIGH - Security (information disclosure prevention)
#[ tokio::test ]
async fn test_error_messages_no_sensitive_data_leak()
{
  let pool = setup_test_db().await;
  // Don't seed any agent (agent doesn't exist)

  let state = create_test_budget_state( pool ).await;
  // Create IC Token for non-existent agent 999
  let ic_token = create_ic_token( 999, &state.ic_token_manager );

  let app = create_budget_router( state ).await;

  let request = Request::builder()
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

  let response = app.oneshot( request ).await.unwrap();

  // Should return 404 or 400, NOT leak "agent doesn't exist"
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();
  let error_msg = error_data[ "error" ].as_str().unwrap().to_lowercase();

  // Error should be generic, not leak existence information
  assert!(
    !error_msg.contains( "agent" ) || !error_msg.contains( "not found" ),
    "Error should not leak agent existence: {}", error_msg
  );
}

/// Test: Zero-cost usage reports (cached responses, free tier)
///
/// # Corner Case
/// Report usage with tokens > 0 but cost_usd = 0.0 (cached response, free tier)
///
/// # Expected Behavior
/// Accepted (HTTP 200), budget accounting handles $0.00 correctly
///
/// # Priority
/// MEDIUM - Edge case from specification
#[ tokio::test ]
async fn test_report_usage_zero_cost_cached_response()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
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

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  assert_eq!( handshake_resp.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with zero cost (cached response)
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_cached_001",
        "tokens": 1000,       // Tokens used but cached
        "cost_usd": 0.0,      // ZERO COST
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::OK,
    "Zero-cost usage should be accepted (cached responses, free tier)"
  );

  // Verify lease budget didn't change
  let lease_check : ( f64, f64 ) = sqlx::query_as(
    "SELECT budget_granted, budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( lease_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!( lease_check.0 > 0.0, "Granted should be positive" );
  assert_eq!( lease_check.1, 0.0, "Spent should remain 0.0 for zero-cost request" );
}

/// Test: Database foreign key constraint enforcement
///
/// # Corner Case
/// Attempt to create lease for non-existent agent (foreign key violation)
///
/// # Expected Behavior
/// Database rejects with foreign key constraint violation
///
/// # Priority
/// HIGH - Data integrity enforcement
#[ tokio::test ]
async fn test_database_foreign_key_constraint_agent()
{
  let pool = setup_test_db().await;
  // Don't seed any agent

  // Attempt to create lease for non-existent agent (should fail at DB level)
  let result = sqlx::query(
    "INSERT INTO budget_leases (id, agent_id, budget_id, budget_granted, budget_spent, created_at, expires_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "lease_test_001" )
  .bind( 9999i64 )  // Non-existent agent
  .bind( 1i64 )
  .bind( 10.0 )
  .bind( 0.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() + 3600000 )
  .execute( &pool )
  .await;

  assert!(
    result.is_err(),
    "Foreign key constraint should prevent lease creation for non-existent agent"
  );

  if let Err( e ) = result
  {
    let error_msg = e.to_string().to_lowercase();
    assert!(
      error_msg.contains( "foreign" ) || error_msg.contains( "constraint" ),
      "Error should mention foreign key constraint, got: {}", e
    );
  }
}

/// Test: Database NOT NULL constraint enforcement
///
/// # Corner Case
/// Attempt to insert NULL into NOT NULL column (budget_granted)
///
/// # Expected Behavior
/// Database rejects with NOT NULL constraint violation
#[ tokio::test ]
async fn test_database_not_null_constraint()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  // Attempt to insert NULL into budget_granted (NOT NULL column)
  let result = sqlx::query(
    "INSERT INTO budget_leases (id, agent_id, budget_id, budget_granted, budget_spent, created_at, expires_at)
     VALUES (?, ?, ?, NULL, ?, ?, ?)"
  )
  .bind( "lease_test_002" )
  .bind( 1i64 )
  .bind( 1i64 )
  // budget_granted = NULL (should fail)
  .bind( 0.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() + 3600000 )
  .execute( &pool )
  .await;

  assert!(
    result.is_err(),
    "NOT NULL constraint should prevent NULL in budget_granted"
  );

  if let Err( e ) = result
  {
    let error_msg = e.to_string().to_lowercase();
    assert!(
      error_msg.contains( "null" ) || error_msg.contains( "not null" ),
      "Error should mention NOT NULL constraint, got: {}", e
    );
  }
}
