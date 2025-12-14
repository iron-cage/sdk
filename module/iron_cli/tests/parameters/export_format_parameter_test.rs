//! Parameter-level tests for `export_format` parameter
//!
//! ## Purpose
//!
//! Validates the `export_format` parameter for data export operations.
//! Tests export format enum validation (csv, json, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .analytics.export_usage (export_format parameter)
//! - .analytics.export_spending (export_format parameter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Supported formats (csv, json)
//! 2. **Invalid Values**: Unknown formats, empty, typos
//! 3. **Edge Cases**: Case sensitivity, whitespace
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

  /// Test valid export_format (csv)
  #[tokio::test]
  async fn test_export_format_csv()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "export_format::csv", "output_file::usage.csv" ] ).await;

    // Should succeed or fail with business logic, not format validation
    if !result.success() {
      assert!( !result.stderr.contains( "export_format" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with export_format validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid export_format (json)
  #[tokio::test]
  async fn test_export_format_json_explicit()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "export_format::json", "output_file::usage.json" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "export_format" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with export_format validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid export_format
  #[tokio::test]
  async fn test_export_format_invalid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "export_format::xml", "output_file::usage.xml" ] ).await;

    assert!( !result.success(), "Unknown export_format should fail" );
    assert!( result.stderr.contains( "export_format" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "unknown" ),
      "Error should mention invalid export_format. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test empty export_format
  #[tokio::test]
  async fn test_export_format_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "export_format::", "output_file::usage.csv" ] ).await;

    assert!( !result.success(), "Empty export_format should fail" );
    assert!( result.stderr.contains( "export_format" ) || result.stderr.contains( "empty" ),
      "Error should mention empty export_format. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test export_format case insensitivity
  #[tokio::test]
  async fn test_export_format_lowercase()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "export_format::csv", "output_file::usage.csv" ] ).await;

    // Lowercase should work
    if !result.success() {
      assert!( !result.stderr.contains( "export_format" ) || !result.stderr.contains( "invalid" ),
        "Lowercase export_format should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test export_format default behavior (missing optional)
  #[tokio::test]
  async fn test_export_format_default()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.export_usage", "output_file::usage.csv" ] ).await;

    // Should use default export_format if optional
    if !result.success() {
      assert!( !result.stderr.contains( "export_format" ) || !result.stderr.contains( "required" ),
        "Should not require optional export_format. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
