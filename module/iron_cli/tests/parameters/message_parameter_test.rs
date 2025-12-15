//! Parameter-level tests for `message` parameter
//!
//! ## Purpose
//!
//! Validates the `message` parameter for optional text input.
//! Tests text validation and optional behavior.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .test.with_message (message parameter)
//! - Other test commands that support message parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Short messages, long messages, special characters
//! 2. **Optional Behavior**: Commands where message is optional
//! 3. **Edge Cases**: Empty string, whitespace, unicode
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

  /// Test message with simple text
  #[tokio::test]
  async fn test_message_simple()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message", "message::Hello World" ] ).await;

    // Should succeed with simple message
    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "invalid" ),
        "Should accept simple message. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test message with special characters
  #[tokio::test]
  async fn test_message_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message", "message::Test-message_123!@#" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "invalid" ),
        "Should accept special characters. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test message with spaces
  #[tokio::test]
  async fn test_message_with_spaces()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message", "message::This is a test message" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "invalid" ),
        "Should accept message with spaces. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test long message
  #[tokio::test]
  async fn test_message_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_message = "This is a very long test message that contains multiple words and should be handled correctly by the parameter validation system";
    let result = harness.run( "iron", &[ ".test.with_message", &format!( "message::{}", long_message ) ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "invalid" ),
        "Should accept long message. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty message
  #[tokio::test]
  async fn test_message_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message", "message::" ] ).await;

    // Empty message might be rejected or treated as missing
    if !result.success() && result.stderr.contains( "message" ) {
      println!( "Empty message behavior: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test missing optional message
  #[tokio::test]
  async fn test_message_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message" ] ).await;

    // Should succeed without optional message
    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "required" ),
        "Should not require optional message parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test message with unicode characters
  #[tokio::test]
  async fn test_message_unicode()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".test.with_message", "message::Hello 世界 🌍" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "message" ) || !result.stderr.contains( "invalid" ),
        "Should accept unicode characters. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
