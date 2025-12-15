//! CORS configuration tests.
//!
//! Verifies CORS origins loaded from ALLOWED_ORIGINS environment variable,
//! not hardcoded in source code.

mod common;
use common::source_analysis::*;

/// Verify CORS origins come from environment variable, not hardcoded array.
///
/// NEGATIVE ACCEPTANCE: Hardcoded CORS arrays forbidden in source code.
///
/// ## Purpose
/// Production deployment requires dynamic CORS configuration for multiple
/// domains (ironcage.ai, app.ironcage.ai). Hardcoded arrays prevent
/// environment-specific configuration.
///
/// ## What This Proves
/// - No hardcoded allow_origin arrays in server source
/// - ALLOWED_ORIGINS environment variable parsing exists
/// - Production enforcement (fails if ALLOWED_ORIGINS missing)
///
/// ## Why This Matters
/// Hardcoded origins:
/// - Prevent multi-domain deployment
/// - Require recompilation for origin changes
/// - Can't use same binary across environments
/// - Create security risk (forgotten dev origins in production)
// test_kind: negative_acceptance
#[ test ]
fn test_no_hardcoded_cors_origins()
{
  let source_path = "src/bin/iron_control_api_server.rs";
  let source = read_source_file( source_path );

  // NEGATIVE ACCEPTANCE: No hardcoded CORS origin arrays
  assert_source_not_contains(
    &source,
    "allow_origin([",
    source_path,
    "Hardcoded CORS array forbidden (must use ALLOWED_ORIGINS env var)"
  );

  assert_source_not_contains(
    &source,
    "vec![\"http://localhost",
    source_path,
    "Hardcoded origin vector forbidden (must use ALLOWED_ORIGINS env var)"
  );

  // Must have env var parsing
  assert_source_contains(
    &source,
    "std::env::var( \"ALLOWED_ORIGINS\" )",
    source_path,
    "ALLOWED_ORIGINS environment variable parsing required"
  );

  // Must have production enforcement (panic/expect if missing)
  let has_enforcement =
    source.contains( "expect(" ) &&
    source.contains( "ALLOWED_ORIGINS" );

  assert!(
    has_enforcement,
    "FAIL: Production enforcement missing in {}\n\
     Must panic/expect if ALLOWED_ORIGINS not set in production",
    source_path
  );
}

/// Verify no fallback defeats purpose of env var.
///
/// ## Purpose
/// Using unwrap_or with default origins defeats the purpose of environment
/// configuration. Server must REQUIRE ALLOWED_ORIGINS explicitly.
/// unwrap_or_else with panic!() is acceptable for error handling.
// test_kind: negative_acceptance
#[ test ]
fn test_no_cors_fallback()
{
  let source = read_source_file( "src/bin/iron_control_api_server.rs" );

  // Check for ALLOWED_ORIGINS with unwrap_or providing a fallback value
  // (unwrap_or_else with panic!() is OK for validation)
  let has_cors_fallback =
    source.contains( "ALLOWED_ORIGINS" ) &&
    {
      // Find ALLOWED_ORIGINS line and check nearby lines for unwrap_or
      let lines: Vec<&str> = source.lines().collect();
      let mut found_fallback = false;
      for ( i, line ) in lines.iter().enumerate()
      {
        if line.contains( "ALLOWED_ORIGINS" )
        {
          // Check this line and next 5 lines for unwrap_or (but not unwrap_or_else with panic)
          for j in 0..6
          {
            if i + j < lines.len()
            {
              let check_line = lines[ i + j ];
              // Detect unwrap_or that provides a fallback (not panic)
              if check_line.contains( ".unwrap_or(" ) ||
                 ( check_line.contains( ".unwrap_or_else" ) &&
                   !check_line.contains( "panic!" ) )
              {
                found_fallback = true;
                break;
              }
            }
          }
        }
      }
      found_fallback
    };

  assert!(
    !has_cors_fallback,
    "FAIL: ALLOWED_ORIGINS uses unwrap_or with fallback value\n\
     unwrap_or defeats purpose (server must REQUIRE env var with expect/panic)"
  );
}

/// Integration test: Verify CORS origins loaded from environment at runtime.
///
/// ## Purpose
/// Prove that CORS middleware actually uses ALLOWED_ORIGINS environment
/// variable at runtime, not just at compile time.
///
/// ## Test Strategy
/// 1. Set ALLOWED_ORIGINS env var
/// 2. Create test server
/// 3. Make OPTIONS request with allowed origin
/// 4. Verify CORS headers present
// test_kind: integration
#[ tokio::test ]
async fn test_cors_respects_allowed_origins_env()
{
  // Set environment variable for test
  std::env::set_var(
    "ALLOWED_ORIGINS",
    "https://ironcage.ai,https://app.ironcage.ai"
  );

  // TODO: Requires test server creation utilities
  // See Phase 0.1 for test server helpers
  //
  // let app = create_test_app().await;
  // let addr = create_test_server( app ).await;
  //
  // let response = test_request( addr, "OPTIONS", "/api/api-tokens" )
  //   .header( "Origin", "https://ironcage.ai" )
  //   .header( "Access-Control-Request-Method", "GET" )
  //   .send()
  //   .await;
  //
  // assert_eq!( response.status(), 200 );
  // assert_eq!(
  //   response.headers().get( "Access-Control-Allow-Origin" ).unwrap(),
  //   "https://ironcage.ai"
  // );

  // Clean up
  std::env::remove_var( "ALLOWED_ORIGINS" );
}

/// Integration test: Verify disallowed origins rejected.
// test_kind: integration
#[ tokio::test ]
async fn test_cors_blocks_disallowed_origins()
{
  std::env::set_var( "ALLOWED_ORIGINS", "https://ironcage.ai" );

  // TODO: Test server utilities
  //
  // let app = create_test_app().await;
  // let addr = create_test_server( app ).await;
  //
  // let response = test_request( addr, "OPTIONS", "/api/api-tokens" )
  //   .header( "Origin", "https://evil.com" )
  //   .send()
  //   .await;
  //
  // // No CORS headers for disallowed origin
  // assert!(
  //   response.headers().get( "Access-Control-Allow-Origin" ).is_none(),
  //   "FAIL: Disallowed origin not blocked"
  // );

  std::env::remove_var( "ALLOWED_ORIGINS" );
}
