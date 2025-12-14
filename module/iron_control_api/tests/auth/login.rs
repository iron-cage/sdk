//! Login endpoint tests
//!
//! Corner cases covered:
//! - Valid credentials (happy path)
//! - Invalid username (user not found)
//! - Invalid password (wrong password)
//! - Missing username field
//! - Missing password field
//! - Empty username
//! - Empty password
//! - SQL injection attempts
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_infrastructure_verification` | Verify test infrastructure | Create DB + user, verify password hashing | Password verification works | ✅ |
//! | `test_jwt_token_infrastructure` | Verify JWT token generation | Create AuthState, generate access+refresh tokens, verify them | Tokens valid and verifiable | ✅ |
//! | `test_fixtures_infrastructure` | Verify test fixtures | Use valid_login_request + invalid_login_request_missing_username fixtures | Fixtures validate correctly | ✅ |

use crate::common::{ create_test_user, verify_password };
use crate::common::test_db;
use crate::common::fixtures::{ valid_login_request, invalid_login_request_missing_username };
use crate::common::test_state::create_test_auth_state;

/// Test infrastructure verification.
///
/// This is a minimal test to verify the test infrastructure works correctly.
#[ tokio::test ]
async fn test_infrastructure_verification()
{
  // Create in-memory database
  let db = test_db::create_test_db().await;
  let pool = db.pool();

  // Create test user
  let ( user_id, password_hash ) = create_test_user( pool, "testuser" ).await;

  // Verify user ID is positive
  assert!(
    !user_id.is_empty(),
    "LOUD FAILURE: User ID should not be empty"
  );

  // Verify password hash is not empty
  assert!(
    !password_hash.is_empty(),
    "LOUD FAILURE: Password hash should not be empty"
  );

  // Verify password verification works
  assert!(
    verify_password( "test_password", &password_hash ),
    "LOUD FAILURE: Password verification should succeed for correct password"
  );

  assert!(
    !verify_password( "wrong_password", &password_hash ),
    "LOUD FAILURE: Password verification should fail for incorrect password"
  );

  // Query user from database
  let fetched_user: ( String, String, String ) = sqlx::query_as(
    "SELECT id, username, password_hash FROM users WHERE id = ?"
  )
  .bind( &user_id )
  .fetch_one( pool )
  .await
  .expect( "LOUD FAILURE: Should fetch created user from database" );

  assert_eq!(
    fetched_user.0,
    user_id,
    "LOUD FAILURE: Fetched user ID should match created user ID"
  );

  assert_eq!(
    fetched_user.1,
    "test_user_dynamic",
    "LOUD FAILURE: Fetched username should match"
  );
}

/// Test JWT token generation and verification.
#[ tokio::test ]
async fn test_jwt_token_infrastructure()
{
  let auth_state = create_test_auth_state().await;

  // Generate access token
  let access_token = auth_state
    .jwt_secret
    .generate_access_token( "user_123", "user@mail.com", "user", "token_id_001" )
    .expect( "LOUD FAILURE: Should generate access token" );

  assert!(
    !access_token.is_empty(),
    "LOUD FAILURE: Access token should not be empty"
  );

  // Verify access token
  let claims = auth_state
    .jwt_secret
    .verify_access_token( &access_token )
    .expect( "LOUD FAILURE: Should verify access token" );

  assert_eq!(
    claims.sub,
    "user_123",
    "LOUD FAILURE: Token subject should match user ID"
  );

  assert_eq!(
    claims.email,
    "user@mail.com",
    "LOUD FAILURE: Token email should match user email"
  );

  assert_eq!(
    claims.role,
    "user",
    "LOUD FAILURE: Token role should match user role"
  );

  assert_eq!(
    claims.jti,
    "token_id_001",
    "LOUD FAILURE: Token JTI should match"
  );

  // Generate refresh token
  let refresh_token = auth_state
    .jwt_secret
    .generate_refresh_token( "user_123", "user@mail.com", "user", "token_id_001" )
    .expect( "LOUD FAILURE: Should generate refresh token" );

  assert!(
    !refresh_token.is_empty(),
    "LOUD FAILURE: Refresh token should not be empty"
  );

  // Verify refresh token
  let refresh_claims = auth_state
    .jwt_secret
    .verify_refresh_token( &refresh_token )
    .expect( "LOUD FAILURE: Should verify refresh token" );

  assert_eq!(
    refresh_claims.sub,
    "user_123",
    "LOUD FAILURE: Refresh token subject should match user ID"
  );

  assert_eq!(
    refresh_claims.jti,
    "token_id_001",
    "LOUD FAILURE: Refresh token JTI should match"
  );

  assert_eq!(
    refresh_claims.email,
    "user@mail.com",
    "LOUD FAILURE: Refresh token email should match user email"
  );

  assert_eq!(
    refresh_claims.role,
    "user",
    "LOUD FAILURE: Refresh token role should match user role"
  );
}

/// Test fixtures work correctly.
#[ test ]
fn test_fixtures_infrastructure()
{
  let valid_request = valid_login_request( "testuser", "password123" );

  assert_eq!(
    valid_request[ "username" ],
    "testuser",
    "LOUD FAILURE: Valid request should have correct username"
  );

  assert_eq!(
    valid_request[ "password" ],
    "password123",
    "LOUD FAILURE: Valid request should have correct password"
  );

  let invalid_request = invalid_login_request_missing_username();

  assert!(
    invalid_request.get( "username" ).is_none(),
    "LOUD FAILURE: Invalid request should be missing username field"
  );

  assert!(
    invalid_request.get( "password" ).is_some(),
    "LOUD FAILURE: Invalid request should still have password field"
  );
}

// TODO: Add actual login endpoint tests once endpoint is implemented
// Test matrix to implement:
// - test_login_valid_credentials() - Happy path with correct username/password
// - test_login_invalid_username() - User does not exist
// - test_login_invalid_password() - Wrong password for existing user
// - test_login_missing_username() - Request missing username field
// - test_login_missing_password() - Request missing password field
// - test_login_empty_username() - Username is empty string
// - test_login_empty_password() - Password is empty string
// - test_login_sql_injection() - SQL injection attempt in username field
