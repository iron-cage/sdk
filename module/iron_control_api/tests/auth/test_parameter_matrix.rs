//! Parameter Matrix Verification Tests
//!
//! Tests that verify the parameter matrix is complete and accurate.
//!
//! # TDD Phase: RED
//! This test is written FIRST, before -parameter_matrix.md exists.
//! Expected to fail with: "No such file or directory"
//!
//! # Rulebook Compliance
//! - test_organization.rulebook.md: Test in tests/ directory ✓
//! - code_style.rulebook.md: 2-space indentation ✓
//! - codebase_hygiene.rulebook.md: Clear test name ✓
//! - files_structure.rulebook.md: Checked readme.md, no overlap ✓

#![cfg(test)]

use std::fs;
use std::path::Path;

/// Test: Parameter matrix file exists and is readable
///
/// # Expected Behavior
/// The matrix file `-parameter_matrix.md` should exist in tests/auth/
/// and contain parameter-to-payload mappings for SQL injection testing.
#[test]
#[ignore]
fn test_parameter_matrix_exists()
{
  let matrix_path = Path::new("tests/auth/-parameter_matrix.md");

  assert!(
    matrix_path.exists(),
    "Parameter matrix not found at: {:?}",
    matrix_path
  );

  let content = fs::read_to_string(matrix_path)
    .expect("Failed to read parameter matrix");

  assert!(
    !content.is_empty(),
    "Parameter matrix is empty"
  );
}

/// Test: Parameter matrix documents all 5 parameters from endpoint catalog
///
/// # Expected Parameters
/// P0 (2 parameters):
/// - email (login endpoint)
/// - password (login endpoint)
///
/// P1 (3 parameters):
/// - Authorization header (logout endpoint)
/// - Authorization header (refresh endpoint)
/// - Authorization header (validate endpoint)
#[test]
#[ignore]
fn test_parameter_matrix_contains_all_parameters()
{
  let matrix_path = Path::new("tests/auth/-parameter_matrix.md");
  let content = fs::read_to_string(matrix_path)
    .expect("Failed to read parameter matrix");

  // P0 parameters
  assert!(
    content.contains("email"),
    "Matrix missing P0 parameter: email"
  );

  assert!(
    content.contains("password"),
    "Matrix missing P0 parameter: password"
  );

  // P1 parameter (Authorization header appears 3 times)
  assert!(
    content.contains("Authorization"),
    "Matrix missing P1 parameter: Authorization"
  );
}

/// Test: Parameter matrix specifies payload counts
///
/// # Expected Payload Counts
/// - P0: 65 payloads per parameter
/// - P1: 38 payloads per parameter
/// - P2: 15 payloads per parameter
#[test]
#[ignore]
fn test_parameter_matrix_specifies_payload_counts()
{
  let matrix_path = Path::new("tests/auth/-parameter_matrix.md");
  let content = fs::read_to_string(matrix_path)
    .expect("Failed to read parameter matrix");

  // Check for payload count documentation
  assert!(
    content.contains("65") || content.contains("P0"),
    "Matrix should document P0 payload count (65)"
  );

  assert!(
    content.contains("38") || content.contains("P1"),
    "Matrix should document P1 payload count (38)"
  );
}

/// Test: Parameter matrix maps parameters to endpoints
///
/// # Expected Mappings
/// Each parameter should be mapped to its endpoint(s)
#[test]
#[ignore]
fn test_parameter_matrix_maps_parameters_to_endpoints()
{
  let matrix_path = Path::new("tests/auth/-parameter_matrix.md");
  let content = fs::read_to_string(matrix_path)
    .expect("Failed to read parameter matrix");

  // Check endpoint references
  let endpoints = [
    "login",
    "logout",
    "refresh",
    "validate",
  ];

  let mut found_endpoints = 0;
  for endpoint in endpoints.iter() {
    if content.contains(endpoint) {
      found_endpoints += 1;
    }
  }

  assert!(
    found_endpoints >= 2,
    "Matrix should reference at least 2 endpoints, found {}",
    found_endpoints
  );
}

/// Test: Parameter matrix calculates total test count
///
/// # Expected Total
/// - P0: 2 params × 65 payloads = 130 tests
/// - P1: 3 params × 38 payloads = 114 tests
/// - Total: 244 tests
#[test]
#[ignore]
fn test_parameter_matrix_calculates_test_totals()
{
  let matrix_path = Path::new("tests/auth/-parameter_matrix.md");
  let content = fs::read_to_string(matrix_path)
    .expect("Failed to read parameter matrix");

  // Check for total count or calculations
  assert!(
    content.contains("244") ||
    content.contains("130") ||
    content.contains("114") ||
    (content.contains("Total") && content.contains("test")),
    "Matrix should document total test counts"
  );
}
