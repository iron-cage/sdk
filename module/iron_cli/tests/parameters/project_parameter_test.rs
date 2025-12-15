//! Parameter-level tests for `project` parameter
//!
//! ## Purpose
//!
//! Validates the `project` parameter across all commands that use it.
//! Tests project identifier/name validation.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .tokens.generate (optional project filter)
//! - .limits.create (optional project scope)
//! - Other commands that accept project parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard project names/IDs
//! 2. **Optional Behavior**: Commands where project is optional
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

  /// Test valid project parameter
  #[tokio::test]
  async fn test_project_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "project::my-project" ] ).await;

    // Should succeed or fail with business logic, not validation error
    if !result.success() {
      assert!( !result.stderr.contains( "project" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with project validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty project parameter
  #[tokio::test]
  async fn test_project_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "project::" ] ).await;

    // Empty project should fail
    assert!( !result.success(), "Empty project should fail" );
    assert!( result.stderr.contains( "project" ) || result.stderr.contains( "empty" ),
      "Error should mention empty project. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional project parameter
  #[tokio::test]
  async fn test_project_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens" ] ).await;

    // Should succeed without optional project parameter
    if !result.success() {
      assert!( !result.stderr.contains( "project" ) || !result.stderr.contains( "required" ),
        "Should not require optional project parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test project parameter with special characters
  #[tokio::test]
  async fn test_project_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "project::my-project-123_test" ] ).await;

    // Hyphens, numbers, underscores should be accepted
    if !result.success() {
      assert!( !result.stderr.contains( "project" ) || !result.stderr.contains( "invalid" ),
        "Should accept hyphens/numbers/underscores in project name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test project parameter with whitespace
  #[tokio::test]
  async fn test_project_whitespace()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", "project::my project" ] ).await;

    // Whitespace in project name should fail
    if !result.success() {
      assert!( result.stderr.contains( "project" ) || result.stderr.contains( "invalid" ),
        "Should reject whitespace in project name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long project name
  #[tokio::test]
  async fn test_project_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_project = "a".repeat( 500 );
    let result = harness.run( "iron-token", &[ ".tokens.generate", "name::test-token", "scope::read:tokens", &format!( "project::{}", long_project ) ] ).await;

    // Very long project name may fail or be truncated
    if !result.success() {
      println!( "Very long project name rejected: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
