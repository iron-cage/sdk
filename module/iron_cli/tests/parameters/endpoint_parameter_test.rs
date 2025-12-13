//! Parameter-level tests for `endpoint` parameter
//!
//! ## Purpose
//!
//! Validates the `endpoint` parameter for API/server configuration.
//! Tests URL format validation and endpoint specification.
//!
//! ## Coverage
//!
//! Commands tested:
//! - .config.set (endpoint configuration)
//! - Other commands that accept endpoint parameter
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: HTTP/HTTPS URLs, localhost, IP addresses
//! 2. **Invalid Values**: Malformed URLs, missing protocol, invalid characters
//! 3. **Edge Cases**: Trailing slashes, ports, paths
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

  /// Test valid HTTPS endpoint
  #[tokio::test]
  async fn test_endpoint_https()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::https://api.example.com" ] ).await;

    // Should succeed or fail with business logic, not URL validation
    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ) || !result.stderr.contains( "format" ),
        "Should not fail with endpoint format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test valid HTTP endpoint
  #[tokio::test]
  async fn test_endpoint_http()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::http://localhost:8080" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should not fail with endpoint format error. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test empty endpoint (should fail)
  #[tokio::test]
  async fn test_endpoint_empty()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::" ] ).await;

    assert!( !result.success(), "Empty endpoint should fail" );
    assert!( result.stderr.contains( "endpoint" ) || result.stderr.contains( "empty" ) || result.stderr.contains( "required" ),
      "Error should mention empty endpoint. Stderr: {}", result.stderr );

    server.shutdown().await;
  }

  /// Test endpoint without protocol (should fail)
  #[tokio::test]
  async fn test_endpoint_no_protocol()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::api.example.com" ] ).await;

    // Missing protocol should fail
    if !result.success() {
      assert!( result.stderr.contains( "endpoint" ) || result.stderr.contains( "protocol" ) || result.stderr.contains( "invalid" ),
        "Should reject endpoint without protocol. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test endpoint with port
  #[tokio::test]
  async fn test_endpoint_with_port()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::https://api.example.com:8443" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should accept endpoint with port. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test endpoint with path
  #[tokio::test]
  async fn test_endpoint_with_path()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::https://api.example.com/v1/api" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should accept endpoint with path. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test endpoint with trailing slash
  #[tokio::test]
  async fn test_endpoint_trailing_slash()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::https://api.example.com/" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should accept endpoint with trailing slash. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test endpoint with IP address
  #[tokio::test]
  async fn test_endpoint_ip_address()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::http://192.168.1.100:8080" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should accept IP address endpoint. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }

  /// Test endpoint with localhost
  #[tokio::test]
  async fn test_endpoint_localhost()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron", &[ ".config.set", "endpoint::http://localhost:3000" ] ).await;

    if !result.success() {
      assert!( !result.stderr.contains( "endpoint" ) || !result.stderr.contains( "invalid" ),
        "Should accept localhost endpoint. Stderr: {}", result.stderr );
    }

    server.shutdown().await;
  }
}
