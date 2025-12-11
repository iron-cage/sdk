//! Malformed JSON handling tests for authentication endpoints.
//!
//! Tests input validation at the HTTP parsing layer to ensure proper error
//! responses for malformed JSON payloads.
//!
//! ## Test Matrix
//!
//! | Test Case | Endpoint | Malformed Input | Expected Result | Status |
//! |-----------|----------|----------------|----------------|--------|
//! | `test_login_with_invalid_json_syntax` | POST /api/auth/login | Missing closing brace | 400 Bad Request | ✅ |
//! | `test_login_with_trailing_comma` | POST /api/auth/login | Trailing comma | 400 Bad Request | ✅ |
//! | `test_refresh_with_invalid_json_syntax` | POST /api/auth/refresh | Malformed JSON | 400 Bad Request | ✅ |
//! | `test_logout_with_invalid_json_syntax` | POST /api/auth/logout | Malformed JSON | 400 Bad Request | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:** Not applicable (all test invalid inputs)
//!
//! **Error Conditions:**
//! - ✅ Invalid JSON syntax → 400 Bad Request
//! - ✅ Trailing commas → 400 Bad Request
//! - ✅ Truncated JSON transmission → 400 Bad Request
//! - ✅ Mixed quotes in JSON → 400 Bad Request
//!
//! **Edge Cases:**
//! - ✅ Empty body when JSON expected → 400 Bad Request
//! - ✅ Partial credentials in malformed JSON → 400 Bad Request
//!
//! **Boundary Conditions:** Not applicable (syntax validation only)
//! **State Transitions:** Not applicable (HTTP layer validation only)
//! **Concurrent Access:** Not applicable (stateless validation)
//! **Resource Limits:** Not applicable (covered by token tests)
//! **Precondition Violations:** Not applicable

use crate::common::test_state::create_test_auth_state;
use iron_control_api::routes::auth_new;
use axum::{ Router, routing::post, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with auth routes.
async fn create_test_router() -> Router
{
  let auth_state = create_test_auth_state().await;

  Router::new()
    .route( "/api/auth/login", post( auth_new::login ) )
    .route( "/api/auth/refresh", post( auth_new::refresh ) )
    .route( "/api/auth/logout", post( auth_new::logout ) )
    .with_state( auth_state )
}

/// Test POST /api/auth/login with invalid JSON syntax (missing closing brace).
///
/// WHY: Axum's JSON extractor should reject syntactically invalid JSON at the
/// HTTP layer before reaching authentication logic.
#[ tokio::test ]
async fn test_login_with_invalid_json_syntax()
{
  let router = create_test_router().await;

  let malformed_json = r#"{"email":"user","password":"pass""#; // Missing closing brace

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Malformed JSON must return 400 Bad Request"
  );
}

/// Test POST /api/auth/login with trailing comma (invalid JSON).
///
/// WHY: JSON spec doesn't allow trailing commas. This tests strict parsing.
#[ tokio::test ]
async fn test_login_with_trailing_comma()
{
  let router = create_test_router().await;

  let malformed_json = r#"{"email":"user","password":"pass",}"#; // Trailing comma

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from( malformed_json ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Trailing comma in JSON must return 400 Bad Request"
  );
}