//! SQL Injection Helper Function Verification Tests
//!
//! Tests that verify the SQL injection helper functions work correctly.
//!
//! # TDD Phase: RED
//! This test is written FIRST, before sql_injection_helpers.rs exists.
//! Expected to fail with compilation error (helper functions not found).
//!
//! # Rulebook Compliance
//! - test_organization.rulebook.md: Test in tests/ directory ✓
//! - code_style.rulebook.md: 2-space indentation ✓
//! - codebase_hygiene.rulebook.md: Clear test name ✓
//! - files_structure.rulebook.md: Checked readme.md, no overlap ✓

#![cfg(test)]

use std::time::Duration;
use crate::common::sql_injection_helpers::*;

/// Test: verify_response_secure() detects SQL keywords in response
#[test]
#[should_panic(expected = "SQL keyword")]
fn test_verify_response_secure_detects_sql_keywords()
{
  let mut response = TestResponse::new();
  response.set_status(401);
  response.set_body("Error: SQL syntax error near 'SELECT'");

  // Should panic because response contains SQL keyword
  verify_response_secure(&response);
}

/// Test: verify_response_secure() detects schema leakage
#[test]
#[should_panic(expected = "table name")]
fn test_verify_response_secure_detects_schema_leakage()
{
  let mut response = TestResponse::new();
  response.set_status(401);
  response.set_body("Error in users table");

  // Should panic because response leaks table name
  verify_response_secure(&response);
}

/// Test: verify_response_secure() passes for secure response
#[test]
fn test_verify_response_secure_passes_for_safe_response()
{
  let mut response = TestResponse::new();
  response.set_status(401);
  response.set_body("Authentication failed");

  // Should NOT panic - this is a secure response
  verify_response_secure(&response);
}

/// Test: verify_authentication_failed() checks 401 status
#[test]
fn test_verify_authentication_failed_checks_status()
{
  let mut response = TestResponse::new();
  response.set_status(401);
  response.set_body("Authentication failed");

  // Should pass - correct auth failure
  verify_authentication_failed(&response);
}

/// Test: verify_no_sql_leakage() detects SQL keywords
#[test]
#[should_panic(expected = "SQL keyword")]
fn test_verify_no_sql_leakage_detects_keywords()
{
  let mut response = TestResponse::new();
  response.set_status(401);
  response.set_body("Error: SELECT failed");

  // Should panic - contains SQL keyword
  verify_no_sql_leakage(&response);
}

/// Test: verify_no_timing_attack() detects slow responses
#[test]
#[should_panic(expected = "timing attack")]
fn test_verify_no_timing_attack_detects_delays()
{
  let slow_response = Duration::from_secs(5);

  // Should panic - response too slow (5 seconds)
  verify_no_timing_attack(slow_response);
}

/// Test: verify_no_timing_attack() passes for fast responses
#[test]
fn test_verify_no_timing_attack_passes_for_fast_response()
{
  let fast_response = Duration::from_millis(100);

  // Should pass - response under 1 second
  verify_no_timing_attack(fast_response);
}

/// Test: send_login_request() sends request with email and password
#[test]
fn test_send_login_request_integration()
{
  let response = send_login_request("test@example.com", "password123");

  // Should return a TestResponse
  assert!(response.status().as_u16() >= 200);
  assert!(response.status().as_u16() < 600);
}

/// Test: send_logout_request() sends request with token
#[test]
fn test_send_logout_request_integration()
{
  let response = send_logout_request("Bearer some_token");

  // Should return a TestResponse
  assert!(response.status().as_u16() >= 200);
  assert!(response.status().as_u16() < 600);
}
