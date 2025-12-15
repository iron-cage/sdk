//! Test database creation using iron_test_db
//!
//! This module provides a thin wrapper around iron_test_db for creating
//! test databases with migrations applied.
//!
//! # Usage
//!
//! ```no_run
//! # use iron_control_api_tests::common::test_db::create_test_db;
//! # async fn example() {
//! let db = create_test_db().await;
//! let pool = db.pool();
//! // Use pool for queries
//! # }
//! ```
//!
//! # Migration History
//!
//! Created during iron_test_db universal adoption migration to replace
//! duplicated test database infrastructure. See module/iron_test_db/tests/
//! migration_verification.rs for full migration rationale.

use iron_test_db::{ TestDatabase, TestDatabaseBuilder };

/// Authentication schema for iron_control_api tests
///
/// Provides tables for user authentication, token blacklisting, and user audit logging
/// that are specific to control API tests (beyond token_manager migrations).
const AUTH_SCHEMA: &str = r#"
-- Users table for authentication tests
CREATE TABLE IF NOT EXISTS users
(
  id TEXT PRIMARY KEY,
  username TEXT NOT NULL UNIQUE,
  password_hash TEXT NOT NULL,
  role TEXT NOT NULL DEFAULT 'user',
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  email TEXT,
  name TEXT,
  last_login INTEGER,
  suspended_at INTEGER,
  suspended_by INTEGER,
  deleted_at INTEGER,
  deleted_by INTEGER,
  force_password_change INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

-- Refresh token blacklist for logout tests
CREATE TABLE IF NOT EXISTS token_blacklist
(
  jti TEXT PRIMARY KEY CHECK (LENGTH(jti) > 0 AND LENGTH(jti) <= 255),
  user_id TEXT NOT NULL,
  blacklisted_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_token_blacklist_user_id ON token_blacklist(user_id);

-- User audit log for user management tests
CREATE TABLE IF NOT EXISTS user_audit_log
(
  id TEXT PRIMARY KEY,
  operation TEXT NOT NULL,
  target_user_id TEXT NOT NULL,
  performed_by TEXT NOT NULL,
  timestamp INTEGER NOT NULL,
  previous_state TEXT,
  new_state TEXT,
  reason TEXT,
  FOREIGN KEY(target_user_id) REFERENCES users(id),
  FOREIGN KEY(performed_by) REFERENCES users(id)
);
"#;

/// Create a test database with all migrations and authentication schema applied.
///
/// This function:
/// 1. Creates an in-memory SQLite database
/// 2. Configures connection pool (size: 5)
/// 3. Applies all migrations from iron_token_manager
/// 4. Applies authentication schema (users, token_blacklist, user_audit_log tables)
///
/// # Panics
///
/// Panics with LOUD FAILURE message if:
/// - Database creation fails
/// - Migration application fails
/// - Authentication schema application fails
///
/// # Examples
///
/// ```no_run
/// # use iron_control_api_tests::common::test_db::create_test_db;
/// # async fn example() {
/// let db = create_test_db().await;
/// let pool = db.pool();
///
/// // Use pool for queries
/// sqlx::query("SELECT * FROM users")
///   .fetch_all(pool)
///   .await
///   .expect("Query failed");
/// # }
/// ```
pub async fn create_test_db() -> TestDatabase
{
  let db = TestDatabaseBuilder::new()
    .in_memory()
    .pool_size( 5 )
    .build()
    .await
    .expect( "LOUD FAILURE: Failed to create test database" );

  iron_token_manager::migrations::apply_all_migrations( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to apply migrations" );

  // Apply authentication-specific schema on top of migrations
  sqlx::raw_sql( AUTH_SCHEMA )
    .execute( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to apply authentication schema" );

  db
}

/// Create an isolated in-memory test database.
///
/// This is the preferred function for test functions that need database isolation.
/// Each call creates a completely isolated in-memory database with fresh schema.
///
/// This function:
/// 1. Creates a fresh in-memory SQLite database
/// 2. Configures connection pool (size: 5)
/// 3. Applies all migrations from iron_token_manager
/// 4. Applies authentication schema (users, token_blacklist, user_audit_log tables)
///
/// # Isolation Guarantee
///
/// Each call to this function creates a new, isolated database instance.
/// Databases are completely independent and dropped when the returned TestDatabase
/// is dropped, ensuring zero data leakage between tests.
///
/// # Panics
///
/// Panics with LOUD FAILURE message if:
/// - Database creation fails
/// - Migration application fails
/// - Authentication schema application fails
///
/// # Examples
///
/// ```no_run
/// # use iron_control_api_tests::common::test_db::create_test_db_isolated;
/// # async fn example() {
/// let db = create_test_db_isolated().await;
/// let pool = db.pool();
///
/// // Use pool for queries - completely isolated from other tests
/// sqlx::query("SELECT * FROM users")
///   .fetch_all(pool)
///   .await
///   .expect("Query failed");
/// # }
/// ```
pub async fn create_test_db_isolated() -> TestDatabase
{
  create_test_db().await
}
