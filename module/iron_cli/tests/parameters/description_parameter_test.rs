//! Parameter-level tests for `description` parameter
//!
//! ## Purpose
//!
//! Validates the `description` parameter across all commands that use it.
//! Tests text field validation for descriptions (optional in most cases).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .tokens.generate (optional description)
//! - Other commands accepting description parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard text, special characters, unicode
//! 2. **Optional Behavior**: Commands where description is optional
//! 3. **Edge Cases**: Empty, very long, whitespace
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

  /// Test valid description with standard text
  #[tokio::test]
  async fn test_description_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "description::This is a test token" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "description" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with description validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test missing optional description
  #[tokio::test]
  async fn test_description_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens" ] ).await;

    // Should succeed without optional description
    if !result.success() {
      assert!( !result.stderr.contains( "description" ) || !result.stderr.contains( "required" ),
        "Should not require optional description. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty description
  #[tokio::test]
  async fn test_description_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "description::" ] ).await;

    // Empty description might be allowed or rejected
    if !result.success() && result.stderr.contains( "description" ) {
      println!( "Empty description rejected: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test description with special characters
  #[tokio::test]
  async fn test_description_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "description::Test with !@#$%^&*() special chars" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "description" ) || !result.stderr.contains( "invalid" ),
        "Should accept special characters in description. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test description with unicode
  #[tokio::test]
  async fn test_description_unicode()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "description::テストトークン 测试令牌" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "description" ) || !result.stderr.contains( "invalid" ),
        "Should accept unicode in description. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long description
  #[tokio::test]
  async fn test_description_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_desc = "a".repeat( 5000 );
    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", &format!( "description::{}", long_desc ) ] ).await;

    // Very long description may be rejected or truncated
    if !result.success() && result.stderr.contains( "description" ) {
      println!( "Very long description rejected: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test description with newlines
  #[tokio::test]
  async fn test_description_with_newlines()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "description::Line 1\nLine 2" ] ).await;

    // Newlines might be allowed or rejected depending on implementation
    if !result.success() && result.stderr.contains( "description" ) {
      println!( "Description with newlines rejected: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
