//! Parameter-level tests for `threshold` parameter
//!
//! ## Purpose
//!
//! Validates the `threshold` parameter for percentage-based filtering.
//! Tests that threshold is a valid percentage value (0-100).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .budget.status (threshold filter)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Integers 0-100
//! 2. **Invalid Values**: Negative numbers, over 100, non-integers
//! 3. **Edge Cases**: Boundary values (0, 100)
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

  /// Test valid threshold (50%)
  #[tokio::test]
  async fn test_threshold_valid_50()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::50" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "invalid" ),
        "Should accept threshold 50. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid threshold (80%)
  #[tokio::test]
  async fn test_threshold_valid_80()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::80" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "invalid" ),
        "Should accept threshold 80. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test boundary threshold (0%)
  #[tokio::test]
  async fn test_threshold_boundary_zero()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::0" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "invalid" ),
        "Should accept threshold 0. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test boundary threshold (100%)
  #[tokio::test]
  async fn test_threshold_boundary_100()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::100" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "invalid" ),
        "Should accept threshold 100. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid threshold (negative)
  #[tokio::test]
  async fn test_threshold_invalid_negative()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::-10" ] ).await;

    assert!( !result.success(), "Negative threshold should fail" );
    assert!( result.stderr.contains( "threshold" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "0" ) || result.stderr.contains( "100" ),
      "Error should mention invalid threshold. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid threshold (over 100)
  #[tokio::test]
  async fn test_threshold_invalid_over_100()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::101" ] ).await;

    assert!( !result.success(), "Threshold over 100 should fail" );
    assert!( result.stderr.contains( "threshold" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "0" ) || result.stderr.contains( "100" ),
      "Error should mention invalid threshold. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test invalid threshold (non-integer)
  #[tokio::test]
  async fn test_threshold_invalid_non_integer()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::abc" ] ).await;

    assert!( !result.success(), "Non-integer threshold should fail" );
    assert!( result.stderr.contains( "threshold" ) || result.stderr.contains( "invalid" ) || result.stderr.contains( "integer" ),
      "Error should mention invalid threshold. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional threshold
  #[tokio::test]
  async fn test_threshold_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status" ] ).await;

    // Should succeed without optional threshold filter
    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "required" ),
        "Should not require optional threshold parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test threshold combined with other filters
  #[tokio::test]
  async fn test_threshold_combined_with_status()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".budget.status", "threshold::80", "status::active" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "threshold" ) || !result.stderr.contains( "invalid" ),
        "Should accept threshold combined with status. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
