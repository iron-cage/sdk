//! Parameter-level tests for `output` parameter
//!
//! ## Purpose
//!
//! Validates the `output` parameter for output file specification.
//! Tests file path validation for output operations.
//!
//! ## Coverage
//!
//! Commands tested:
//! - Commands that support output file parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Relative paths, absolute paths, filenames
//! 2. **Optional Behavior**: Commands where output is optional
//! 3. **Edge Cases**: Special characters, long paths, stdout
//!
//! ## TDD Status
//!
//! RED: Writing tests (below)
//! GREEN: Implementation pending
//! REFACTOR: Pending

#[cfg(test)]
mod tests
{
  use crate::fixtures::{ IntegrationTestHarness, TestData, TestServer };

  /// Test valid output (relative path)
  #[tokio::test]
  async fn test_output_relative_path()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal", "output::output.txt" ] ).await;

    // Should succeed or fail with business logic, not path validation
    if !result.success() {
      assert!( !result.stderr.contains( "output" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with output validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test missing optional output
  #[tokio::test]
  async fn test_output_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal" ] ).await;

    // Should succeed without optional output (uses stdout)
    assert!( result.success() || !result.stderr.contains( "output" ) || !result.stderr.contains( "required" ),
      "Should not require optional output parameter. Stdout: {}, Stderr: {}", result.stdout, result.stderr );

    server.shutdown().await;
  }

  /// Test empty output
  #[tokio::test]
  async fn test_output_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal", "output::" ] ).await;

    // Empty output might be treated as optional or rejected
    if !result.success() && result.stderr.contains( "output" ) {
      println!( "Empty output behavior: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output with absolute path
  #[tokio::test]
  async fn test_output_absolute_path()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal", "output::/tmp/output.txt" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output" ) || !result.stderr.contains( "invalid" ),
        "Should accept absolute path. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output with subdirectories
  #[tokio::test]
  async fn test_output_with_subdirectories()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal", "output::reports/daily/output.txt" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output" ) || !result.stderr.contains( "invalid" ),
        "Should accept path with subdirectories. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output with special characters
  #[tokio::test]
  async fn test_output_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.minimal", "output::report-2024_01_15.txt" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output" ) || !result.stderr.contains( "invalid" ),
        "Should accept hyphens/underscores in filename. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
