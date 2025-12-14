//! Protocol 005 database state corner case tests
//!
//! Tests database-dependent corner cases for Protocol 005 (Budget Control Protocol):
//! - Non-existent agents
//! - Zero/insufficient budgets
//! - Non-existent/expired leases
//! - Budget enforcement at boundaries
//!
//! # Corner Case Coverage
//!
//! Tests address the following critical gaps from gap analysis:
//! 1. Handshake with non-existent agent (security)
//! 2. Handshake with zero agent budget (enforcement boundary)
//! 3. Handshake with insufficient budget for lease (enforcement)
//! 4. Report usage with non-existent lease (prevents phantom usage)
//! 5. Report usage on expired lease (time boundary)
//! 6. Report usage exceeding lease budget (CRITICAL enforcement)
//! 7. Refresh with insufficient agent budget (enforcement)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_handshake_with_nonexistent_agent` | Handshake for nonexistent agent | POST /api/budget/handshake with IC token for agent_id=999 (not in DB) | 404 Not Found or 403 Forbidden | ✅ |
//! | `test_handshake_with_zero_agent_budget` | Handshake with zero budget | Agent with budget_remaining=0.0, POST /api/budget/handshake | 403 Forbidden (insufficient budget) | ✅ |
//! | `test_handshake_with_insufficient_budget_for_lease` | Handshake with budget < lease requirement | Agent with budget=5.0 USD, request lease requiring 10.0 USD | 403 Forbidden (insufficient budget) | ✅ |
//! | `test_report_usage_with_nonexistent_lease` | Report usage for nonexistent lease | POST /api/budget/report with lease_id="nonexistent_lease" | 404 Not Found | ✅ |
//! | `test_report_usage_on_expired_lease` | Report usage on expired lease | Create lease, expire it, POST /api/budget/report | 403 Forbidden or 400 Bad Request (lease expired) | ✅ |
//! | `test_report_usage_exceeding_lease_budget` | Usage exceeds lease budget limit | Lease with 10.0 USD, POST /api/budget/report with cost_usd=15.0 | 403 Forbidden (budget exceeded) | ✅ |
//! | `test_refresh_with_insufficient_agent_budget` | Refresh with insufficient agent budget | Agent budget=5.0 USD, POST /api/budget/refresh requesting 10.0 USD | 403 Forbidden (insufficient budget) | ✅ |

mod common;

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
  let provider_key_master : [ u8; 32 ] = [ 42u8; 32 ];

  let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
  );
  let provider_key_crypto = Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master ).unwrap()
  );
  let crypto_service = Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master ).unwrap()
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
    provider_key_crypto,
    db_pool: pool,
    jwt_secret,
    crypto_service: Some( crypto_service ),
  }
}

/// Helper: Generate IC Token for test agent
fn create_ic_token( agent_id: i64, manager: &IcTokenManager ) -> String
{
  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    None, // No expiration
  );

  manager.generate_token( &claims ).expect("LOUD FAILURE: Should generate IC Token")
}

/// Helper: Seed agent with specific budget and provider key
///
/// # Fix(issue-database-state-unique-001)
/// Root cause: Hardcoded agent_id=1 and provider_key id=1 conflicted with migration 017 seeded data
/// Pitfall: Always use unique IDs for test data; use agent_id > 100 and provider_key id = agent_id * 1000 to avoid conflicts
async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_microdollars: i64 )
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

  // Insert agent budget (using microdollars)
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( budget_microdollars )
  .bind( budget_microdollars )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert into ai_provider_keys (actual table name from migration 004)
  // Use unique provider key ID based on agent_id to avoid conflicts between tests
  // Create real encrypted provider key for testing
  let test_provider_key = format!( "sk-test_key_for_agent_{}", agent_id );
  let provider_key_master : [ u8; 32 ] = [ 42u8; 32 ]; // Test master key (must match create_test_budget_state)
  let crypto_service = iron_secrets::crypto::CryptoService::new( &provider_key_master )
    .expect( "LOUD FAILURE: Should create crypto service" );
  let encrypted = crypto_service.encrypt( &test_provider_key )
    .expect( "LOUD FAILURE: Should encrypt provider key" );

  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 )  // Unique provider key ID per test (e.g., agent 114 → key 114000)
  .bind( "openai" )
  .bind( encrypted.ciphertext_base64() )
  .bind( encrypted.nonce_base64() )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert usage_limits for test_user (required for budget validation)
  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( 10_000_000_000_i64 )  // $10,000 USD limit (in microdollars)
  .bind( 0i64 )                 // No spending yet
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();
}


