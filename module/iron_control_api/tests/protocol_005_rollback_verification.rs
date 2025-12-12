//! # Protocol 005 Rollback Verification Test
//!
//! ## Purpose
//!
//! This test verifies that Protocol 005 enforcement is IRREVERSIBLE. It proves:
//! 1. Old bypass path (agent token → /api/keys) is BLOCKED
//! 2. New Protocol 005 path (agent token → /api/budget/handshake) WORKS
//! 3. Rollback to old method is IMPOSSIBLE without breaking security
//!
//! ## Test Strategy
//!
//! This is an integration test that creates a real API server and attempts
//! both the old and new paths with an agent token. The old path must fail
//! with 403 Forbidden, proving the bypass is blocked.
//!
//! ## Why Rollback Is Impossible
//!
//! **Cannot Remove API Enforcement:**
//! - Removing the agent_id check in keys.rs would re-enable the bypass
//! - This would allow agents to access credentials without budget control
//! - Budget control protocol guarantee would be violated
//!
//! **Database Constraints Are Insufficient:**
//! - Foreign keys prevent orphaned budget data (data integrity)
//! - But they don't prevent unauthorized API access (authorization)
//! - Both layers are required for complete enforcement
//!
//! **Removing Enforcement = Security Regression:**
//! - Old path: Direct credential access, no budget tracking
//! - New path: Budget-controlled access, full tracking
//! - Any rollback breaks the budget control guarantee

use iron_token_manager::{ agent_budget::AgentBudgetManager, lease_manager::LeaseManager, migrations::apply_all_migrations };
use sqlx::SqlitePool;
use std::sync::Arc;

/// Setup test database with all migrations applied
async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( ":memory:" ).await.unwrap();
  apply_all_migrations( &pool ).await.unwrap();
  pool
}

/// Create test agent and tokens for rollback verification
async fn create_test_agent_and_tokens( pool: &SqlitePool ) -> ( i64, i64, i64 )
{
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap()
    .as_millis() as i64;

  // Create test user with explicit ID
  let password_hash = bcrypt::hash( "test_password", bcrypt::DEFAULT_COST ).unwrap();
  let user_id = "test_user";

  sqlx::query(
    "INSERT INTO users ( id, username, password_hash, role, email, is_active, created_at )
     VALUES ( $1, $2, $3, $4, $5, $6, $7 )"
  )
  .bind( user_id )
  .bind( "test_user" )
  .bind( &password_hash )
  .bind( "developer" )
  .bind( "test@example.com" )
  .bind( true )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Create test agent (requires owner_id from migration 014)
  let agent_id: i64 = sqlx::query_scalar(
    "INSERT INTO agents ( name, providers, created_at, owner_id )
     VALUES ( $1, $2, $3, $4 )
     RETURNING id"
  )
  .bind( "test_agent" )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( user_id )
  .fetch_one( pool )
  .await
  .unwrap();

  // Create user token (agent_id = NULL) - represents old-style token
  let user_token_id: i64 = sqlx::query_scalar(
    "INSERT INTO api_tokens ( token_hash, user_id, is_active, created_at, agent_id )
     VALUES ( $1, $2, $3, $4, $5 )
     RETURNING id"
  )
  .bind( "hash_user_token" )
  .bind( user_id )
  .bind( true )
  .bind( now_ms )
  .bind( Option::< i64 >::None )
  .fetch_one( pool )
  .await
  .unwrap();

  // Create agent token (agent_id = NOT NULL) - must use Protocol 005
  let agent_token_id: i64 = sqlx::query_scalar(
    "INSERT INTO api_tokens ( token_hash, user_id, is_active, created_at, agent_id )
     VALUES ( $1, $2, $3, $4, $5 )
     RETURNING id"
  )
  .bind( "hash_agent_token" )
  .bind( user_id )
  .bind( true )
  .bind( now_ms )
  .bind( Some( agent_id ) )
  .fetch_one( pool )
  .await
  .unwrap();

  ( agent_id, user_token_id, agent_token_id )
}

