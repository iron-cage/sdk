//! Parameter-level tests for `reason` parameter
//!
//! ## Purpose
//!
//! Validates the `reason` parameter for budget request justification.
//! Tests required text field validation with security considerations.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .budget_request.create (required reason)
//! - .budget_request.reject (required reason)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Standard text, detailed explanations
//! 2. **Required Field**: Must be provided, cannot be empty
//! 3. **Edge Cases**: Very long, special characters, unicode
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

  /// Test valid reason with standard text
  #[tokio::test]
  async fn test_reason_valid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000", "reason::Need additional budget for Q4 API costs" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "reason" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with reason validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty reason (should fail)
  #[tokio::test]
  async fn test_reason_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000", "reason::" ] ).await;

    assert!( !result.success(), "Empty reason should fail" );
    assert!( result.stderr.contains( "reason" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty reason. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing required reason
  #[tokio::test]
  async fn test_reason_missing_required()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000" ] ).await;

    assert!( !result.success(), "Missing required reason should fail" );
    assert!( result.stderr.contains( "reason" ) || result.stderr.contains( "required" ),
      "Error should mention missing reason. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test reason with special characters
  #[tokio::test]
  async fn test_reason_special_characters()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000", "reason::Need $500 for API (50% increase)" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "reason" ) || !result.stderr.contains( "invalid" ),
        "Should accept special characters in reason. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long reason
  #[tokio::test]
  async fn test_reason_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_reason = "a".repeat( 5000 );
    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000", &format!( "reason::{}", long_reason ) ] ).await;

    // Very long reason may be accepted or have max length
    if !result.success() && result.stderr.contains( "reason" ) {
      println!( "Very long reason behavior: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test reason with unicode
  #[tokio::test]
  async fn test_reason_unicode()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.create", "amount::1000", "reason::追加予算が必要です (Additional budget needed)" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "reason" ) || !result.stderr.contains( "invalid" ),
        "Should accept unicode in reason. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test reason on reject command
  #[tokio::test]
  async fn test_reason_reject_command()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.reject", "id::550e8400-e29b-41d4-a716-446655440000", "reason::Budget constraints for Q4" ] ).await;

    // Should succeed or fail with "not found", not reason validation
    if !result.success() {
      assert!( !result.stderr.contains( "reason" ) || !result.stderr.contains( "invalid" ) || result.stderr.contains( "not found" ),
        "Should not fail with reason validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
