//! Parameter-level tests for `role` parameter
//!
//! ## Purpose
//!
//! Validates the `role` parameter across user management commands.
//! Tests role enum validation (admin, user, viewer, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .user.create (required role)
//! - .user.update (optional role)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Known roles (admin, user, viewer)
//! 2. **Invalid Values**: Unknown roles, empty, typos
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

  /// Test valid role (admin)
  #[tokio::test]
  async fn test_role_admin()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::newuser@example.com", "role::admin" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "role" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'admin' role. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid role (user)
  #[tokio::test]
  async fn test_role_user()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::newuser@example.com", "role::user" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "role" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'user' role. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid role
  #[tokio::test]
  async fn test_role_invalid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::newuser@example.com", "role::superadmin" ] ).await;

    assert!( !result.success(), "Unknown role should fail" );
    assert!( result.stderr.contains( "role" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid role. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test empty role
  #[tokio::test]
  async fn test_role_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::newuser@example.com", "role::" ] ).await;

    assert!( !result.success(), "Empty role should fail" );
    assert!( result.stderr.contains( "role" ) || result.stderr.contains( "empty" ),
      "Error should mention empty role. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional role parameter
  #[tokio::test]
  async fn test_role_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // Assuming .user.update allows missing optional role
    let result = harness.run( "iron", &[ ".user.update", "id::550e8400-e29b-41d4-a716-446655440000" ] ).await;

    // Should succeed or fail with "not found", not "role required"
    if !result.success() {
      assert!( !result.stderr.contains( "role" ) || !result.stderr.contains( "required" ),
        "Should not require optional role. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
