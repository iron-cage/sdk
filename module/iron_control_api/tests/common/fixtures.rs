//! Test data factories for creating valid test fixtures.
//!
//! All fixtures use explicit values (never default parameters) to prevent fragile tests.
//!
//! Fragile Test Prevention: Every parameter must be explicitly provided, no defaults.

use serde_json::json;

/// Create valid login request with explicit credentials.
///
/// Fragile Test Prevention: All parameters explicit, no defaults.
pub fn valid_login_request( username: &str, password: &str ) -> serde_json::Value
{
  json!({
    "username": username,
    "password": password,
  })
}

/// Create valid refresh request with explicit token.
#[ allow( dead_code ) ]
pub fn valid_refresh_request( refresh_token: &str ) -> serde_json::Value
{
  json!({
    "refresh_token": refresh_token,
  })
}

/// Create valid logout request with explicit token.
#[ allow( dead_code ) ]
pub fn valid_logout_request( refresh_token: &str ) -> serde_json::Value
{
  json!({
    "refresh_token": refresh_token,
  })
}

/// Create valid token creation request with explicit parameters.
#[ allow( dead_code ) ]
pub fn valid_token_request( provider: &str, project_id: &str, name: &str ) -> serde_json::Value
{
  json!({
    "provider": provider,
    "project_id": project_id,
    "name": name,
  })
}

/// Create valid limit creation request with explicit parameters.
///
/// Fragile Test Prevention: All parameters explicit, no defaults.
pub fn valid_limit_request(
  limit_type: &str,
  limit_value: i64,
  period: &str,
) -> serde_json::Value
{
  json!({
    "limit_type": limit_type,
    "limit_value": limit_value,
    "period": period,
  })
}

/// Create valid usage record with explicit parameters.
#[ allow( dead_code ) ]
pub fn valid_usage_record(
  provider: &str,
  model: &str,
  input_tokens: i64,
  output_tokens: i64,
  cost_cents: i64,
) -> serde_json::Value
{
  json!({
    "provider": provider,
    "model": model,
    "input_tokens": input_tokens,
    "output_tokens": output_tokens,
    "cost_cents": cost_cents,
  })
}

/// Create valid trace record with explicit parameters.
#[ allow( dead_code ) ]
pub fn valid_trace_record(
  provider: &str,
  model: &str,
  endpoint: &str,
  duration_ms: i64,
) -> serde_json::Value
{
  json!({
    "provider": provider,
    "model": model,
    "endpoint": endpoint,
    "duration_ms": duration_ms,
  })
}

// Invalid request fixtures for error testing

/// Create invalid login request (missing username).
#[ allow( dead_code ) ]
pub fn invalid_login_request_missing_username() -> serde_json::Value
{
  json!({
    "password": "test_password",
  })
}

/// Create invalid login request (missing password).
#[ allow( dead_code ) ]
pub fn invalid_login_request_missing_password() -> serde_json::Value
{
  json!({
    "username": "testuser",
  })
}

/// Create invalid login request (empty username).
#[ allow( dead_code ) ]
pub fn invalid_login_request_empty_username() -> serde_json::Value
{
  json!({
    "username": "",
    "password": "test_password",
  })
}

/// Create invalid login request (empty password).
#[ allow( dead_code ) ]
pub fn invalid_login_request_empty_password() -> serde_json::Value
{
  json!({
    "username": "testuser",
    "password": "",
  })
}

/// Create invalid limit request (invalid type).
#[ allow( dead_code ) ]
pub fn invalid_limit_request_type() -> serde_json::Value
{
  json!({
    "limit_type": "invalid_type",
    "limit_value": 100,
    "period": "monthly",
  })
}

/// Create invalid limit request (negative value).
#[ allow( dead_code ) ]
pub fn invalid_limit_request_negative() -> serde_json::Value
{
  json!({
    "limit_type": "budget",
    "limit_value": -10,
    "period": "monthly",
  })
}

/// Create invalid limit request (zero value).
#[ allow( dead_code ) ]
pub fn invalid_limit_request_zero() -> serde_json::Value
{
  json!({
    "limit_type": "budget",
    "limit_value": 0,
    "period": "monthly",
  })
}

/// Create invalid limit request (invalid period).
#[ allow( dead_code ) ]
pub fn invalid_limit_request_period() -> serde_json::Value
{
  json!({
    "limit_type": "budget",
    "limit_value": 100,
    "period": "invalid_period",
  })
}

/// Create invalid token request (missing provider).
#[ allow( dead_code ) ]
pub fn invalid_token_request_missing_provider() -> serde_json::Value
{
  json!({
    "project_id": "project_123",
    "name": "test_token",
  })
}

/// Create invalid token request (invalid provider).
#[ allow( dead_code ) ]
pub fn invalid_token_request_invalid_provider() -> serde_json::Value
{
  json!({
    "provider": "invalid_provider",
    "project_id": "project_123",
    "name": "test_token",
  })
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_valid_login_request()
  {
    let request = valid_login_request( "testuser", "password123" );
    assert_eq!( request[ "username" ], "testuser" );
    assert_eq!( request[ "password" ], "password123" );
  }

  #[ test ]
  fn test_invalid_login_request_missing_username()
  {
    let request = invalid_login_request_missing_username();
    assert!( request.get( "username" ).is_none() );
    assert!( request.get( "password" ).is_some() );
  }

  #[ test ]
  fn test_valid_limit_request()
  {
    let request = valid_limit_request( "budget", 1000, "monthly" );
    assert_eq!( request[ "limit_type" ], "budget" );
    assert_eq!( request[ "limit_value" ], 1000 );
    assert_eq!( request[ "period" ], "monthly" );
  }
}
