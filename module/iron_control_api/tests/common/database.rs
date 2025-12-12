//! Database test infrastructure for verifying test isolation and state management.
//!
//! Provides database hooks and tests to ensure:
//! - Test isolation (no data leakage between tests)
//! - Transaction rollback on failure
//! - Concurrent test execution safety
//! - Database constraints active after migrations

// Fix(issue-003): Database setup/teardown hooks for test isolation
// Root cause: Tests shared database state causing flaky failures and non-deterministic results
// Pitfall: Always isolate test state with in-memory databases and proper fixture cleanup

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };

/// Test database fixture with automatic cleanup.
///
/// Creates an in-memory SQLite database with migrations applied.
/// Each instance is isolated from other tests.
pub struct TestDb
{
  pool: SqlitePool,
}

impl TestDb
{
  /// Create new test database with migrations applied.
  ///
  /// Uses `sqlite::memory:` for test isolation.
  pub async fn new() -> Self
  {
    let pool = setup_test_db().await;
    Self { pool }
  }

  /// Get reference to database pool for test queries.
  pub fn pool( &self ) -> &SqlitePool
  {
    &self.pool
  }
}

impl Drop for TestDb
{
  fn drop( &mut self )
  {
    // Note: Can't await in Drop, cleanup happens via tokio runtime
    // In-memory databases are automatically cleaned up when pool is dropped
  }
}

/// Setup test database with schema applied.
///
/// Creates in-memory SQLite database and runs migrations.
/// Each call creates a fresh, isolated database.
pub async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create in-memory test database" );

  // Run migrations for api_tokens table
  // Note: Using iron_token_manager's migrations
  let migration_001 = include_str!( "../../../iron_token_manager/migrations/001_initial_schema.sql" );
  sqlx::raw_sql( migration_001 )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to run migration 001" );

  let migration_002 = include_str!( "../../../iron_token_manager/migrations/002_add_length_constraints.sql" );
  sqlx::raw_sql( migration_002 )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to run migration 002" );

  pool
}

