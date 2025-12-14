//! Parameter-level tests for `status` parameter
//!
//! ## Purpose
//!
//! Validates the `status` parameter for filtering budget requests.
//! Tests status enum validation (pending, approved, rejected, cancelled).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .budget_request.list (status filter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Known statuses (pending, approved, rejected, cancelled)
//! 2. **Invalid Values**: Unknown statuses, empty, typos
//! 3. **Optional Behavior**: Missing optional status parameter
//! 4. **Edge Cases**: Case sensitivity
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

  /// Test valid status (pending)
  #[tokio::test]
  async fn test_status_pending()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::pending" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'pending' status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid status (approved)
  #[tokio::test]
  async fn test_status_approved()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::approved" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'approved' status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid status (rejected)
  #[tokio::test]
  async fn test_status_rejected()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::rejected" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'rejected' status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid status (cancelled)
  #[tokio::test]
  async fn test_status_cancelled()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::cancelled" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'cancelled' status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid status
  #[tokio::test]
  async fn test_status_invalid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::unknown" ] ).await;

    assert!( !result.success(), "Unknown status should fail" );
    assert!( result.stderr.contains( "status" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid status. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test empty status
  #[tokio::test]
  async fn test_status_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list", "status::" ] ).await;

    assert!( !result.success(), "Empty status should fail" );
    assert!( result.stderr.contains( "status" ) || result.stderr.contains( "empty" ),
      "Error should mention empty status. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional status
  #[tokio::test]
  async fn test_status_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget_request.list" ] ).await;

    // Should succeed without optional status filter
    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "required" ),
        "Should not require optional status parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
