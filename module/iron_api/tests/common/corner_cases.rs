//! Corner case test utilities and reusable test data
//!
//! Provides comprehensive test vectors for security, edge cases, and boundary testing.
//! Used across all endpoint tests for systematic corner case coverage.

#![allow(dead_code)]

/// SQL injection attack vectors
pub const SQL_INJECTIONS: &[&str] = &[
  "' OR '1'='1",
  "'; DROP TABLE users; --",
  "' UNION SELECT * FROM tokens --",
  "admin'--",
  "1' OR '1' = '1' --",
  "'; DELETE FROM limits WHERE '1'='1",
];

/// Cross-site scripting (XSS) attack vectors
pub const XSS_VECTORS: &[&str] = &[
  "<script>alert('XSS')</script>",
  "<img src=x onerror=alert(1)>",
  "<svg/onload=alert('XSS')>",
  "javascript:alert(1)",
  "<iframe src=\"javascript:alert('XSS')\"></iframe>",
];

/// Path traversal attack vectors
pub const PATH_TRAVERSAL: &[&str] = &[
  "../../../etc/passwd",
  "..\\..\\..\\windows\\system32",
  "%2e%2e%2f%2e%2e%2f",
  "....//....//....//etc/passwd",
];

/// Command injection attack vectors
pub const COMMAND_INJECTION: &[&str] = &[
  "; ls -la",
  "| cat /etc/passwd",
  "`whoami`",
  "$(rm -rf /)",
];

/// Unicode test strings (various languages and special characters)
pub const UNICODE_STRINGS: &[&str] = &[
  "用户-user-123",           // Chinese
  "пользователь",            // Russian (Cyrillic)
  "مستخدم",                  // Arabic (RTL)
  "ユーザー",                // Japanese
  "👤user🔑token",          // Emoji
  "café_résumé",             // Accents
  "user\u{200B}hidden",      // Zero-width space
  "user\nwith\nnewlines",    // Embedded newlines
];

/// Very long strings for DoS testing
pub fn long_string(length: usize) -> String {
  "A".repeat(length)
}

/// Empty and whitespace strings
pub const EMPTY_WHITESPACE: &[&str] = &[
  "",                // Empty
  " ",               // Single space
  "   ",             // Multiple spaces
  "\t",              // Tab
  "\n",              // Newline
  "\r\n",            // CRLR
  " \t\n\r ",        // Mixed whitespace
];

/// Control characters
pub const CONTROL_CHARS: &[&str] = &[
  "\x00",            // Null byte
  "\x01\x02\x03",    // SOH, STX, ETX
  "\x1B",            // ESC
  "\x7F",            // DEL
];

/// Special characters that might break parsing
pub const SPECIAL_CHARS: &[&str] = &[
  "!@#$%^&*()",
  "{}[]|\\",
  "\"'`",
  "<>?/",
  "user@domain.com",
  "user+tag@domain.com",
];

/// URL encoding edge cases
pub const URL_ENCODING: &[&str] = &[
  "%20",             // Space
  "%3C%3E",          // <>
  "%00",             // Null byte
  "user%2Btest",     // +
];

/// Numeric boundary test values for i64
pub const I64_BOUNDARIES: &[i64] = &[
  i64::MIN,          // -9223372036854775808
  i64::MIN + 1,
  -1_000_000,
  -1,
  0,
  1,
  1_000_000,
  i64::MAX - 1,
  i64::MAX,          // 9223372036854775807
];

/// Numeric boundary test values for Option<i64> (cost fields)
pub const OPTIONAL_I64_BOUNDARIES: &[Option<i64>] = &[
  None,
  Some(0),
  Some(1),
  Some(-1),
  Some(i64::MIN),
  Some(i64::MAX),
];

/// Invalid JSON payloads
pub const INVALID_JSON: &[&str] = &[
  "{",               // Incomplete
  "}",               // No opening
  "{{}",             // Malformed
  "{'key': 'value'}", // Single quotes
  "{key: value}",    // Unquoted keys
  "null",            // Valid JSON but wrong type
  "[]",              // Array instead of object
];

/// Content-Type header test values
pub const CONTENT_TYPES: &[&str] = &[
  "application/json",
  "application/json; charset=utf-8",
  "text/plain",
  "application/xml",
  "multipart/form-data",
  "",                // Missing
];

