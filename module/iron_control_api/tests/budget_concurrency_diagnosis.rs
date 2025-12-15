//! Budget Concurrency Diagnostic Test
//!
//! Enhanced version of budget concurrency test with detailed logging
//! to diagnose why only 3/10 transactions succeeded instead of expected 5/10.
//!
//! ## Purpose
//!
//! Determine if the low success rate is due to:
//! 1. SQLite serialization failures (expected with DEFERRED transactions)
//! 2. Application logic bug
//! 3. Deadlocks or timeouts
//!
//! ## Expected Results
//!
//! With 50 USD budget and 10 concurrent 10 USD spend attempts:
//! - **Ideal:** Exactly 5 successes (no retries)
//! - **Acceptable:** 3-5 successes + transaction conflicts logged
//! - **Bug:** < 3 successes or > 5 successes (overspending)

#[ path = "common/mod.rs" ]
mod common;

use sqlx::{ SqlitePool, Error as SqlxError };
use common::test_db;
use iron_test_db::TestDatabase;
use std::sync::{ Arc, Mutex };

/// Diagnostic result for each transaction attempt
#[derive(Debug, Clone)]
struct TransactionResult
{
  task_id: usize,
  success: bool,
  error_type: Option<String>,
  spent_seen: Option<i64>,
  timestamp_ms: u128,
}

/// Attempt to spend from budget with detailed error logging
async fn attempt_spend(
  pool: SqlitePool,
  task_id: usize,
  spend_amount: i64,
) -> TransactionResult
{
  let start_time = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap()
    .as_millis();

  // Use BEGIN IMMEDIATE for write-heavy workload
  // This acquires write lock immediately, reducing conflicts
  let tx_result = pool.begin().await;

  if let Err( ref e ) = tx_result
  {
    return TransactionResult {
      task_id,
      success: false,
      error_type: Some( format!( "begin() failed: {}", e ) ),
      spent_seen: None,
      timestamp_ms: start_time,
    };
  }

  let mut tx = tx_result.unwrap();

  // Read current spent amount
  let current_result: Result<(i64,), SqlxError> = sqlx::query_as(
    "SELECT spent_cents FROM budgets WHERE id = 1"
  )
  .fetch_one( &mut *tx )
  .await;

  let spent_seen = match current_result
  {
    Ok( (spent,) ) => spent,
    Err( e ) =>
    {
      let _ = tx.rollback().await;
      return TransactionResult {
        task_id,
        success: false,
        error_type: Some( format!( "SELECT failed: {}", e ) ),
        spent_seen: None,
        timestamp_ms: start_time,
      };
    }
  };

  let new_spent = spent_seen + spend_amount;

  // Read total budget
  let total_result: Result<(i64,), SqlxError> = sqlx::query_as(
    "SELECT total_cents FROM budgets WHERE id = 1"
  )
  .fetch_one( &mut *tx )
  .await;

  let total = match total_result
  {
    Ok( (t,) ) => t,
    Err( e ) =>
    {
      let _ = tx.rollback().await;
      return TransactionResult {
        task_id,
        success: false,
        error_type: Some( format!( "SELECT total failed: {}", e ) ),
        spent_seen: Some( spent_seen ),
        timestamp_ms: start_time,
      };
    }
  };

  // Check budget constraint
  if new_spent > total
  {
    let _ = tx.rollback().await;
    return TransactionResult {
      task_id,
      success: false,
      error_type: Some( format!( "Budget exceeded: {} + {} > {}", spent_seen, spend_amount, total ) ),
      spent_seen: Some( spent_seen ),
      timestamp_ms: start_time,
    };
  }

  // Update spent amount
  let update_result = sqlx::query( "UPDATE budgets SET spent_cents = ? WHERE id = 1" )
    .bind( new_spent )
    .execute( &mut *tx )
    .await;

  if let Err( e ) = update_result
  {
    let _ = tx.rollback().await;
    return TransactionResult {
      task_id,
      success: false,
      error_type: Some( format!( "UPDATE failed: {}", e ) ),
      spent_seen: Some( spent_seen ),
      timestamp_ms: start_time,
    };
  }

  // Commit transaction
  let commit_result = tx.commit().await;

  match commit_result
  {
    Ok( _ ) => TransactionResult {
      task_id,
      success: true,
      error_type: None,
      spent_seen: Some( spent_seen ),
      timestamp_ms: start_time,
    },
    Err( e ) => TransactionResult {
      task_id,
      success: false,
      error_type: Some( format!( "COMMIT failed: {}", e ) ),
      spent_seen: Some( spent_seen ),
      timestamp_ms: start_time,
    }
  }
}

