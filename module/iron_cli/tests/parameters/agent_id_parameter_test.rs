//! Parameter-level tests for `agent_id` parameter
//!
//! ## Purpose
//!
//! Validates the `agent_id` parameter for agent identification and filtering.
//! Tests UUID format validation for agent resources.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .agent.get (agent_id parameter)
//! - .agent.update (agent_id parameter)
//! - .agent.delete (agent_id parameter)
//! - .analytics.usage_by_agent (agent_id filter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard UUID format
//! 2. **Invalid Values**: Non-UUID strings, empty, malformed
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

  /// Test valid agent_id with UUID
  #[tokio::test]
  async fn test_agent_id_valid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "agent_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( result.stderr.contains( "not found" ) || result.stderr.contains( "does not exist" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty agent_id (should fail)
  #[tokio::test]
  async fn test_agent_id_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "agent_id::" ] ).await;

    assert!( !result.success(), "Empty agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid agent_id format (not a UUID)
  #[tokio::test]
  async fn test_agent_id_invalid_format()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get", "agent_id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required agent_id
  #[tokio::test]
  async fn test_agent_id_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.get" ] ).await;

    assert!( !result.success(), "Missing required agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "required" ),
      "Error should mention missing agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test agent_id on update command
  #[tokio::test]
  async fn test_agent_id_update_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.update", "agent_id::550e8400-e29b-41d4-a716-446655440000", "name::updated-agent" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( result.stderr.contains( "not found" ) || result.stderr.contains( "does not exist" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test agent_id on delete command
  #[tokio::test]
  async fn test_agent_id_delete_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".agent.delete", "agent_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( result.stderr.contains( "not found" ) || result.stderr.contains( "does not exist" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test agent_id in analytics filtering
  #[tokio::test]
  async fn test_agent_id_analytics_filter()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed (empty result set is acceptable for analytics)
    if !result.success() {
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ),
        "Valid UUID should be accepted for filtering. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
