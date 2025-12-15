//! HTTP method validation tests for authentication endpoints.
//!
//! Tests that auth endpoints properly reject unsupported HTTP methods with
//! 405 Method Not Allowed.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Wrong Method | Expected Result | Status |
//! |-----------|----------|-------------|----------------|--------|
//! | `test_login_get_method_rejected` | POST /api/auth/login | GET | 405 Method Not Allowed | ✅ |
//! | `test_refresh_get_method_rejected` | POST /api/auth/refresh | GET | 405 Method Not Allowed | ✅ |
//! | `test_logout_get_method_rejected` | POST /api/auth/logout | GET | 405 Method Not Allowed | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:** Not applicable (all test error conditions)
//!
//! **Error Conditions:**
//! - ✅ Wrong HTTP method → 405 Method Not Allowed
//! - ✅ All auth endpoints require POST
//!
//! **Edge Cases:** Explicit API contract documentation
//! **Boundary Conditions:** Not applicable
//! **State Transitions:** Not applicable
//! **Concurrent Access:** Not applicable
//! **Resource Limits:** Not applicable
//! **Precondition Violations:** Not applicable

use crate::common::test_state::create_test_auth_state;
use iron_control_api::routes::auth;
use axum::{ Router, routing::post, http::{ Request, StatusCode }, extract::ConnectInfo };
use axum::body::Body;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use tower::ServiceExt;

/// Create test router with auth routes.
async fn create_test_router() -> Router
{
  let auth_state = create_test_auth_state().await;
  let test_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);

  Router::new()
    .route( "/api/auth/login", post( auth::login ) )
    .route( "/api/auth/refresh", post( auth::refresh ) )
    .route( "/api/auth/logout", post( auth::logout ) )
    .layer(axum::Extension(ConnectInfo(test_addr)))
    .with_state( auth_state )
}

/// Test POST /api/auth/login with GET method → 405 Method Not Allowed.
///
/// WHY: Login is a mutating operation (creates session), GET should be rejected.
#[ tokio::test ]
async fn test_login_get_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/auth/login" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: GET method on /api/auth/login must return 405 Method Not Allowed"
  );
}

/// Test POST /api/auth/refresh with GET method → 405 Method Not Allowed.
///
/// WHY: Token refresh is a mutating operation, GET should be rejected.
#[ tokio::test ]
async fn test_refresh_get_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/auth/refresh" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: GET method on /api/auth/refresh must return 405 Method Not Allowed"
  );
}

/// Test POST /api/auth/logout with GET method → 405 Method Not Allowed.
///
/// WHY: Logout is a mutating operation (invalidates session), GET should be rejected.
#[ tokio::test ]
async fn test_logout_get_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/auth/logout" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: GET method on /api/auth/logout must return 405 Method Not Allowed"
  );
}

/// Test POST /api/auth/login with PUT method → 405 Method Not Allowed.
///
/// WHY: Login uses POST, not PUT (not an update operation).
#[ tokio::test ]
async fn test_login_put_method_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from( r#"{"username":"user","password":"pass"}"# ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: PUT method on /api/auth/login must return 405 Method Not Allowed"
  );
}
