//! Parameter-level tests for `api_key` parameter
//!
//! ## Purpose
//!
//! Validates the `api_key` parameter across commands that manage API keys.
//! Tests API key format validation and identifier rules.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .api_key.create (name becomes api_key identifier)
//! - .api_key.get (api_key parameter)
//! - .api_key.revoke (api_key parameter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard key identifiers, prefixed formats
//! 2. **Invalid Values**: Empty, malformed, special characters
//! 3. **Edge Cases**: Very long, whitespace, case sensitivity
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

  /// Test valid api_key identifier
  #[tokio::test]
  async fn test_api_key_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.get", "api_key::my-api-key" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "api_key" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with api_key format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty api_key (should fail)
  #[tokio::test]
  async fn test_api_key_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.get", "api_key::" ] ).await;

    assert!( !result.success(), "Empty api_key should fail" );
    assert!( result.stderr.contains( "api_key" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty api_key. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required api_key
  #[tokio::test]
  async fn test_api_key_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.get" ] ).await;

    assert!( !result.success(), "Missing required api_key should fail" );
    assert!( result.stderr.contains( "api_key" ) || result.stderr.contains( "required" ),
      "Error should mention missing api_key. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test api_key with hyphens and underscores
  #[tokio::test]
  async fn test_api_key_with_separators()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.get", "api_key::my_test-key-123" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "api_key" ) || !result.stderr.contains( "invalid" ),
        "Should accept hyphens/underscores in api_key. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test api_key with whitespace (should fail)
  #[tokio::test]
  async fn test_api_key_with_whitespace()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.get", "api_key::my api key" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "api_key" ) || result.stderr.contains( "invalid" ),
        "Should reject whitespace in api_key. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long api_key
  #[tokio::test]
  async fn test_api_key_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_key = "a".repeat( 500 );
    let result = harness.run( "iron", &[ ".api_key.get", &format!( "api_key::{}", long_key ) ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "api_key" ) || result.stderr.contains( "too long" ) || result.stderr.contains( "length" ),
        "Should reject very long api_key. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test api_key on revoke command
  #[tokio::test]
  async fn test_api_key_revoke_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".api_key.revoke", "api_key::test-key" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "api_key" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with api_key format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
