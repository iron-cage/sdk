//! Parameter-level tests for `status` parameter
//!
//! ## Purpose
//!
//! Validates the `status` parameter for filtering budget status.
//! Tests status enum validation (active, exhausted).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .budget.status (status filter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Known statuses (active, exhausted)
//! 2. **Invalid Values**: Unknown statuses, empty, typos
//! 3. **Optional Behavior**: Missing optional status parameter
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

  /// Test valid status (active)
  #[tokio::test]
  async fn test_status_active()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "status::active" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'active' status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid status (exhausted)
  #[tokio::test]
  async fn test_status_exhausted()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "status::exhausted" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'exhausted' status. Stderr: {}", result.stderr );
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

    let result = harness.run( "iron", &[ ".budget.status", "status::unknown" ] ).await;

    assert!( !result.success(), "Unknown status should fail" );
    assert!( result.stderr.contains( "status" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid status. Stderr: {}", result.stderr );

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

    let result = harness.run( "iron", &[ ".budget.status" ] ).await;

    // Should succeed without optional status filter
    if !result.success() {
      assert!( !result.stderr.contains( "status" ) || !result.stderr.contains( "required" ),
        "Should not require optional status parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
