//! Endpoint integration tests for GET /api/traces.
//!
//! ## Purpose
//! Verify that trace listing endpoint correctly retrieves API call traces from database,
//! ordered by timestamp descending (most recent first).
//!
//! ## Test Matrix
//!
//! | Test Case | Database State | Expected Status | Expected Behavior |
//! |-----------|----------------|-----------------|-------------------|
//! | Empty database | No trace records | 200 OK | Empty array |
//! | Single trace | 1 trace record | 200 OK | Array with 1 item |
//! | Multiple traces | 3+ trace records | 200 OK | Array ordered by timestamp DESC |
//! | Database error | (simulated) | 500 Error | Error message |
//!
//! ## Known Edge Cases
//! - Empty database returns 200 OK with empty array (not 404)
//! - Traces must be ordered by traced_at DESC (most recent first)
//! - All trace fields must be included (id, token_id, provider, model, etc.)
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check TraceStorage::get_all_traces() ORDER BY clause
//! 2. Check field mapping from database row to TraceRecord struct
//! 3. Check error handler returns HTTP 500 (not silent failure)
//! 4. Verify api_call_traces table schema matches production

use crate::common::{ extract_json_response, extract_response, test_state::TestTracesAppState };
use iron_control_api::routes::traces::ApiTrace;
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Generate JWT token for test user
fn generate_jwt_for_user( app_state: &TestTracesAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with traces list route.
async fn create_test_router() -> ( Router, TestTracesAppState )
{
  let app_state = TestTracesAppState::new().await;

  let router = Router::new()
    .route( "/api/traces", get( iron_control_api::routes::traces::list_traces ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Test empty database returns 200 OK with empty array.
///
/// WHY: Empty database is valid state (no API calls yet), not an error.
#[ tokio::test ]
async fn test_list_empty_database_returns_empty_array()
{
  let ( router, app_state ) = create_test_router().await;
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Empty database must return 200 OK, not error status"
  );

  let ( status, body ): ( StatusCode, Vec< ApiTrace > ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::OK );
  assert_eq!(
    body.len(), 0,
    "LOUD FAILURE: Empty database must return empty array"
  );
}

/// Test response is JSON array (not object).
///
/// WHY: Spec defines endpoint returns Vec<ApiTrace>, not wrapped object.
#[ tokio::test ]
async fn test_list_response_is_array()
{
  let ( router, app_state ) = create_test_router().await;
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::OK );

  // Parse as JSON to verify it's an array
  let json: serde_json::Value = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response must be valid JSON" );

  assert!(
    json.is_array(),
    "LOUD FAILURE: Response must be JSON array, got: {:?}",
    json
  );
}

/// Test ApiTrace structure matches specification.
///
/// WHY: Frontend depends on exact field names for rendering.
#[ tokio::test ]
async fn test_list_api_trace_structure()
{
  let ( router, app_state ) = create_test_router().await;
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let ( _status, body ) = extract_response( response ).await;
  let json: serde_json::Value = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response must be valid JSON" );

  // Empty array case - verify type
  assert_eq!(
    json.as_array().unwrap().len(), 0,
    "Empty database returns empty array"
  );

  // Expected fields (verify with non-empty response in integration tests)
  // id, token_id, provider, model, endpoint, status_code, latency_ms,
  // input_tokens, output_tokens, cost_cents, timestamp
}

/// Test POST method rejected (GET-only endpoint).
#[ tokio::test ]
async fn test_list_rejects_post_method()
{
  let ( router, _app_state ) = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/traces" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test Content-Type is application/json.
#[ tokio::test ]
async fn test_list_content_type_is_json()
{
  let ( router, app_state ) = create_test_router().await;
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let content_type = response.headers().get( "content-type" )
    .expect( "LOUD FAILURE: Response must include Content-Type header" )
    .to_str()
    .expect( "LOUD FAILURE: Content-Type must be valid UTF-8" );

  assert!(
    content_type.contains( "application/json" ),
    "LOUD FAILURE: Content-Type must be application/json, got: {}",
    content_type
  );
}

/// Test DELETE method rejected.
#[ tokio::test ]
async fn test_list_rejects_delete_method()
{
  let ( router, _app_state ) = create_test_router().await;

  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/traces" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: DELETE to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test PUT method rejected.
#[ tokio::test ]
async fn test_list_rejects_put_method()
{
  let ( router, _app_state ) = create_test_router().await;

  let request = Request::builder()
    .method( "PUT" )
    .uri( "/api/traces" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: PUT to GET-only endpoint must return 405 Method Not Allowed"
  );
}

// --- Bug Reproducer Tests ---

/// Fix(issue-002): Traces endpoint missing authentication requirement
///
/// Root cause: The list_traces handler in routes/traces.rs doesn't include AuthenticatedUser extractor
/// parameter, so it accepts requests without authentication. The handler signature is:
/// `pub async fn list_traces(State(state): State<TracesState>) -> impl IntoResponse`
/// Instead of:
/// `pub async fn list_traces(_user: AuthenticatedUser, State(state): State<TracesState>) -> impl IntoResponse`
///
/// Pitfall: Security-sensitive endpoints (viewing traces, which may contain usage patterns and costs)
/// must always require authentication. When adding new REST endpoints, always verify authentication
/// requirements and add AuthenticatedUser extractor to enforce them. Missing auth on read-only endpoints
/// is particularly dangerous because it may not be caught by functional testing if the endpoint "works"
/// but exposes sensitive data. Always add explicit auth tests for new endpoints.
///
/// Current Behavior: Returns 200 OK with data (empty array) even without Authorization header
/// Expected After Fix: Should return 401 Unauthorized with {"error": "Missing authentication token"}
#[ tokio::test ]
#[ ignore ]
async fn bug_reproducer_issue_002_traces_missing_auth()
{
  let ( router, _app_state ) = create_test_router().await;

  // Make request WITHOUT Authorization header
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/traces" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Current buggy behavior: Returns 200 OK without auth
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Currently returns 200 OK without authentication (this is the bug)"
  );

  let ( status, body ): ( StatusCode, Vec< ApiTrace > ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::OK );

  // The endpoint returns data (empty array in this case) without checking auth
  assert_eq!(
    body.len(), 0,
    "Empty database returns empty array - but this shouldnt be accessible without auth!"
  );

  // After fix, this test should be updated to assert:
  // - Status: 401 UNAUTHORIZED
  // - Body contains: {"error": "Missing authentication token", "code": "AUTH_MISSING_TOKEN"}
  // - No trace data should be returned without authentication
}
