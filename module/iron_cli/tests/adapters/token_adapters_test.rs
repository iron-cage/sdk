//! Token adapter tests
//!
//! ## Test Coverage
//!
//! Tests for 5 token management adapters: generate, list, get, rotate, revoke
//! Total: 25 tests (5 per adapter)
//!
//! ## Architecture Under Test
//!
//! ```text
//! VerifiedCommand → token_adapter() → token_handler() → TokenService
//!                         ↓                   ↓              ↓
//!                    Extract params      Validate      Async I/O
//! ```
//!
//! ## Testing Strategy
//!
//! Uses InMemoryAdapter (real implementation, no mocking):
//! - Predictable: HashMap-based token storage
//! - Fast: No network/DB overhead
//! - Real: Same interface as SqlxAdapter
//!
//! ## Test Matrix
//!
//! See tests/adapters/-test_matrix.md for complete specification

use iron_cli::adapters::{ AdapterError, ServiceError, TokenService };
use iron_cli::adapters::implementations::InMemoryAdapter;
use iron_cli::adapters::auth::HasParams;
use iron_cli::formatting::{ TreeFmtFormatter, OutputFormat };
use std::collections::HashMap;
use std::sync::Arc;

/// Helper: Create test adapter with empty token storage
fn create_test_adapter() -> Arc<InMemoryAdapter>
{
  Arc::new( InMemoryAdapter::new() )
}

/// Helper: Create test adapter with pre-seeded tokens
async fn create_adapter_with_tokens() -> Arc<InMemoryAdapter>
{
  let adapter = Arc::new( InMemoryAdapter::new() );

  // Seed some tokens
  let _ = adapter.generate( "test-token-1", "read:tokens", Some( 3600 ) ).await;
  let _ = adapter.generate( "test-token-2", "write:tokens", Some( 7200 ) ).await;
  let _ = adapter.generate( "test-token-3", "admin:tokens", None ).await;

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
// .tokens.generate adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_generate_token_adapter_success()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[
      ("name", "my-token"),
      ("scope", "read:tokens"),
      ("ttl", "3600"),
    ],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid params" );

  let output = result.unwrap();
  assert!(
    output.contains( "my-token" ) || output.contains( "tok_" ),
    "Output should contain token name or ID"
  );
}

#[ tokio::test ]
async fn test_generate_token_adapter_missing_name()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[("scope", "read:tokens")],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without name" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError( e ) =>
    {
      assert!(
        e.to_string().contains( "name" ),
        "Error should mention missing name"
      );
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_generate_token_adapter_invalid_scope()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[
      ("name", "my-token"),
      ("scope", "invalid_scope_no_colon"),
    ],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid scope format" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError( e ) =>
    {
      assert!(
        e.to_string().contains( "scope" ),
        "Error should mention scope validation failure"
      );
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_generate_token_adapter_storage_error()
{
  let adapter = create_test_adapter();
  adapter.set_failure_mode( "database_error" );

  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[
      ("name", "my-token"),
      ("scope", "read:tokens"),
    ],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail on storage error" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::DatabaseError( _ ) ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_generate_token_adapter_dry_run()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.generate",
    &[
      ("name", "my-token"),
      ("scope", "read:tokens"),
      ("dry_run", "true"),
    ],
  );

  let result = iron_cli::adapters::tokens::generate_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Dry-run should succeed" );

  // Verify token was NOT actually created
  let tokens_list = adapter.list( None ).await.unwrap();
  assert_eq!( tokens_list.len(), 0, "No tokens should be created in dry-run mode" );
}

// ============================================================================
// .tokens.list adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_list_tokens_adapter_success_empty()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".tokens.list", &[] );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with empty list" );

  let output = result.unwrap();
  assert!(
    output.contains( "No tokens" ) || output.is_empty() || output.contains( "0" ),
    "Output should indicate empty list"
  );
}

#[ tokio::test ]
async fn test_list_tokens_adapter_success_multiple()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".tokens.list", &[] );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with multiple tokens" );

  let output = result.unwrap();
  assert!(
    output.contains( "test-token" ) || output.contains( "tok_" ),
    "Output should contain token information"
  );
}

#[ tokio::test ]
async fn test_list_tokens_adapter_with_filter()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.list",
    &[("filter", "active")],
  );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with filter" );
}

#[ tokio::test ]
async fn test_list_tokens_adapter_storage_error()
{
  let adapter = create_test_adapter();
  adapter.set_failure_mode( "database_error" );

  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".tokens.list", &[] );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail on storage error" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::DatabaseError( _ ) ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_list_tokens_adapter_json_format()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Json );

  let command = create_verified_command(
    ".tokens.list",
    &[("format", "json")],
  );

  let result = iron_cli::adapters::tokens::list_tokens_adapter(
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
// .tokens.get adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_get_token_adapter_success()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.get",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::get_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with valid token_id" );

  let output = result.unwrap();
  assert!(
    output.contains( "test-token-1" ) || output.contains( "tok_" ),
    "Output should contain token details"
  );
}

