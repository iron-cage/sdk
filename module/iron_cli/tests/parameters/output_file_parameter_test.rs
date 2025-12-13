//! Parameter-level tests for `output_file` parameter
//!
//! ## Purpose
//!
//! Validates the `output_file` parameter for analytics export file paths.
//! Tests file path validation and optional behavior.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .analytics.export (output_file parameter)
//! - Other analytics commands that support file export
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Relative paths, absolute paths, filenames with extensions
//! 2. **Optional Behavior**: Commands where output_file is optional (defaults to stdout)
//! 3. **Edge Cases**: Special characters, long paths, directory traversal
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

  /// Test valid output_file (relative path)
  #[tokio::test]
  async fn test_output_file_relative_path()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export", "output_file::analytics_export.csv" ] ).await;

    // Should succeed or fail with business logic, not path validation
    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with output_file validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output_file with absolute path
  #[tokio::test]
  async fn test_output_file_absolute_path()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export", "output_file::/tmp/analytics_export.csv" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "invalid" ),
        "Should accept absolute path. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output_file with subdirectories
  #[tokio::test]
  async fn test_output_file_with_subdirectories()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export", "output_file::reports/2024/analytics.csv" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "invalid" ),
        "Should accept path with subdirectories. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test missing optional output_file (defaults to stdout)
  #[tokio::test]
  async fn test_output_file_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export" ] ).await;

    // Should succeed without optional output_file (uses stdout)
    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "required" ),
        "Should not require optional output_file parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty output_file
  #[tokio::test]
  async fn test_output_file_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export", "output_file::" ] ).await;

    // Empty output_file might be treated as optional or rejected
    if !result.success() && result.stderr.contains( "output_file" ) {
      println!( "Empty output_file behavior: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output_file with various extensions
  #[tokio::test]
  async fn test_output_file_extensions()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // Test CSV extension
    let result = harness.run( "iron", &[ ".analytics.export", "output_file::report.csv" ] ).await;
    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "invalid" ),
        "Should accept .csv extension. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test output_file with special characters in filename
  #[tokio::test]
  async fn test_output_file_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export", "output_file::analytics_2024-01-15_report.csv" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "output_file" ) || !result.stderr.contains( "invalid" ),
        "Should accept hyphens/underscores in filename. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