/// HTTP methods for testing unsupported methods
pub const HTTP_METHODS: &[&str] = &[
  "GET",
  "POST",
  "PUT",
  "DELETE",
  "PATCH",
  "HEAD",
  "OPTIONS",
  "TRACE",
  "CONNECT",
];

/// Helper function to generate SQL injection test for string field
///
/// # Example
/// ```
/// let test_cases = sql_injection_cases("user_id");
/// assert_eq!(test_cases.len(), 6);
/// ```
pub fn sql_injection_cases(field_name: &str) -> Vec<(String, &'static str)> {
  SQL_INJECTIONS
    .iter()
    .map(|injection| (format!("{}: {}", field_name, injection), *injection))
    .collect()
}

/// Helper function to generate XSS test cases for string field
pub fn xss_cases(field_name: &str) -> Vec<(String, &'static str)> {
  XSS_VECTORS
    .iter()
    .map(|xss| (format!("{}: {}", field_name, xss), *xss))
    .collect()
}

/// Helper function to generate Unicode test cases
pub fn unicode_cases(field_name: &str) -> Vec<(String, &'static str)> {
  UNICODE_STRINGS
    .iter()
    .map(|unicode| (format!("{}: {}", field_name, unicode), *unicode))
    .collect()
}

/// Helper function to generate empty/whitespace test cases
pub fn empty_whitespace_cases(field_name: &str) -> Vec<(String, &'static str)> {
  EMPTY_WHITESPACE
    .iter()
    .map(|ws| (format!("{}: '{}'", field_name, ws), *ws))
    .collect()
}

/// Test case for numeric boundary testing
#[derive(Debug, Clone)]
pub struct NumericBoundaryTest {
  pub description: String,
  pub value: i64,
  pub expected_valid: bool,
}

impl NumericBoundaryTest {
  pub fn new(description: impl Into<String>, value: i64, expected_valid: bool) -> Self {
    Self {
      description: description.into(),
      value,
      expected_valid,
    }
  }
}

/// Generate numeric boundary test cases for cost fields (must be positive)
pub fn cost_boundary_cases() -> Vec<NumericBoundaryTest> {
  vec![
    NumericBoundaryTest::new("i64::MIN", i64::MIN, false),
    NumericBoundaryTest::new("negative large", -1_000_000, false),
    NumericBoundaryTest::new("negative one", -1, false),
    NumericBoundaryTest::new("zero", 0, false),
    NumericBoundaryTest::new("one cent", 1, true),
    NumericBoundaryTest::new("normal value", 10_000, true),
    NumericBoundaryTest::new("large value", 1_000_000_000, true),
    NumericBoundaryTest::new("i64::MAX", i64::MAX, true),
  ]
}

/// Generate numeric boundary test cases for ID fields (must be positive)
pub fn id_boundary_cases() -> Vec<NumericBoundaryTest> {
  vec![
    NumericBoundaryTest::new("i64::MIN", i64::MIN, false),
    NumericBoundaryTest::new("negative", -1, false),
    NumericBoundaryTest::new("zero", 0, false),
    NumericBoundaryTest::new("one", 1, true),
    NumericBoundaryTest::new("normal ID", 12345, true),
    NumericBoundaryTest::new("very large ID", 999_999_999, true),
    NumericBoundaryTest::new("i64::MAX", i64::MAX, true),
  ]
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_sql_injection_cases() {
    let cases = sql_injection_cases("user_id");
    assert_eq!(cases.len(), 6);
    assert!(cases[0].1.contains("OR"));
  }

  #[test]
  fn test_xss_cases() {
    let cases = xss_cases("description");
    assert_eq!(cases.len(), 5);
    assert!(cases[0].1.contains("script"));
  }

  #[test]
  fn test_unicode_cases() {
    let cases = unicode_cases("project_id");
    assert_eq!(cases.len(), 8);
  }

  #[test]
  fn test_cost_boundary_cases() {
    let cases = cost_boundary_cases();
    assert_eq!(cases.len(), 8);
    assert!(!cases[0].expected_valid); // i64::MIN invalid
    assert!(cases[4].expected_valid);  // 1 cent valid
  }

  #[test]
  fn test_long_string_generation() {
    let s = long_string(10_000);
    assert_eq!(s.len(), 10_000);
    assert!(s.chars().all(|c| c == 'A'));
  }
}
