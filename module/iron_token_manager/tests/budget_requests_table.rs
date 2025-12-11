//! Budget Change Requests Table Schema Tests (Protocol 012)
//!
//! Tests database schema for `budget_change_requests` table created by migration 011.
//! Verifies table structure, constraints, foreign keys, and indexes.

mod common;
use common::create_test_db_v2;

#[ tokio::test ]
async fn test_budget_change_requests_table_exists()
{
  let db = create_test_db_v2().await;

  let result = sqlx::query( "SELECT COUNT(*) FROM budget_change_requests" )
    .fetch_one( db.pool() )
    .await;

  assert!( result.is_ok(), "budget_change_requests table should exist after migrations" );
}

#[ tokio::test ]
async fn test_budget_request_constraint_justification_min_length()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let result = sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test" )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Short" )  // Too short (< 20 chars)
  .bind( "pending" )
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject justification shorter than 20 characters" );
}

#[ tokio::test ]
async fn test_budget_request_constraint_justification_max_length()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let long_justification = "a".repeat( 501 );  // Too long (> 500 chars)

  let result = sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test" )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( long_justification )
  .bind( "pending" )
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject justification longer than 500 characters" );
}

#[ tokio::test ]
async fn test_budget_request_constraint_status_enum()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let result = sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test" )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Valid justification text that is at least 20 characters long" )
  .bind( "invalid_status" )  // Invalid - not in (pending, approved, rejected, cancelled)
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject status not in enum (pending/approved/rejected/cancelled)" );
}

#[ tokio::test ]
async fn test_budget_request_foreign_key_constraint()
{
  let db = create_test_db_v2().await;

  let result = sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test" )
  .bind( 999 )  // Nonexistent agent_id
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Valid justification text that is at least 20 characters long" )
  .bind( "pending" )
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject budget request for nonexistent agent (foreign key constraint)" );
}

#[ tokio::test ]
async fn test_budget_request_cascade_delete()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  // Insert budget request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test" )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Valid justification for cascade delete test with sufficient length" )
  .bind( "pending" )
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await
  .expect( "Should insert budget request" );

  // Delete agent
  sqlx::query( "DELETE FROM agents WHERE id = ?" )
    .bind( 1 )
    .execute( db.pool() )
    .await
    .expect( "Should delete agent" );

  // Verify budget request was cascade deleted
  let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM budget_change_requests WHERE id = ?" )
    .bind( "breq_test" )
    .fetch_one( db.pool() )
    .await
    .expect( "Should count" );

  assert_eq!( count, 0, "Budget request should be cascade deleted when agent is deleted" );
}

#[ tokio::test ]
async fn test_budget_request_indexes_exist()
{
  let db = create_test_db_v2().await;

  // Check status index exists
  let status_index = sqlx::query( "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_budget_requests_status'" )
    .fetch_optional( db.pool() )
    .await
    .expect( "Should query indexes" );
  assert!( status_index.is_some(), "Index idx_budget_requests_status should exist" );

  // Check agent index exists
  let agent_index = sqlx::query( "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_budget_requests_agent'" )
    .fetch_optional( db.pool() )
    .await
    .expect( "Should query indexes" );
  assert!( agent_index.is_some(), "Index idx_budget_requests_agent should exist" );
}
