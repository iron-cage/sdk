//! Tests for budget request approval workflow
//!
//! Verifies that approving a budget request:
//! 1. Updates request status to approved
//! 2. Updates agent budget to requested amount
//! 3. Records change in `budget_modification_history`
//! 4. Uses optimistic locking (fails if concurrent approval)

#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::too_many_lines ) ]

use sqlx::{ SqlitePool, Row };

async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect( "Failed to apply migrations" );
  pool
}

#[ tokio::test ]
async fn test_approve_budget_request_applies_budget_change()
{
  let pool = setup_test_db().await;

  // Create test agent
  sqlx::query( "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( 1 )
    .bind( "test-agent" )
    .bind( "[]" )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( &pool )
    .await
    .expect( "Failed to create agent" );

  // Create agent budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( 1 )
  .bind( 100.0 )
  .bind( 0.0 )
  .bind( 100.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() )
  .execute( &pool )
  .await
  .expect( "Failed to create agent budget" );

  // Create budget request
  let request_id = "breq_test_apply";
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )  // $100 current
  .bind( 250_000_000 )  // $250 requested (+$150 increase)
  .bind( "Need more budget for expanded testing" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .expect( "Failed to create request" );

  // Approve the request
  let approve_time = chrono::Utc::now().timestamp_millis();
  let result = iron_token_manager::budget_request::approve_budget_request(
    &pool,
    request_id,
    "admin-approver",
    approve_time,
  )
  .await;

  assert!( result.is_ok(), "Approval should succeed: {:?}", result.err() );

  // Verify 1: Request status changed to approved
  let request_status: String = sqlx::query( "SELECT status FROM budget_change_requests WHERE id = ?" )
    .bind( request_id )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch request" )
    .get( "status" );

  assert_eq!(
    request_status, "approved",
    "Request status should be approved"
  );

  // Verify 2: Agent budget updated to requested amount ($250)
  let agent_budget: f64 = sqlx::query( "SELECT total_allocated FROM agent_budgets WHERE agent_id = ?" )
    .bind( 1 )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch agent budget" )
    .get( "total_allocated" );

  assert_eq!(
    agent_budget, 250.0,
    "Agent budget should be updated to requested amount"
  );

  let budget_remaining: f64 = sqlx::query( "SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?" )
    .bind( 1 )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch budget remaining" )
    .get( "budget_remaining" );

  assert_eq!(
    budget_remaining, 250.0,
    "Budget remaining should also be updated"
  );

  // Verify 3: History record created
  let history_count: i64 = sqlx::query( "SELECT COUNT(*) as count FROM budget_modification_history WHERE agent_id = ?" )
    .bind( 1 )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch history count" )
    .get( "count" );

  assert_eq!(
    history_count, 1,
    "Should have one history record"
  );

  // Verify history details
  let history_row = sqlx::query(
    "SELECT modification_type, old_budget_micros, new_budget_micros, change_amount_micros, modifier_id, related_request_id
     FROM budget_modification_history WHERE agent_id = ?"
  )
  .bind( 1 )
  .fetch_one( &pool )
  .await
  .expect( "Should fetch history" );

  assert_eq!(
    history_row.get::< String, _ >( "modification_type" ), "increase",
    "History should show increase type (budget went from $100 to $250)"
  );
  assert_eq!(
    history_row.get::< i64, _ >( "old_budget_micros" ), 100_000_000,
    "History should record old budget"
  );
  assert_eq!(
    history_row.get::< i64, _ >( "new_budget_micros" ), 250_000_000,
    "History should record new budget"
  );
  assert_eq!(
    history_row.get::< i64, _ >( "change_amount_micros" ), 150_000_000,
    "History should record delta (+$150)"
  );
  assert_eq!(
    history_row.get::< String, _ >( "modifier_id" ), "admin-approver",
    "History should record approver"
  );
  assert_eq!(
    history_row.get::< String, _ >( "related_request_id" ), request_id,
    "History should link to request"
  );
}

#[ tokio::test ]
async fn test_approve_budget_request_optimistic_locking()
{
  let pool = setup_test_db().await;

  // Create test agent
  sqlx::query( "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( 1 )
    .bind( "test-agent" )
    .bind( "[]" )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( &pool )
    .await
    .expect( "Failed to create agent" );

  // Create agent budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( 1 )
  .bind( 100.0 )
  .bind( 0.0 )
  .bind( 100.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() )
  .execute( &pool )
  .await
  .expect( "Failed to create agent budget" );

  // Create budget request
  let request_id = "breq_test_concurrent";
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Need more budget for testing" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .expect( "Failed to create request" );

  // First approval - should succeed
  let approve_time = chrono::Utc::now().timestamp_millis();
  let result1 = iron_token_manager::budget_request::approve_budget_request(
    &pool,
    request_id,
    "admin-1",
    approve_time,
  )
  .await;

  assert!( result1.is_ok(), "First approval should succeed" );

  // Second approval (concurrent attempt) - should fail
  let result2 = iron_token_manager::budget_request::approve_budget_request(
    &pool,
    request_id,
    "admin-2",
    approve_time + 1000,
  )
  .await;

  assert!(
    result2.is_err(),
    "Second approval should fail due to optimistic locking"
  );

  // Verify budget only changed once
  let agent_budget: f64 = sqlx::query( "SELECT total_allocated FROM agent_budgets WHERE agent_id = ?" )
    .bind( 1 )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch agent budget" )
    .get( "total_allocated" );

  assert_eq!(
    agent_budget, 200.0,
    "Budget should only reflect single approval"
  );

  // Verify only one history record
  let history_count: i64 = sqlx::query( "SELECT COUNT(*) as count FROM budget_modification_history WHERE agent_id = ?" )
    .bind( 1 )
    .fetch_one( &pool )
    .await
    .expect( "Should fetch history count" )
    .get( "count" );

  assert_eq!(
    history_count, 1,
    "Should have only one history record"
  );
}
