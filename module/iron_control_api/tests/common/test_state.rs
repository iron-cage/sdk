//! Test state builders for creating Axum application state.
//!
//! Provides builders for:
//! - AuthState (JWT authentication)
//! - TokenState (token management)
//! - UsageState (usage tracking)
//! - Combined application state

use sqlx::SqlitePool;
use iron_control_api::routes::auth::AuthState;
use iron_control_api::routes::tokens::TokenState;
use iron_control_api::routes::usage::UsageState;

/// Test JWT secret for all tests (consistent across test runs).
pub const TEST_JWT_SECRET: &str = "test_jwt_secret_key_for_testing_12345";

/// Seed common test users for token tests.
///
/// Creates users that token tests expect:
/// - Normal test users (user_test, user_minimal, etc.)
/// - Security test users (command injection, unicode, etc.)
///
/// This is required because migration 013 added FK constraint from `api_tokens` to users.
async fn seed_test_users_for_tokens( pool: &SqlitePool )
{
  let now_ms = chrono::Utc::now().timestamp_millis();
  let password_hash = "test_hash";

  // Common test users used across token tests
  let mut test_users = vec![
    "user_test", "user_minimal", "user_abc", "user_xyz", "user_001", "user_002",
    "user_003", "user_admin", "user_developer", "user_viewer", "testuser1",
    "testuser2", "testuser3", "testuser4", "testuser5", "test-user", "test_user",
    "user_1", "user1", "user_123", "user_2", "user2", "user_rotate",
    "user_timestamp_test", "user\nid_with_newline",
    // State transition test users
    "user_revoke_test", "user_metadata_test", "user_double_revoke",
    "user_rotation_failure", "user_cascade_test",
    // Corner case test users
    "user_plaintext_test", "user_sha256_test", "user_null_project", "user_valid",
    // Content-type test user
    "test", "test_rotate",
    // Concurrency test users
    "user_rotate_concurrent", "user_revoke_concurrent", "user_rotate_revoke_race",
    // Audit logging test users
    "user_audit_test", "user_revoke_audit",
    // Rate limiting test users
    "user_rate_limit_test", "user_rate_limit_creation",
  ];

  // Security test users (command injection, SQL injection, XSS, unicode, etc.)
  // These are used in corner case and security tests
  test_users.extend_from_slice( &[
    // Command injection
    "; ls -la",
    "| cat /etc/passwd",
    "`whoami`",
    "$(rm -rf /)",
    // SQL injection
    "' OR '1'='1",
    "admin' OR '1'='1",
    "'; DROP TABLE users; --",
    "' UNION SELECT * FROM tokens --",
    "admin'--",
    "1' OR '1' = '1' --",
    "'; DELETE FROM limits WHERE '1'='1",
    // XSS vectors
    "<script>alert('XSS')</script>",
    "<img src=x onerror=alert(1)>",
    "<svg/onload=alert('XSS')>",
    "javascript:alert(1)",
    "<iframe src=\"javascript:alert('XSS')\"></iframe>",
    // Path traversal
    "../../../etc/passwd",
    "..\\..\\..\\windows\\system32\\config\\sam",
    // Unicode
    "用户-user-123",
    "пользователь",
    "مستخدم",
    "ユーザー",
    "👤user🔑token",
    "café_résumé",
    "user\u{200B}hidden",
    "user\nwith\nnewlines",
  ] );

  // Add numbered users for concurrency and uniqueness tests
  for i in 0..=100
  {
    test_users.push( Box::leak( format!( "user_{i}" ).into_boxed_str() ) );
    if i <= 20
    {
      test_users.push( Box::leak( format!( "concurrent_user_{i}" ).into_boxed_str() ) );
      test_users.push( Box::leak( format!( "user_concurrent_{i}" ).into_boxed_str() ) );
    }
  }

  // Add boundary test users for length constraint tests
  test_users.push( Box::leak( "C".repeat( 255 ).into_boxed_str() ) );  // Max valid length (255 chars, matches users.id)

  for user_id in test_users
  {
    // Create email that fits within 255 char limit
    // For long user_ids, truncate to fit
    let max_user_id_for_email = 255 - "@example.com".len();  // 243 chars
    let email = if user_id.len() > max_user_id_for_email
    {
      // Truncate user_id to fit within email constraint
      format!( "{}@example.com", &user_id[ ..max_user_id_for_email ] )
    }
    else
    {
      format!( "{user_id}@example.com" )
    };

    let _ = sqlx::query(
      "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind( user_id )
    .bind( user_id )
    .bind( password_hash )
    .bind( email )
    .bind( "user" )
    .bind( 1 )
    .bind( now_ms )
    .execute( pool )
    .await;
  }
}

/// Create test AuthState with known JWT secret and in-memory database.
pub async fn create_test_auth_state() -> AuthState
{
  AuthState::new( TEST_JWT_SECRET.to_string(), "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create test AuthState" )
}

/// Create test TokenState with in-memory database and seed test users.
pub async fn create_test_token_state() -> TokenState
{
  let token_state = TokenState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create test TokenState" );

  // Seed test users for FK constraint compliance
  seed_test_users_for_tokens( token_state.storage.pool() ).await;

  token_state
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
  pub tokens: TokenState,
  pub database: SqlitePool,
}

impl TestAppState
{
  /// Create new test application state with in-memory database.
  pub async fn new() -> Self
  {
    let auth = create_test_auth_state().await;
    let tokens = create_test_token_state().await;
    let database = super::create_test_database().await;

    Self { auth, tokens, database }
  }

  /// Create new test application state with custom database path.
  ///
  /// Used for concurrency tests where shared database path is needed.
  pub async fn with_db_path( db_path: &str ) -> Self
  {
    let auth = AuthState::new( TEST_JWT_SECRET.to_string(), db_path )
      .await
      .expect( "LOUD FAILURE: Failed to create test AuthState with custom db_path" );

    let tokens = TokenState::new( db_path )
      .await
      .expect( "LOUD FAILURE: Failed to create test TokenState with custom db_path" );

    // Seed test users for FK constraint compliance
    seed_test_users_for_tokens( tokens.storage.pool() ).await;

    let database = SqlitePool::connect( db_path )
      .await
      .expect( "LOUD FAILURE: Failed to connect to custom database path" );

    Self { auth, tokens, database }
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

/// Enable TokenState extraction from TestAppState.
impl axum::extract::FromRef< TestAppState > for TokenState
{
  fn from_ref( state: &TestAppState ) -> Self
  {
    state.tokens.clone()
  }
}

/// Enable SqlitePool extraction from TestAppState.
impl axum::extract::FromRef< TestAppState > for SqlitePool
{
  fn from_ref( state: &TestAppState ) -> Self
  {
    state.database.clone()
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
    let token = auth_state.jwt_secret.generate_access_token( "user_123", "test_user@mail.com", "user", "token_id_001" );
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
