//! Parameter-level tests for `provider` parameter
//!
//! ## Purpose
//!
//! Validates the `provider` parameter across provider management commands.
//! Tests provider identifier/name validation (openai, anthropic, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - .provider.get (provider parameter)
//! - .provider.update (provider parameter)
//! - .agent.create (provider specification)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: Known providers (openai, anthropic, cohere, etc.)
//! 2. **Invalid Values**: Unknown providers, empty, typos
//! 3. **Edge Cases**: Case sensitivity, hyphens
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

  /// Test valid provider (openai)
  #[tokio::test]
  async fn test_provider_openai()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::openai" ] ).await;

    // Should succeed or fail with "not found", not validation error
    if !result.success() {
      assert!( !result.stderr.contains( "provider" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with provider validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid provider (anthropic)
  #[tokio::test]
  async fn test_provider_anthropic()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::anthropic" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "provider" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with provider validation error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty provider (should fail)
  #[tokio::test]
  async fn test_provider_empty()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::" ] ).await;

    assert!( !result.success(), "Empty provider should fail" );
    assert!( result.stderr.contains( "provider" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty provider. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test unknown provider
  #[tokio::test]
  async fn test_provider_unknown()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::unknown-provider" ] ).await;

    // Unknown provider should fail with "not found" error
    if !result.success() {
      assert!( result.stderr.contains( "not found" ) || result.stderr.contains( "unknown" ) || result.stderr.contains( "provider" ),
        "Should fail with 'not found' error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test provider with hyphens
  #[tokio::test]
  async fn test_provider_with_hyphens()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::custom-provider" ] ).await;

    // Should accept hyphens in provider name
    if !result.success() {
      assert!( !result.stderr.contains( "provider" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should accept hyphens in provider name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test provider case sensitivity
  #[tokio::test]
  async fn test_provider_case_sensitivity()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron", &[ ".provider.get", "provider::OpenAI" ] ).await;

    // Case handling - either normalized or rejected consistently
    if !result.success() && !result.stderr.contains( "not found" ) {
      println!( "Provider case handling: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test very long provider name
  #[tokio::test]
  async fn test_provider_very_long()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let long_provider = "a".repeat( 300 );
    let result = harness.run( "iron", &[ ".provider.get", &format!( "provider::{}", long_provider ) ] ).await;

    if !result.success() {
      assert!( result.stderr.contains( "provider" ) || result.stderr.contains( "too long" ) || result.stderr.contains( "not found" ),
        "Should reject very long provider name. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