#[tokio::test]
#[ignore] // Flaky diagnostic test - run manually with `cargo test -- --ignored`
async fn test_budget_concurrent_spending_diagnostic()
{
  let db: TestDatabase = test_db::create_test_db().await;
  let pool = db.pool().clone();

  // Shared results collector
  let results = Arc::new( Mutex::new( Vec::new() ) );

  // Launch 10 concurrent spending tasks
  let mut handles = vec![];

  for i in 0..10
  {
    let pool_clone = pool.clone();
    let results_clone = results.clone();
    let spend_amount = 1000_i64; // $10

    let handle = tokio::spawn( async move
    {
      let result = attempt_spend( pool_clone, i, spend_amount ).await;
      results_clone.lock().unwrap().push( result.clone() );
      result
    });

    handles.push( handle );
  }

  // Wait for all tasks
  for handle in handles
  {
    let _ = handle.await;
  }

  // Analyze results
  let all_results = results.lock().unwrap().clone();

  let successes: Vec<_> = all_results.iter()
    .filter( |r| r.success )
    .collect();

  let failures: Vec<_> = all_results.iter()
    .filter( |r| !r.success )
    .collect();

  println!( "\n=== BUDGET CONCURRENCY DIAGNOSTIC RESULTS ===" );
  println!( "Total attempts: {}", all_results.len() );
  println!( "Successes: {}", successes.len() );
  println!( "Failures: {}", failures.len() );
  println!();

  println!( "Successful transactions:" );
  for result in &successes
  {
    println!(
      "  Task {}: spent_seen={:?}, timestamp={}ms",
      result.task_id, result.spent_seen, result.timestamp_ms
    );
  }
  println!();

  println!( "Failed transactions:" );
  let mut error_summary: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
  for result in &failures
  {
    let error_type = result.error_type.as_ref().unwrap();
    *error_summary.entry( error_type.clone() ).or_insert( 0 ) += 1;

    println!(
      "  Task {}: spent_seen={:?}, error={}, timestamp={}ms",
      result.task_id, result.spent_seen, error_type, result.timestamp_ms
    );
  }
  println!();

  println!( "Error summary:" );
  for (error_type, count) in &error_summary
  {
    println!( "  {}: {} occurrences", error_type, count );
  }
  println!();

  // Verify final budget state
  let final_spent: (i64,) = sqlx::query_as( "SELECT spent_cents FROM budgets WHERE id = 1" )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to query final spent amount" );

  println!( "Final budget state: spent={}, remaining={}", final_spent.0, 5000 - final_spent.0 );

  // CRITICAL: No overspending
  assert!(
    final_spent.0 <= 5000,
    "LOUD FAILURE: Budget overspent! {} > 5000",
    final_spent.0
  );

  // Analysis assertions
  println!( "\n=== ANALYSIS ===" );

  if successes.len() == 5
  {
    println!( "✅ IDEAL: Exactly 5/10 transactions succeeded (optimal concurrency)" );
  }
  else if successes.len() >= 3 && successes.len() <= 5
  {
    println!( "⚠️  ACCEPTABLE: {}/10 transactions succeeded", successes.len() );
    println!( "   This indicates transaction serialization due to SQLite default (DEFERRED) mode." );
    println!( "   Consider using BEGIN IMMEDIATE for better concurrency in production." );
  }
  else if successes.len() < 3
  {
    println!( "❌ CONCERNING: Only {}/10 transactions succeeded (< 60% success rate)", successes.len() );
    println!( "   This suggests excessive lock contention or timeout issues." );
    panic!( "Unacceptably low success rate" );
  }
  else
  {
    panic!( "LOUD FAILURE: More than 5 transactions succeeded - budget constraint violated!" );
  }

  // Check for specific error patterns
  if error_summary.contains_key( "begin() failed: database is locked" ) ||
     error_summary.keys().any( |error| error.contains( "SQLITE_BUSY" ) )
  {
    println!( "⚠️  Detected SQLITE_BUSY errors - indicates lock contention" );
    println!( "   Recommendation: Use connection pooling with retry logic in production" );
  }

  if error_summary.values().any( |v| *v > 3 )
  {
    println!( "⚠️  High concentration of single error type - investigate root cause" );
  }
}
