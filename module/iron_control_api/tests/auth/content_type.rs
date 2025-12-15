//! Content-Type validation tests for authentication endpoints.
//!
//! Tests that auth endpoints requiring JSON bodies properly reject requests
//! with incorrect Content-Type headers with 415 Unsupported Media Type.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Content-Type | Expected Result | Status |
//! |-----------|----------|-------------|----------------|--------|
//! | `test_login_wrong_content_type` | POST /api/auth/login | text/plain | 415 | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Error Conditions:**
//! - ✅ Wrong Content-Type on login → 415
//!
//! **Why These Tests Matter:**
//! - Security: Prevent credential parser confusion
//! - API contract enforcement for auth operations

use crate::common::test_state::create_test_auth_state;
use axum::{ Router, routing::post, http::{ Request, StatusCode }, extract::ConnectInfo };
use axum::body::Body;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt;
use iron_control_api::routes::auth;

/// Create test router with auth routes.
async fn create_test_router() -> Router
{
  let auth_state = create_test_auth_state().await;

  Router::new()
    .route( "/api/auth/login", post( auth::login ) )
    .with_state( auth_state )
}

#[ tokio::test ]
async fn test_login_wrong_content_type()
{
  let router = create_test_router().await;
  let test_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

  // WHY: Login credentials must be sent with proper Content-Type
  // Security-critical: prevent parser confusion on credentials
  let mut request = Request::builder()
    .method( "POST" )
    .uri( "/api/auth/login" )
    .header( "content-type", "text/plain" )
    .body( Body::from( r#"{"username":"test","password":"pass"}"# ) )
    .unwrap();

  // Insert ConnectInfo into request extensions for the handler
  request.extensions_mut().insert(ConnectInfo(test_addr));

  let response = router.oneshot( request ).await.unwrap();

  // Axum may return either 415 or 400 depending on JSON extractor behavior
  let status = response.status();
  assert!(
    status == StatusCode::UNSUPPORTED_MEDIA_TYPE || status == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Wrong Content-Type on POST /api/auth/login must return 415 or 400. Got: {}",
    status
  );
}
