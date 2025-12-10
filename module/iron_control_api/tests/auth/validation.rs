//! Auth request validation tests (LoginRequest, RefreshRequest, LogoutRequest)
//!
//! Test Matrix: LoginRequest validation
//!
//! | Field    | Valid Values         | Invalid Values           | Edge Cases              |
//! |----------|----------------------|--------------------------|-------------------------|
//! | username | Non-empty string     | "" (empty), whitespace   | 1 char OK, 255 max      |
//! | password | Non-empty string     | "" (empty), whitespace   | 1 char OK, 1000 max     |
//!
//! Test Matrix: RefreshRequest validation
//!
//! | Field         | Valid Values     | Invalid Values           | Edge Cases          |
//! |---------------|------------------|--------------------------|---------------------|
//! | refresh_token | Non-empty JWT    | "" (empty), whitespace   | 1 char OK, 2000 max |
//!
//! Test Matrix: LogoutRequest validation
//!
//! | Field         | Valid Values     | Invalid Values           | Edge Cases          |
//! |---------------|------------------|--------------------------|---------------------|
//! | refresh_token | Non-empty JWT    | "" (empty), whitespace   | 1 char OK, 2000 max |
//!
//! Corner cases covered:
//! - Empty fields (username, password, refresh_token)
//! - Whitespace-only fields
//! - Valid minimal inputs (1 char)
//! - Field length limits (DoS prevention)
//! - Valid complete requests

use iron_control_api::routes::auth::{ LoginRequest, RefreshRequest, LogoutRequest };

/// Test empty username is rejected.
#[ tokio::test ]
async fn test_empty_username_rejected()
{
  let request = LoginRequest
  {
    username: "".to_string(),
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty username must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "username" ),
    "LOUD FAILURE: Error message must mention 'username'"
  );
}

/// Test empty password is rejected.
#[ tokio::test ]
async fn test_empty_password_rejected()
{
  let request = LoginRequest
  {
    username: "valid_user".to_string(),
    password: "".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty password must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "password" ),
    "LOUD FAILURE: Error message must mention 'password'"
  );
}

/// Test whitespace-only username is rejected.
#[ tokio::test ]
async fn test_whitespace_username_rejected()
{
  let request = LoginRequest
  {
    username: "   ".to_string(),
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only username must be rejected"
  );
}

/// Test whitespace-only password is rejected.
#[ tokio::test ]
async fn test_whitespace_password_rejected()
{
  let request = LoginRequest
  {
    username: "valid_user".to_string(),
    password: "   ".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only password must be rejected"
  );
}

/// Test single-character username is accepted.
#[ tokio::test ]
async fn test_single_char_username_accepted()
{
  let request = LoginRequest
  {
    username: "a".to_string(),
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single-character username must be accepted"
  );
}

/// Test single-character password is accepted.
#[ tokio::test ]
async fn test_single_char_password_accepted()
{
  let request = LoginRequest
  {
    username: "valid_user".to_string(),
    password: "p".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single-character password must be accepted"
  );
}

/// Test username too long is rejected (>255 chars).
#[ tokio::test ]
async fn test_username_too_long_rejected()
{
  let long_username = "a".repeat( 256 );
  let request = LoginRequest
  {
    username: long_username,
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Username exceeding 255 characters must be rejected"
  );

  let error_msg = result.unwrap_err();
  assert!(
    error_msg.contains( "username" ) && error_msg.contains( "255" ),
    "LOUD FAILURE: Error must specify username length limit, got: {}", error_msg
  );
}

/// Test password too long is rejected (>1000 chars).
#[ tokio::test ]
async fn test_password_too_long_rejected()
{
  let long_password = "p".repeat( 1001 );
  let request = LoginRequest
  {
    username: "valid_user".to_string(),
    password: long_password,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Password exceeding 1000 characters must be rejected"
  );

  let error_msg = result.unwrap_err();
  assert!(
    error_msg.contains( "password" ) && error_msg.contains( "1000" ),
    "LOUD FAILURE: Error must specify password length limit, got: {}", error_msg
  );
}

/// Test username at max length is accepted (255 chars).
#[ tokio::test ]
async fn test_username_max_length_accepted()
{
  let max_username = "a".repeat( 255 );
  let request = LoginRequest
  {
    username: max_username,
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Username at 255 characters must be accepted"
  );
}

/// Test password at max length is accepted (1000 chars).
#[ tokio::test ]
async fn test_password_max_length_accepted()
{
  let max_password = "p".repeat( 1000 );
  let request = LoginRequest
  {
    username: "valid_user".to_string(),
    password: max_password,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Password at 1000 characters must be accepted"
  );
}

