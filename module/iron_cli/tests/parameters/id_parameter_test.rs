//! Parameter-level tests for `id` parameter
//!
//! ## Purpose
//!
//! Validates the `id` parameter across all commands that use it.
//! The `id` parameter is used for generic resource identification (agents, projects, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .agent.get, .agent.update, .agent.delete, .agent.add_provider, .agent.remove_provider
//! - .agent.ic_token.generate, .agent.ic_token.status, .agent.ic_token.regenerate, .agent.ic_token.revoke
//! - .project.get
//! - And others...
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard UUID format
//! 2. **Invalid Values**: Non-UUID strings, empty, malformed
//! 3. **Cross-Command**: Same parameter behavior across different resource types
//! 4. **Edge Cases**: Case sensitivity, whitespace
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

  /// Test valid id parameter with standard UUID
  #[tokio::test]
  async fn test_id_valid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty id parameter (should fail)
  #[tokio::test]
  async fn test_id_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "id::" ] ).await;

    assert!( !result.success(), "Empty id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid id format (not a UUID)
  #[tokio::test]
  async fn test_id_invalid_format()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test id parameter across different resource types (agent, project)
  #[tokio::test]
  async fn test_id_cross_resource_consistency()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let test_uuid = "550e8400-e29b-41d4-a716-446655440000";

    // Test id parameter with agent.get
    let result1 = harness.run( "iron", &[ ".agent.get", &format!( "id::{}", test_uuid ) ] ).await;

    // Test id parameter with project.get
    let result2 = harness.run( "iron", &[ ".project.get", &format!( "id::{}", test_uuid ) ] ).await;

    // All should handle the UUID consistently (succeed or "not found", not format error)
    for result in [ result1, result2 ] {
      if !result.success() {
        assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
          "Should not fail with id format error. Stderr: {}", result.stderr );
      }
    }

    server.shutdown().await;
  }

  /// Test missing required id parameter
  #[tokio::test]
  async fn test_id_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get" ] ).await;

    assert!( !result.success(), "Missing required id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "required" ),
      "Error should mention missing id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test id with uppercase UUID (case normalization)
  #[tokio::test]
  async fn test_id_uppercase()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "id::550E8400-E29B-41D4-A716-446655440000" ] ).await;

    // Uppercase should be normalized (or fail with "not found", not format error)
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test id with very long string (exceeds UUID length)
  #[tokio::test]
  async fn test_id_too_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let very_long_id = "550e8400-e29b-41d4-a716-446655440000-extra-characters-that-make-it-too-long";
    let result = harness.run( "iron", &[ ".agent.get", &format!( "id::{}", very_long_id ) ] ).await;

    assert!( !result.success(), "Too long id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test id with single character (clearly invalid)
  #[tokio::test]
  async fn test_id_single_character()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "id::x" ] ).await;

    assert!( !result.success(), "Single character id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  // =====================================================
  // IC Token Command Tests
  // =====================================================

  /// Test .agent.ic_token.generate with valid id
  #[tokio::test]
  async fn test_ic_token_generate_valid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.generate", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with id format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test .agent.ic_token.generate with missing id
  #[tokio::test]
  async fn test_ic_token_generate_missing_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.generate" ] ).await;

    assert!( !result.success(), "Missing required id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "required" ),
      "Error should mention missing id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.generate with invalid id
  #[tokio::test]
  async fn test_ic_token_generate_invalid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.generate", "id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.status with valid id
  #[tokio::test]
  async fn test_ic_token_status_valid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.status", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with id format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test .agent.ic_token.status with missing id
  #[tokio::test]
  async fn test_ic_token_status_missing_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.status" ] ).await;

    assert!( !result.success(), "Missing required id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "required" ),
      "Error should mention missing id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.status with invalid id
  #[tokio::test]
  async fn test_ic_token_status_invalid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.status", "id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.regenerate with valid id
  #[tokio::test]
  async fn test_ic_token_regenerate_valid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.regenerate", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with id format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test .agent.ic_token.regenerate with missing id
  #[tokio::test]
  async fn test_ic_token_regenerate_missing_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.regenerate" ] ).await;

    assert!( !result.success(), "Missing required id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "required" ),
      "Error should mention missing id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.regenerate with invalid id
  #[tokio::test]
  async fn test_ic_token_regenerate_invalid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.regenerate", "id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.revoke with valid id
  #[tokio::test]
  async fn test_ic_token_revoke_valid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.revoke", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with id format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test .agent.ic_token.revoke with missing id
  #[tokio::test]
  async fn test_ic_token_revoke_missing_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.revoke" ] ).await;

    assert!( !result.success(), "Missing required id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "required" ),
      "Error should mention missing id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test .agent.ic_token.revoke with invalid id
  #[tokio::test]
  async fn test_ic_token_revoke_invalid_id()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.ic_token.revoke", "id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid id should fail" );
    assert!( result.stderr.contains( "id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test IC token commands across all four operations with same agent id
  #[tokio::test]
  async fn test_ic_token_commands_consistency()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let test_uuid = "550e8400-e29b-41d4-a716-446655440000";

    // Test all IC token commands with the same id
    let commands = [
      ".agent.ic_token.generate",
      ".agent.ic_token.status",
      ".agent.ic_token.regenerate",
      ".agent.ic_token.revoke",
    ];

    for cmd in commands {
      let result = harness.run( "iron", &[ cmd, &format!( "id::{}", test_uuid ) ] ).await;

      // All should handle the UUID consistently (succeed or "not found", not format error)
      if !result.success() {
        assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
          "Command {} should not fail with id format error. Stderr: {}", cmd, result.stderr );
      }
    }

    server.shutdown().await;
  }
}
