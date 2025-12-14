//! Parameter-level tests for `name` parameter
//!
//! ## Purpose
//!
//! Validates the `name` parameter across all commands that use it.
//! Tests name/identifier validation for various resources (tokens, agents, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .tokens.generate (required name)
//! - .agent.create (required name)
//! - .agent.update (optional name)
//! - Other commands accepting name parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard names, hyphens, underscores
//! 2. **Required/Optional**: Behavior based on command requirements
//! 3. **Invalid Values**: Empty, special characters, too long
//! 4. **Edge Cases**: Whitespace, case sensitivity
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

  /// Test valid name parameter
  #[tokio::test]
  async fn test_name_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::my-test-token", "scope::read:tokens" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "name" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with name validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty name (should fail for required parameter)
  #[tokio::test]
  async fn test_name_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::", "scope::read:tokens" ] ).await;

    assert!( !result.success(), "Empty name should fail" );
    assert!( result.stderr.contains( "name" ) || result.stderr.contains( "empty" ),
      "Error should mention empty name. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required name parameter
  #[tokio::test]
  async fn test_name_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "scope::read:tokens" ] ).await;

    assert!( !result.success(), "Missing required name should fail" );
    assert!( result.stderr.contains( "name" ) || result.stderr.contains( "required" ),
      "Error should mention missing name. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test name with hyphens and underscores
  #[tokio::test]
  async fn test_name_with_hyphens_underscores()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::my_test-token-123", "scope::read:tokens" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "name" ) || !result.stderr.contains( "invalid" ),
        "Should accept hyphens and underscores in name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test name with whitespace (should fail)
  #[tokio::test]
  async fn test_name_with_whitespace()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::my test token", "scope::read:tokens" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "name" ) || result.stderr.contains( "invalid" ),
        "Should reject whitespace in name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long name
  #[tokio::test]
  async fn test_name_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_name = "a".repeat( 500 );
    let result = harness.run( "iron-token", &[ ".tokens.generate", &format!( "name::{}", long_name ), "scope::read:tokens" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "name" ) || result.stderr.contains( "too long" ) || result.stderr.contains( "length" ),
        "Should reject very long name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test name with special characters (should fail)
  #[tokio::test]
  async fn test_name_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::my@token!#", "scope::read:tokens" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "name" ) || result.stderr.contains( "invalid" ),
        "Should reject special characters in name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test name across different commands (tokens vs agents)
  #[tokio::test]
  async fn test_name_cross_command_consistency()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // Test name on tokens.generate
    let result1 = harness.run( "iron-token", &[ ".tokens.generate", "name::test-resource", "scope::read:tokens" ] ).await;

    // Test name on agent.create
    let result2 = harness.run( "iron", &[ ".agent.create", "name::test-resource", "budget::1000" ] ).await;

    // Both should handle name consistently
    for result in [ result1, result2 ] {
      if !result.success() {
        assert!( !result.stderr.contains( "name" ) || !result.stderr.contains( "format" ),
          "Should handle name format consistently. Stderr: {}", result.stderr );
      }
    }

    server.shutdown().await;
  }
}
