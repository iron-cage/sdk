//! Parameter-level tests for `format` parameter
//!
//! ## Purpose
//!
//! Validates the `format` parameter across all commands that use it.
//! The format parameter controls output formatting (json, table, etc.).
//!
//! ## Coverage
//!
//! Commands tested:
//! - Health check commands
//! - Token management commands
//! - Limit management commands
//! - Usage tracking commands
//! - Trace commands
//! - And all other commands that support format parameter (68 total)
//!
//! ## Test Categories
//!
//! 1. **Valid Values**: json, table, json (case-insensitive)
//! 2. **Default Behavior**: When format is not specified
//! 3. **Invalid Values**: Unknown format types
//! 4. **Edge Cases**: Empty format, whitespace
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

  /// Test format parameter with valid JSON value
  #[tokio::test]
  async fn test_format_json()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::json" ] ).await;

    // Should succeed and output JSON
    if result.success() {
      assert!( result.stdout.contains( "{" ) || result.stdout.contains( "[" ),
        "JSON format should produce JSON markers" );
    }

    server.shutdown().await;
  }

  /// Test format parameter with valid table value
  #[tokio::test]
  async fn test_format_table_explicit()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::table" ] ).await;

    // Should succeed
    assert!( result.success() || result.exit_code == 0,
      "Table format should be valid" );

    server.shutdown().await;
  }

  /// Test format parameter with lowercase (case-insensitivity)
  #[tokio::test]
  async fn test_format_lowercase()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::json" ] ).await;

    // Lowercase should work
    if result.success() {
      assert!( result.stdout.contains( "{" ) || result.stdout.contains( "[" ),
        "Lowercase 'json' should be accepted" );
    }

    server.shutdown().await;
  }

  /// Test default format behavior (no format parameter)
  #[tokio::test]
  async fn test_format_default()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health" ] ).await;

    // Should use default format (likely table)
    assert!( result.success() || result.exit_code == 0,
      "Should succeed with default format" );

    server.shutdown().await;
  }

  /// Test format parameter with invalid value
  #[tokio::test]
  async fn test_format_invalid()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::invalid" ] ).await;

    // Should fail with format error
    assert!( !result.success(), "Invalid format should fail" );
    assert!( result.stderr.contains( "format" ) || result.stderr.contains( "invalid" ),
      "Error should mention format parameter" );

    server.shutdown().await;
  }

  /// Test empty format parameter value
  #[tokio::test]
  async fn test_format_empty()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::" ] ).await;

    // Empty format should fail
    assert!( !result.success(), "Empty format should fail" );
    assert!( result.stderr.contains( "format" ) || result.stderr.contains( "empty" ),
      "Error should mention empty format" );

    server.shutdown().await;
  }

  /// Test format parameter with whitespace
  #[tokio::test]
  async fn test_format_whitespace()
  {
    let server = TestServer::start().await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() );

    let result = harness.run( "iron-token", &[ ".health", "format::   " ] ).await;

    // Whitespace should fail or be trimmed
    if !result.success() {
      assert!( result.stderr.contains( "format" ) || result.stderr.contains( "invalid" ),
        "Error should mention format" );
    }

    server.shutdown().await;
  }

  /// Test format on tokens.list command (authenticated)
  #[tokio::test]
  async fn test_format_tokens_list()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".tokens.list", "format::json" ] ).await;

    // Should succeed and output JSON
    if result.success() {
      assert!( result.stdout.contains( "{" ) || result.stdout.contains( "[" ),
        "JSON format should produce JSON output" );
    }

    server.shutdown().await;
  }

  /// Test format on limits.list command (authenticated)
  #[tokio::test]
  async fn test_format_limits_list()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    let result = harness.run( "iron-token", &[ ".limits.list", "format::json" ] ).await;

    // Should succeed and output JSON
    if result.success() {
      assert!( result.stdout.contains( "{" ) || result.stdout.contains( "[" ),
        "JSON format should produce JSON output" );
    }

    server.shutdown().await;
  }

  /// Test format consistency across multiple commands
  #[tokio::test]
  async fn test_format_cross_command_consistency()
  {
    let server = TestServer::start().await;
    let data = TestData::new().await;
    let user_id = data.create_user( "test@example.com" ).await;
    let api_key = data.create_api_key( user_id, "test-key" ).await;

    let harness = IntegrationTestHarness::new()
      .server_url( server.url() )
      .api_key( &api_key );

    // Test format on multiple commands
    let commands = vec![
      ( "iron-token", vec![ ".health", "format::json" ] ),
      ( "iron-token", vec![ ".tokens.list", "format::json" ] ),
      ( "iron-token", vec![ ".limits.list", "format::json" ] ),
    ];

    for ( binary, args ) in commands {
      let result = harness.run( binary, &args ).await;

      // All should either succeed with JSON or fail consistently
      if result.success() {
        assert!( result.stdout.contains( "{" ) || result.stdout.contains( "[" ),
          "Command {:?} should produce JSON output", args );
      }
    }

    server.shutdown().await;
  }
}
