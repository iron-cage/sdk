//! Path parameter validation tests for usage endpoints
//!
//! ## Purpose
//! Verify that usage endpoints validate path parameters (project_id, provider)
//! to prevent DoS attacks via excessively long strings and reject malformed input.
//!
//! ## Test Matrix
//!
//! | Parameter   | Invalid Values           | Expected Status | Expected Behavior           |
//! |-------------|--------------------------|-----------------|----------------------------|
//! | project_id  | Whitespace-only          | 400 BAD_REQUEST | Descriptive error message  |
//! | project_id  | Exceeds max length (1000)| 400 BAD_REQUEST | DoS prevention             |
//! | provider    | Whitespace-only          | 400 BAD_REQUEST | Descriptive error message  |
//! | provider    | Exceeds max length (100) | 400 BAD_REQUEST | DoS prevention             |
//!
//! ## Rationale
//!
//! **Why Validate Path Parameters?**
//! 1. **DoS Prevention**: Unbounded strings in SQL queries can cause memory exhaustion
//! 2. **Consistency**: Request body fields (CreateTokenRequest.project_id) are validated
//! 3. **Defense in Depth**: Don't rely solely on URL length limits (vary by server: 2KB-8KB)
//! 4. **Explicit Errors**: Return 400 with clear message vs generic 500 database error
//!
//! **Why These Limits?**
//! - project_id max 1000 chars: Typical UUID (36) or readable ID (50), 1000 is generous
//! - provider max 100 chars: Known providers ~10 chars ("anthropic"), 100 is generous
//!
//! ## Known Edge Cases
//! - Empty path param (e.g., `/api/usage/by-project/`) returns 404 from routing (not our handler)
//! - URL encoding handled by Axum automatically (%20 → space)
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check handler validates path param before database query
//! 2. Check validation returns 400 BAD_REQUEST (not 500 INTERNAL_SERVER_ERROR)
//! 3. Check error message is descriptive (mentions field name and constraint)

use crate::common::extract_response;
use iron_api::routes::usage::UsageState;
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with usage routes.
async fn create_test_router() -> Router
{
  let usage_state = UsageState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create usage state with in-memory database" );

  Router::new()
    .route(
      "/api/usage/by-project/:project_id",
      get( iron_api::routes::usage::get_usage_by_project )
    )
    .route(
      "/api/usage/by-provider/:provider",
      get( iron_api::routes::usage::get_usage_by_provider )
    )
    .with_state( usage_state )
}

// ========================================
// GET /api/usage/by-project/:project_id
// ========================================

/// Test whitespace-only project_id is rejected.
///
/// WHY: Whitespace-only IDs are invalid - should return 400, not pass to database.
#[ tokio::test ]
async fn test_by_project_whitespace_only_rejected()
{
  let router = create_test_router().await;

  // URL-encoded spaces: %20
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/%20%20%20" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Whitespace-only project_id must return 400 BAD_REQUEST"
  );

  let ( _, body ) = extract_response( response ).await;
  assert!(
    body.contains( "project_id" ) && ( body.contains( "empty" ) || body.contains( "whitespace" ) ),
    "LOUD FAILURE: Error message must mention project_id and describe issue. Got: {}",
    body
  );
}

/// Test excessively long project_id is rejected (DoS prevention).
///
/// WHY: Long strings in SQL queries can cause memory exhaustion and slow queries.
/// Limit: 1000 chars (generous, typical UUIDs are 36 chars).
#[ tokio::test ]
async fn test_by_project_too_long_rejected()
{
  let router = create_test_router().await;

  // Create 1001-character project_id (exceeds limit)
  let long_id = "a".repeat( 1001 );
  let uri = format!( "/api/usage/by-project/{}", long_id );

  let request = Request::builder()
    .method( "GET" )
    .uri( &uri )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: project_id exceeding 1000 chars must return 400 BAD_REQUEST for DoS prevention"
  );

  let ( _, body ) = extract_response( response ).await;
  assert!(
    body.contains( "project_id" ) && ( body.contains( "too long" ) || body.contains( "1000" ) ),
    "LOUD FAILURE: Error message must mention project_id length limit. Got: {}",
    body
  );
}

/// Test valid project_id is accepted (baseline).
///
/// WHY: Ensure validation doesn't break valid requests.
#[ tokio::test ]
async fn test_by_project_valid_id_accepted()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/proj-12345" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should return 200 OK (with zero usage for non-existent project, but validation passed)
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Valid project_id must pass validation and return 200 OK"
  );
}

// ========================================
// GET /api/usage/by-provider/:provider
// ========================================

/// Test whitespace-only provider is rejected.
///
/// WHY: Whitespace-only provider names are invalid.
#[ tokio::test ]
async fn test_by_provider_whitespace_only_rejected()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/%20" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Whitespace-only provider must return 400 BAD_REQUEST"
  );

  let ( _, body ) = extract_response( response ).await;
  assert!(
    body.contains( "provider" ) && ( body.contains( "empty" ) || body.contains( "whitespace" ) ),
    "LOUD FAILURE: Error message must mention provider and describe issue. Got: {}",
    body
  );
}

/// Test excessively long provider name is rejected (DoS prevention).
///
/// WHY: Prevent memory exhaustion from malicious long strings.
/// Limit: 100 chars (known providers are ~10 chars, 100 is generous).
#[ tokio::test ]
async fn test_by_provider_too_long_rejected()
{
  let router = create_test_router().await;

  let long_provider = "a".repeat( 101 );
  let uri = format!( "/api/usage/by-provider/{}", long_provider );

  let request = Request::builder()
    .method( "GET" )
    .uri( &uri )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: provider exceeding 100 chars must return 400 BAD_REQUEST for DoS prevention"
  );

  let ( _, body ) = extract_response( response ).await;
  assert!(
    body.contains( "provider" ) && ( body.contains( "too long" ) || body.contains( "100" ) ),
    "LOUD FAILURE: Error message must mention provider length limit. Got: {}",
    body
  );
}

/// Test valid provider is accepted (baseline).
///
/// WHY: Ensure validation doesn't break valid requests.
#[ tokio::test ]
async fn test_by_provider_valid_name_accepted()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/openai" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Valid provider must pass validation and return 200 OK"
  );
}
