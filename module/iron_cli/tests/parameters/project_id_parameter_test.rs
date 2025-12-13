//! Parameter-level tests for `project_id` parameter
//!
//! ## Purpose
//!
//! Validates the `project_id` parameter for project identification.
//! Tests UUID format validation for project resources.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .project.get (project_id parameter)
//! - .project.update (project_id parameter)
//! - .project.delete (project_id parameter)
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

  /// Test valid project_id with UUID
  #[tokio::test]
  async fn test_project_id_valid_uuid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.get", "project_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found' error, not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty project_id (should fail)
  #[tokio::test]
  async fn test_project_id_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.get", "project_id::" ] ).await;

    assert!( !result.success(), "Empty project_id should fail" );
    assert!( result.stderr.contains( "project_id" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty project_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid project_id format (not a UUID)
  #[tokio::test]
  async fn test_project_id_invalid_format()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.get", "project_id::not-a-uuid" ] ).await;

    assert!( !result.success(), "Invalid project_id should fail" );
    assert!( result.stderr.contains( "project_id" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "UUID" ),
      "Error should mention invalid project_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required project_id
  #[tokio::test]
  async fn test_project_id_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.get" ] ).await;

    assert!( !result.success(), "Missing required project_id should fail" );
    assert!( result.stderr.contains( "project_id" ) || result.stderr.contains( "required" ),
      "Error should mention missing project_id. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test project_id on update command
  #[tokio::test]
  async fn test_project_id_update_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.update", "project_id::550e8400-e29b-41d4-a716-446655440000", "name::updated-project" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test project_id on delete command
  #[tokio::test]
  async fn test_project_id_delete_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".project.delete", "project_id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not format error
    if !result.success() {
      assert!( !result.stderr.contains( "id" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should fail with 'not found', not format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
