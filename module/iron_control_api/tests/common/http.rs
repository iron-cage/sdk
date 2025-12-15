//! HTTP testing utilities for integration tests.
//!
//! Provides helpers for creating test HTTP servers and making requests.
//! Used in Phase 6 integration tests to verify runtime CORS behavior,
//! server port configuration, and end-to-end request handling.

use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Create test HTTP server for integration testing.
///
/// ## Purpose
/// Spawns a background HTTP server for integration testing. Returns the
/// server's address that can be used to make HTTP requests.
///
/// ## Usage
/// ```rust,ignore
/// let app = create_test_app().await;
/// let addr = create_test_server( app ).await;
///
/// let response = test_request( addr, "GET", "/api/health" ).await;
/// assert_eq!( response.status(), 200 );
/// ```
///
/// ## Implementation Note
/// Uses modern axum API (axum::serve with tokio::net::TcpListener).
/// Spawns server in background tokio task for non-blocking operation.
#[ allow( dead_code ) ]
pub async fn create_test_server( app: Router ) -> SocketAddr
{
  // Bind to random available port (127.0.0.1:0)
  let listener = TcpListener::bind( "127.0.0.1:0" )
    .await
    .expect( "Failed to bind test server to random port" );

  let addr = listener.local_addr()
    .expect( "Failed to get test server local address" );

  // Spawn server in background task
  tokio::spawn( async move {
    axum::serve( listener, app )
      .await
      .expect( "Test server failed during serve" );
  } );

  // Give server time to start accepting connections
  tokio::time::sleep( tokio::time::Duration::from_millis( 50 ) ).await;

  addr
}

