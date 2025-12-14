//! Source code analysis utilities for NEGATIVE ACCEPTANCE tests.
//!
//! ## Purpose
//!
//! Provides helpers for analyzing source code to verify:
//! - Old patterns are absent (NEGATIVE ACCEPTANCE)
//! - New patterns are present (positive validation)
//! - Code structure matches expectations
//!
//! ## Usage
//!
//! ```rust,no_run
//! use common::source_analysis::*;
//!
//! fn test_no_hardcoded_values() {
//!   let source = read_source_file("src/config.rs");
//!   assert_source_not_contains(
//!     &source,
//!     "localhost:3001",
//!     "src/config.rs",
//!     "Hardcoded URLs forbidden (use environment variables)"
//!   );
//! }
//! ```

use std::fs;

/// Read source file for analysis.
///
/// ## Purpose
///
/// Centralize source file reading for NEGATIVE ACCEPTANCE tests.
/// Provides clear error messages if file missing or unreadable.
///
/// ## Parameters
///
/// - `path`: Relative path from crate root (e.g., "src/config.rs")
///
/// ## Returns
///
/// File contents as String
///
/// ## Panics
///
/// Panics with LOUD FAILURE if:
/// - File doesn't exist
/// - File not readable
/// - File contains invalid UTF-8
///
/// ## Example
///
/// ```rust
/// let source = read_source_file("src/middleware/mod.rs");
/// assert!(!source.contains("old_middleware"));
/// ```
pub fn read_source_file( path: &str ) -> String
{
  fs::read_to_string( path )
    .unwrap_or_else( |_| panic!(
      "LOUD FAILURE: Failed to read source file: {}\n\
       This may indicate:\n\
       - File moved or deleted\n\
       - Incorrect path (check from crate root)\n\
       - Permissions issue",
      path
    ) )
}

/// Assert source does NOT contain pattern.
///
/// ## Purpose
///
/// NEGATIVE ACCEPTANCE: Prove old patterns absent from source code.
/// Used to verify backward compatibility code deleted, hardcoded values removed, etc.
///
/// ## Parameters
///
/// - `source`: Source code content (from read_source_file)
/// - `pattern`: Pattern that must NOT exist
/// - `file_path`: Path for error message
/// - `message`: Reason why pattern forbidden
///
/// ## Panics
///
/// Panics with detailed message if pattern found in source.
///
/// ## Example
///
/// ```rust
/// let source = read_source_file("src/config.rs");
/// assert_source_not_contains(
///   &source,
///   "allow_origin([",
///   "src/config.rs",
///   "Hardcoded CORS arrays forbidden (must use env var)"
/// );
/// ```
pub fn assert_source_not_contains(
  source: &str,
  pattern: &str,
  file_path: &str,
  message: &str,
)
{
  assert!(
    !source.contains( pattern ),
    "FAIL: Pattern '{}' found in {}\n\
     Reason: {}\n\
     \n\
     NEGATIVE ACCEPTANCE violation: This pattern must NOT exist in source code.\n\
     \n\
     Fix: Remove all occurrences of this pattern.",
    pattern,
    file_path,
    message
  );
}

/// Assert source DOES contain pattern.
///
/// ## Purpose
///
/// Positive validation: Prove new patterns present in source code.
/// Used to verify environment variable parsing added, new config present, etc.
///
/// ## Parameters
///
/// - `source`: Source code content (from read_source_file)
/// - `pattern`: Pattern that MUST exist
/// - `file_path`: Path for error message
/// - `message`: Reason why pattern required
///
/// ## Panics
///
/// Panics with detailed message if pattern missing from source.
///
/// ## Example
///
/// ```rust
/// let source = read_source_file("src/config.rs");
/// assert_source_contains(
///   &source,
///   "std::env::var(\"ALLOWED_ORIGINS\")",
///   "src/config.rs",
///   "ALLOWED_ORIGINS environment variable parsing required"
/// );
/// ```
pub fn assert_source_contains(
  source: &str,
  pattern: &str,
  file_path: &str,
  message: &str,
)
{
  assert!(
    source.contains( pattern ),
    "FAIL: Required pattern '{}' missing in {}\n\
     Reason: {}\n\
     \n\
     Expected: This pattern must exist in source code.\n\
     \n\
     Fix: Add the required implementation.",
    pattern,
    file_path,
    message
  );
}

