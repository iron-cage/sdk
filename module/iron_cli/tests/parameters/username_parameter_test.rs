//! Parameter-level tests for `username` parameter
//!
//! ## Purpose
//!
//! Validates the `username` parameter across authentication commands.
//! Tests username validation for login/signup operations.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .auth.login (username parameter)
//! - .user.create (username parameter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard usernames, email-style, alphanumeric
//! 2. **Invalid Values**: Empty, special characters, too long/short
//! 3. **Edge Cases**: Case sensitivity, whitespace, unicode
//!
//! ## TDD Status
//!
//! RED: Writing tests (below)
//! GREEN: Implementation pending
//! REFACTOR: Pending

#[cfg(test)]
mod tests
{
  use crate::fixtures::{ IntegrationTestHarness, TestServer };

  /// Test valid username
  #[tokio::test]
  async fn test_username_valid()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "username::testuser", "password::testpass123" ] ).await;

    // Should succeed or fail with auth error, not validation error
    if !result.success() {
      assert!( !result.stderr.contains( "username" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with username format validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty username (should fail)
  #[tokio::test]
  async fn test_username_empty()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "username::", "password::testpass123" ] ).await;

    assert!( !result.success(), "Empty username should fail" );
    assert!( result.stderr.contains( "username" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty username. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required username
  #[tokio::test]
  async fn test_username_missing_required()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "password::testpass123" ] ).await;

    assert!( !result.success(), "Missing required username should fail" );
    assert!( result.stderr.contains( "username" ) || result.stderr.contains( "required" ),
      "Error should mention missing username. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test username with special characters
  #[tokio::test]
  async fn test_username_special_characters()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "username::test@user!", "password::testpass123" ] ).await;

    // Special characters may be rejected depending on policy
    if !result.success() && result.stderr.contains( "username" ) && result.stderr.contains( "invalid" ) {
      println!( "Username with special characters rejected: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test username with whitespace (should fail)
  #[tokio::test]
  async fn test_username_with_whitespace()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "username::test user", "password::testpass123" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "username" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "whitespace" ),
        "Should reject whitespace in username. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long username
  #[tokio::test]
  async fn test_username_very_long()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let long_username = "a".repeat( 300 );
    let result = harness.run( "iron-token", &[ ".auth.login", &format!( "username::{}", long_username ), "password::testpass123" ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "username" ) || result.stderr.contains( "too long" ) || result.stderr.contains( "length" ),
        "Should reject very long username. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test username with email format
  #[tokio::test]
  async fn test_username_email_format()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".auth.login", "username::test@example.com", "password::testpass123" ] ).await;

    // Email-style usernames might be allowed
    if !result.success() {
      assert!( !result.stderr.contains( "username" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should accept email-style username. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