// ============================================================================
// TEST 1: Handshake with Non-Existent Agent
// ============================================================================

/// TEST 1: Handshake with non-existent agent
///
/// # Corner Case
///
/// IC Token contains agent_id that doesnt exist in database
///
/// # Expected Behavior
///
/// - HTTP 403 "No budget allocated for agent"
/// - Prevents token forgery attacks (attacker cannot create IC Token for fake agent)
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Without this check, an attacker could forge IC Tokens
/// for non-existent agents and potentially access provider keys without budget tracking.
///
/// **Prevention**: Database lookup MUST occur before any budget operations.
#[ tokio::test ]
async fn test_handshake_with_nonexistent_agent()
{
  let pool = setup_test_db().await;

  // Create test user and provider key for testing (agent 999 won't exist)
  let now_ms = chrono::Utc::now().timestamp_millis();
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
  .execute( &pool )
  .await
  .unwrap();

  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( 999000i64 )
  .bind( "openai" )
  .bind( "encrypted_test_key_base64" )
  .bind( "test_nonce_base64" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_test_budget_state( pool.clone() ).await;

  // Create IC Token for agent that doesnt exist (agent_id = 999)
  let ic_token = create_ic_token( 999, &state.ic_token_manager );

  // Build handshake request
  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 999000
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 401 Unauthorized (generic error to prevent agent enumeration)
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Handshake with non-existent agent should return 401 Unauthorized (security: prevent enumeration)"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Invalid IC Token" ),
    "Error message should be generic to prevent agent enumeration: {}",
    body_str
  );
}

// ============================================================================
// TEST 2: Handshake with Zero Agent Budget
// ============================================================================

/// TEST 2: Handshake with zero agent budget
///
/// # Corner Case
///
/// Agent exists in database but budget.remaining_micros = 0
///
/// # Expected Behavior
///
/// - HTTP 403 "Insufficient budget"
/// - Enforces budget boundary at exactly $0.00
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Zero budget is a boundary condition that could bypass
/// enforcement if not explicitly checked.
///
/// **Prevention**: Budget check MUST use `<` not `<=` to reject zero budgets.
#[ tokio::test ]
async fn test_handshake_with_zero_agent_budget()
{
  let pool = setup_test_db().await;

  // Seed agent with exactly $0.00 budget
  seed_agent_with_budget( &pool, 114, 0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 114, &state.ic_token_manager );

  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 114000
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 403 Forbidden
  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "Handshake with zero budget should return 403 Forbidden"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Budget limit exceeded" ),
    "Error message should indicate budget limit exceeded: {}",
    body_str
  );
}

// ============================================================================
// TEST 3: Handshake with Insufficient Budget for Lease
// ============================================================================

/// TEST 3: Handshake with partial budget availability
///
/// # Corner Case
///
/// Agent budget = $5.00, default lease amount = $10.00
///
/// # Expected Behavior
///
/// - HTTP 200 OK with granted budget = $5.00
/// - Implementation grants min(default_lease, remaining_budget) to allow partial leases
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Validates that partial leases are granted when remaining budget
/// is less than default lease amount, enabling multiple concurrent leases.
///
/// **Prevention**: Ensures implementation uses min(default, remaining) formula correctly.
#[ tokio::test ]
async fn test_handshake_with_insufficient_budget_for_lease()
{
  let pool = setup_test_db().await;

  // Seed agent with $5.00 (less than $10.00 default lease)
  seed_agent_with_budget( &pool, 115, 5_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 115, &state.ic_token_manager );

  let request_body = json!(
  {
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 115000
  } );

  let app = Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK (partial lease granted)
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Handshake should succeed with partial lease when budget < default"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  // Verify partial lease granted (should be min($10, $5) = $5)
  let granted = response_json[ "budget_granted" ].as_i64().unwrap();
  assert_eq!(
    granted, 5_000_000,
    "Should grant partial lease of $5, got {} microdollars",
    granted
  );
}