/// Assert file does NOT exist.
///
/// ## Purpose
///
/// NEGATIVE ACCEPTANCE: Prove files deleted (not just renamed or moved).
/// Used to verify backward compatibility code completely removed.
///
/// ## Parameters
///
/// - `file_path`: Path that must NOT exist
/// - `message`: Reason why file forbidden
///
/// ## Panics
///
/// Panics with detailed message if file exists.
///
/// ## Example
///
/// ```rust
/// assert_file_not_exists(
///   "src/middleware/url_redirect.rs",
///   "Backward compatibility middleware must be deleted completely"
/// );
/// ```
pub fn assert_file_not_exists( file_path: &str, message: &str )
{
  assert!(
    !std::path::Path::new( file_path ).exists(),
    "FAIL: File '{}' still exists\n\
     Reason: {}\n\
     \n\
     NEGATIVE ACCEPTANCE violation: File must be deleted (not renamed, not moved).\n\
     \n\
     Fix: Delete file completely using:\n\
     git rm {}\n\
     or\n\
     rm {}",
    file_path,
    message,
    file_path,
    file_path
  );
}

/// Assert file DOES exist.
///
/// ## Purpose
///
/// Positive validation: Prove required files created.
///
/// ## Parameters
///
/// - `file_path`: Path that MUST exist
/// - `message`: Reason why file required
///
/// ## Panics
///
/// Panics with detailed message if file missing.
///
/// ## Example
///
/// ```rust
/// assert_file_exists(
///   "src/config/production.rs",
///   "Production configuration must exist"
/// );
/// ```
pub fn assert_file_exists( file_path: &str, message: &str )
{
  assert!(
    std::path::Path::new( file_path ).exists(),
    "FAIL: Required file '{}' missing\n\
     Reason: {}\n\
     \n\
     Expected: File must exist.\n\
     \n\
     Fix: Create the required file.",
    file_path,
    message
  );
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_read_source_file_success()
  {
    // Read this test file itself
    let source = read_source_file( "tests/common/source_analysis.rs" );
    assert!( source.contains( "test_read_source_file_success" ) );
  }

  #[ test ]
  #[ should_panic( expected = "LOUD FAILURE" ) ]
  fn test_read_source_file_missing()
  {
    let _ = read_source_file( "nonexistent_file.rs" );
  }

  #[ test ]
  fn test_assert_source_not_contains_pass()
  {
    let source = "fn main() {}";
    assert_source_not_contains(
      source,
      "evil_code",
      "test.rs",
      "Evil code forbidden"
    );
  }

  #[ test ]
  #[ should_panic( expected = "FAIL" ) ]
  fn test_assert_source_not_contains_fail()
  {
    let source = "fn main() { evil_code(); }";
    assert_source_not_contains(
      source,
      "evil_code",
      "test.rs",
      "Evil code forbidden"
    );
  }

  #[ test ]
  fn test_assert_source_contains_pass()
  {
    let source = "fn main() { good_code(); }";
    assert_source_contains(
      source,
      "good_code",
      "test.rs",
      "Good code required"
    );
  }

  #[ test ]
  #[ should_panic( expected = "FAIL" ) ]
  fn test_assert_source_contains_fail()
  {
    let source = "fn main() {}";
    assert_source_contains(
      source,
      "missing_code",
      "test.rs",
      "Missing code required"
    );
  }

  #[ test ]
  fn test_assert_file_exists_pass()
  {
    assert_file_exists(
      "tests/common/source_analysis.rs",
      "Test file must exist"
    );
  }

  #[ test ]
  #[ should_panic( expected = "FAIL" ) ]
  fn test_assert_file_exists_fail()
  {
    assert_file_exists(
      "nonexistent_file.rs",
      "File must exist"
    );
  }

  #[ test ]
  fn test_assert_file_not_exists_pass()
  {
    assert_file_not_exists(
      "nonexistent_file.rs",
      "File must not exist"
    );
  }

  #[ test ]
  #[ should_panic( expected = "FAIL" ) ]
  fn test_assert_file_not_exists_fail()
  {
    assert_file_not_exists(
      "tests/common/source_analysis.rs",
      "File must not exist"
    );
  }
}
