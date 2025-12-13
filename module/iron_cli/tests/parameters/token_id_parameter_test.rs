//! Parameter-level tests for `token_id` parameter
//!
//! ## Purpose
//!
//! Validates the `token_id` parameter across all commands that use it.
//! Tests UUID format validation and token lookup behavior.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .tokens.get
//! - .tokens.rotate
//! - .tokens.revoke
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard UUID format with hyphens
//! 2. **Invalid Values**: Non-UUID strings, empty, malformed UUIDs
//! 3. **Edge Cases**: Uppercase, no hyphens, wrong length
//! 4. **Not Found**: Valid UUID that doesn't exist in database
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

  /// Test valid token_id with standard UUID format
  #[tokio::test]
  async fn test_token_id_valid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get", "token_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty token_id value (should fail)
  #[tokio::test]
  async fn test_token_id_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get", "token_id::" ] ).await;

    assert!( !result.success(), "Empty token_id should fail" );
    assert!( result.stderr.contains( "token_id" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty token_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid token_id (not a UUID)
  #[tokio::test]
  async fn test_token_id_invalid_format()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get", "token_id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid token_id should fail" );
    assert!( result.stderr.contains( "token_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid token_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test token_id with missing required parameter
  #[tokio::test]
  async fn test_token_id_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get" ] ).await;

    assert!( !result.success(), "Missing required token_id should fail" );
    assert!( result.stderr.contains( "token_id" ) || result.stderr.contains( "required" ),
      "Error should mention missing token_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test token_id on .tokens.rotate command
  #[tokio::test]
  async fn test_token_id_rotate_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.rotate", "token_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test token_id on .tokens.revoke command
  #[tokio::test]
  async fn test_token_id_revoke_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.revoke", "token_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test token_id with uppercase UUID (case normalization)
  #[tokio::test]
  async fn test_token_id_uppercase()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get", "token_id::550E8400-E29B-41D4-A716-446655440000" ] ).await;

    // Uppercase should be normalized and work (or fail with "not found", not format error)
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test token_id with whitespace (should fail or be trimmed)
  #[tokio::test]
  async fn test_token_id_with_whitespace()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.get", "token_id:: 550e8400-e29b-41d4-a716-446655440000 " ] ).await;

    // Whitespace handling - either trimmed and succeeds, or fails with format error
    if !result.success() && !result.stderr.contains( "not found" ) {
      assert!( result.stderr.contains( "token_id" ) || result.stderr.contains( "invalid" ),
        "Should handle whitespace gracefully. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
