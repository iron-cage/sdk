//! Health and version handler tests
//!
//! ## Test Coverage
//!
//! Covers health and version handlers.
//! Total: 3 test cases

use std::collections::HashMap;
use iron_cli::handlers::health_handlers::*;

// ============================================================================
// .health tests (2 tests)
// ============================================================================

#[test]
fn test_health_handler_success()
{
  let params = HashMap::new();

  let result = health_handler(&params);

  assert!(result.is_ok(), "Should succeed");
  let output = result.unwrap();
  assert!(!output.is_empty(), "Should return health status");
}

#[test]
fn test_health_handler_format_json()
{
  let mut params = HashMap::new();
  params.insert("format".into(), "json".into());

  let result = health_handler(&params);

  assert!(result.is_ok(), "Should succeed with json format");
}

// ============================================================================
// .version tests (1 test)
// ============================================================================

#[test]
fn test_version_handler_success()
{
  let params = HashMap::new();

  let result = version_handler(&params);

  assert!(result.is_ok(), "Should succeed");
  let output = result.unwrap();
  assert!(!output.is_empty(), "Should return version string");
}
