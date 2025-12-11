//! Test state builders for creating Axum application state.
//!
//! Provides builders for:
//! - AuthState (JWT authentication)
//! - UsageState (usage tracking)
//! - Combined application state

use sqlx::SqlitePool;
use iron_control_api::routes::auth_new::AuthState;
use iron_control_api::routes::usage::UsageState;

/// Test JWT secret for all tests (consistent across test runs).
pub const TEST_JWT_SECRET: &str = "test_jwt_secret_key_for_testing_12345";

/// Create test AuthState with known JWT secret and in-memory database.
pub async fn create_test_auth_state() -> AuthState
{
  AuthState::new( TEST_JWT_SECRET.to_string(), "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create test AuthState" )
}

/// Create test UsageState with in-memory database.
///
/// Note: This requires iron_token_manager's UsageTracker to support in-memory database.
/// If it doesn't, this will need to be updated to use a file-based test database.
#[ allow( dead_code ) ]
pub async fn create_test_usage_state() -> UsageState
{
  UsageState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create test usage state" )
}

/// Combined application state for integration tests.
#[ derive( Clone ) ]
pub struct TestAppState
{
  pub auth: AuthState,
  pub database: SqlitePool,
}

impl TestAppState
{
  /// Create new test application state with in-memory database.
  pub async fn new() -> Self
  {
    let auth = create_test_auth_state().await;
    let database = super::create_test_database().await;

    Self { auth, database }
  }

  /// Get JWT secret for token generation in tests.
  pub fn jwt_secret( &self ) -> String
  {
    TEST_JWT_SECRET.to_string()
  }
}

/// Enable AuthState extraction from TestAppState.
impl axum::extract::FromRef< TestAppState > for AuthState
{
  fn from_ref( state: &TestAppState ) -> Self
  {
    state.auth.clone()
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ tokio::test ]
  async fn test_create_test_auth_state()
  {
    let auth_state = create_test_auth_state().await;

    // Verify JWT generation works (implicitly validates TEST_JWT_SECRET is valid)
    let token = auth_state.jwt_secret.generate_access_token( 1, "test_user", "user" );
    assert!( token.is_ok() );
  }

  #[ tokio::test ]
  async fn test_create_test_app_state()
  {
    let app_state = TestAppState::new().await;
    assert_eq!( app_state.jwt_secret(), TEST_JWT_SECRET );

    // Verify database is accessible
    let result = sqlx::query( "SELECT COUNT(*) FROM users" )
      .fetch_one( &app_state.database )
      .await;

    assert!( result.is_ok(), "Database should be queryable" );
  }
}
