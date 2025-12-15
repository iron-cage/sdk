//! Limit creation validation tests.
//!
//! ## Test Matrix: CreateLimitRequest Validation
//!
//! | Field | Valid Values | Invalid Values | Edge Cases |
//! |-------|--------------|----------------|------------|
//! | max_tokens_per_day | 1..MAX_SAFE | 0, -1, i64::MAX | NULL (None) OK |
//! | max_requests_per_minute | 1..MAX_SAFE | 0, -1, i64::MAX | NULL (None) OK |
//! | max_cost_per_month_microdollars | 1..MAX_SAFE | 0, -1, i64::MAX | NULL (None) OK |
//! | (all limits) | at least one set | all None | - |
//!
//! ## Corner Cases Covered
//! - ✅ All None (must reject - at least one limit required)
//! - ✅ Zero values (must reject - positive only)
//! - ✅ Negative values (must reject)
//! - ✅ Overflow values (i64::MAX - must reject)
//! - ✅ Valid single limit (must accept)
//! - ✅ Valid multiple limits (must accept)
//! - ✅ Mixed valid/invalid (must reject)

use iron_control_api::routes::limits::CreateLimitRequest;

/// Test valid single limit (tokens).
#[ tokio::test ]
async fn test_valid_single_limit_tokens()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( 1000000 ),
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid single limit (tokens) must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test valid single limit (requests).
#[ tokio::test ]
async fn test_valid_single_limit_requests()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: Some( 60 ),
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid single limit (requests) must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test valid single limit (cost).
#[ tokio::test ]
async fn test_valid_single_limit_cost()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: Some( "project_123".to_string() ),
    max_tokens_per_day: None,
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: Some( 500000 ),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid single limit (cost) must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test valid multiple limits.
#[ tokio::test ]
async fn test_valid_multiple_limits()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( 1000000 ),
    max_requests_per_minute: Some( 100 ),
    max_cost_per_month_microdollars: Some( 500000 ),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Valid multiple limits must pass validation. Error: {:?}",
    result.err()
  );
}

/// Test all None rejected (at least one limit required).
#[ tokio::test ]
async fn test_all_none_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Request with all None limits must be rejected"
  );

  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "at least one" ) || err_msg.contains( "required" ),
    "LOUD FAILURE: Error message must indicate at least one limit is required. Got: {}",
    err_msg
  );
}

/// Test zero tokens_per_day rejected.
#[ tokio::test ]
async fn test_zero_tokens_per_day_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( 0 ),
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Zero max_tokens_per_day must be rejected"
  );

  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "positive" ) || err_msg.contains( "greater than" ) || err_msg.contains( "must be" ),
    "LOUD FAILURE: Error message must indicate value must be positive. Got: {}",
    err_msg
  );
}

/// Test negative tokens_per_day rejected.
#[ tokio::test ]
async fn test_negative_tokens_per_day_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( -100 ),
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Negative max_tokens_per_day must be rejected"
  );
}

/// Test zero requests_per_minute rejected.
#[ tokio::test ]
async fn test_zero_requests_per_minute_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: Some( 0 ),
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Zero max_requests_per_minute must be rejected"
  );
}

/// Test negative requests_per_minute rejected.
#[ tokio::test ]
async fn test_negative_requests_per_minute_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: Some( -10 ),
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Negative max_requests_per_minute must be rejected"
  );
}

/// Test zero cost_per_month_cents rejected.
#[ tokio::test ]
async fn test_zero_cost_per_month_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: Some( 0 ),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Zero max_cost_per_month_microdollars must be rejected"
  );
}

/// Test negative cost_per_month_cents rejected.
#[ tokio::test ]
async fn test_negative_cost_per_month_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: Some( -500 ),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Negative max_cost_per_month_microdollars must be rejected"
  );
}

/// Test overflow tokens_per_day rejected.
#[ tokio::test ]
async fn test_overflow_tokens_per_day_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( i64::MAX ),
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: i64::MAX max_tokens_per_day must be rejected (overflow risk)"
  );

  let err_msg = result.unwrap_err().to_string();
  assert!(
    err_msg.contains( "too large" ) || err_msg.contains( "maximum" ) || err_msg.contains( "overflow" ),
    "LOUD FAILURE: Error must mention value is too large. Got: {}",
    err_msg
  );
}

/// Test overflow requests_per_minute rejected.
#[ tokio::test ]
async fn test_overflow_requests_per_minute_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: Some( i64::MAX ),
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: i64::MAX max_requests_per_minute must be rejected (overflow risk)"
  );
}

/// Test overflow cost_per_month_cents rejected.
#[ tokio::test ]
async fn test_overflow_cost_per_month_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: None,
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: Some( i64::MAX ),
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: i64::MAX max_cost_per_month_microdollars must be rejected (overflow risk)"
  );
}

/// Test mixed valid and invalid (one positive, one negative) rejected.
#[ tokio::test ]
async fn test_mixed_valid_invalid_rejected()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( 1000000 ),  // Valid
    max_requests_per_minute: Some( -10 ), // Invalid
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_err(),
    "LOUD FAILURE: Request with mix of valid and invalid limits must be rejected"
  );
}

/// Test boundary value 1 accepted (minimum valid).
#[ tokio::test ]
async fn test_boundary_value_one_accepted()
{
  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( 1 ),
    max_requests_per_minute: Some( 1 ),
    max_cost_per_month_microdollars: Some( 1 ),
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: Boundary value 1 (minimum valid) must be accepted. Error: {:?}",
    result.err()
  );
}

/// Test MAX_SAFE_LIMIT accepted (maximum safe value).
#[ tokio::test ]
async fn test_max_safe_limit_accepted()
{
  const MAX_SAFE: i64 = i64::MAX / 2;

  let request = CreateLimitRequest
  {
    user_id: "user_test".to_string(),
    project_id: None,
    max_tokens_per_day: Some( MAX_SAFE ),
    max_requests_per_minute: None,
    max_cost_per_month_microdollars: None,
  };

  let result = request.validate();
  assert!(
    result.is_ok(),
    "LOUD FAILURE: MAX_SAFE_LIMIT value must be accepted. Error: {:?}",
    result.err()
  );
}
