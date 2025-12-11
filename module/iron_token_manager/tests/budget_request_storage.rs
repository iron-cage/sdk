//! Budget Request Storage Tests (Protocol 012 & 017)

mod common;
use common::{ create_test_db_v2, seed_test_agent };
use iron_token_manager::budget_request::*;

#[ tokio::test ]
async fn test_create_and_get()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 200_000_000, justification: "Need more budget for testing".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  create_budget_request( db.pool(), &req ).await.expect( "Should create request" );
  let fetched = get_budget_request( db.pool(), "breq_1" ).await.expect( "Should fetch" ).expect( "Should exist" );
  assert_eq!( fetched.id, "breq_1" );
  assert_eq!( fetched.agent_id, 1 );
}

#[ tokio::test ]
async fn test_get_nonexistent()
{
  let db = create_test_db_v2().await;
  let result = get_budget_request( db.pool(), "nonexistent" ).await.expect( "Should return Ok" );
  assert!( result.is_none(), "Nonexistent request should return None" );
}

#[ tokio::test ]
async fn test_list_by_status()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req1 = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 150_000_000, justification: "Testing pending status listing".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  let req2 = BudgetChangeRequest { id: "breq_2".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 150_000_000, requested_budget_micros: 200_000_000, justification: "Another pending request test".into(), status: RequestStatus::Pending, created_at: 2000, updated_at: 2000 };
  create_budget_request( db.pool(), &req1 ).await.expect( "Should create req1" );
  create_budget_request( db.pool(), &req2 ).await.expect( "Should create req2" );
  let pending = list_budget_requests_by_status( db.pool(), RequestStatus::Pending ).await.expect( "Should list" );
  assert_eq!( pending.len(), 2, "Should have 2 pending requests" );
}

#[ tokio::test ]
async fn test_list_by_agent()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  seed_test_agent( db.pool(), 2 ).await;
  let req1 = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 150_000_000, justification: "Request for agent 1 only".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  let req2 = BudgetChangeRequest { id: "breq_2".into(), agent_id: 2, requester_id: "user-1".into(), current_budget_micros: 200_000_000, requested_budget_micros: 250_000_000, justification: "Request for agent 2 only".into(), status: RequestStatus::Pending, created_at: 2000, updated_at: 2000 };
  create_budget_request( db.pool(), &req1 ).await.expect( "Should create req1" );
  create_budget_request( db.pool(), &req2 ).await.expect( "Should create req2" );
  let agent1_reqs = list_budget_requests_by_agent( db.pool(), 1 ).await.expect( "Should list" );
  assert_eq!( agent1_reqs.len(), 1, "Agent 1 should have 1 request" );
  assert_eq!( agent1_reqs[ 0 ].id, "breq_1" );
}

#[ tokio::test ]
async fn test_update_status()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 150_000_000, justification: "Testing status update".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  create_budget_request( db.pool(), &req ).await.expect( "Should create" );
  let affected = update_budget_request_status( db.pool(), "breq_1", RequestStatus::Approved, 2000 ).await.expect( "Should update" );
  assert_eq!( affected, 1, "Should update 1 row" );
  let updated = get_budget_request( db.pool(), "breq_1" ).await.expect( "Should fetch" ).expect( "Should exist" );
  assert_eq!( updated.status, RequestStatus::Approved );
}

#[ tokio::test ]
async fn test_approve_and_reject()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req1 = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 150_000_000, justification: "Testing approve function".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  let req2 = BudgetChangeRequest { id: "breq_2".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 50_000_000, justification: "Testing reject function".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  create_budget_request( db.pool(), &req1 ).await.expect( "Should create req1" );
  create_budget_request( db.pool(), &req2 ).await.expect( "Should create req2" );
  approve_budget_request( db.pool(), "breq_1", "admin-1", 2000 ).await.expect( "Should approve" );
  reject_budget_request( db.pool(), "breq_2", 2000 ).await.expect( "Should reject" );
  let approved = get_budget_request( db.pool(), "breq_1" ).await.expect( "Should fetch" ).expect( "Should exist" );
  let rejected = get_budget_request( db.pool(), "breq_2" ).await.expect( "Should fetch" ).expect( "Should exist" );
  assert_eq!( approved.status, RequestStatus::Approved );
  assert_eq!( rejected.status, RequestStatus::Rejected );
}

#[ tokio::test ]
async fn test_record_and_get_history()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 200_000_000, justification: "Testing budget history with related request".into(), status: RequestStatus::Approved, created_at: 1000, updated_at: 2000 };
  create_budget_request( db.pool(), &req ).await.expect( "Should create request" );
  let hist = BudgetModificationHistory { id: "hist_1".into(), agent_id: 1, modification_type: ModificationType::Increase, old_budget_micros: 100_000_000, new_budget_micros: 200_000_000, change_amount_micros: 100_000_000, modifier_id: "admin-1".into(), reason: "Approved budget increase request".into(), related_request_id: Some( "breq_1".into() ), created_at: 3000 };
  record_budget_modification( db.pool(), &hist ).await.expect( "Should record" );
  let history = get_budget_history( db.pool(), 1 ).await.expect( "Should fetch history" );
  assert_eq!( history.len(), 1, "Should have 1 history entry" );
  assert_eq!( history[ 0 ].id, "hist_1" );
  assert_eq!( history[ 0 ].modification_type, ModificationType::Increase );
}

