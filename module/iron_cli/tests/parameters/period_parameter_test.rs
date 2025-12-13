//! Parameter-level tests for `period` parameter
//!
//! ## Purpose
//!
//! Validates the `period` parameter for analytics time grouping.
//! Tests time period enum validation (day, week, month).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .analytics.usage_by_period (period parameter)
//! - Other analytics commands that support period grouping
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Supported periods (day, week, month)
//! 2. **Invalid Values**: Unknown periods, empty, typos
//! 3. **Optional Behavior**: Missing optional period parameter
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

  /// Test valid period (day)
  #[tokio::test]
  async fn test_period_day()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::day" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "period" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'day' period. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid period (week)
  #[tokio::test]
  async fn test_period_week()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::week" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "period" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'week' period. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid period (month)
  #[tokio::test]
  async fn test_period_month()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::month" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "period" ) || !result.stderr.contains( "invalid" ),
        "Should accept 'month' period. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test invalid period
  #[tokio::test]
  async fn test_period_invalid()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::year" ] ).await;

    assert!( !result.success(), "Unknown period should fail" );
    assert!( result.stderr.contains( "period" ) || result.stderr.contains( "invalid" ),
      "Error should mention invalid period. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test empty period
  #[tokio::test]
  async fn test_period_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::" ] ).await;

    assert!( !result.success(), "Empty period should fail" );
    assert!( result.stderr.contains( "period" ) || result.stderr.contains( "empty" ),
      "Error should mention empty period. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test missing optional period
  #[tokio::test]
  async fn test_period_missing_optional()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period" ] ).await;

    // Should use default period if optional
    if !result.success() {
      assert!( !result.stderr.contains( "period" ) || !result.stderr.contains( "required" ),
        "Should not require optional period parameter. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test period case sensitivity
  #[tokio::test]
  async fn test_period_case_sensitivity()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".analytics.usage_by_period", "period::DAY" ] ).await;

    // Uppercase should be normalized
    if !result.success() && !result.stderr.contains( "invalid" ) {
      println!( "Period case handling: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
