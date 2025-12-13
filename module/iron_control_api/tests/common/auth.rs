//! Authentication test helpers - Protocol 007 test infrastructure
//!
//! Provides real (non-mocked) test utilities for authentication endpoint testing.
//!
//! # Functions
//!
//! - `setup_auth_test_db()` - Create in-memory database with auth schema
//! - `seed_test_user()` - Create test user with specified credentials
//! - `create_auth_router()` - Create Axum router with auth endpoints

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use axum::{ Router, routing::post };
use iron_control_api::routes::auth::{ login, logout, refresh, validate, AuthState };
use std::sync::Arc;

/// Setup in-memory SQLite database with auth schema for testing
///
/// Creates:
/// - users table
/// - token_blacklist table
/// - user_audit_log table
///
/// Uses real database (not mocked) to catch integration issues
#[allow(dead_code)]
pub async fn setup_auth_test_db() -> SqlitePool
{
  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( "sqlite::memory:?cache=shared" )
    .await
    .expect( "LOUD FAILURE: Failed to create in-memory database for auth tests" );

  // Apply schema (same as common::TEST_SCHEMA)
  let schema = r#"
    CREATE TABLE IF NOT EXISTS users
    (
      id TEXT PRIMARY KEY,
      username TEXT NOT NULL UNIQUE,
      password_hash TEXT NOT NULL,
      role TEXT NOT NULL DEFAULT 'user',
      is_active INTEGER NOT NULL DEFAULT 1,
      created_at INTEGER NOT NULL,
      email TEXT,
      last_login INTEGER,
      suspended_at INTEGER,
      suspended_by INTEGER,
      deleted_at INTEGER,
      deleted_by INTEGER,
      force_password_change INTEGER NOT NULL DEFAULT 0
    );

    CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);

    CREATE TABLE IF NOT EXISTS token_blacklist
    (
      jti TEXT PRIMARY KEY CHECK (LENGTH(jti) > 0 AND LENGTH(jti) <= 255),
      user_id TEXT NOT NULL,
      blacklisted_at INTEGER NOT NULL,
      expires_at INTEGER NOT NULL
    );

    CREATE INDEX IF NOT EXISTS idx_token_blacklist_user_id ON token_blacklist(user_id);

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

  sqlx::raw_sql( schema )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to apply auth schema" );

  pool
}

/// Seed test user with specified credentials
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `email` - User email (also used as username)
/// * `password` - Plain text password (will be hashed with bcrypt)
/// * `role` - User role ("admin", "user", etc.)
/// * `is_active` - Whether account is active
///
/// # Returns
///
/// User ID (for assertions)
#[allow(dead_code)]
pub async fn seed_test_user(
  pool: &SqlitePool,
  email: &str,
  password: &str,
  role: &str,
  is_active: bool
) -> String
{
  let password_hash = bcrypt::hash( password, 4 )
    .expect( "LOUD FAILURE: Failed to hash test password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect("LOUD FAILURE: Time went backwards")
    .as_secs() as i64;

  let user_id = format!( "user_{}", uuid::Uuid::new_v4() );

  sqlx::query(
    "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &user_id )
  .bind( email )  // Use email as username
  .bind( email )
  .bind( &password_hash )
  .bind( role )
  .bind( if is_active { 1 } else { 0 } )
  .bind( now )
  .execute( pool )
  .await
  .unwrap_or_else( |_| panic!(
    "LOUD FAILURE: Failed to seed test user '{}'",
    email
  ) );

  user_id
}

/// Create Axum router with auth endpoints for testing
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Axum router with:
/// - POST /api/v1/auth/login
/// - POST /api/v1/auth/logout
/// - POST /api/v1/auth/refresh
/// - POST /api/v1/auth/validate
#[allow(dead_code)]
pub async fn create_auth_router( pool: SqlitePool ) -> Router
{
  let auth_state = AuthState
  {
    jwt_secret: Arc::new( iron_control_api::jwt_auth::JwtSecret::new(
      "test_jwt_secret_for_authentication_tests_only".to_string()
    ) ),
    db_pool: pool,
    rate_limiter: iron_control_api::rate_limiter::LoginRateLimiter::new(),
  };

  Router::new()
    .route( "/api/v1/auth/login", post( login ) )
    .route( "/api/v1/auth/logout", post( logout ) )
    .route( "/api/v1/auth/refresh", post( refresh ) )
    .route( "/api/v1/auth/validate", post( validate ) )
    .with_state( auth_state )
}