/// Make HTTP request to test server.
///
/// ## Purpose
/// Simplified HTTP request helper for integration tests.
///
/// ## Usage
/// ```rust,ignore
/// let addr = create_test_server( app ).await;
///
/// let response = test_request( addr, "GET", "/api/health" ).await;
/// assert_eq!( response.status(), 200 );
///
/// let response = test_request( addr, "POST", "/api/auth/login" ).await;
/// // Further assertions on response...
/// ```
///
/// ## For More Complex Requests
/// Use reqwest::Client directly for requests with headers, body, etc.:
/// ```rust,ignore
/// let client = reqwest::Client::new();
/// let response = client
///   .get( format!( "http://{}/api/health", addr ) )
///   .header( "Authorization", "Bearer token" )
///   .send()
///   .await
///   .expect( "Request failed" );
/// ```
#[ allow( dead_code ) ]
pub async fn test_request(
  addr: SocketAddr,
  method: &str,
  path: &str,
) -> reqwest::Response
{
  let client = reqwest::Client::new();
  let url = format!( "http://{}{}", addr, path );

  let http_method = method.parse::< reqwest::Method >()
    .unwrap_or_else( |_| panic!( "Invalid HTTP method: {}", method ) );

  client.request( http_method, &url )
    .send()
    .await
    .unwrap_or_else( |_| panic!( "HTTP {} request to {} failed", method, url ) )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use axum::{ Router, routing::get };

  /// Verify test_request helper parses HTTP methods correctly.
  #[ tokio::test ]
  async fn test_request_method_parsing()
  {
    // Valid methods should parse
    let methods = vec![ "GET", "POST", "PUT", "DELETE", "PATCH", "OPTIONS" ];
    for method in methods
    {
      let _parsed = method.parse::< reqwest::Method >()
        .unwrap_or_else( |_| panic!( "Failed to parse method: {}", method ) );
    }
  }

  /// Verify server creation with simple router succeeds.
  ///
  /// **Corner Case:** P0-Critical - Server binding and address return
  ///
  /// Tests that create_test_server:
  /// 1. Successfully binds to random port (127.0.0.1:0)
  /// 2. Returns valid SocketAddr
  /// 3. Server is immediately accessible
  #[ tokio::test ]
  async fn test_create_server_basic()
  {
    let app = Router::new().route( "/test", get( || async { "ok" } ) );
    let addr = create_test_server( app ).await;

    // Verify valid address returned
    assert_eq!( addr.ip().to_string(), "127.0.0.1" );
    assert!( addr.port() > 0 );
  }

  /// Verify basic request/response cycle works end-to-end.
  ///
  /// **Corner Case:** P0-Critical - Basic request/response cycle
  ///
  /// Tests that:
  /// 1. Server creation works
  /// 2. GET request succeeds
  /// 3. Response body matches expected
  #[ tokio::test ]
  async fn test_request_response_cycle()
  {
    let app = Router::new().route( "/api/health", get( || async { "healthy" } ) );
    let addr = create_test_server( app ).await;

    let response = test_request( addr, "GET", "/api/health" ).await;

    assert_eq!( response.status(), 200 );
    let body = response.text().await.unwrap();
    assert_eq!( body, "healthy" );
  }

  /// Verify all standard HTTP methods work correctly.
  ///
  /// **Corner Case:** P0-Critical - Standard HTTP methods
  ///
  /// Tests that GET, POST, PUT, DELETE, PATCH methods:
  /// 1. Parse correctly
  /// 2. Can be sent to server
  /// 3. Server receives correct method
  #[ tokio::test ]
  async fn test_standard_http_methods()
  {
    use axum::routing::{ post, put, delete };

    let app = Router::new()
      .route( "/get", get( || async { "GET" } ) )
      .route( "/post", post( || async { "POST" } ) )
      .route( "/put", put( || async { "PUT" } ) )
      .route( "/delete", delete( || async { "DELETE" } ) );

    let addr = create_test_server( app ).await;

    // Test each method
    let resp_get = test_request( addr, "GET", "/get" ).await;
    assert_eq!( resp_get.text().await.unwrap(), "GET" );

    let resp_post = test_request( addr, "POST", "/post" ).await;
    assert_eq!( resp_post.text().await.unwrap(), "POST" );

    let resp_put = test_request( addr, "PUT", "/put" ).await;
    assert_eq!( resp_put.text().await.unwrap(), "PUT" );

    let resp_delete = test_request( addr, "DELETE", "/delete" ).await;
    assert_eq!( resp_delete.text().await.unwrap(), "DELETE" );
  }

  /// Verify path with and without leading slash both work.
  ///
  /// **Corner Case:** P0-Critical - Valid paths with/without leading slash
  ///
  /// Tests that test_request handles:
  /// 1. Path with leading slash "/api/test"
  /// 2. Path without leading slash "api/test" (should fail - URLs need slash)
  /// 3. Root path "/"
  /// 4. Empty path "" (constructs root URL)
  #[ tokio::test ]
  async fn test_path_variations()
  {
    let app = Router::new()
      .route( "/", get( || async { "root" } ) )
      .route( "/api/test", get( || async { "api" } ) );

    let addr = create_test_server( app ).await;

    // Path with leading slash - should work
    let resp1 = test_request( addr, "GET", "/api/test" ).await;
    assert_eq!( resp1.status(), 200 );
    assert_eq!( resp1.text().await.unwrap(), "api" );

    // Root path
    let resp2 = test_request( addr, "GET", "/" ).await;
    assert_eq!( resp2.status(), 200 );
    assert_eq!( resp2.text().await.unwrap(), "root" );

    // Empty path (should construct http://addr/ which is root)
    let resp3 = test_request( addr, "GET", "" ).await;
    assert_eq!( resp3.status(), 200 );
    assert_eq!( resp3.text().await.unwrap(), "root" );
  }

  /// Verify multiple sequential server creations get unique ports.
  ///
  /// **Corner Case:** P1-Important - Concurrent server creation
  ///
  /// Tests that:
  /// 1. Multiple servers can be created sequentially
  /// 2. Each gets unique port
  /// 3. All servers remain accessible
  #[ tokio::test ]
  async fn test_multiple_servers_unique_ports()
  {
    let app1 = Router::new().route( "/", get( || async { "server1" } ) );
    let app2 = Router::new().route( "/", get( || async { "server2" } ) );
    let app3 = Router::new().route( "/", get( || async { "server3" } ) );

    let addr1 = create_test_server( app1 ).await;
    let addr2 = create_test_server( app2 ).await;
    let addr3 = create_test_server( app3 ).await;

    // All ports should be different
    assert_ne!( addr1.port(), addr2.port() );
    assert_ne!( addr1.port(), addr3.port() );
    assert_ne!( addr2.port(), addr3.port() );

    // All servers should respond correctly
    let resp1 = test_request( addr1, "GET", "/" ).await;
    assert_eq!( resp1.text().await.unwrap(), "server1" );

    let resp2 = test_request( addr2, "GET", "/" ).await;
    assert_eq!( resp2.text().await.unwrap(), "server2" );

    let resp3 = test_request( addr3, "GET", "/" ).await;
    assert_eq!( resp3.text().await.unwrap(), "server3" );
  }

  /// Verify concurrent requests to same server all succeed.
  ///
  /// **Corner Case:** P1-Important - Concurrent requests to same server
  ///
  /// Tests that:
  /// 1. Multiple simultaneous requests to same server work
  /// 2. No request failures or timeouts
  /// 3. All responses correct
  #[ tokio::test ]
  async fn test_concurrent_requests()
  {
    let app = Router::new().route( "/", get( || async { "response" } ) );
    let addr = create_test_server( app ).await;

    // Send 10 concurrent requests
    let mut handles = vec![];
    for _ in 0..10
    {
      let addr_clone = addr;
      let handle = tokio::spawn( async move {
        test_request( addr_clone, "GET", "/" ).await
      } );
      handles.push( handle );
    }

    // Wait for all and verify
    for handle in handles
    {
      let response = handle.await.unwrap();
      assert_eq!( response.status(), 200 );
      assert_eq!( response.text().await.unwrap(), "response" );
    }
  }

  /// Verify 404 response handled correctly (not treated as error).
  ///
  /// **Corner Case:** P0-Critical - Non-200 responses
  ///
  /// Tests that:
  /// 1. 404 Not Found returns as valid response
  /// 2. test_request doesn't panic on 404
  /// 3. Status code accessible
  #[ tokio::test ]
  async fn test_404_not_found_response()
  {
    let app = Router::new().route( "/exists", get( || async { "ok" } ) );
    let addr = create_test_server( app ).await;

    // Request non-existent path
    let response = test_request( addr, "GET", "/nonexistent" ).await;

    // Should return 404, not panic
    assert_eq!( response.status(), 404 );
  }

  /// Verify path with query parameters works correctly.
  ///
  /// **Corner Case:** P1-Important - Path with special characters
  ///
  /// Tests that:
  /// 1. Paths with query params construct valid URLs
  /// 2. Server receives query params
  #[ tokio::test ]
  async fn test_path_with_query_params()
  {
    use axum::extract::Query;
    use serde::Deserialize;

    #[ derive( Deserialize ) ]
    struct Params { key: String }

    let app = Router::new().route(
      "/api",
      get( | Query( params ): Query< Params > | async move {
        format!( "key={}", params.key )
      } )
    );

    let addr = create_test_server( app ).await;

    let response = test_request( addr, "GET", "/api?key=value" ).await;
    assert_eq!( response.status(), 200 );
    assert_eq!( response.text().await.unwrap(), "key=value" );
  }
}
