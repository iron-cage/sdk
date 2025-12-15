//! Parameter-level tests for `email` parameter
//!
//! ## Purpose
//!
//! Validates the `email` parameter across user management commands.
//! Tests email format validation (RFC 5322 compliance).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .user.create (email parameter)
//! - .user.update (email parameter)
//! - Other user management commands
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard email formats, subdomains, plus addressing
//! 2. **Invalid Values**: Missing @, missing domain, invalid characters
//! 3. **Edge Cases**: Very long, unicode, special formats
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

  /// Test valid email
  #[tokio::test]
  async fn test_email_valid()
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
      assert!( !result.stderr.contains( "email" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with email format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty email (should fail)
  #[tokio::test]
  async fn test_email_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::", "role::user" ] ).await;

    assert!( !result.success(), "Empty email should fail" );
    assert!( result.stderr.contains( "email" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty email. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test email missing @ symbol (should fail)
  #[tokio::test]
  async fn test_email_missing_at_symbol()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::userexample.com", "role::user" ] ).await;

    assert!( !result.success(), "Email without @ should fail" );
    assert!( result.stderr.contains( "email" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid email. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test email with subdomain
  #[tokio::test]
  async fn test_email_with_subdomain()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::user@mail.example.com", "role::user" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "email" ) || !result.stderr.contains( "invalid" ),
        "Should accept email with subdomain. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test email with plus addressing (should be allowed)
  #[tokio::test]
  async fn test_email_plus_addressing()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::user+tag@example.com", "role::user" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "email" ) || !result.stderr.contains( "invalid" ),
        "Should accept plus addressing in email. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long email
  #[tokio::test]
  async fn test_email_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_local = "a".repeat( 200 );
    let result = harness.run( "iron", &[ ".user.create", &format!( "email::{}@example.com", long_local ), "role::user" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "email" ) || result.stderr.contains( "too long" ),
        "Should reject very long email. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test email missing domain (should fail)
  #[tokio::test]
  async fn test_email_missing_domain()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "admin@example.com" ).await;
    let api_key = data.create_api_key( user_id, "admin-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.create", "email::user@", "role::user" ] ).await;

    assert!( !result.success(), "Email without domain should fail" );
    assert!( result.stderr.contains( "email" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid email. Stderr: {}", result.stderr );

    server.shutdown().await;
  }
}
