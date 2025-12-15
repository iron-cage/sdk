//! Server port configuration tests.
//!
//! Verifies SERVER_PORT loaded from environment variable, not hardcoded.

mod common;
use common::source_analysis::*;

/// Verify server port comes from environment variable, not hardcoded.
///
/// NEGATIVE ACCEPTANCE: Hardcoded port numbers forbidden in server source.
///
/// ## Purpose
/// Production deployment requires dynamic port configuration for different
/// environments (local dev, staging, production). Hardcoded ports prevent
/// environment-specific configuration.
///
/// ## What This Proves
/// - No hardcoded SocketAddr with port number in server source
/// - SERVER_PORT environment variable parsing exists
/// - Production enforcement (fails if SERVER_PORT missing)
///
/// ## Why This Matters
/// Hardcoded ports:
/// - Prevent multi-environment deployment
/// - Require recompilation for port changes
/// - Can't use same binary across environments
/// - Create port conflicts in containerized deployments
// test_kind: negative_acceptance
#[ test ]
fn test_no_hardcoded_server_port()
{
  let source_path = "src/bin/iron_control_api_server.rs";
  let source = read_source_file( source_path );

  // NEGATIVE ACCEPTANCE: No hardcoded SocketAddr with port
  assert_source_not_contains(
    &source,
    "SocketAddr::from( ( [0, 0, 0, 0], 3001 ) )",
    source_path,
    "Hardcoded port 3001 forbidden (must use SERVER_PORT env var)"
  );

  assert_source_not_contains(
    &source,
    "SocketAddr::from( ( [0, 0, 0, 0], 3000 ) )",
    source_path,
    "Hardcoded port 3000 forbidden (must use SERVER_PORT env var)"
  );

  assert_source_not_contains(
    &source,
    "SocketAddr::from(([0, 0, 0, 0], 3001))",
    source_path,
    "Hardcoded port forbidden (must use SERVER_PORT env var)"
  );

  // Must have env var parsing
  assert_source_contains(
    &source,
    "std::env::var( \"SERVER_PORT\" )",
    source_path,
    "SERVER_PORT environment variable parsing required"
  );

  // Must have production enforcement (panic/expect if missing)
  let has_enforcement =
    source.contains( "expect(" ) &&
    source.contains( "SERVER_PORT" );

  assert!(
    has_enforcement,
    "FAIL: Production enforcement missing in {}\\n\\
     Must panic/expect if SERVER_PORT not set in production",
    source_path
  );
}

/// Verify no fallback defeats purpose of env var.
///
/// ## Purpose
/// Using unwrap_or with default port defeats the purpose of environment
/// configuration. Server must REQUIRE SERVER_PORT explicitly.
/// unwrap_or_else with panic!() is acceptable for error handling.
// test_kind: negative_acceptance
#[ test ]
fn test_no_port_fallback()
{
  let source = read_source_file( "src/bin/iron_control_api_server.rs" );

  // Check for SERVER_PORT with unwrap_or providing a fallback value
  // (unwrap_or_else with panic!() is OK for validation)
  let has_port_fallback =
    source.contains( "SERVER_PORT" ) &&
    {
      // Find SERVER_PORT line and check nearby lines for unwrap_or
      let lines: Vec<&str> = source.lines().collect();
      let mut found_fallback = false;
      for ( i, line ) in lines.iter().enumerate()
      {
        if line.contains( "SERVER_PORT" )
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
    !has_port_fallback,
    "FAIL: SERVER_PORT uses unwrap_or with fallback value\\n\\
     unwrap_or defeats purpose (server must REQUIRE env var with expect/panic)"
  );
}

/// Verify port number is parsed as u16.
///
/// ## Purpose
/// Port numbers must be valid u16 (1-65535). Server must validate port
/// parsing and fail loudly if invalid.
// test_kind: negative_acceptance
#[ test ]
fn test_port_parsing_validation()
{
  let source = read_source_file( "src/bin/iron_control_api_server.rs" );

  // Must parse port as u16
  let has_u16_parsing =
    source.contains( "SERVER_PORT" ) &&
    ( source.contains( "parse::<u16>" ) || source.contains( "parse::< u16 >" ) );

  assert!(
    has_u16_parsing,
    "FAIL: SERVER_PORT must be parsed as u16\\n\\
     Port numbers are 16-bit unsigned integers (1-65535)"
  );
}
