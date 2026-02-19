//! Budget Approval Approver Context Tracking Test
//!
//! # Test Context
//!
//! **Feature:** Protocol 012 - Budget Request Workflow
//! **Component:** Approver identity tracking from JWT authentication
//! **File:** `routes/budget/request_workflow.rs:652`
//!
//! ## Feature Overview
//!
//! Budget approval workflow must record WHO approved each budget change for audit trail
//! integrity. The approver identity comes from the authenticated user's JWT claims (sub field).
//!
//! ## Current Implementation Status
//!
//! The approve_budget_request endpoint currently uses a hardcoded placeholder:
//!
//! ```rust
//! // request_workflow.rs:652-653
//! // TODO: Get approver_id from authenticated user context instead of using placeholder
//! let approver_id = "system-admin";
//! ```
//!
//! This test verifies that real approver identity is extracted from JWT and stored in audit trail.
//!
//! ## Test Strategy
//!
//! This test follows the TDD RED-GREEN-REFACTOR cycle:
//!
//! 1. **RED:** Test fails because approver_id is hardcoded placeholder "system-admin"
//! 2. **GREEN:** Implement JWT user_id extraction - test passes
//! 3. **REFACTOR:** Ensure auth middleware populates user context correctly
//!
//! ## Test Requirements
//!
//! - ✅ Create budget request in pending status
//! - ✅ Authenticate as specific user (JWT with user_id="test_admin_user_123")
//! - ✅ Call approve endpoint with JWT
//! - ✅ Verify approver_id in database matches JWT user_id (not "system-admin")

use axum::{ body::Body, http::{ Request, StatusCode } };
use serde_json::Value;
use tower::ServiceExt;

mod common;

/// Test: Approve endpoint extracts approver_id from JWT claims
///
/// **Test Flow:**
/// 1. Create pending budget request in database
/// 2. Create JWT token for user "test_admin_user_123"
/// 3. Call PATCH /api/v1/budget/requests/:id/approve with JWT auth
/// 4. Verify response is 200 OK
/// 5. Query budget_modification_history table
/// 6. Verify approver_id = "test_admin_user_123" (NOT "system-admin" placeholder)
///
/// **Expected Behavior:**
/// - Approver ID should match JWT user_id claim
/// - Audit trail must record actual approver, not placeholder
#[tokio::test]
async fn test_approve_budget_request_tracks_real_approver()
{
  // Setup: Create test database and state
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  // Setup: Create agent with budget
  let agent_id = 300i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  // Setup: Create pending budget request
  let request_id = "req_test_approver_001";
  let requester_id = "user_requester_456";
  let requested_budget_cents = 500_000i64; // $5,000 in cents
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT INTO budget_change_requests (id, agent_id, requester_id, current_budget_micros, requested_budget_micros, justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?)"
  )
  .bind( request_id )
  .bind( agent_id )
  .bind( requester_id )
  .bind( 100_000_000i64 ) // current_budget_micros (current budget $100 USD in microdollars)
  .bind( requested_budget_cents * 10_000 ) // requested_budget_micros ($5,000 in cents → microdollars)
  .bind( "Test budget request for approver tracking validation test case" ) // 20+ chars
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .expect( "LOUD FAILURE: Should insert pending budget request" );

  // Setup: Create JWT for approving user
  let approving_user_id = "test_admin_user_123";
  let approving_user_email = "admin@test.com";
  let access_token = common::create_test_access_token(
    approving_user_id,
    approving_user_email,
    "admin",
    "test_jwt_secret" // Must match create_test_budget_state jwt_secret
  );

  // Execute: Call approve endpoint with JWT authentication
  let app = common::budget::create_budget_router( state.clone() ).await;

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .header( "authorization", format!( "Bearer {}", access_token ) )
    .header( "content-type", "application/json" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Verify: Approval should succeed
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Approval should succeed with valid JWT authentication"
  );

  // Verify: Parse response body
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body : Value = serde_json::from_slice( &body_bytes )
    .expect( "LOUD FAILURE: Response should be valid JSON" );

  assert_eq!(
    body[ "status" ].as_str(),
    Some( "approved" ),
    "Budget request should be approved"
  );

  // Verify: Query budget_modification_history for modifier_id (approver)
  let history_record : ( String, ) = sqlx::query_as(
    "SELECT modifier_id FROM budget_modification_history
     WHERE agent_id = ? AND modification_type = 'increase'
     ORDER BY created_at DESC LIMIT 1"
  )
  .bind( agent_id )
  .fetch_one( &pool )
  .await
  .expect( "LOUD FAILURE: Should find budget modification history record" );

  let stored_modifier_id = history_record.0;

  // CRITICAL ASSERTION: Modifier ID must match JWT user_id, NOT hardcoded placeholder
  assert_eq!(
    stored_modifier_id,
    approving_user_id,
    "Modifier ID (approver) in audit trail should match JWT user_id (not hardcoded placeholder)"
  );

  assert_ne!(
    stored_modifier_id,
    "system-admin",
    "Modifier ID should NOT be the hardcoded placeholder 'system-admin'"
  );
}

/// Test: Approve endpoint requires valid JWT authentication
///
/// **Test Flow:**
/// 1. Create pending budget request
/// 2. Call approve endpoint WITHOUT JWT
/// 3. Verify response is 401 Unauthorized
///
/// **Expected Behavior:**
/// - Cannot approve without authentication
#[tokio::test]
async fn test_approve_budget_request_requires_authentication()
{
  let pool = common::budget::setup_test_db().await;
  let state = common::budget::create_test_budget_state( pool.clone() ).await;

  let agent_id = 301i64;
  common::budget::seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  // Create pending budget request
  let request_id = "req_test_auth_required_001";
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT INTO budget_change_requests (id, agent_id, requester_id, current_budget_micros, requested_budget_micros, justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, 'pending', ?, ?)"
  )
  .bind( request_id )
  .bind( agent_id )
  .bind( "user_requester" )
  .bind( 100_000_000i64 ) // current budget micros
  .bind( 1_000_000i64 ) // requested $100 in micros
  .bind( "Test request for authentication validation use case" ) // 20+ chars
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  // Call approve WITHOUT authentication
  let app = common::budget::create_budget_router( state ).await;

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .header( "content-type", "application/json" )
    // NO Authorization header
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Verify: Should reject with 401 Unauthorized
  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Approve endpoint should require JWT authentication"
  );
}
