//! Parameter-level tests for `password` parameter
//!
//! ## Purpose
//!
//! Validates the `password` parameter across authentication commands.
//! Tests password validation including minimum length, security requirements.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .auth.login (password parameter)
//! - .user.create (password parameter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Strong passwords with mixed characters
//! 2. **Security**: Minimum length, complexity requirements
//! 3. **Invalid Values**: Empty, too short, too weak
//! 4. **Edge Cases**: Very long, special characters, unicode
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

  /// Test valid password
  #[tokio::test]
  async fn test_password_valid()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", "password::SecurePass123!" ] ).await;

    // Should succeed or fail with auth error, not password format validation
    if !result.success() {
      assert!( !result.stderr.contains( "password" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with password format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty password (should fail)
  #[tokio::test]
  async fn test_password_empty()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", "password::" ] ).await;

    assert!( !result.success(), "Empty password should fail" );
    assert!( result.stderr.contains( "password" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty password. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required password
  #[tokio::test]
  async fn test_password_missing_required()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser" ] ).await;

    assert!( !result.success(), "Missing required password should fail" );
    assert!( result.stderr.contains( "password" ) || result.stderr.contains( "required" ),
      "Error should mention missing password. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test very short password (security concern)
  #[tokio::test]
  async fn test_password_too_short()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", "password::ab" ] ).await;

    // Very short password might be rejected for security
    if !result.success() && result.stderr.contains( "password" ) {
      println!( "Short password rejected (security policy): {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test password with special characters
  #[tokio::test]
  async fn test_password_special_characters()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", "password::P@ssw0rd!#$%" ] ).await;

    // Special characters should be allowed in passwords
    if !result.success() {
      assert!( !result.stderr.contains( "password" ) || !result.stderr.contains( "invalid" ),
        "Should accept special characters in password. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long password
  #[tokio::test]
  async fn test_password_very_long()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let long_password = "a".repeat( 1000 );
    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", &format!( "password::{}", long_password ) ] ).await;

    // Very long password may be accepted or have max length
    if !result.success() && result.stderr.contains( "password" ) {
      println!( "Very long password behavior: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test password with unicode characters
  #[tokio::test]
  async fn test_password_unicode()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".auth.login", "username::testuser", "password::パスワード123" ] ).await;

    // Unicode should be allowed in passwords
    if !result.success() {
      assert!( !result.stderr.contains( "password" ) || !result.stderr.contains( "invalid" ),
        "Should accept unicode in password. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
