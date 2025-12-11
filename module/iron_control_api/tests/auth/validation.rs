//! Auth request validation tests (LoginRequest, RefreshRequest, LogoutRequest)
//!
//! Test Matrix: LoginRequest validation
//!
//! | Field    | Valid Values         | Invalid Values           | Edge Cases              |
//! |----------|----------------------|--------------------------|-------------------------|
//! | email    | Non-empty string     | "" (empty), whitespace   | 1 char OK, 255 max      |
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
//! - Empty fields (email, password, refresh_token)
//! - Whitespace-only fields
//! - Valid minimal inputs (1 char)
//! - Field length limits (DoS prevention)
//! - Valid complete requests

use iron_control_api::routes::auth_new::{ LoginRequest };

/// Test empty email is rejected.
#[ tokio::test ]
async fn test_empty_email_rejected()
{
  let request = LoginRequest
  {
    email: "".to_string(),
    password: "valid_password".to_string(),
    
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Empty email must be rejected"
  );
  assert!(
    result.unwrap_err().contains( "email" ),
    "LOUD FAILURE: Error message must mention 'email'"
  );
}

/// Test empty password is rejected.
#[ tokio::test ]
async fn test_empty_password_rejected()
{
  let request = LoginRequest
  {
    email: "valid_user".to_string(),
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

/// Test whitespace-only email is rejected.
#[ tokio::test ]
async fn test_whitespace_email_rejected()
{
  let request = LoginRequest
  {
    email: "   ".to_string(),
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only email must be rejected"
  );
}

/// Test whitespace-only password is rejected.
#[ tokio::test ]
async fn test_whitespace_password_rejected()
{
  let request = LoginRequest
  {
    email: "valid_user".to_string(),
    password: "   ".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Whitespace-only password must be rejected"
  );
}

/// Test single-character email is accepted.
#[ tokio::test ]
async fn test_single_char_email_accepted()
{
  let request = LoginRequest
  {
    email: "a".to_string(),
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single-character email must be accepted"
  );
}

/// Test single-character password is accepted.
#[ tokio::test ]
async fn test_single_char_password_accepted()
{
  let request = LoginRequest
  {
    email: "valid_user".to_string(),
    password: "p".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Single-character password must be accepted"
  );
}

/// Test email too long is rejected (>255 chars).
#[ tokio::test ]
async fn test_email_too_long_rejected()
{
  let long_email = "a".repeat( 256 );
  let request = LoginRequest
  {
    email: long_email,
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: email exceeding 255 characters must be rejected"
  );

  let error_msg = result.unwrap_err();
  assert!(
    error_msg.contains( "email" ) && error_msg.contains( "255" ),
    "LOUD FAILURE: Error must specify email length limit, got: {}", error_msg
  );
}

/// Test password too long is rejected (>1000 chars).
#[ tokio::test ]
async fn test_password_too_long_rejected()
{
  let long_password = "p".repeat( 1001 );
  let request = LoginRequest
  {
    email: "valid_user".to_string(),
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

/// Test email at max length is accepted (255 chars).
#[ tokio::test ]
async fn test_email_max_length_accepted()
{
  let max_email = "a".repeat( 255 );
  let request = LoginRequest
  {
    email: max_email,
    password: "valid_password".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: email at 255 characters must be accepted"
  );
}

/// Test password at max length is accepted (1000 chars).
#[ tokio::test ]
async fn test_password_max_length_accepted()
{
  let max_password = "p".repeat( 1000 );
  let request = LoginRequest
  {
    email: "valid_user".to_string(),
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
    email: "testuser".to_string(),
    password: "testpassword123".to_string(),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid login request must be accepted"
  );
}