/// Test valid complete request is accepted.
#[ tokio::test ]
async fn test_valid_complete_request()
{
  let request = LoginRequest
  {
    username: "testuser".to_string(),
    password: "testpassword123".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid login request must be accepted"
  );
}

// ============================================================================
// RefreshRequest Validation Tests
// ============================================================================

/// Test empty refresh_token is rejected in RefreshRequest.
#[ tokio::test ]
async fn test_refresh_empty_token_rejected()
{
  let request = RefreshRequest
  {
    refresh_token: "".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty refresh_token must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "refresh_token" ),
    "LOUD FAILURE: Error message must mention 'refresh_token'"
  );
}

/// Test whitespace-only refresh_token is rejected in RefreshRequest.
#[ tokio::test ]
async fn test_refresh_whitespace_token_rejected()
{
  let request = RefreshRequest
  {
    refresh_token: "   ".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only refresh_token must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "refresh_token" ),
    "LOUD FAILURE: Error message must mention 'refresh_token'"
  );
}

/// Test single character refresh_token is accepted in RefreshRequest.
#[ tokio::test ]
async fn test_refresh_single_char_token_accepted()
{
  let request = RefreshRequest
  {
    refresh_token: "a".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single character refresh_token must be accepted"
  );
}

/// Test refresh_token at max length is accepted (2000 chars).
#[ tokio::test ]
async fn test_refresh_token_max_length_accepted()
{
  let max_token = "t".repeat( 2000 );
  let request = RefreshRequest
  {
    refresh_token: max_token,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Refresh token at 2000 characters must be accepted"
  );
}

/// Test refresh_token exceeding max length is rejected (>2000 chars).
#[ tokio::test ]
async fn test_refresh_token_too_long_rejected()
{
  let long_token = "t".repeat( 2001 );
  let request = RefreshRequest
  {
    refresh_token: long_token,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Refresh token exceeding 2000 characters must be rejected"
  );

  let error_msg = result.unwrap_err();
  assert!(
    error_msg.contains( "refresh_token" ) && error_msg.contains( "2000" ),
    "LOUD FAILURE: Error must specify refresh_token length limit, got: {}", error_msg
  );
}

/// Test valid RefreshRequest is accepted.
#[ tokio::test ]
async fn test_refresh_valid_request()
{
  let request = RefreshRequest
  {
    refresh_token: "valid.jwt.token".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid refresh request must be accepted"
  );
}

// ============================================================================
// LogoutRequest Validation Tests
// ============================================================================

/// Test empty refresh_token is rejected in LogoutRequest.
#[ tokio::test ]
async fn test_logout_empty_token_rejected()
{
  let request = LogoutRequest
  {
    refresh_token: "".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty refresh_token must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "refresh_token" ),
    "LOUD FAILURE: Error message must mention 'refresh_token'"
  );
}

/// Test whitespace-only refresh_token is rejected in LogoutRequest.
#[ tokio::test ]
async fn test_logout_whitespace_token_rejected()
{
  let request = LogoutRequest
  {
    refresh_token: "   ".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only refresh_token must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "refresh_token" ),
    "LOUD FAILURE: Error message must mention 'refresh_token'"
  );
}

/// Test single character refresh_token is accepted in LogoutRequest.
#[ tokio::test ]
async fn test_logout_single_char_token_accepted()
{
  let request = LogoutRequest
  {
    refresh_token: "a".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single character refresh_token must be accepted"
  );
}

/// Test refresh_token at max length is accepted in LogoutRequest (2000 chars).
#[ tokio::test ]
async fn test_logout_token_max_length_accepted()
{
  let max_token = "t".repeat( 2000 );
  let request = LogoutRequest
  {
    refresh_token: max_token,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Refresh token at 2000 characters must be accepted"
  );
}

/// Test refresh_token exceeding max length is rejected in LogoutRequest (>2000 chars).
#[ tokio::test ]
async fn test_logout_token_too_long_rejected()
{
  let long_token = "t".repeat( 2001 );
  let request = LogoutRequest
  {
    refresh_token: long_token,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Refresh token exceeding 2000 characters must be rejected"
  );

  let error_msg = result.unwrap_err();
  assert!(
    error_msg.contains( "refresh_token" ) && error_msg.contains( "2000" ),
    "LOUD FAILURE: Error must specify refresh_token length limit, got: {}", error_msg
  );
}

/// Test valid LogoutRequest is accepted.
#[ tokio::test ]
async fn test_logout_valid_request()
{
  let request = LogoutRequest
  {
    refresh_token: "valid.jwt.token".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid logout request must be accepted"
  );
}
