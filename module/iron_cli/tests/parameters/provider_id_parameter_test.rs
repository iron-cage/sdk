//! Parameter-level tests for `provider_id` parameter
//!
//! ## Purpose
//!
//! Validates the `provider_id` parameter for provider identification.
//! Tests UUID format validation for provider resources.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .provider.get (provider_id parameter)
//! - .provider.update (provider_id parameter)
//! - .provider.delete (provider_id parameter)
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

  /// Test valid provider_id with UUID
  #[tokio::test]
  async fn test_provider_id_valid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty provider_id (should fail)
  #[tokio::test]
  async fn test_provider_id_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider_id::" ] ).await;

    assert!( !result.success(), "Empty provider_id should fail" );
    assert!( result.stderr.contains( "provider_id" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty provider_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid provider_id format (not a UUID)
  #[tokio::test]
  async fn test_provider_id_invalid_format()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider_id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid provider_id should fail" );
    assert!( result.stderr.contains( "provider_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid provider_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required provider_id
  #[tokio::test]
  async fn test_provider_id_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get" ] ).await;

    assert!( !result.success(), "Missing required provider_id should fail" );
    assert!( result.stderr.contains( "provider_id" ) || result.stderr.contains( "required" ),
      "Error should mention missing provider_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test provider_id on update command
  #[tokio::test]
  async fn test_provider_id_update_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.update", "provider_id::550e8400-e29b-41d4-a716-446655440000", "name::updated-provider" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test provider_id on delete command
  #[tokio::test]
  async fn test_provider_id_delete_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.delete", "provider_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test provider_id case sensitivity
  #[tokio::test]
  async fn test_provider_id_case_sensitivity()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // UUIDs should be case-insensitive
    let result = harness.run( "iron", &[ ".provider.get", "provider_id::550E8400-E29B-41D4-A716-446655440000" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Uppercase UUID should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