// ============================================================================
// TEST 4: Report Usage with Non-Existent Lease
// ============================================================================

/// TEST 4: Report usage with non-existent lease
///
/// # Corner Case
///
/// Lease ID in usage report doesnt exist in database
///
/// # Expected Behavior
///
/// - HTTP 404 "Lease not found"
/// - Prevents phantom usage reports from affecting budgets
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Without this check, malicious clients could report usage
/// on fake leases to manipulate budget accounting.
///
/// **Prevention**: Lease lookup MUST occur before any usage accounting.
#[ tokio::test ]
async fn test_report_usage_with_nonexistent_lease()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool.clone() ).await;

  // Create usage report for non-existent lease
  let request_body = json!(
  {
    "lease_id": "lease_00000000-0000-0000-0000-000000000000",
    "request_id": "req_12345",
    "tokens": 1000,
    "cost_microdollars": 50_000,
    "model": "gpt-4",
    "provider": "openai"
  } );

  let app = Router::new()
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 404 Not Found
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Report usage with non-existent lease should return 404 Not Found"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Lease not found" ) || body_str.contains( "not found" ),
    "Error message should indicate lease not found: {}",
    body_str
  );
}

// ============================================================================
// TEST 5: Report Usage on Expired Lease
// ============================================================================

/// TEST 5: Report usage on expired lease
///
/// # Corner Case
///
/// Lease exists but expires_at < current_time
///
/// # Expected Behavior
///
/// - HTTP 403 "Lease expired"
/// - Enforces time boundary for lease validity
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Expired leases should not accept new usage reports
/// to prevent stale usage from affecting budgets.
///
/// **Prevention**: Expiry check MUST occur before usage accounting.
///
/// # Bug Reproducer
///
/// This test exposes a bug in the current implementation: `report_usage` endpoint
/// (budget.rs:575-656) does NOT check lease expiry before accepting usage reports.
/// The implementation only checks if lease exists (line 590-610), not if its expired.
#[ tokio::test ]
async fn test_report_usage_on_expired_lease()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 116, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Create lease that expired 1 hour ago
  let now_ms = chrono::Utc::now().timestamp_millis();
  let one_hour_ago = now_ms - ( 60 * 60 * 1000 );
  let lease_id = "lease_expired_test";

  state
    .lease_manager
    .create_lease( lease_id, 116, 116, 10_000_000, Some( one_hour_ago ) ) // $10
    .await
    .unwrap();

  // Try to report usage on expired lease
  let request_body = json!(
  {
    "lease_id": lease_id,
    "request_id": "req_12345",
    "tokens": 1000,
    "cost_microdollars": 50_000,
    "model": "gpt-4",
    "provider": "openai"
  } );

  let app = Router::new()
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 403 Forbidden (lease expired)
  // NOTE: This assertion will FAIL, exposing the bug
  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "BUG EXPOSED: Report usage on expired lease should return 403 Forbidden, \
     but implementation at budget.rs:590-610 doesnt check expiry"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "expired" ) || body_str.contains( "Lease expired" ),
    "Error message should indicate lease expired: {}",
    body_str
  );
}