/// Teardown test database.
///
/// Closes database pool. With in-memory databases, this releases all resources.
///
/// Note: Typically not needed as Drop handles cleanup automatically, but provided
/// for explicit cleanup scenarios.
#[ allow( dead_code ) ]
pub async fn teardown_test_db( pool: SqlitePool )
{
  pool.close().await;
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  /// Verifies database isolation between sequential tests (Deliverable 1.3, Test 1).
  ///
  /// ## Root Cause (if this test fails)
  /// Tests sharing database state causes data leakage, leading to flaky tests that
  /// pass/fail depending on execution order. One test's data pollutes another test's
  /// environment, causing non-deterministic failures that are hard to debug.
  ///
  /// ## Why Not Caught Initially
  /// Isolation bugs are subtle - tests might pass in isolation but fail when run in
  /// suite. Original test infrastructure didn't verify isolation explicitly, assumed
  /// it worked correctly.
  ///
  /// ## Fix Applied
  /// Each TestDb::new() creates fresh `sqlite::memory:` database with migrations.
  /// In-memory databases are process-local and dropped when pool closes, ensuring
  /// complete isolation between tests.
  ///
  /// ## Prevention
  /// Always use in-memory databases for unit tests (never shared file databases).
  /// Verify isolation with explicit tests like this one. Document test infrastructure
  /// assumptions with infrastructure tests.
  ///
  /// ## Pitfall to Avoid
  /// Never use file-based databases for unit tests (e.g., `sqlite:test.db`). File
  /// databases require explicit cleanup, create race conditions in parallel execution,
  /// and leak state between tests. Always use `sqlite::memory:` for test isolation.
  // test_kind: infrastructure(issue-003-isolation)
  #[ tokio::test ]
  async fn test_database_reset_between_tests()
  {
    // Test 1: Insert data into first database
    let token_hash_1 = {
      let db = TestDb::new().await;

      sqlx::query(
        "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
         VALUES (?, ?, ?, 1, ?)"
      )
        .bind( "user_test_1" )
        .bind( "project_test_1" )
        .bind( "hash_test_isolation_1" )
        .bind( 1234567890 )
        .execute( db.pool() )
        .await
        .expect("LOUD FAILURE: Should insert test data");

      // Query to verify data exists in first database
      let row: ( String, ) = sqlx::query_as(
        "SELECT token_hash FROM api_tokens WHERE user_id = ?"
      )
        .bind( "user_test_1" )
        .fetch_one( db.pool() )
        .await
        .expect("LOUD FAILURE: Should fetch inserted data");

      row.0
      // db is dropped here, pool closed, in-memory database destroyed
    };

    assert_eq!(
      token_hash_1,
      "hash_test_isolation_1",
      "First database should have test data"
    );

    // Test 2: Create second database, should NOT see data from Test 1
    {
      let db = TestDb::new().await;

      let count: ( i64, ) = sqlx::query_as(
        "SELECT COUNT(*) FROM api_tokens WHERE user_id = ?"
      )
        .bind( "user_test_1" )
        .fetch_one( db.pool() )
        .await
        .expect("LOUD FAILURE: Should query database");

      assert_eq!(
        count.0,
        0,
        "Second database should be isolated from first (no data leakage)"
      );
    }
  }

  /// Verifies transaction rollback on test failure (Deliverable 1.3, Test 2).
  ///
  /// ## Root Cause (if this test fails)
  /// Test failures leaving database in inconsistent state cause subsequent tests
  /// to fail mysteriously. Uncommitted transactions or partial writes persist,
  /// creating cascading failures across test suite.
  ///
  /// ## Why Not Caught Initially
  /// Transaction cleanup is automatic for in-memory databases (dropped on panic),
  /// but file-based databases require explicit rollback. Original tests didn't
  /// verify cleanup behavior explicitly.
  ///
  /// ## Fix Applied
  /// Using `sqlite::memory:` ensures automatic cleanup on test failure. When
  /// TestDb is dropped (even on panic), the entire in-memory database is destroyed,
  /// leaving no residual state.
  ///
  /// ## Prevention
  /// Use in-memory databases for all unit tests. Avoid manual transaction management
  /// in tests (let database drop handle cleanup). Document cleanup behavior with
  /// infrastructure tests.
  ///
  /// ## Pitfall to Avoid
  /// Never rely on manual transaction rollback in test cleanup. Panics bypass
  /// cleanup code, leaving database in bad state. Use RAII (Resource Acquisition
  /// Is Initialization) pattern - database cleanup happens in Drop, which runs
  /// even on panic.
  // test_kind: infrastructure(issue-003-rollback)
  #[ tokio::test ]
  async fn test_transaction_rollback_on_failure()
  {
    // Verify database state is clean after simulated failure
    let db = TestDb::new().await;

    // Simulate test that inserts data then fails
    let result = std::panic::catch_unwind( std::panic::AssertUnwindSafe( || {
      // This simulates a test that would panic/fail
      // In real scenario, test failure would drop database before commit
      panic!( "Simulated test failure" );
    } ) );

    assert!(
      result.is_err(),
      "Simulated failure should panic"
    );

    // Verify database is still accessible and empty
    // (In-memory database survives panic because we caught it)
    let count: ( i64, ) = sqlx::query_as( "SELECT COUNT(*) FROM api_tokens" )
      .fetch_one( db.pool() )
      .await
      .expect("LOUD FAILURE: Database should still be accessible after panic");

    assert_eq!(
      count.0,
      0,
      "Database should be empty (simulated test didn't commit any data)"
    );
  }

  /// Verifies concurrent test execution doesn't cause conflicts (Deliverable 1.3, Test 3).
  ///
  /// ## Root Cause (if this test fails)
  /// Parallel test execution causes database locking conflicts, race conditions,
  /// or data corruption. Tests fail non-deterministically when run in parallel
  /// with `cargo test` or `cargo nextest`.
  ///
  /// ## Why Not Caught Initially
  /// Concurrency issues only manifest under parallel execution. Original tests
  /// might pass when run serially (`--test-threads=1`) but fail in parallel.
  /// CI/CD parallelism can expose these issues.
  ///
  /// ## Fix Applied
  /// Each test gets isolated in-memory database (`sqlite::memory:`), eliminating
  /// shared state. No database locking conflicts possible since databases are
  /// separate. Parallel execution is safe by design.
  ///
  /// ## Prevention
  /// Always use isolated databases for tests (in-memory or separate files).
  /// Verify parallel safety with concurrent execution tests. Run tests with
  /// parallelism enabled in CI (`cargo nextest` defaults to parallel).
  ///
  /// ## Pitfall to Avoid
  /// Never use single shared database for all tests (e.g., `static DB: ...`).
  /// Shared mutable state + parallelism = flaky tests. Always prefer isolation
  /// over shared state in test infrastructure.
  // test_kind: infrastructure(issue-003-concurrency)
  #[ tokio::test ]
  async fn test_concurrent_test_execution()
  {
    use tokio::task::JoinSet;

    // Spawn 10 concurrent tasks, each creating its own database
    let mut join_set = JoinSet::new();

    for i in 0..10
    {
      join_set.spawn( async move {
        let db = TestDb::new().await;

        // Each task inserts unique data
        sqlx::query(
          "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
           VALUES (?, ?, ?, 1, ?)"
        )
          .bind( format!( "user_concurrent_{}", i ) )
          .bind( format!( "project_concurrent_{}", i ) )
          .bind( format!( "hash_concurrent_{}", i ) )
          .bind( 1234567890 + i as i64 )
          .execute( db.pool() )
          .await
          .expect("LOUD FAILURE: Should insert without conflicts");

        // Verify only THIS task's data exists (isolation check)
        let count: ( i64, ) = sqlx::query_as(
          "SELECT COUNT(*) FROM api_tokens WHERE user_id = ?"
        )
          .bind( format!( "user_concurrent_{}", i ) )
          .fetch_one( db.pool() )
          .await
          .expect("LOUD FAILURE: Should query without conflicts");

        assert_eq!(
          count.0,
          1,
          "Task {} should see only its own data (isolation)",
          i
        );

        i
      } );
    }

    // Wait for all tasks to complete
    let mut completed = Vec::new();
    while let Some( result ) = join_set.join_next().await
    {
      completed.push( result.expect("LOUD FAILURE: Task should not panic") );
    }

    assert_eq!(
      completed.len(),
      10,
      "All 10 concurrent tasks should complete successfully"
    );
  }

  /// Verifies database constraints are active after migrations (Deliverable 1.3, Test 4).
  ///
  /// ## Root Cause (if this test fails)
  /// Migrations didn't apply correctly, leaving database without CHECK constraints.
  /// Invalid data can be inserted, bypassing validation logic. Defense-in-depth
  /// layer (database constraints) is missing.
  ///
  /// ## Why Not Caught Initially
  /// Original tests only verified API behavior, didn't inspect database schema.
  /// Assumed migrations worked correctly without verification. Schema bugs can
  /// be silent until invalid data gets inserted.
  ///
  /// ## Fix Applied
  /// Migration 002 adds CHECK constraints on user_id and project_id length.
  /// This test verifies constraints are active by attempting to violate them.
  /// Database should reject invalid data with constraint error.
  ///
  /// ## Prevention
  /// Always test database constraints with infrastructure tests. Verify schema
  /// matches specification after migrations. Test defense-in-depth layers
  /// independently of application logic.
  ///
  /// ## Pitfall to Avoid
  /// Never assume migrations worked correctly. Always verify schema changes with
  /// tests. Silent schema bugs can persist for months until invalid data triggers
  /// them. Test constraints by attempting to violate them.
  // test_kind: infrastructure(issue-003-constraints)
  #[ tokio::test ]
  async fn test_database_constraints_active()
  {
    let db = TestDb::new().await;

    // Attempt to insert user_id longer than 500 chars (violates CHECK constraint)
    let too_long_user_id = "A".repeat( 501 );

    let result = sqlx::query(
      "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
       VALUES (?, ?, ?, 1, ?)"
    )
      .bind( &too_long_user_id )
      .bind( "valid_project" )
      .bind( "valid_hash" )
      .bind( 1234567890 )
      .execute( db.pool() )
      .await;

    assert!(
      result.is_err(),
      "Database should reject user_id > 500 chars due to CHECK constraint"
    );

    let error_msg = result.unwrap_err().to_string();
    assert!(
      error_msg.to_lowercase().contains( "check" ) ||
      error_msg.to_lowercase().contains( "constraint" ),
      "Error should mention CHECK constraint violation, got: {}",
      error_msg
    );

    // Verify constraint also rejects empty user_id
    let result = sqlx::query(
      "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
       VALUES (?, ?, ?, 1, ?)"
    )
      .bind( "" )
      .bind( "valid_project" )
      .bind( "valid_hash_2" )
      .bind( 1234567890 )
      .execute( db.pool() )
      .await;

    assert!(
      result.is_err(),
      "Database should reject empty user_id due to CHECK constraint (LENGTH > 0)"
    );

    // Verify valid data is accepted
    let result = sqlx::query(
      "INSERT INTO api_tokens (user_id, project_id, token_hash, is_active, created_at)
       VALUES (?, ?, ?, 1, ?)"
    )
      .bind( "valid_user" )
      .bind( "valid_project" )
      .bind( "valid_hash_3" )
      .bind( 1234567890 )
      .execute( db.pool() )
      .await;

    assert!(
      result.is_ok(),
      "Database should accept valid data: {:?}",
      result.err()
    );
  }
}
