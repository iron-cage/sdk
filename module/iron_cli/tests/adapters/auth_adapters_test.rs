//! Authentication adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 3 authentication adapters: login, refresh, logout
//! Total: 15 tests (5 per adapter)
//!
//! ## Architecture Under Test
//!
//! ```text
//! VerifiedCommand → login_adapter() → login_handler() → AuthService
//!                         ↓                   ↓              ↓
//!                    Extract params      Validate      Async I/O
//! ```
//!
//! ## Testing Strategy
//!
//! Uses InMemoryAdapter (real implementation, no mocking):
//! - Predictable: HashMap-based storage
//! - Fast: No network/DB overhead
//! - Real: Same interface as SqlxAdapter
//!
//! ## Test Matrix
//!
//! See tests/adapters/-test_matrix.md for complete specification

use iron_cli::adapters::{ AdapterError, ServiceError, AuthService };
use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ Formatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

/// Helper: Create test adapter with empty storage
fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

/// Helper: Create test adapter with pre-seeded user
fn create_adapter_with_user() -> Arc<InMemoryAdapter>
{
  let adapter = Arc::new( InMemoryAdapter::new() );
  adapter.seed_user( "alice@example.com", "password123" );
  adapter
}

/// Helper: Create VerifiedCommand mock for testing
fn create_verified_command(command: &str, args: &[(&str, &str)]) -> MockVerifiedCommand
{
  let mut params = HashMap::new();
  for (key, value) in args
  {
    params.insert( key.to_string(), value.to_string() );
  }

  MockVerifiedCommand {
    command: command.to_string(),
    params,
  }
}

/// Mock VerifiedCommand for testing (until unilang types available)
struct MockVerifiedCommand
{
  #[ allow( dead_code ) ]
  command: String,
  params: HashMap<String, String>,
}

impl HasParams for MockVerifiedCommand
{
  fn get_params(&self) -> HashMap<String, String>
  {
    self.params.clone()
  }
}

// ============================================================================
// .auth.login adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_login_adapter_success()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".auth.login",
    &[
      ("username", "alice@example.com"),
      ("password", "password123"),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid credentials" );

  let output = result.unwrap();
  assert!( output.contains( "alice" ) || output.contains( "login" ), "Output should mention user or login" );

  // Verify tokens were stored
  assert!( adapter.has_tokens(), "Tokens should be stored after login" );
}

