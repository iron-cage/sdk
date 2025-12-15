//! url_redirect deletion verification.
//!
//! NEGATIVE ACCEPTANCE: Proves url_redirect middleware completely removed.
//!
//! ## Purpose
//!
//! Verifies that backward compatibility middleware url_redirect has been
//! completely deleted (not just disabled or commented out).
//!
//! ## Why This Matters
//!
//! The url_redirect middleware served zero actual clients. Keeping unused
//! backward compatibility code creates:
//! - Maintenance burden (must update with API changes)
//! - API confusion (which endpoint should users call?)
//! - False sense of compatibility (nobody actually uses it)
//!
//! ## What This Test Suite Proves
//!
//! 1. File `src/middleware/url_redirect.rs` deleted
//! 2. Module export removed from `src/middleware/mod.rs`
//! 3. Router doesn't reference the middleware
//! 4. No import statements remain anywhere
//! 5. Old `/api/tokens` route definition removed

mod common;
use common::source_analysis::*;

/// Verify url_redirect.rs file deleted.
///
/// ## Purpose
///
/// Backward compatibility middleware url_redirect served zero clients.
/// Must be completely removed, not just disabled.
///
/// ## What This Proves
///
/// - url_redirect.rs file doesn't exist
/// - Not renamed (e.g., url_redirect_old.rs)
/// - Not moved to deprecated/ directory
/// - Completely deleted from repository
///
/// ## Why This Matters
///
/// Keeping unused code creates:
/// - Maintenance burden (must update with API changes)
/// - API confusion (which endpoint to use?)
/// - False sense of compatibility (nobody actually uses it)
// test_kind: negative_acceptance
#[ test ]
fn test_url_redirect_file_deleted()
{
  // NEGATIVE ACCEPTANCE: File must NOT exist
  let file_path = "src/middleware/url_redirect.rs";
  assert!(
    !std::path::Path::new( file_path ).exists(),
    "FAIL: url_redirect.rs still exists at {}\n\
     Must DELETE file completely (not rename, not comment out)\n\
     \n\
     Fix: git rm src/middleware/url_redirect.rs",
    file_path
  );
}

/// Verify no url_redirect module export.
///
/// ## Purpose
///
/// Module export must be removed along with file deletion.
/// Dead exports confuse developers and break compilation if someone tries to use them.
// test_kind: negative_acceptance
#[ test ]
fn test_no_url_redirect_module_export()
{
  let mod_file = read_source_file( "src/middleware/mod.rs" );

  assert_source_not_contains(
    &mod_file,
    "pub mod url_redirect",
    "src/middleware/mod.rs",
    "Module export must be removed (file deleted)"
  );

  assert_source_not_contains(
    &mod_file,
    "url_redirect",
    "src/middleware/mod.rs",
    "No references to url_redirect allowed (even in comments)"
  );
}

/// Verify no url_redirect usage in router.
///
/// ## Purpose
///
/// Router must not try to apply deleted middleware.
/// Dead middleware application causes compilation errors.
// test_kind: negative_acceptance
#[ test ]
fn test_no_url_redirect_in_router()
{
  let server = read_source_file( "src/bin/iron_control_api_server.rs" );

  assert_source_not_contains(
    &server,
    "url_redirect",
    "src/bin/iron_control_api_server.rs",
    "Router must not reference deleted middleware"
  );
}

/// Verify no url_redirect imports anywhere.
///
/// ## Purpose
///
/// Dead imports confuse code readers and may cause compilation errors.
/// All references must be removed, not just commented out.
// test_kind: negative_acceptance
#[ test ]
fn test_no_url_redirect_imports()
{
  // Search entire src/ directory for url_redirect imports
  let mut found_imports = Vec::new();

  for entry in walkdir::WalkDir::new( "src" )
  {
    let entry = entry.expect( "Failed to read directory entry" );
    if entry.path().extension().and_then( |s| s.to_str() ) == Some( "rs" )
    {
      let content = std::fs::read_to_string( entry.path() )
        .expect( "Failed to read file" );

      if content.contains( "use" ) && content.contains( "url_redirect" )
      {
        found_imports.push( entry.path().display().to_string() );
      }
    }
  }

  assert!(
    found_imports.is_empty(),
    "FAIL: Found url_redirect imports in {} files:\n{:#?}\n\
     All imports must be removed\n\
     \n\
     Fix: Remove all 'use' statements referencing url_redirect",
    found_imports.len(),
    found_imports
  );
}

/// Verify /api/tokens route doesn't exist.
///
/// ## Purpose
///
/// Old route /api/tokens redirected to /api/api-tokens.
/// With middleware deleted, old route must not be defined anywhere.
///
/// ## What This Proves
///
/// - No route handler for `/api/tokens`
/// - Only correct route `/api/api-tokens` exists
/// - No accidental route duplication
// test_kind: negative_acceptance
#[ test ]
fn test_old_tokens_route_removed()
{
  let server = read_source_file( "src/bin/iron_control_api_server.rs" );

  // Search for old route definition
  // Must NOT find standalone "/api/tokens" (without "api-" prefix)
  let lines: Vec<&str> = server.lines().collect();

  for ( i, line ) in lines.iter().enumerate()
  {
    // Look for route definitions containing "/api/tokens"
    if line.contains( "\"/api/tokens\"" )
      && !line.contains( "/api/api-tokens" )
      && ( line.contains( "route" ) || line.contains( "get" ) || line.contains( "post" ) )
    {
      panic!(
        "FAIL: Old route '/api/tokens' found at line {}\n\
         Line: {}\n\
         \n\
         Old /api/tokens route must be completely removed.\n\
         Only /api/api-tokens (correct route) should exist.\n\
         \n\
         Fix: Remove route definition for /api/tokens",
        i + 1,
        line
      );
    }
  }
}