//
// # Bug Fix Documentation: Missing Lease Expiry Check (issue-budget-001)
//
// ## 1. Root Cause
//
// The `report_usage` endpoint (budget.rs:575-656) was implemented without lease expiry validation.
// After successfully fetching the lease from the database (lines 590-610), the implementation
// immediately proceeded to record usage without checking if `lease.expires_at` timestamp had passed.
// This occurred because the initial implementation focused on database schema validation (does lease
// exist?) rather than business logic validation (is lease still valid for use?).
//
// ## 2. Why Not Caught
//
// No existing Protocol 005 tests covered expired lease scenarios. The test suite (budget_routes.rs,
// protocol_005_*.rs) focused on:
// - Request validation (empty fields, malformed data)
// - Token verification (IC Token, IP Token)
// - Happy path flows (handshake → report → refresh)
//
// But NO tests verified time-based enforcement boundaries. This gap existed because the original
// test implementation (26 tests) covered API contract validation but not database state corner cases.
//
// ## 3. Fix Applied
//
// Added lease expiry check at budget.rs:605-617 (immediately after lease fetch, before usage recording):
//
// ```rust
// // Check if lease has expired
// if let Some( expires_at ) = lease.expires_at
// {
//   let now_ms = chrono::Utc::now().timestamp_millis();
//   if expires_at < now_ms
//   {
//     return (
//       StatusCode::FORBIDDEN,
//       Json( serde_json::json!({ "error": "Lease expired" }) ),
//     )
//       .into_response();
//   }
// }
// ```
//
// This check runs BEFORE usage recording, ensuring expired leases cannot affect budgets.
//
// ## 4. Prevention
//
// **Test Coverage**: Create database state corner case tests for ALL time-bounded resources (leases,
// tokens, sessions). Test both "just expired" (t = expiry + 1ms) and "long expired" (t = expiry + 1 hour).
//
// **Code Review Checklist**: For all endpoint handlers, verify that ALL validation checks occur
// BEFORE side effects (database writes, state mutations). Separation of concerns: validate first,
// mutate second.
//
// **Specification Requirement**: Protocol specifications must explicitly enumerate ALL enforcement
// checks including temporal boundaries. Protocol 005 spec should list: request validation, IC Token
// verification, lease existence, lease expiry, lease budget sufficiency.
//
// ## 5. Pitfall
//
// **Time-Based Validation Gaps**: Implementing database fetch without subsequent state validation
// creates a gap between "resource exists" and "resource is valid for operation". This pitfall applies
// to ANY time-bounded resource (API tokens, sessions, leases, credentials).
//
// **Example Pattern**: If implementation checks `if lease.is_some()` but not `if lease.is_valid()`,
// temporal constraints are silently violated.
//
// **Detection**: Any endpoint accepting a resource ID (lease_id, token_id, session_id) must verify
// BOTH existence AND validity (expiry, revocation status, enabled flag) before usage.
//

// ============================================================================
// TEST 6: Report Usage Exceeding Lease Budget (CRITICAL)
// ============================================================================

/// TEST 6: Report usage exceeding lease budget
///
/// # Corner Case
///
/// Lease budget_granted = $1.00, budget_spent = $0.90, usage report cost = $0.50
/// Remaining = $0.10, but trying to spend $0.50
///
/// # Expected Behavior
///
/// - HTTP 403 "Insufficient lease budget"
/// - CRITICAL: Budget enforcement at lease level
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: This is the PRIMARY budget enforcement mechanism.
/// Without this check, agents could exceed lease budgets indefinitely.
///
/// **Prevention**: Lease budget check MUST occur BEFORE updating budget_spent.
///
/// # Bug Reproducer
///
/// This test exposes a CRITICAL bug: `report_usage` endpoint (budget.rs:612-624)
/// does NOT check if lease has sufficient remaining budget before recording usage.
/// It blindly calls `record_usage()` which just adds to budget_spent unconditionally.
#[ tokio::test ]
async fn test_report_usage_exceeding_lease_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 117, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Create lease with $1.00 budget
  let lease_id = "lease_budget_test";
  state
    .lease_manager
    .create_lease( lease_id, 117, 117, 1_000_000, None ) // $1.00
    .await
    .unwrap();

  // Record $0.90 usage (leaving $0.10 remaining)
  state.lease_manager.record_usage( lease_id, 900_000 ).await.unwrap(); // $0.90

  // Try to report $0.50 usage (exceeds remaining $0.10)
  let request_body = json!(
  {
    "lease_id": lease_id,
    "request_id": "req_12345",
    "tokens": 5000,
    "cost_microdollars": 500_000,
    "model": "gpt-4",
    "provider": "openai"
  } );

  let app = Router::new()
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 403 Forbidden (insufficient lease budget)
  // NOTE: This assertion will FAIL, exposing CRITICAL bug
  assert_eq!(
    response.status(),
    StatusCode::FORBIDDEN,
    "CRITICAL BUG EXPOSED: Report usage exceeding lease budget should return 403 Forbidden, \
     but implementation at budget.rs:612-624 doesnt check lease remaining budget"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Insufficient" ) || body_str.contains( "budget" ),
    "Error message should indicate insufficient budget: {}",
    body_str
  );
}

