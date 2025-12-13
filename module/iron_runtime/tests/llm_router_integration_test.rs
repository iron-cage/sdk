//! Integration tests for LLM Router
//!
//! These tests require:
//! - IC_TOKEN environment variable set
//! - IC_SERVER environment variable set (Iron Cage server URL)
//! - Provider key configured in Iron Cage dashboard
//!
//! Tests skip gracefully when env vars are not set (for CI compatibility).

use tracing::debug;

use iron_runtime::llm_router::LlmRouter;
use std::env;

// =============================================================================
// Helper functions
// =============================================================================

/// Get test credentials, returning None if env vars not set.
/// This allows tests to skip gracefully in CI without env vars.
fn get_test_credentials() -> Option< ( String, String ) >
{
  let token = env::var( "IC_TOKEN" ).ok()?;
  let server = env::var( "IC_SERVER" ).ok()?;
  Some( ( token, server ) )
}

/// Macro to skip test if credentials not available
macro_rules! require_credentials {
  () => {
    match get_test_credentials() {
      Some( creds ) => creds,
      None => {
        eprintln!( "SKIPPED: IC_TOKEN/IC_SERVER not set" );
        return;
      }
    }
  };
}

// =============================================================================
// Router lifecycle tests
// =============================================================================

#[test]
fn test_router_starts_and_stops()
{
  let ( token, server ) = require_credentials!();

  let mut router = LlmRouter::create( token, server, 300 ).expect("LOUD FAILURE: Failed to create router");

  assert!( router.running() );
  assert!( router.port > 0, "Router should be assigned a valid port" );
  assert!( router.get_base_url().contains( "127.0.0.1" ) );
  assert!( router.get_base_url().contains( "/v1" ) );

  // Provider should be detected
  let provider = &router.provider;
  assert!(
    provider == "openai" || provider == "anthropic" || provider == "unknown",
    "Unexpected provider: {}",
    provider
  );

  router.shutdown();
  assert!( !router.running() );
}

#[test]
fn test_router_provider_detection()
{
  let ( token, server ) = require_credentials!();

  let mut router = LlmRouter::create( token, server, 300 ).expect("LOUD FAILURE: Failed to create router");

  let provider = &router.provider;
  debug!( "Detected provider: {}", provider );

  // Provider should be one of the known values
  assert!(
    provider == "openai" || provider == "anthropic" || provider == "unknown",
    "Unexpected provider: {}",
    provider
  );

  router.shutdown();
}

#[test]
fn test_router_base_url_format()
{
  let ( token, server ) = require_credentials!();

  let mut router = LlmRouter::create( token, server, 300 ).expect("LOUD FAILURE: Failed to create router");

  let base_url = router.get_base_url();

  // Should be a valid local URL
  assert!( base_url.starts_with( "http://127.0.0.1:" ) );
  assert!( base_url.ends_with( "/v1" ) );

  // Port should be in valid range
  let port = router.port;
  assert!( port > 1024, "Port should be > 1024, got {}", port );
  assert!( port < 65535, "Port should be < 65535, got {}", port );

  router.shutdown();
}

#[test]
fn test_router_api_key_passthrough()
{
  let ( token, server ) = require_credentials!();

  let mut router = LlmRouter::create( token.clone(), server, 300 ).expect("LOUD FAILURE: Failed to create router");

  // API key should be the same as the input token
  assert_eq!( router.api_key, token, "Router should store the provided API key" );

  router.shutdown();
}

#[test]
fn test_router_custom_cache_ttl()
{
  let ( token, server ) = require_credentials!();

  // Create with custom cache TTL
  let mut router = LlmRouter::create( token, server, 60 ).expect("LOUD FAILURE: Failed to create router");

  assert!( router.running() );

  router.shutdown();
}

// =============================================================================
// Error handling tests
// =============================================================================

#[test]
fn test_router_invalid_server_url()
{
  let token = "test_token".to_string();
  let server = "http://invalid-server-that-does-not-exist:9999".to_string();

  // Router should still start (connection errors happen on first request)
  let mut router = LlmRouter::create( token, server, 300 ).expect("LOUD FAILURE: Failed to create router");

  assert!( router.running() );
  // Provider will be "unknown" since it couldn't fetch the key
  assert_eq!( router.provider, "unknown", "Provider should be unknown when server is unreachable" );

  router.shutdown();
}