#[ tokio::test ]
async fn test_get_token_adapter_missing_id()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command( ".tokens.get", &[] );

  let result = iron_cli::adapters::tokens::get_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail without token_id" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError( e ) =>
    {
      assert!(
        e.to_string().contains( "token_id" ) || e.to_string().contains( "token-id" ),
        "Error should mention missing token_id"
      );
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_get_token_adapter_not_found()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.get",
    &[("token_id", "tok_nonexistent")],
  );

  let result = iron_cli::adapters::tokens::get_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with nonexistent token" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NotFound ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_get_token_adapter_invalid_format()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.get",
    &[("token_id", "invalid_format_no_prefix")],
  );

  let result = iron_cli::adapters::tokens::get_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with invalid token_id format" );

  match result.unwrap_err()
  {
    AdapterError::HandlerError( e ) =>
    {
      assert!(
        e.to_string().contains( "tok_" ) || e.to_string().contains( "format" ),
        "Error should mention token_id format requirement"
      );
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_get_token_adapter_expanded_format()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Expanded );

  let command = create_verified_command(
    ".tokens.get",
    &[
      ("token_id", "tok_test-token-1"),
      ("format", "expanded"),
    ],
  );

  let result = iron_cli::adapters::tokens::get_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with expanded format" );
}

// ============================================================================
// .tokens.rotate adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_rotate_token_adapter_success()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.rotate",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::rotate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed rotating existing token" );

  let output = result.unwrap();
  assert!(
    output.contains( "rotated" ) || output.contains( "success" ),
    "Output should confirm rotation"
  );
}

#[ tokio::test ]
async fn test_rotate_token_adapter_not_found()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.rotate",
    &[("token_id", "tok_nonexistent")],
  );

  let result = iron_cli::adapters::tokens::rotate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with nonexistent token" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NotFound ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_rotate_token_adapter_preserves_scope()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Get original scope
  let original_token = adapter.get( "tok_test-token-1" ).await.unwrap();
  let original_scope = original_token.scope.clone();

  let command = create_verified_command(
    ".tokens.rotate",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::rotate_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Rotation should succeed" );

  // Verify scope unchanged
  let rotated_token = adapter.get( "tok_test-token-1" ).await.unwrap();
  assert_eq!(
    original_scope,
    rotated_token.scope,
    "Scope should be preserved after rotation"
  );
}

#[ tokio::test ]
async fn test_rotate_token_adapter_storage_error()
{
  let adapter = create_adapter_with_tokens().await;
  adapter.set_failure_mode( "database_error" );

  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.rotate",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::rotate_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail on storage error" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::DatabaseError( _ ) ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_rotate_token_adapter_dry_run()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // Get original token state
  let original_token = adapter.get( "tok_test-token-1" ).await.unwrap();

  let command = create_verified_command(
    ".tokens.rotate",
    &[
      ("token_id", "tok_test-token-1"),
      ("dry_run", "true"),
    ],
  );

  let result = iron_cli::adapters::tokens::rotate_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Dry-run should succeed" );

  // Verify token unchanged
  let current_token = adapter.get( "tok_test-token-1" ).await.unwrap();
  assert_eq!(
    original_token.expires_at,
    current_token.expires_at,
    "Token should not change in dry-run mode"
  );
}

// ============================================================================
// .tokens.revoke adapter tests (5 tests)
// ============================================================================

#[ tokio::test ]
async fn test_revoke_token_adapter_success()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.revoke",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed revoking existing token" );

  // Verify token was actually revoked (should not exist)
  let get_result = adapter.get( "tok_test-token-1" ).await;
  assert!( get_result.is_err(), "Token should not exist after revocation" );
}

#[ tokio::test ]
async fn test_revoke_token_adapter_not_found()
{
  let adapter = create_test_adapter();
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.revoke",
    &[("token_id", "tok_nonexistent")],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail with nonexistent token" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NotFound ) =>
    {
      // Expected error type
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_revoke_token_adapter_with_reason()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.revoke",
    &[
      ("token_id", "tok_test-token-1"),
      ("reason", "compromised"),
    ],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_ok(), "Should succeed with reason" );

  let output = result.unwrap();
  assert!(
    output.contains( "compromised" ) || output.contains( "revoked" ),
    "Output should mention revocation/reason"
  );
}

#[ tokio::test ]
async fn test_revoke_token_adapter_already_revoked()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  // First revocation
  let command1 = create_verified_command(
    ".tokens.revoke",
    &[("token_id", "tok_test-token-1")],
  );

  let _ = iron_cli::adapters::tokens::revoke_token_adapter(
    &command1,
    adapter.clone(),
    &formatter,
  ).await;

  // Second revocation (should fail - token already gone)
  let command2 = create_verified_command(
    ".tokens.revoke",
    &[("token_id", "tok_test-token-1")],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command2,
    adapter,
    &formatter,
  ).await;

  assert!( result.is_err(), "Should fail when token already revoked" );

  match result.unwrap_err()
  {
    AdapterError::ServiceError( ServiceError::NotFound ) =>
    {
      // Expected: token already gone
    }
    other => panic!( "Wrong error type: {:?}", other ),
  }
}

#[ tokio::test ]
async fn test_revoke_token_adapter_dry_run()
{
  let adapter = create_adapter_with_tokens().await;
  let formatter = TreeFmtFormatter::new( OutputFormat::Table );

  let command = create_verified_command(
    ".tokens.revoke",
    &[
      ("token_id", "tok_test-token-1"),
      ("dry_run", "true"),
    ],
  );

  let result = iron_cli::adapters::tokens::revoke_token_adapter(
    &command,
    adapter.clone(),
    &formatter,
  ).await;

  assert!( result.is_ok(), "Dry-run should succeed" );

  // Verify token still exists
  let get_result = adapter.get( "tok_test-token-1" ).await;
  assert!( get_result.is_ok(), "Token should still exist in dry-run mode" );
}