//
// # Bug Fix Documentation: Missing Lease Budget Check (issue-budget-002) - CRITICAL
//
// ## 1. Root Cause
//
// The `report_usage` endpoint (budget.rs:575-656) immediately recorded usage in the lease without
// verifying that the lease had sufficient remaining budget. After fetching the lease (lines 590-610)
// and checking expiry (lines 605-617), the code directly called `lease_manager.record_usage()` at
// line 631 without calculating `lease.budget_granted - lease.budget_spent` and comparing it to
// `request.cost_usd`.
//
// This is a CRITICAL security bug because it allows agents to exceed their allocated budgets. An agent
// with a $1.00 lease could report usage of $100.00 and the system would accept it, violating the core
// budget control guarantee of Protocol 005.
//
// ## 2. Why Not Caught
//
// No existing Protocol 005 tests verified budget enforcement at the lease level. The test suite
// (budget_routes.rs, protocol_005_*.rs) tested:
// - Request validation (negative costs, zero tokens)
// - Token verification (IC Token, IP Token)
// - Happy path success cases
//
// But NO tests covered boundary conditions where reported cost exceeds remaining lease budget. This
// gap existed because the original tests focused on API contract compliance (is the request valid?)
// rather than business logic enforcement (does the system honor budget constraints?).
//
// The oversight occurred because lease budget enforcement seems "obvious" but wasn't explicitly
// verified. This is a classic pitfall: critical enforcement logic that's assumed to work but never tested.
//
// ## 3. Fix Applied
//
// Added lease budget sufficiency check at budget.rs:619-628 (after expiry check, before usage recording):
//
// ```rust
// // Check if lease has sufficient remaining budget
// let lease_remaining = lease.budget_granted - lease.budget_spent;
// if lease_remaining < request.cost_usd
// {
//   return (
//     StatusCode::FORBIDDEN,
//     Json( serde_json::json!({ "error": "Insufficient lease budget" }) ),
//   )
//     .into_response();
// }
// ```
//
// This check enforces the fundamental budget control invariant: agents cannot spend more than allocated.
//
// ## 4. Prevention
//
// **Test Coverage**: For ALL resource-constrained operations (budget leases, token limits, rate limits),
// create tests that verify enforcement at boundaries:
// - Request exactly at limit (remaining = cost) → should succeed
// - Request 1 cent over limit (remaining < cost) → should fail with 403
// - Request far over limit (remaining << cost) → should fail with 403
//
// **Specification Requirement**: Protocol specifications must explicitly state ALL enforcement
// invariants as testable properties. Protocol 005 should state: "The system MUST reject usage reports
// where `cost_usd > (lease.budget_granted - lease.budget_spent)`".
//
// **Code Review Pattern**: Any operation that consumes a limited resource (budget, tokens, quota)
// MUST verify sufficiency BEFORE consumption. Look for this pattern in code reviews:
// ```rust
// // ❌ BAD: Consume first, check later
// resource.consume(amount)?;
// if resource.remaining < 0 { /* too late */ }
//
// // ✅ GOOD: Check first, consume only if sufficient
// if resource.remaining < amount { return Err(...); }
// resource.consume(amount)?;
// ```
//
// ## 5. Pitfall
//
// **Assumed Enforcement**: Never assume that "obvious" business rules are automatically enforced.
// Budget limits, rate limits, quota constraints, and permission checks MUST be explicitly coded
// AND explicitly tested. If there's no test proving enforcement works, assume it doesn't work.
//
// **Resource Consumption Pattern**: Any code path that modifies a limited resource (budgets, quotas,
// tokens) without a sufficiency check is a potential vulnerability. This pattern appears in:
// - Budget systems (this bug)
// - Rate limiters (allow request before checking remaining quota)
// - Token allocators (mint tokens before checking pool capacity)
// - Permission systems (perform action before checking authorization)
//
// **Detection Strategy**: Search codebase for resource modification operations (`.record_usage()`,
// `.consume()`, `.allocate()`, `.spend()`) and verify that each has a corresponding sufficiency
// check immediately before it. If the check is missing or occurs after modification, it's a bug.
//
// **Historical Context**: This bug class (check-after-modify vs check-before-modify) is analogous to
// TOCTOU (time-of-check-time-of-use) vulnerabilities in security. The fix pattern is identical:
// perform ALL checks BEFORE ANY modifications.
//

