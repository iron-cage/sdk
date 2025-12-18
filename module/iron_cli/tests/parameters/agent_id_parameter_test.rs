//! Parameter-level tests for `agent_id` parameter
//!
//! ## Purpose
//!
//! Validates the `agent_id` parameter for agent identification and filtering.
//! Tests UUID format validation for agent resources.
//! Also tests integer format validation for budget filtering.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .agent.get (agent_id parameter - UUID)
//! - .agent.update (agent_id parameter - UUID)
//! - .agent.delete (agent_id parameter - UUID)
//! - .analytics.usage_by_agent (agent_id filter - UUID)
//! - .budget.status (agent_id filter - positive integer)
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
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with agent_id format error. Stderr: {}", result.stderr );
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
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with agent_id format error. Stderr: {}", result.stderr );
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
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with agent_id format error. Stderr: {}", result.stderr );
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

  // =====================================================
  // Budget Status Command Tests (agent_id as Integer)
  // =====================================================

  /// Test valid agent_id in budget.status (positive integer)
  #[tokio::test]
  async fn test_budget_status_agent_id_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::1" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ),
        "Should accept positive integer agent_id. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid agent_id in budget.status (larger positive integer)
  #[tokio::test]
  async fn test_budget_status_agent_id_valid_large()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::12345" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ),
        "Should accept larger positive integer agent_id. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid agent_id in budget.status (zero)
  #[tokio::test]
  async fn test_budget_status_agent_id_invalid_zero()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::0" ] ).await;

    assert!( !result.success(), "Zero agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "positive" ),
      "Error should mention invalid agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid agent_id in budget.status (negative)
  #[tokio::test]
  async fn test_budget_status_agent_id_invalid_negative()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::-1" ] ).await;

    assert!( !result.success(), "Negative agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "positive" ),
      "Error should mention invalid agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid agent_id in budget.status (non-integer)
  #[tokio::test]
  async fn test_budget_status_agent_id_invalid_non_integer()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::abc" ] ).await;

    assert!( !result.success(), "Non-integer agent_id should fail" );
    assert!( result.stderr.contains( "agent_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "integer" ),
      "Error should mention invalid agent_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional agent_id in budget.status
  #[tokio::test]
  async fn test_budget_status_agent_id_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status" ] ).await;

    // Should succeed without optional agent_id filter
    if !result.success() {
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "required" ),
        "Should not require optional agent_id parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test agent_id combined with other filters in budget.status
  #[tokio::test]
  async fn test_budget_status_agent_id_combined()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "agent_id::1", "threshold::80", "status::active" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "agent_id" ) || !result.stderr.contains( "invalid" ),
        "Should accept agent_id combined with other filters. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
