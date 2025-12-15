//! Test: Login endpoint requires ConnectInfo for per-IP rate limiting
//!
//! ## Root Cause
//!
//! The login handler in `auth/handlers.rs` uses `ConnectInfo<SocketAddr>` extractor
//! for per-IP rate limiting (Fix issue-GAP-006). However, the server startup code
//! in `bin/iron_control_api_server.rs` uses `axum::serve(listener, app)` which does
//! NOT provide the ConnectInfo extension. Axum requires explicit opt-in via
//! `.into_make_service_with_connect_info::<SocketAddr>()` to make client socket
//! addresses available to handlers.
//!
//! When a request reaches the login endpoint, Axum tries to extract ConnectInfo
//! but finds the extension missing, resulting in a 500 Internal Server Error with
//! message: "Missing request extension: Extension of type `ConnectInfo<SocketAddr>`
//! was not found."
//!
//! ## Why Not Caught
//!
//! 1. **No Integration Tests:** The existing auth tests use `TestServer` helper
//!    which may provide ConnectInfo automatically, masking the production issue
//! 2. **Manual Testing Gap:** The Iron Cage Pilot launch plan focused on API
//!    token endpoints but didnt include comprehensive auth endpoint testing
//! 3. **Code Review Blind Spot:** The ConnectInfo requirement was added in
//!    Fix(issue-GAP-006) but the reviewer didnt verify that the server startup
//!    code provides this extension
//!
//! ## Fix Applied
//!
//! Modified `bin/iron_control_api_server.rs` server startup:
//!
//! ```rust,no_run
//! // Before (BROKEN):
//! axum::serve( listener, app ).await?;
//!
//! // After (FIXED):
//! axum::serve(
//!   listener,
//!   app.into_make_service_with_connect_info::<SocketAddr>()
//! ).await?;
//! ```
//!
//! This explicitly opts into ConnectInfo extraction, making client socket addresses
//! available to all route handlers.
//!
//! ## Prevention
//!
//! 1. **Integration Test Coverage:** Add test that starts actual server (not
//!    TestServer helper) and verifies login works end-to-end
//! 2. **Handler Signature Validation:** If handler uses ConnectInfo extractor,
//!    CI must verify server provides it (could use compile-time check or runtime
//!    assertion on startup)
//! 3. **Manual Testing Checklist:** Include auth endpoints in launch verification,
//!    not just new feature endpoints
//! 4. **Documentation:** Add comment in server.rs next to serve() call explaining
//!    ConnectInfo requirement and consequences of removing it
//!
//! ## Pitfall
//!
//! **Never assume Axum extractors work without configuration.** Extractors like
//! ConnectInfo, Extension, and State require server-side setup. Always:
//! - Check extractor documentation for setup requirements
//! - Verify server provides required extensions/layers
//! - Test with real server instance (not just test helpers)
//! - Add comments documenting why specific server config is needed
//!
//! When adding extractors to handlers, audit the entire request path:
//! handler signature → middleware layers → server startup → listener config.
//! Missing any piece causes runtime failures.

#[ cfg( feature = "enabled" ) ]
#[ tokio::test ]
async fn bug_reproducer_login_requires_connect_info()
{
  use std::net::SocketAddr;
  use iron_control_api::routes::auth::AuthState;
  use axum::{ Router, routing::post };

  // Create in-memory database for testing (same pattern as common/auth.rs)
  let database_url = "sqlite::memory:?cache=shared";

  // Initialize auth state
  let auth_state = AuthState::new( "test-secret".to_string(), database_url )
    .await
    .expect( "Failed to initialize auth state" );

  // Build router WITHOUT ConnectInfo (reproduces bug)
  let app_without_connect_info = Router::new()
    .route( "/api/v1/auth/login", post( iron_control_api::routes::auth::login ) )
    .with_state( auth_state.clone() );

  // Start server WITHOUT into_make_service_with_connect_info (BROKEN)
  let listener = tokio::net::TcpListener::bind( "127.0.0.1:0" )
    .await
    .expect( "Failed to bind port" );
  let addr = listener.local_addr().expect( "Failed to get local addr" );

  tokio::spawn( async move
  {
    // BUG REPRODUCTION: Using plain serve without ConnectInfo
    axum::serve( listener, app_without_connect_info )
      .await
      .expect( "Server failed" );
  } );

  // Wait for server to start
  tokio::time::sleep( tokio::time::Duration::from_millis( 100 ) ).await;

  // Attempt login request
  let client = reqwest::Client::new();
  let response = client
    .post( format!( "http://{}/api/v1/auth/login", addr ) )
    .json( &serde_json::json!({
      "email": "admin@admin.com",
      "password": "testpass"
    }) )
    .send()
    .await
    .expect( "Request failed" );

  // Should fail with 500 due to missing ConnectInfo
  assert_eq!(
    response.status(),
    500,
    "Expected 500 Internal Server Error due to missing ConnectInfo"
  );

  let body = response.text().await.expect( "Failed to read body" );
  assert!(
    body.contains( "ConnectInfo" ),
    "Error message should mention missing ConnectInfo extension"
  );

  // Now test WITH ConnectInfo (shows fix works)
  let app_with_connect_info = Router::new()
    .route( "/api/v1/auth/login", post( iron_control_api::routes::auth::login ) )
    .with_state( auth_state );

  let listener2 = tokio::net::TcpListener::bind( "127.0.0.1:0" )
    .await
    .expect( "Failed to bind port" );
  let addr2 = listener2.local_addr().expect( "Failed to get local addr" );

  tokio::spawn( async move
  {
    // FIX: Using into_make_service_with_connect_info
    axum::serve(
      listener2,
      app_with_connect_info.into_make_service_with_connect_info::<SocketAddr>()
    )
      .await
      .expect( "Server failed" );
  } );

  tokio::time::sleep( tokio::time::Duration::from_millis( 100 ) ).await;

  // Attempt login request with fixed server
  let response2 = client
    .post( format!( "http://{}/api/v1/auth/login", addr2 ) )
    .json( &serde_json::json!({
      "email": "admin@admin.com",
      "password": "testpass"
    }) )
    .send()
    .await
    .expect( "Request failed" );

  // Should succeed (or fail with 401 if credentials wrong, but NOT 500)
  assert_ne!(
    response2.status(),
    500,
    "Should not return 500 when ConnectInfo is provided"
  );
}