#[ tokio::test ]
async fn test_justification_constraint()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let req = BudgetChangeRequest { id: "breq_1".into(), agent_id: 1, requester_id: "user-1".into(), current_budget_micros: 100_000_000, requested_budget_micros: 150_000_000, justification: "Short".into(), status: RequestStatus::Pending, created_at: 1000, updated_at: 1000 };
  let result = create_budget_request( db.pool(), &req ).await;
  assert!( result.is_err(), "Should reject short justification" );
}

#[ tokio::test ]
async fn test_reason_constraint()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;
  let hist = BudgetModificationHistory { id: "hist_1".into(), agent_id: 1, modification_type: ModificationType::Increase, old_budget_micros: 100_000_000, new_budget_micros: 200_000_000, change_amount_micros: 100_000_000, modifier_id: "admin-1".into(), reason: "Short".into(), related_request_id: None, created_at: 3000 };
  let result = record_budget_modification( db.pool(), &hist ).await;
  assert!( result.is_err(), "Should reject short reason" );
}

/// ## Root Cause
///
/// The `reject_budget_request` function lacked optimistic locking (WHERE status='pending' clause).
/// This allowed concurrent operations to both succeed, potentially causing inconsistent state where
/// a request could be rejected after being approved, or vice versa.
///
/// ## Why Not Caught
///
/// No concurrent operation tests existed. The test `test_approve_and_reject` only tested
/// sequential operations on different requests. The API layer has status validation but that
/// doesnt prevent storage layer TOCTTOU (Time-Of-Check-Time-Of-Use) race conditions.
///
/// ## Fix Applied
///
/// Added optimistic locking to `reject_budget_request` by checking `rows_affected` after UPDATE.
/// The function now returns `RowNotFound` error if the request status was already changed by
/// another concurrent operation, matching the behavior of `approve_budget_request`.
///
/// ## Prevention
///
/// All status transition functions should use optimistic locking with WHERE clauses on the
/// expected current status. Add concurrent operation tests for all critical state transitions.
///
/// ## Pitfall
///
/// API-layer status validation alone is insufficient for preventing race conditions. Database-level
/// optimistic locking (via WHERE clauses and `rows_affected` checks) is required to ensure atomicity
/// of check-then-modify operations in concurrent environments.
#[ tokio::test ]
async fn test_reject_concurrent_race_condition()
{
  let db = create_test_db_v2().await;
  seed_test_agent( db.pool(), 1 ).await;

  // Create pending request
  let req = BudgetChangeRequest
  {
    id: "breq_concurrent_test".into(),
    agent_id: 1,
    requester_id: "user-1".into(),
    current_budget_micros: 100_000_000,
    requested_budget_micros: 200_000_000,
    justification: "Testing concurrent reject race condition".into(),
    status: RequestStatus::Pending,
    created_at: 1000,
    updated_at: 1000,
  };
  create_budget_request( db.pool(), &req ).await.expect( "Should create request" );

  // Spawn two concurrent reject operations
  let pool1 = db.pool().clone();
  let pool2 = db.pool().clone();

  let task1 = tokio::spawn( async move
  {
    reject_budget_request( &pool1, "breq_concurrent_test", 2000 ).await
  });

  let task2 = tokio::spawn( async move
  {
    reject_budget_request( &pool2, "breq_concurrent_test", 2001 ).await
  });

  let result1 = task1.await.expect( "Task should complete" );
  let result2 = task2.await.expect( "Task should complete" );

  // One should succeed, one should fail with RowNotFound (optimistic lock failure)
  let success_count = [ result1.is_ok(), result2.is_ok() ].iter().filter( |&&x| x ).count();
  let failure_count = [ result1.is_err(), result2.is_err() ].iter().filter( |&&x| x ).count();

  assert_eq!( success_count, 1, "Exactly one reject operation should succeed" );
  assert_eq!( failure_count, 1, "Exactly one reject operation should fail due to optimistic lock" );

  // Verify the failed operation returned RowNotFound
  let failed_result = if result1.is_err() { result1 } else { result2 };
  match failed_result
  {
    Err( sqlx::Error::RowNotFound ) => {}, // Expected
    other => panic!( "Expected RowNotFound error, got: {other:?}" ),
  }

  // Verify final status is rejected
  let final_req = get_budget_request( db.pool(), "breq_concurrent_test" )
    .await
    .expect( "Should fetch" )
    .expect( "Should exist" );
  assert_eq!( final_req.status, RequestStatus::Rejected );
}