#[ tokio::test ]
async fn test_login_adapter_missing_username()
{
  let adapter = create_test_adapter();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".auth.login",
    &[("password", "password123")],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without username" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError( e ) =>
    {
      assert!(
        e.to_string().contains( "username" ),
        "Error should mention missing username"
      );
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_login_adapter_invalid_credentials()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".auth.login",
    &[
      ("username", "alice@example.com"),
      ("password", "wrong_password"),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with wrong password" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::Unauthorized ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }

  // Verify no tokens stored
  assert!( !adapter.has_tokens(), "Tokens should not be stored on failed login" );
}

#[ tokio::test ]
async fn test_login_adapter_network_error()
{
  let adapter = create_test_adapter();
  adapter.set_failure_mode( "network_error" ); // InMemoryAdapter simulates failures

  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".auth.login",
    &[
      ("username", "alice@example.com"),
      ("password", "password123"),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail on network error" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NetworkError( _ ) ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_login_adapter_json_format()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".auth.login",
    &[
      ("username", "alice@example.com"),
      ("password", "password123"),
      ("format", "json"),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with JSON format" );

  let output = result.unwrap();

  // Verify JSON structure
  let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str( &output );
  assert!( parsed.is_ok(), "Output should be valid JSON" );
}

// ============================================================================
// .auth.refresh adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_refresh_adapter_success()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // First login to get tokens
  adapter.login( "alice@example.com", "password123" ).await.unwrap();

  let command = create_verified_command( ".auth.refresh", &[] );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid refresh token" );

  let output = result.unwrap();
  assert!( output.contains( "refresh" ) || output.contains( "token" ), "Output should mention refresh or token" );

  // Verify new tokens stored
  assert!( adapter.has_tokens(), "New tokens should be stored after refresh" );
}

#[ tokio::test ]
async fn test_refresh_adapter_no_stored_token()
{
  let adapter = create_test_adapter(); // No login, no tokens
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".auth.refresh", &[] );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without stored refresh token" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NotFound ) =>
    {
      // Expected: no token found in storage
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_refresh_adapter_expired_token()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login and then expire the token
  adapter.login( "alice@example.com", "password123" ).await.unwrap();
  adapter.expire_tokens(); // InMemoryAdapter test helper

  let command = create_verified_command( ".auth.refresh", &[] );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with expired token" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::Unauthorized ) =>
    {
      // Expected: expired token = unauthorized
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_refresh_adapter_dry_run()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login to get initial tokens
  adapter.login( "alice@example.com", "password123" ).await.unwrap();
  let initial_tokens = adapter.get_tokens().unwrap();

  let command = create_verified_command(
    ".auth.refresh",
    &[("dry_run", "true")],
  );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Dry-run should succeed" );

  // Verify tokens UNCHANGED
  let current_tokens = adapter.get_tokens().unwrap();
  assert_eq!(
    initial_tokens.access_token,
    current_tokens.access_token,
    "Tokens should not change in dry-run mode"
  );
}

#[ tokio::test ]
async fn test_refresh_adapter_yaml_format()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Yaml );

  // Login first
  adapter.login( "alice@example.com", "password123" ).await.unwrap();

  let command = create_verified_command(
    ".auth.refresh",
    &[("format", "yaml")],
  );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with YAML format" );

  let output = result.unwrap();

  // Verify YAML structure (simple check for YAML markers)
  assert!(
    output.contains( ":" ) && !output.starts_with( "{" ),
    "Output should be YAML format (key: value)"
  );
}

// ============================================================================
// .auth.logout adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_logout_adapter_success()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login first
  adapter.login( "alice@example.com", "password123" ).await.unwrap();
  assert!( adapter.has_tokens(), "Should have tokens before logout" );

  let command = create_verified_command( ".auth.logout", &[] );

  let result = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Logout should succeed" );

  let output = result.unwrap();
  assert!( output.contains( "logout" ) || output.contains( "success" ), "Output should confirm logout" );

  // Verify tokens cleared
  assert!( !adapter.has_tokens(), "Tokens should be cleared after logout" );
}

#[ tokio::test ]
async fn test_logout_adapter_no_token()
{
  let adapter = create_test_adapter(); // No login
  let formatter = Formatter::new( OutputFormat::Table );

  let command = create_verified_command( ".auth.logout", &[] );

  let result = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  // Logout when not logged in should succeed (idempotent)
  assert!( result.is_ok(), "Logout should succeed even without tokens" );

  let output = result.unwrap();
  assert!(
    output.contains( "not logged in" ) || output.contains( "already" ),
    "Output should indicate already logged out"
  );
}

#[ tokio::test ]
async fn test_logout_adapter_clears_storage()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login and verify tokens exist
  adapter.login( "alice@example.com", "password123" ).await.unwrap();
  assert!( adapter.has_tokens(), "Tokens should exist before logout" );

  let command = create_verified_command( ".auth.logout", &[] );

  let _ = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  // Explicitly verify storage is empty
  assert!(
    !adapter.has_tokens(),
    "Storage should be empty after logout"
  );

  let tokens_result = adapter.get_tokens();
  assert!(
    tokens_result.is_none(),
    "get_tokens() should return None after logout"
  );
}

#[ tokio::test ]
async fn test_logout_adapter_dry_run()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login first
  adapter.login( "alice@example.com", "password123" ).await.unwrap();
  assert!( adapter.has_tokens(), "Should have tokens before dry-run logout" );

  let command = create_verified_command(
    ".auth.logout",
    &[("dry_run", "true")],
  );

  let result = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Dry-run logout should succeed" );

  // Verify tokens STILL EXIST (dry-run = no state changes)
  assert!(
    adapter.has_tokens(),
    "Tokens should remain after dry-run logout"
  );
}

#[ tokio::test ]
async fn test_logout_adapter_table_format()
{
  let adapter = create_adapter_with_user();
  let formatter = Formatter::new( OutputFormat::Table );

  // Login first
  adapter.login( "alice@example.com", "password123" ).await.unwrap();

  let command = create_verified_command(
    ".auth.logout",
    &[("format", "table")],
  );

  let result = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with table format" );

  let output = result.unwrap();

  // Verify table format (simple text output)
  assert!(
    !output.starts_with( "{" ) && !output.starts_with( "[" ),
    "Table format should not be JSON"
  );
}
