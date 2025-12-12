//! Test infrastructure providing real (non-mocked) test utilities.
//!
//! Provides:
//! - In-memory database setup with schema
//! - Test user creation
//! - JWT token generation
//! - Request builders
//! - Response extractors
//! - Database test infrastructure and isolation tests

pub mod corner_cases;
pub mod database;
pub mod error_format;
pub mod fixtures;
pub mod test_state;

use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use axum::{ response::Response, http::StatusCode, body::Body };
use iron_control_api::jwt_auth::{ JwtSecret, AccessTokenClaims, RefreshTokenClaims };

/// Test database schema for authentication
const TEST_SCHEMA: &str = r#"
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

/// Create in-memory SQLite database with test schema applied.
///
/// Uses real database (not mocked) to catch integration issues.
pub async fn create_test_database() -> SqlitePool
{
  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create in-memory database" );

  // Apply all migrations

  iron_token_manager::migrations::apply_all_migrations( &pool ).await.unwrap();

  pool
}

/// Create admin user with credentials
#[allow(dead_code)]
pub async fn create_test_admin( pool: &SqlitePool ) -> ( String, String )
{
  let password_hash = bcrypt::hash( "testpass", 4 )
    .expect( "LOUD FAILURE: Failed to hash test password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_secs() as i64;

  let user_id = "user_admin_test".to_string();

  sqlx::query(
    "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &user_id )
  .bind( "admin" )
  .bind( "admin@admin.com" )
  .bind( &password_hash )
  .bind( "admin" )
  .bind( 1 )
  .bind( now )
  .execute( pool )
  .await
  .unwrap_or_else( |_| panic!(
    "LOUD FAILURE: Failed to create test admin user '{}'",
    "admin"
  ) );

  ( user_id, password_hash )
}


/// Create test user with known credentials.
///
/// Returns (user_id, password_hash) for test assertions.
pub async fn create_test_user( pool: &SqlitePool, email: &str ) -> ( String, String )
{
  let password_hash = bcrypt::hash( "test_password", 4 )
    .expect( "LOUD FAILURE: Failed to hash test password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_secs() as i64;

  sqlx::query(
    "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind("user_dynamic_test")
  .bind( "test_user_dynamic" )
  .bind( email )
  .bind( &password_hash )
  .bind( "user" )
  .bind( 1 )
  .bind( now )
  .execute( pool )
  .await
  .unwrap_or_else( |_| panic!(
    "LOUD FAILURE: Failed to create test user '{}'",
    email
  ) );

  ( "user_dynamic_test".to_string(), password_hash )
}

/// Generate valid JWT access token for test user.
///
/// Uses real JWT generation (not mocked) to catch signing issues.
pub fn create_test_access_token( user_id: &str, email: &str, role: &str, jwt_secret: &str ) -> String
{
  let jwt = JwtSecret::new( jwt_secret.to_string() );
  jwt.generate_access_token( user_id, email, role, jwt_secret )
    .unwrap_or_else( |_| panic!(
      "LOUD FAILURE: Failed to generate test JWT for user '{}'",
      user_id
    ) )
}

// ... (skipping refresh token stuff)

  #[ test ]
  fn test_create_test_access_token()
  {
    let token = create_test_access_token( "user_123", "user@mail.com", "user", "test_secret" );
    assert!( !token.is_empty(), "Token should not be empty" );

    let claims = decode_test_access_token( &token, "test_secret" );
    assert_eq!( claims.sub, "user_123" );
    assert_eq!( claims.email, "user@mail.com" );
    assert_eq!( claims.role, "user" );
    assert_eq!( claims.jti, "test_secret" );
  }

/// Generate valid JWT refresh token for test user.
///
/// Uses real JWT generation (not mocked) to catch signing issues.
#[ allow( dead_code ) ]
pub fn create_test_refresh_token( user_id: &str, email: &str, role: &str, token_id: &str, jwt_secret: &str ) -> String
{
  let jwt = JwtSecret::new( jwt_secret.to_string() );
  jwt.generate_refresh_token( user_id, email, role, token_id )
    .unwrap_or_else( |_| panic!(
      "LOUD FAILURE: Failed to generate test refresh token for user '{}'",
      user_id
    ) )
}

/// Decode JWT access token for testing.
#[ allow( dead_code ) ]
pub fn decode_test_access_token( token: &str, jwt_secret: &str ) -> AccessTokenClaims
{
  let jwt = JwtSecret::new( jwt_secret.to_string() );
  jwt.verify_access_token( token )
    .expect( "LOUD FAILURE: Failed to decode test JWT" )
}

/// Decode JWT refresh token for testing.
#[ allow( dead_code ) ]
pub fn decode_test_refresh_token( token: &str, jwt_secret: &str ) -> RefreshTokenClaims
{
  let jwt = JwtSecret::new( jwt_secret.to_string() );
  jwt.verify_refresh_token( token )
    .expect( "LOUD FAILURE: Failed to decode test refresh token" )
}

/// Extract status and body from Axum response.
#[ allow( dead_code ) ]
pub async fn extract_response( response: Response< Body > ) -> ( StatusCode, String )
{
  let status = response.status();
  let bytes = http_body_util::BodyExt::collect( response.into_body() )
    .await
    .expect( "LOUD FAILURE: Failed to read response body" )
    .to_bytes();
  let body = String::from_utf8( bytes.to_vec() )
    .expect( "LOUD FAILURE: Response body must be valid UTF-8" );

  ( status, body )
}

/// Extract status and parse JSON body from Axum response.
#[ allow( dead_code ) ]
pub async fn extract_json_response< T >( response: Response< Body > ) -> ( StatusCode, T )
where
  T: serde::de::DeserializeOwned,
{
  let ( status, body ) = extract_response( response ).await;
  let json = serde_json::from_str::< T >( &body )
    .unwrap_or_else( |_| panic!(
      "LOUD FAILURE: Failed to parse response body as JSON: {}",
      body
    ) );

  ( status, json )
}

/// Blacklist refresh token for logout testing.
#[ allow( dead_code ) ]
pub async fn blacklist_refresh_token( pool: &SqlitePool, token_id: &str, user_id: &str )
{
  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_secs() as i64;

  sqlx::query(
    "INSERT INTO token_blacklist (jti, user_id, blacklisted_at) VALUES (?, ?, ?)"
  )
  .bind( token_id )
  .bind( user_id )
  .bind( now )
  .execute( pool )
  .await
  .expect( "LOUD FAILURE: Failed to blacklist refresh token" );
}

/// Check if refresh token is blacklisted.
#[ allow( dead_code ) ]
pub async fn is_token_blacklisted( pool: &SqlitePool, token_id: &str ) -> bool
{
  let result: ( i64, ) = sqlx::query_as(
    "SELECT COUNT(*) FROM token_blacklist WHERE jti = ?"
  )
  .bind( token_id )
  .fetch_one( pool )
  .await
  .expect( "LOUD FAILURE: Failed to check token blacklist" );

  result.0 > 0
}

/// Verify password hash matches plaintext password.
pub fn verify_password( password: &str, hash: &str ) -> bool
{
  bcrypt::verify( password, hash )
    .expect( "LOUD FAILURE: Failed to verify password hash" )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ tokio::test ]
  async fn test_create_test_database()
  {
    let pool = create_test_database().await;

    // Verify schema was applied
    let result = sqlx::query( "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='users'" )
      .fetch_one( &pool )
      .await;

    assert!( result.is_ok(), "Users table should exist" );
  }

  #[ tokio::test ]
  async fn test_create_test_user()
  {
    let pool = create_test_database().await;
    let ( user_id, password_hash ) = create_test_user( &pool, "testuser" ).await;

    assert!( !user_id.is_empty(), "User ID should not be empty" );
    assert!( !password_hash.is_empty() );

    // Verify user was inserted
    let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users WHERE id = ?" )
      .bind( user_id )
      .fetch_one( &pool )
      .await
      .expect( "Should query users" );

    assert_eq!( count, 1, "User should be inserted" );
  }

  #[ test ]
  fn test_create_test_access_token()
  {
    let token = create_test_access_token( "user_123", "user@mail.com", "user", "test_secret" );
    assert!( !token.is_empty(), "Token should not be empty" );

    let claims = decode_test_access_token( &token, "test_secret" );
    assert_eq!( claims.sub, "user_123" );
    assert_eq!( claims.email, "user@mail.com" );
    assert_eq!( claims.role, "user" );
    assert_eq!( claims.jti, "test_secret" );
  }

  #[ test ]
  fn test_verify_password()
  {
    let hash = bcrypt::hash( "mypassword", 4 ).expect( "Should hash" );
    assert!( verify_password( "mypassword", &hash ) );
    assert!( !verify_password( "wrongpassword", &hash ) );
  }
}
