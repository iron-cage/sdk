//! Integration tests for iron_cli
//!
//! ## Test Coverage
//!
//! Tests complete end-to-end workflows:
//! - Handler validation (pure logic)
//! - Adapter orchestration (async I/O)
//! - Service integration (InMemoryAdapter)
//! - Configuration integration
//! - Formatter output
//!
//! ## Testing Strategy
//!
//! Uses InMemoryAdapter (real implementation, no mocking).
//! Tests verify complete stack integration.
//!
//! **Why InMemoryAdapter for Tests?**
//! - Fast: No network I/O overhead
//! - Deterministic: Same behavior every run
//! - Self-contained: No external dependencies (no API server needed)

// Test files are allowed to use println!/eprintln! for debugging
#![allow(clippy::disallowed_macros)]
//! - Real implementation: Not a mock - uses actual HashMap storage
//!
//! **Important: Feature Flag Required**
//! Integration tests compile as separate crates and don't get automatic cfg(test).
//! They require `feature = "test-adapter"` to access InMemoryAdapter, which is
//! enforced via compile_error! in production builds. See:
//! - `module/iron_cli/src/adapters/implementations/in_memory.rs` (obsolescence guard)
//! - `module/iron_cli/src/adapters/implementations/mod.rs` (cfg guards)
//!
//! Production code uses HttpAdapter for real API integration.

use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::{ AuthService, StorageService };
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use iron_cli::config::Config;
use std::collections::HashMap;
use std::sync::Arc;

fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

fn create_verified_command(command: &str, args: &[(&str, &str)]) -> MockVerifiedCommand
{
  let mut params = HashMap::new();
  for (key, value) in args
  {
    params.insert( key.to_string(), value.to_string() );
  }

  MockVerifiedCommand {
    #[ allow( dead_code ) ]
    command: command.to_string(),
    params,
  }
}

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
// Authentication Integration Tests
// ============================================================================

#[ tokio::test ]
async fn test_login_integration_success()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Setup: register user
  adapter.seed_user( "alice", "password123" );

  // Execute login
  let command = create_verified_command(
    ".auth.login",
    &[
      ( "email", "alice" ),
      ( "password", "password123" ),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Login should succeed" );
  let output = result.unwrap();
  assert!( output.contains( "success" ) );
}

#[ tokio::test ]
async fn test_login_integration_invalid_credentials()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Don't register user - should fail

  let command = create_verified_command(
    ".auth.login",
    &[
      ( "email", "nonexistent" ),
      ( "password", "wrong" ),
    ],
  );

  let result = iron_cli::adapters::auth::login_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid credentials" );
}

#[ tokio::test ]
async fn test_logout_integration_success()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Setup: login first
  adapter.seed_user( "bob", "secret" );
  let tokens = adapter.login( "bob", "secret" ).await.unwrap();
  let _ = adapter.save_tokens( &tokens ).await;

  // Execute logout
  let command = create_verified_command( ".auth.logout", &[] );

  let result = iron_cli::adapters::auth::logout_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Logout should succeed" );

  // Verify tokens cleared
  let tokens_after = adapter.load_tokens().await.unwrap();
  assert!( tokens_after.is_none(), "Tokens should be cleared" );
}

#[ tokio::test ]
async fn test_refresh_integration_success()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Setup: login first
  adapter.seed_user( "charlie", "pass123" );
  let tokens = adapter.login( "charlie", "pass123" ).await.unwrap();
  let _ = adapter.save_tokens( &tokens ).await;

  // Execute refresh
  let command = create_verified_command( ".auth.refresh", &[] );

  let result = iron_cli::adapters::auth::refresh_adapter(
    &command,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Refresh should succeed" );
}

// ============================================================================
// Token Management Integration Tests
// ============================================================================

#[ tokio::test ]
async fn test_list_tokens_integration()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".tokens.list", &[] );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "List should succeed" );
}

#[ tokio::test ]
async fn test_generate_token_integration()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[
      ( "name", "test-token" ),
      ( "scope", "read:tokens" ),
    ],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  if let Err(ref e) = result {
    eprintln!("Generate token error: {:?}", e);
  }
  assert!( result.is_ok(), "Generate should succeed" );
}

#[ tokio::test ]
async fn test_revoke_token_integration_not_found()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.revoke",
    &[( "name", "nonexistent" )],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail when token not found" );
}

// ============================================================================
// Configuration Integration Tests
// ============================================================================

#[ test ]
fn test_config_integration_cli_args()
{
  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );
  cli_args.insert( "api_url".to_string(), "https://custom.api".to_string() );

  let config = Config::with_cli_args( cli_args );

  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );
  assert_eq!( config.get( "api_url" ), Some( "https://custom.api".to_string() ) );
}

#[ test ]
fn test_config_integration_env_fallback()
{
  std::env::set_var( "IRON_CLI_FORMAT", "yaml" );

  let config = Config::from_env();

  assert_eq!( config.get( "format" ), Some( "yaml".to_string() ) );

  std::env::remove_var( "IRON_CLI_FORMAT" );
}

#[ test ]
fn test_config_integration_precedence()
{
  std::env::set_var( "IRON_CLI_FORMAT", "yaml" );

  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );

  let config = Config::builder()
    .with_iron_config()
    .with_cli_args( cli_args )
    .build();

  // CLI should override env
  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );

  std::env::remove_var( "IRON_CLI_FORMAT" );
}

// ============================================================================
// Multi-Layer Workflow Tests
// ============================================================================

#[ tokio::test ]
async fn test_full_auth_workflow()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  // 1. Register user
  adapter.seed_user( "workflow-user", "pass" );

  // 2. Login
  let login_cmd = create_verified_command(
    ".auth.login",
    &[
      ( "email", "workflow-user" ),
      ( "password", "pass" ),
    ],
  );

  let login_result = iron_cli::adapters::auth::login_adapter(
    &login_cmd,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( login_result.is_ok(), "Login should succeed" );

  // 3. Refresh
  let refresh_cmd = create_verified_command( ".auth.refresh", &[] );

  let refresh_result = iron_cli::adapters::auth::refresh_adapter(
    &refresh_cmd,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( refresh_result.is_ok(), "Refresh should succeed" );

  // 4. Logout
  let logout_cmd = create_verified_command( ".auth.logout", &[] );

  let logout_result = iron_cli::adapters::auth::logout_adapter(
    &logout_cmd,
    adapter.clone(),
    adapter.clone(),
    &formatter,
  ).await;

  assert!( logout_result.is_ok(), "Logout should succeed" );

  // 5. Verify tokens cleared
  let tokens = adapter.load_tokens().await.unwrap();
  assert!( tokens.is_none(), "Tokens should be cleared after logout" );
}

#[ tokio::test ]
async fn test_formatter_integration_all_formats()
{
  let adapter = create_test_adapter();

  let formats = vec![
    OutputFormat::Table,
    OutputFormat::Expanded,
    OutputFormat::Json,
    OutputFormat::Yaml,
  ];

  for format in formats
  {
    let formatter = TreeFmtFormatter::new( format );
    let command = create_verified_command( ".tokens.list", &[] );

    let result = iron_cli::adapters::tokens::list_tokens_adapter(
      &command,
      adapter.clone(),
      &formatter,
    ).await;

    assert!( result.is_ok(), "Should work with all formats" );
  }
}
