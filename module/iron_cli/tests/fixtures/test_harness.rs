//! IntegrationTestHarness - Real CLI execution for integration testing
//!
//! ## Purpose
//!
//! Executes real CLI binaries (iron-token, iron) via process spawn
//! for end-to-end parameter validation testing.
//!
//! ## No Mocking Policy
//!
//! This uses REAL CLI execution:
//! - Real process spawn via std::process::Command
//! - Real CLI binary execution (not mocked functions)
//! - Real stdout/stderr capture
//! - Real exit codes
//!
//! ## Architecture
//!
//! ```text
//! Test → TestHarness → spawn() → CLI Binary → HTTP → TestServer → TestData
//! ```
//!
//! ## TDD Status
//!
//! RED: ✅ Tests written and verified failing
//! GREEN: ✅ Minimal implementation passes all tests
//! REFACTOR: ✅ Code quality improvements applied
//!
//! ## Design Decisions
//!
//! **Why process spawn?**
//! - Tests complete CLI stack including argument parsing
//! - Catches integration issues that unit tests miss
//! - Validates real user experience
//!
//! **Why capture stdout/stderr?**
//! - Verifies output formatting
//! - Captures error messages for debugging
//! - Tests both success and error paths
//!
//! ## Usage Example
//!
//! ```rust
//! #[tokio::test]
//! async fn test_cli_parameter_validation()
//! {
//!   let server = TestServer::start().await;
//!   let data = TestData::new().await;
//!   let api_key = data.create_api_key(user_id, "test-key").await;
//!
//!   let harness = IntegrationTestHarness::new()
//!     .server_url(server.url())
//!     .api_key(&api_key);
//!
//!   let result = harness.run("iron-token", &[".health"]).await;
//!
//!   assert!(result.success(), "Command should succeed");
//!   assert!(result.stdout.contains("healthy"));
//! }
//! ```

use std::process::Command;

/// Result from CLI execution
pub struct CliResult
{
  pub stdout: String,
  pub stderr: String,
  pub exit_code: i32,
}

impl CliResult
{
  /// Check if command succeeded (exit code 0)
  pub fn success( &self ) -> bool
  {
    self.exit_code == 0
  }
}

pub struct IntegrationTestHarness
{
  server_url: Option< String >,
  api_key: Option< String >,
}

impl IntegrationTestHarness
{
  /// Create new test harness
  pub fn new() -> Self
  {
    Self {
      server_url: None,
      api_key: None,
    }
  }

  /// Set server URL for CLI to connect to
  pub fn server_url( mut self, url: impl Into< String > ) -> Self
  {
    self.server_url = Some( url.into() );
    self
  }

  /// Set API key for authentication
  pub fn api_key( mut self, key: impl Into< String > ) -> Self
  {
    self.api_key = Some( key.into() );
    self
  }

  /// Execute CLI command
  ///
  /// # Arguments
  ///
  /// * `binary` - Name of binary ("iron-token" or "iron")
  /// * `args` - Command arguments
  ///
  /// # Returns
  ///
  /// CLI execution result with stdout, stderr, and exit code
  ///
  /// # Panics
  ///
  /// Panics if CLI binary cannot be executed.
  /// This is acceptable for test infrastructure.
  pub async fn run( &self, binary: &str, args: &[ &str ] ) -> CliResult
  {
    // Execute via cargo run to ensure binary is up-to-date
    let mut cmd = Command::new( "cargo" );
    cmd.arg( "run" )
      .arg( "--bin" )
      .arg( binary )
      .arg( "--" );

    // Add CLI arguments
    for arg in args
    {
      cmd.arg( arg );
    }

    // Set environment variables if provided
    if let Some( url ) = &self.server_url
    {
      cmd.env( "IRON_CLI_API_URL", url );
    }

    if let Some( key ) = &self.api_key
    {
      cmd.env( "IRON_CLI_API_KEY", key );
    }

    // Execute and capture output
    let output = cmd.output()
      .expect( "LOUD FAILURE: Failed to execute CLI command" );

    CliResult {
      stdout: String::from_utf8_lossy( &output.stdout ).to_string(),
      stderr: String::from_utf8_lossy( &output.stderr ).to_string(),
      exit_code: output.status.code().unwrap_or( -1 ),
    }
  }
}

#[cfg(test)]
mod tests
{
  use super::*;

  /// RED Phase Test: Harness creates successfully
  #[tokio::test]
  async fn test_harness_creates()
  {
    let _harness = IntegrationTestHarness::new();
    // If we get here without panic, harness created successfully
  }

  /// RED Phase Test: Can execute CLI command
  #[tokio::test]
  async fn test_execute_cli_command()
  {
    let harness = IntegrationTestHarness::new()
      .server_url( "http://localhost:8080" )
      .api_key( "test-key" );

    let result = harness.run( "iron-token", &[ ".health" ] ).await;

    // Result should have stdout/stderr/exit_code
    assert!( !result.stdout.is_empty() || !result.stderr.is_empty(),
      "Should capture output" );
  }

  /// RED Phase Test: Success detection works
  #[tokio::test]
  async fn test_success_detection()
  {
    let harness = IntegrationTestHarness::new();

    let result = harness.run( "iron-token", &[ ".version" ] ).await;

    // .version should always succeed
    assert!( result.success(), "Version command should succeed" );
  }
}
