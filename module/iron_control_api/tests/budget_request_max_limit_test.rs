//! Budget request maximum limit validation tests
//!
//! Tests for GAP-002: $10K maximum budget limit enforcement
//!
//! ## Root Cause (issue-GAP-002)
//!
//! Protocol 012 specification requires max budget of $10,000 for pilot deployment,
//! but original implementation used $1 trillion limit (technical overflow boundary).
//! This allowed budget requests orders of magnitude beyond operational requirements,
//! creating excessive financial exposure risk during pilot phase.
//!
//! ## Why Not Caught
//!
//! - No automated tests existed for budget request workflow validation
//! - Handshake endpoint had $100 limit but budget approval workflow was untested
//! - Protocol 012 maturity tracking showed 92% but missing validation went undetected
//!
//! ## Fix Applied
//!
//! Changed `MAX_BUDGET_USD` constant from `1_000_000_000_000.0` to `10_000.0` in
//! `src/routes/budget/request_workflow.rs:49` with updated rationale documentation.
//!
//! ## Prevention
//!
//! - Created comprehensive test suite (8 tests) covering boundary conditions
//! - Automated validation ensures protocol compliance is testable, not just documented
//! - Tests validate both business logic (pilot limit) and technical limits (overflow)
//!
//! ## Pitfall
//!
//! When protocol specifications define operational limits (pilot $10K) separate from
//! technical limits (i64 overflow $9.2T), implementation must enforce BOTH constraints.
//! Test the operational limit with boundary tests, not just the technical limit.
//! A passing overflow test doesnt validate business requirements.
//!
//! ## Test Coverage
//!
//! 1. ✅ Budget at exact limit ($10,000)
//! 2. ✅ Budget just under limit ($9,999.99)
//! 3. ✅ Budget just over limit ($10,000.01)
//! 4. ✅ Budget significantly over limit ($50,000)
//! 5. ✅ Budget at minimum ($0.01)
//! 6. ✅ Budget validation error message format
//! 7. ✅ Large valid budget ($9,500)
//! 8. ✅ Multiple requests with different amounts

mod common;

use iron_control_api::routes::budget::request_workflow::CreateBudgetRequestRequest;

/// TEST 1: Budget request at exact maximum limit ($10,000)
///
/// # Expected Behavior
///
/// - Request with budget = $10,000 succeeds validation
/// - Boundary condition - maximum allowed value
#[ test ]
fn test_budget_request_at_exact_limit()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 10_000.0, // Exact limit
    justification: "Pilot deployment budget allocation for Q1 2026".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_ok(),
    "Budget request at exact $10K limit should succeed, got: {:?}",
    result
  );
}

/// TEST 2: Budget request just under maximum limit ($9,999.99)
///
/// # Expected Behavior
///
/// - Request with budget = $9,999.99 succeeds validation
/// - Just below boundary - should pass
#[ test ]
fn test_budget_request_just_under_limit()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 9_999.99, // Just under limit
    justification: "Large pilot deployment requiring near-maximum budget".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_ok(),
    "Budget request at $9,999.99 should succeed, got: {:?}",
    result
  );
}

/// TEST 3: Budget request just over maximum limit ($10,000.01)
///
/// # Expected Behavior
///
/// - Request with budget = $10,000.01 fails validation
/// - Error message indicates budget exceeds maximum
#[ test ]
fn test_budget_request_just_over_limit()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 10_000.01, // Just over limit
    justification: "Budget request slightly exceeding pilot limit".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_err(),
    "Budget request at $10,000.01 should fail validation"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "exceeds maximum" ) || error_msg.contains( "10000" ),
    "Error message should mention exceeds/maximum/10000, got: {}",
    error_msg
  );
}

/// TEST 4: Budget request significantly over maximum limit ($50,000)
///
/// # Expected Behavior
///
/// - Request with budget = $50,000 fails validation
/// - Error message indicates budget exceeds maximum
#[ test ]
fn test_budget_request_significantly_over_limit()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 50_000.0, // 5x over limit
    justification: "Large budget request exceeding pilot constraints".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_err(),
    "Budget request at $50,000 should fail validation"
  );

  let error_msg = result.unwrap_err().to_string();
  assert!(
    error_msg.contains( "exceeds maximum" ) && error_msg.contains( "10000" ),
    "Error should mention 'exceeds maximum' and '10000', got: {}",
    error_msg
  );
}

/// TEST 5: Budget request at minimum valid amount ($0.01)
///
/// # Expected Behavior
///
/// - Request with budget = $0.01 succeeds validation
/// - Minimum positive value - should pass
#[ test ]
fn test_budget_request_at_minimum()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 0.01, // Minimum valid
    justification: "Minimal budget allocation for testing purposes only".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_ok(),
    "Budget request at $0.01 should succeed, got: {:?}",
    result
  );
}

/// TEST 6: Error message format verification
///
/// # Expected Behavior
///
/// - Error message includes both "exceeds maximum" and the actual limit value
/// - Provides actionable information for API clients
#[ test ]
fn test_error_message_format()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 25_000.0,
    justification: "Request exceeding pilot deployment maximum budget limit".to_string(),
  };

  let result = request.validate();

  assert!( result.is_err(), "Should fail validation" );

  let error_msg = result.unwrap_err().to_string();

  // Error should be specific and actionable
  assert!(
    error_msg.contains( "requested_budget_usd" ),
    "Error should mention field name, got: {}",
    error_msg
  );

  assert!(
    error_msg.contains( "10000" ),
    "Error should include limit value, got: {}",
    error_msg
  );

  assert!(
    error_msg.contains( "exceeds" ) || error_msg.contains( "maximum" ),
    "Error should explain why it failed, got: {}",
    error_msg
  );
}

/// TEST 7: Large valid budget within limit ($9,500)
///
/// # Expected Behavior
///
/// - Request with budget = $9,500 succeeds validation
/// - Common large request amount - should pass
#[ test ]
fn test_budget_request_large_but_valid()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 9_500.0,
    justification: "Major pilot deployment requiring substantial budget allocation".to_string(),
  };

  let result = request.validate();

  assert!(
    result.is_ok(),
    "Budget request at $9,500 should succeed, got: {:?}",
    result
  );
}

/// TEST 8: Multiple validation failures - budget over limit
///
/// # Expected Behavior
///
/// - Validation stops at first error (over budget limit)
/// - Returns budget-specific error, not other validation errors
#[ test ]
fn test_validation_fails_on_budget_limit_first()
{
  let request = CreateBudgetRequestRequest
  {
    agent_id: 1,
    requester_id: "user_001".to_string(),
    requested_budget_usd: 100_000.0, // Over limit
    justification: "Invalid".to_string(), // Also too short (< 20 chars)
  };

  let result = request.validate();

  assert!( result.is_err(), "Should fail validation" );

  let error_msg = result.unwrap_err().to_string();

  // Should fail on budget limit before reaching justification length check
  // (validation order matters for error reporting)
  assert!(
    error_msg.contains( "budget" ) || error_msg.contains( "exceeds" ) || error_msg.contains( "10000" ),
    "Should fail on budget limit check first, got: {}",
    error_msg
  );
}
