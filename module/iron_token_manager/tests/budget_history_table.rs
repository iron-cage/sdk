//! Budget Modification History Table Schema Tests (Protocol 017)
//!
//! Tests database schema for `budget_modification_history` table created by migration 012.
//! Verifies table structure, constraints, foreign keys, and indexes.

mod common;
use common::create_test_db_v2;

#[ tokio::test ]
async fn test_budget_modification_history_table_exists()
{
  let db = create_test_db_v2().await;

  let result = sqlx::query( "SELECT COUNT(*) FROM budget_modification_history" )
    .fetch_one( db.pool() )
    .await;

  assert!( result.is_ok(), "budget_modification_history table should exist after migrations" );
}

#[ tokio::test ]
async fn test_budget_history_constraint_reason_min_length()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let result = sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 1 )
  .bind( "increase" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( "Short" )  // Too short (< 10 chars)
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject reason shorter than 10 characters" );
}

#[ tokio::test ]
async fn test_budget_history_constraint_reason_max_length()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let long_reason = "a".repeat( 501 );  // Too long (> 500 chars)

  let result = sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 1 )
  .bind( "increase" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( long_reason )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject reason longer than 500 characters" );
}

#[ tokio::test ]
async fn test_budget_history_constraint_modification_type_enum()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  let result = sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 1 )
  .bind( "invalid_type" )  // Invalid - not in (increase, decrease, reset)
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( "Valid reason that is at least 10 characters" )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject modification_type not in enum (increase/decrease/reset)" );
}

#[ tokio::test ]
async fn test_budget_history_foreign_key_agent_constraint()
{
  let db = create_test_db_v2().await;

  let result = sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 999 )  // Nonexistent agent_id
  .bind( "increase" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( "Valid reason for testing foreign key" )
  .bind( 1000 )
  .execute( db.pool() )
  .await;

  assert!( result.is_err(), "Should reject history entry for nonexistent agent (foreign key constraint)" );
}

#[ tokio::test ]
async fn test_budget_history_cascade_delete()
{
  let db = create_test_db_v2().await;
  common::seed_test_agent( db.pool(), 1 ).await;

  // Insert history entry
  sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 1 )
  .bind( "increase" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( "Valid reason for cascade delete test with sufficient length" )
  .bind( 1000 )
  .execute( db.pool() )
  .await
  .expect( "Should insert history entry" );

  // Delete agent
  sqlx::query( "DELETE FROM agents WHERE id = ?" )
    .bind( 1 )
    .execute( db.pool() )
    .await
    .expect( "Should delete agent" );

  // Verify history entry was cascade deleted
  let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM budget_modification_history WHERE id = ?" )
    .bind( "bhist_test" )
    .fetch_one( db.pool() )
    .await
    .expect( "Should count" );

  assert_eq!( count, 0, "History entry should be cascade deleted when agent is deleted" );
}

#[ tokio::test ]
async fn test_budget_history_related_request_set_null()
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
  .bind( "Valid justification for testing SET NULL behavior" )
  .bind( "approved" )
  .bind( 1000 )
  .bind( 1000 )
  .execute( db.pool() )
  .await
  .expect( "Should insert budget request" );

  // Insert history entry linked to request
  sqlx::query(
    "INSERT INTO budget_modification_history
     (id, agent_id, modification_type, old_budget_micros, new_budget_micros,
      change_amount_micros, modifier_id, reason, related_request_id, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "bhist_test" )
  .bind( 1 )
  .bind( "increase" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( 100_000_000 )
  .bind( "admin-123" )
  .bind( "Approved budget increase from request" )
  .bind( "breq_test" )
  .bind( 1000 )
  .execute( db.pool() )
  .await
  .expect( "Should insert history entry" );

  // Delete budget request
  sqlx::query( "DELETE FROM budget_change_requests WHERE id = ?" )
    .bind( "breq_test" )
    .execute( db.pool() )
    .await
    .expect( "Should delete request" );

  // Verify history entry still exists but related_request_id is NULL
  let related_id: Option< String > = sqlx::query_scalar(
    "SELECT related_request_id FROM budget_modification_history WHERE id = ?"
  )
  .bind( "bhist_test" )
  .fetch_one( db.pool() )
  .await
  .expect( "Should fetch history entry" );

  assert_eq!( related_id, None, "related_request_id should be NULL after request deletion (SET NULL)" );
}

#[ tokio::test ]
async fn test_budget_history_index_exists()
{
  let db = create_test_db_v2().await;

  // Check agent index exists
  let agent_index = sqlx::query( "SELECT name FROM sqlite_master WHERE type='index' AND name='idx_budget_history_agent'" )
    .fetch_optional( db.pool() )
    .await
    .expect( "Should query indexes" );
  assert!( agent_index.is_some(), "Index idx_budget_history_agent should exist" );
}