// ============================================================================
// TEST 7: Refresh with Insufficient Agent Budget
// ============================================================================

/// TEST 7: Refresh with insufficient agent budget
///
/// # Corner Case
///
/// Agent budget remaining = $5.00, refresh request amount = $10.00
///
/// # Expected Behavior
///
/// - HTTP 200 OK with status="denied" and reason="insufficient_budget"
/// - Prevents lease refresh when agent cannot afford it
///
/// # Root Cause Prevention
///
/// **Why This Test Exists**: Refresh operations must check agent budget
/// availability before granting additional lease budget.
///
/// **Prevention**: Agent budget check MUST occur before lease refresh.
///
/// # Implementation Note
///
/// Based on budget.rs:737-748, the refresh endpoint DOES check agent budget
/// and returns status="denied" (HTTP 200 with denial) when insufficient.
/// This test verifies the check works correctly.
#[ tokio::test ]
async fn test_refresh_with_insufficient_agent_budget()
{
  let pool = setup_test_db().await;

  // Seed agent with $5.00 (insufficient for $10.00 default refresh)
  seed_agent_with_budget( &pool, 118, 5_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Generate IC Token for agent 118
  let ic_token = create_ic_token( 118, &state.ic_token_manager );

  // Create active lease
  let lease_id = "lease_refresh_test";
  state
    .lease_manager
    .create_lease( lease_id, 118, 118, 10_000_000, None ) // $10
    .await
    .unwrap();

  // Try to refresh with $10 (exceeds remaining $5)
  let request_body = json!(
  {
    "ic_token": ic_token,
    "current_lease_id": lease_id,
    "requested_budget": 10_000_000  // $10
  } );

  // Create valid JWT token for authorization (GAP-003: refresh endpoint requires JWT)
  let access_token = common::create_test_access_token(
    "test_user",
    "test@example.com",
    "admin",
    "test_jwt_secret"
  );

  let app = Router::new()
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", access_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK (refresh endpoint returns 200 even for denials)
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Refresh request should return 200 OK"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  // Parse JSON response
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect("LOUD FAILURE: Response should be valid JSON");

  // Fix(issue-budget-008): Refresh now reserves budget via check_and_reserve_budget
  //
  // Root cause: Test expected denial when budget < requested, but check_and_reserve_budget
  // supports partial grants. Agent has $5, requests $10, receives $5 granted.
  //
  // Pitfall: When fixing budget accounting bugs, check ALL tests that validate budget
  // enforcement behavior. Partial grant support changes "insufficient budget" semantics
  // from "deny" to "approve with available amount".

  // Assert: Status is "approved" (partial grant)
  assert_eq!(
    response_json[ "status" ].as_str().unwrap(),
    "approved",
    "Refresh with partial budget should return status='approved' (partial grant)"
  );

  // Assert: budget_granted is $5 (partial grant, not full $10)
  assert_eq!(
    response_json[ "budget_granted" ].as_i64().unwrap(),
    5_000_000,
    "Partial grant should provide available budget ($5, not requested $10)"
  );

  // Assert: Reason is None (approved, not denied)
  assert!(
    response_json[ "reason" ].is_null(),
    "Approved refresh should have no denial reason"
  );
}
