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
use axum::
{
  Router,
  routing::post,
  extract::{ Request, ConnectInfo },
  middleware::{ self, Next },
  response::Response,
};
use iron_control_api::routes::auth::{ login, logout, refresh, validate, AuthState };
use std::sync::Arc;
use std::net::{ SocketAddr, IpAddr, Ipv4Addr };

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
      name TEXT,
      last_login INTEGER,
      suspended_at INTEGER,
      suspended_by INTEGER,
      deleted_at INTEGER,
      deleted_by INTEGER,
      force_password_change INTEGER NOT NULL DEFAULT 0,
      failed_login_count INTEGER NOT NULL DEFAULT 0,
      last_failed_login INTEGER,
      locked_until INTEGER
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

  // Extract username from email (before @ sign)
  let username = email.split('@').next().unwrap_or(email).replace('.', "_");

  sqlx::query(
    "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at) VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &user_id )
  .bind( &username )
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

/// Seed test user with specified credentials AND name field
///
/// # Arguments
///
/// * `pool` - Database connection pool
/// * `email` - User email (also used as username)
/// * `password` - Plain text password (will be hashed with bcrypt)
/// * `role` - User role ("admin", "user", etc.)
/// * `is_active` - Whether account is active
/// * `name` - User display name
///
/// # Returns
///
/// User ID (for assertions)
#[allow(dead_code)]
pub async fn seed_test_user_with_name(
  pool: &SqlitePool,
  email: &str,
  password: &str,
  role: &str,
  is_active: bool,
  name: &str
) -> String
{
  let password_hash = bcrypt::hash( password, 4 )
    .expect( "LOUD FAILURE: Failed to hash test password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect("LOUD FAILURE: Time went backwards")
    .as_secs() as i64;

  let user_id = format!( "user_{}", uuid::Uuid::new_v4() );

  // Extract username from email (before @ sign)
  let username = email.split('@').next().unwrap_or(email).replace('.', "_");

  sqlx::query(
    "INSERT INTO users (id, username, email, password_hash, role, is_active, created_at, name) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( &user_id )
  .bind( &username )
  .bind( email )
  .bind( &password_hash )
  .bind( role )
  .bind( if is_active { 1 } else { 0 } )
  .bind( now )
  .bind( name )
  .execute( pool )
  .await
  .unwrap_or_else( |_| panic!(
    "LOUD FAILURE: Failed to seed test user '{}' with name '{}'",
    email,
    name
  ) );

  user_id
}

/// Middleware to inject ConnectInfo for tests
///
/// In production, ConnectInfo is provided by `into_make_service_with_connect_info`.
/// For tests using `oneshot()`, we manually inject a fake SocketAddr.
///
/// # Test SocketAddr
///
/// - Default: Uses 127.0.0.1:54321 as the test client address
/// - Custom IP: If x-test-client-ip header is present, uses that IP address
///   (This allows testing IP-based rate limiting with different IPs)
///
/// **Note:** x-test-client-ip is ONLY for testing. Production uses real TCP ConnectInfo.
async fn inject_connect_info( mut request: Request, next: Next ) -> Response
{
  // Check for custom test IP header
  let ip = if let Some( test_ip ) = request.headers().get( "x-test-client-ip" )
  {
    if let Ok( ip_str ) = test_ip.to_str()
    {
      if let Ok( parsed_ip ) = ip_str.parse::<IpAddr>()
      {
        parsed_ip
      }
      else
      {
        IpAddr::V4( Ipv4Addr::new( 127, 0, 0, 1 ) )
      }
    }
    else
    {
      IpAddr::V4( Ipv4Addr::new( 127, 0, 0, 1 ) )
    }
  }
  else
  {
    IpAddr::V4( Ipv4Addr::new( 127, 0, 0, 1 ) )
  };

  // Create fake test socket address
  let addr = SocketAddr::new( ip, 54321 );

  // Insert ConnectInfo extension
  request.extensions_mut().insert( ConnectInfo( addr ) );

  // Continue to next middleware/handler
  next.run( request ).await
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
    .layer( middleware::from_fn( inject_connect_info ) )
}

/// Combined test state (mimics AppState pattern from main server)
///
/// Allows AuthenticatedUser extractor to access AuthState even when
/// routes use UserManagementState (via FromRef trait).
#[derive(Clone)]
struct TestAppState
{
  auth: AuthState,
  users: iron_control_api::routes::users::UserManagementState,
}

/// Enable AuthenticatedUser extractor to access AuthState from TestAppState
impl axum::extract::FromRef< TestAppState > for AuthState
{
  fn from_ref( state: &TestAppState ) -> Self
  {
    state.auth.clone()
  }
}

/// Enable user routes to access UserManagementState from TestAppState
impl axum::extract::FromRef< TestAppState > for iron_control_api::routes::users::UserManagementState
{
  fn from_ref( state: &TestAppState ) -> Self
  {
    state.users.clone()
  }
}

/// Create Axum router with auth + users endpoints for testing
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
/// - POST /api/v1/users (create user - requires admin)
#[allow(dead_code)]
pub async fn create_full_router( pool: SqlitePool ) -> Router
{
  use iron_control_api::routes::users::{ create_user, UserManagementState };
  use iron_control_api::rbac::PermissionChecker;

  // Create auth state
  let jwt_secret = Arc::new( iron_control_api::jwt_auth::JwtSecret::new(
    "test_jwt_secret_for_authentication_tests_only".to_string()
  ) );

  let auth_state = AuthState
  {
    jwt_secret: jwt_secret.clone(),
    db_pool: pool.clone(),
    rate_limiter: iron_control_api::rate_limiter::LoginRateLimiter::new(),
  };

  // Create user management state
  let permission_checker = Arc::new( PermissionChecker::new() );

  let user_state = UserManagementState
  {
    db_pool: pool.clone(),
    permission_checker,
  };

  // Create combined state (allows AuthenticatedUser extractor to work on user routes)
  let app_state = TestAppState
  {
    auth: auth_state,
    users: user_state,
  };

  // Create router with combined state
  Router::new()
    .route( "/api/v1/auth/login", post( login ) )
    .route( "/api/v1/auth/logout", post( logout ) )
    .route( "/api/v1/auth/refresh", post( refresh ) )
    .route( "/api/v1/auth/validate", post( validate ) )
    .route( "/api/v1/users", post( create_user ) )
    .with_state( app_state )
    .layer( middleware::from_fn( inject_connect_info ) )
}
