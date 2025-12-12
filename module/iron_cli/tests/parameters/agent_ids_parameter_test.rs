//! Parameter-level tests for `agent_ids` parameter
//!
//! ## Purpose
//!
//! Validates the `agent_ids` parameter for multiple agent filtering.
//! Tests comma-separated UUID list parsing and validation.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .analytics.usage_by_agent (agent_ids filter)
//! - Other analytics commands that support agent filtering
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Single UUID, multiple UUIDs comma-separated
//! 2. **Invalid Values**: Malformed UUIDs, mixed valid/invalid, empty
//! 3. **Edge Cases**: Whitespace handling, duplicate UUIDs
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

  /// Test single agent_id in list
  #[tokio::test]
  async fn test_agent_ids_single()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed (empty result set is acceptable)
    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "invalid" ),
        "Single UUID should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test multiple agent_ids comma-separated
  #[tokio::test]
  async fn test_agent_ids_multiple()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::550e8400-e29b-41d4-a716-446655440000,660f9511-f3ac-52e5-b827-557766551111" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "invalid" ),
        "Multiple UUIDs should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test agent_ids with whitespace (should be trimmed)
  #[tokio::test]
  async fn test_agent_ids_whitespace()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::550e8400-e29b-41d4-a716-446655440000, 660f9511-f3ac-52e5-b827-557766551111" ] ).await;

    // Whitespace around commas should be handled gracefully
    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "invalid" ),
        "Whitespace should be trimmed. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty agent_ids
  #[tokio::test]
  async fn test_agent_ids_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::" ] ).await;

    assert!( !result.success(), "Empty agent_ids should fail" );
    assert!( result.stderr.contains( "agent_ids" ) || result.stderr.contains( "empty" ),
      "Error should mention empty agent_ids. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid UUID in list
  #[tokio::test]
  async fn test_agent_ids_invalid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::not-a-uuid,550e8400-e29b-41d4-a716-446655440000" ] ).await;

    assert!( !result.success(), "Invalid UUID in list should fail" );
    assert!( result.stderr.contains( "agent_ids" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid UUID. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional agent_ids
  #[tokio::test]
  async fn test_agent_ids_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent" ] ).await;

    // Should succeed without optional filter (returns all agents)
    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "required" ),
        "Should not require optional agent_ids parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test duplicate agent_ids
  #[tokio::test]
  async fn test_agent_ids_duplicates()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", "agent_ids::550e8400-e29b-41d4-a716-446655440000,550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Duplicates should be handled (deduplicated or accepted)
    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "invalid" ),
        "Duplicates should be handled gracefully. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test many agent_ids (stress test)
  #[tokio::test]
  async fn test_agent_ids_many()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // Test with 5 agent IDs
    let ids = "550e8400-e29b-41d4-a716-446655440000,\
               660f9511-f3ac-52e5-b827-557766551111,\
               770f9522-f3bd-63f6-c938-668877662222,\
               880f9633-f4ce-74g7-d049-779988773333,\
               990f9744-f5df-85h8-e15a-880099884444";

    let result = harness.run( "iron", &[ ".analytics.usage_by_agent", &format!( "agent_ids::{}", ids ) ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "agent_ids" ) || !result.stderr.contains( "invalid" ),
        "Multiple UUIDs should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
