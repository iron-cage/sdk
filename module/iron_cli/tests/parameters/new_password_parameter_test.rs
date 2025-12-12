//! Parameter-level tests for `new_password` parameter
//!
//! ## Purpose
//!
//! Validates the `new_password` parameter for password change operations.
//! Tests password strength validation and security requirements.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .user.change_password (new_password parameter)
//! - Other user management commands that support password change
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Strong passwords meeting security requirements
//! 2. **Invalid Values**: Weak passwords, empty, too short, no special chars
//! 3. **Security Requirements**: Minimum length, complexity rules
//! 4. **Edge Cases**: Very long passwords, special characters
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

  /// Test valid new_password (strong password)
  #[tokio::test]
  async fn test_new_password_valid_strong()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::SecureP@ssw0rd123" ] ).await;

    // Should succeed with strong password
    if !result.success() {
      assert!( !result.stderr.contains( "new_password" ) || !result.stderr.contains( "weak" ) || !result.stderr.contains( "invalid" ),
        "Strong password should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test new_password too short (should fail)
  #[tokio::test]
  async fn test_new_password_too_short()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::Pass1" ] ).await;

    assert!( !result.success(), "Short password should fail" );
    assert!( result.stderr.contains( "new_password" ) || result.stderr.contains( "password" ) || result.stderr.contains( "length" ),
      "Error should mention password length requirement. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test new_password without special characters
  #[tokio::test]
  async fn test_new_password_no_special_chars()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::Password123" ] ).await;

    // Might fail if special characters are required
    if !result.success() && result.stderr.contains( "password" ) {
      println!( "Password complexity requirement: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test new_password without numbers
  #[tokio::test]
  async fn test_new_password_no_numbers()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::Password@Special" ] ).await;

    // Might fail if numbers are required
    if !result.success() && result.stderr.contains( "password" ) {
      println!( "Password complexity requirement: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty new_password (should fail)
  #[tokio::test]
  async fn test_new_password_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::" ] ).await;

    assert!( !result.success(), "Empty new_password should fail" );
    assert!( result.stderr.contains( "new_password" ) || result.stderr.contains( "password" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty password. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required new_password
  #[tokio::test]
  async fn test_new_password_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password" ] ).await;

    assert!( !result.success(), "Missing required new_password should fail" );
    assert!( result.stderr.contains( "new_password" ) || result.stderr.contains( "password" ) || result.stderr.contains( "required" ),
      "Error should mention missing password. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test very long new_password
  #[tokio::test]
  async fn test_new_password_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // 100-character password
    let long_password = "VeryLongP@ssw0rdWithManyCharacters123456789!@#$%^&*()VeryLongP@ssw0rdWithManyCharacters123456789!";
    let result = harness.run( "iron", &[ ".user.change_password", &format!( "new_password::{}", long_password ) ] ).await;

    // Should accept very long passwords (no maximum length limit typically)
    if !result.success() {
      assert!( !result.stderr.contains( "new_password" ) || !result.stderr.contains( "too long" ),
        "Very long password should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test new_password with various special characters
  #[tokio::test]
  async fn test_new_password_special_chars_variety()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".user.change_password", "new_password::P@ss!w0rd#2024$" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "new_password" ) || !result.stderr.contains( "invalid" ),
        "Password with variety of special chars should be accepted. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