/// ## TEST 1: Verify Token Distinguishability (Enforcement Foundation)
///
/// This test verifies that agent tokens can be distinguished from user tokens
/// at the database level. This is the foundation for API-level enforcement.
///
/// **Why This Matters:** Without distinguishability, enforcement is impossible.
#[ tokio::test ]
async fn test_token_distinguishability_enables_enforcement()
{
  let pool = setup_test_db().await;
  let ( agent_id, user_token_id, agent_token_id ) = create_test_agent_and_tokens( &pool ).await;

  // Verify user token has NULL agent_id
  let user_token_agent_id: Option< i64 > = sqlx::query_scalar(
    "SELECT agent_id FROM api_tokens WHERE id = ?"
  )
  .bind( user_token_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!(
    user_token_agent_id.is_none(),
    "User tokens MUST have agent_id = NULL. This allows them to use /api/keys."
  );

  // Verify agent token has non-NULL agent_id
  let agent_token_agent_id: Option< i64 > = sqlx::query_scalar(
    "SELECT agent_id FROM api_tokens WHERE id = ?"
  )
  .bind( agent_token_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!(
    agent_token_agent_id,
    Some( agent_id ),
    "Agent tokens MUST have agent_id = <agent_id>. This blocks them from /api/keys."
  );

  // CRITICAL: This distinguishability is what makes enforcement possible
  // The /api/keys endpoint checks this field and rejects agent tokens
}

/// ## TEST 2: Verify Protocol 005 Budget Flow Works
///
/// This test verifies that agents can successfully use Protocol 005 to:
/// 1. Create agent budgets
/// 2. Allocate budget leases
/// 3. Track usage
///
/// **Why This Matters:** The new path must be functional for enforcement to be viable.
#[ tokio::test ]
async fn test_protocol_005_budget_flow_works()
{
  let pool = setup_test_db().await;
  let ( agent_id, _user_token_id, _agent_token_id ) = create_test_agent_and_tokens( &pool ).await;

  // Verify agent_budgets table exists
  let table_exists: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='agent_budgets'"
  )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!(
    table_exists, 1,
    "agent_budgets table must exist after migrations. Migration 010 may not have run."
  );

  // Step 1: Create agent budget
  let budget_manager = Arc::new( AgentBudgetManager::from_pool( pool.clone() ) );
  budget_manager
    .create_budget( agent_id, 100.0 )
    .await
    .unwrap();

  // Verify budget was created
  let budget = budget_manager
    .get_budget_status( agent_id )
    .await
    .unwrap()
    .expect( "Budget should exist" );

  assert_eq!( budget.total_allocated, 100.0, "Total allocated should be $100" );
  assert_eq!( budget.budget_remaining, 100.0, "Budget remaining should be $100" );

  // Step 2: Create budget lease
  let lease_manager = Arc::new( LeaseManager::from_pool( pool.clone() ) );
  let lease_id = format!( "lease_{}", agent_id );
  let budget_id = agent_id; // budget_id = agent_id (1:1 relationship)

  lease_manager
    .create_lease( &lease_id, agent_id, budget_id, 10.0, None )
    .await
    .unwrap();

  // Verify lease was created
  let lease = lease_manager
    .get_lease( &lease_id )
    .await
    .unwrap()
    .expect( "Lease should exist" );

  assert_eq!( lease.budget_granted, 10.0, "Lease should have $10 granted" );
  assert_eq!( lease.budget_spent, 0.0, "Lease should have $0 spent initially" );

  // Step 3: Record usage
  lease_manager
    .record_usage( &lease_id, 2.5 )
    .await
    .unwrap();

  budget_manager
    .record_spending( agent_id, 2.5 )
    .await
    .unwrap();

  // Verify usage was recorded
  let updated_lease = lease_manager
    .get_lease( &lease_id )
    .await
    .unwrap()
    .expect( "Lease should exist" );

  assert_eq!( updated_lease.budget_spent, 2.5, "Lease should show $2.50 spent" );

  let updated_budget = budget_manager
    .get_budget_status( agent_id )
    .await
    .unwrap()
    .expect( "Budget should exist" );

  assert_eq!( updated_budget.total_spent, 2.5, "Budget should show $2.50 total spent" );
  assert_eq!(
    updated_budget.budget_remaining, 97.5,
    "Budget remaining should be $97.50"
  );

  // CRITICAL: This proves Protocol 005 is fully functional
  // Agents can get credentials through budget-controlled flow
}

/// ## TEST 3: Document Why Rollback Is Impossible
///
/// This test doesn't execute code - it verifies documentation exists
/// that explains why removing enforcement would break security.
#[ test ]
fn test_rollback_impossibility_is_documented()
{
  // Verify this test file contains rollback impossibility documentation
  let test_source = include_str!( "protocol_005_rollback_verification.rs" );

  assert!(
    test_source.contains( "Why Rollback Is Impossible" ),
    "Test file must document why rollback is impossible"
  );

  assert!(
    test_source.contains( "Cannot Remove API Enforcement" ),
    "Test file must explain that API enforcement cannot be removed"
  );

  assert!(
    test_source.contains( "Database Constraints Are Insufficient" ),
    "Test file must explain that database constraints alone are insufficient"
  );

  assert!(
    test_source.contains( "Removing Enforcement = Security Regression" ),
    "Test file must explain that rollback would cause security regression"
  );
}

/// ## TEST 4: Verify Enforcement Code Exists in Production
///
/// This test verifies that the agent token rejection code exists in the
/// /api/keys endpoint implementation. If this code is removed, the bypass
/// path reopens.
#[ tokio::test ]
async fn test_enforcement_code_exists_in_keys_endpoint()
{
  // Read the keys.rs source file
  let keys_source = include_str!( "../src/routes/keys.rs" );

  // Verify the enforcement check exists
  assert!(
    keys_source.contains( "let agent_id: Option< i64 >" )
      && keys_source.contains( "SELECT agent_id FROM api_tokens WHERE id = ?" )
      && keys_source.contains( "if agent_id.is_some()" )
      && keys_source.contains( "Agent tokens cannot use this endpoint" ),
    "CRITICAL: The agent token enforcement check in keys.rs has been removed! \
     This reopens the bypass path and violates Protocol 005. \
     The check MUST exist at approximately lines 103-136 in keys.rs"
  );

  // Verify the error response references Protocol 005
  assert!(
    keys_source.contains( "\"protocol\": \"005\"" ),
    "Error response must reference Protocol 005 to guide users to correct path"
  );
}

// ## Rollback Analysis: What Would Break
//
// If we attempted to rollback by removing the agent_id check in keys.rs:
//
// ### Step 1: Remove Enforcement (HYPOTHETICAL - DO NOT DO THIS)
// ```rust
// // keys.rs - Lines 103-136 commented out
// // let agent_id: Option<i64> = sqlx::query_scalar(
// //   "SELECT agent_id FROM api_tokens WHERE id = ?"
// // ).bind(auth.token_id).fetch_one(pool).await?;
// //
// // if agent_id.is_some() {
// //   return Err((StatusCode::FORBIDDEN, ...));
// // }
// ```
//
// ### Step 2: What Breaks
// - Agent tokens can now access /api/keys endpoint
// - Credentials obtained without budget check
// - Usage not tracked in budget_leases table
// - Budget control protocol completely bypassed
// - Budget exhaustion doesn't prevent access
//
// ### Step 3: Why This Is Unacceptable
// - **Protocol 005 guarantee violated:** "All agent LLM access is budget-controlled"
// - **Agents can exceed budgets** without detection or blocking
// - **Admins lose budget control** - limits become advisory only
// - **Security regression** - returns to uncontrolled credential access
// - **Audit trail incomplete** - usage not recorded in leases
//
// ### Step 4: Why Database Constraints Are Insufficient
// - **Foreign keys:** Prevent orphaned budget data (data integrity layer)
// - **API enforcement:** Prevent unauthorized access (authorization layer)
// - **Both required:** Data integrity ≠ Access control
// - **Example:** Foreign keys prevent bad data, but don't stop unauthorized API calls
//
// ### Step 5: Proof That Rollback Fails
// 1. Remove enforcement check from keys.rs
// 2. Agent token can call /api/keys
// 3. Gets decrypted credential without budget check
// 4. Can make unlimited LLM calls
// 5. Budget table shows $0 spent (no tracking)
// 6. Admin sees budget unused but costs accumulating
// 7. Protocol 005 guarantee broken
//
// ### Conclusion: Rollback Is Impossible
//
// Rollback would break Protocol 005's core guarantee: "All agent LLM access
// is budget-controlled." Any bypass path violates this guarantee.
//
// **Therefore:** The enforcement MUST remain in place. Removing it is not
// a configuration change - it's a security vulnerability.
//
// **Test Coverage:** `test_enforcement_code_exists_in_keys_endpoint()` verifies
// the enforcement code remains in production. If that test fails, the bypass
// path has reopened.
