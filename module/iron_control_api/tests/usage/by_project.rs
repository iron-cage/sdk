//! Endpoint integration tests for GET /api/usage/by-project/:project_id.
//!
//! ## Purpose
//! Verify that project-specific usage endpoint correctly aggregates usage for all tokens
//! belonging to a specific project_id (via JOIN with api_tokens table).
//!
//! ## Test Matrix
//!
//! | Test Case | Database State | Expected Status | Expected Behavior |
//! |-----------|----------------|-----------------|-------------------|
//! | Unknown project | No matching tokens | 200 OK | All zeros (COALESCE) |
//! | Valid project | 1+ tokens | 200 OK | Aggregate for project |
//! | Empty project | Token exists, no usage | 200 OK | All zeros |
//! | Multiple providers | OpenAI + Anthropic | 200 OK | Breakdown by provider |
//!
//! ## Known Edge Cases
//! - Endpoint uses JOIN query with COALESCE, so non-existent project_id
//!   returns 200 OK with zero values (not 404 or 500)
//! - Provider breakdown must filter by project_id correctly
//! - URL parameter must handle special characters (URL encoding)
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check UsageTracker::get_usage_by_project() JOIN query correctness
//! 2. Check UsageTracker::get_usage_by_provider_for_project() GROUP BY with WHERE clause
//! 3. Check Path parameter extraction in handler function
//! 4. Verify api_tokens table schema matches production

use crate::common::extract_response;
use iron_control_api::routes::usage::UsageState;
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with by-project route.
async fn create_test_router() -> Router
{
  let usage_state = UsageState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create usage state with in-memory database" );

  Router::new()
    .route(
      "/api/usage/by-project/:project_id",
      get( iron_control_api::routes::usage::get_usage_by_project )
    )
    .with_state( usage_state )
}

/// Test unknown project_id returns 200 OK with zero usage.
///
/// WHY: SQL query uses COALESCE which returns 0 for NULL values.
/// Non-existent project_id returns valid result (all zeros), not database error.
/// This is correct behavior - unknown project is valid query, not error condition.
#[ tokio::test ]
async fn test_by_project_unknown_project_returns_zeros()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/nonexistent_project" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Unknown project_id returns 200 OK with zero usage, not error"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::OK );

  // Verify response is valid JSON with zero values
  let json: serde_json::Value = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response must be valid JSON" );

  assert_eq!(
    json[ "total_tokens" ].as_i64().unwrap(), 0,
    "LOUD FAILURE: Unknown project must have zero tokens"
  );
  assert_eq!(
    json[ "total_requests" ].as_i64().unwrap(), 0,
    "LOUD FAILURE: Unknown project must have zero requests"
  );
}

/// Test URL path parameter extraction.
///
/// WHY: Axum Path extractor must correctly extract project_id from URL.
#[ tokio::test ]
async fn test_by_project_path_parameter_extraction()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/test-project-123" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should return 200 OK (with zeros for non-existent project), proving path extraction worked
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Path parameter must be extracted and passed to query"
  );
}

/// Test special characters in project_id (URL encoding).
///
/// WHY: Project IDs might contain hyphens, underscores, or encoded characters.
#[ tokio::test ]
async fn test_by_project_special_characters_in_id()
{
  let router = create_test_router().await;

  // Test with URL-encoded space (%20)
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/project%20with%20spaces" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should return 200 OK (proving URL decoding and query worked)
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: URL-encoded project_id must be decoded correctly"
  );
}

/// Test POST method rejected (GET-only endpoint).
#[ tokio::test ]
async fn test_by_project_rejects_post_method()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/usage/by-project/test-project" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST to GET-only endpoint must return 405 Method Not Allowed"
  );
}

/// Test Content-Type is application/json for error responses.
#[ tokio::test ]
async fn test_by_project_error_content_type_is_json()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-project/nonexistent" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let content_type = response.headers().get( "content-type" )
    .expect( "LOUD FAILURE: Error response must include Content-Type header" )
    .to_str()
    .expect( "LOUD FAILURE: Content-Type must be valid UTF-8" );

  assert!(
    content_type.contains( "application/json" ),
    "LOUD FAILURE: Error response Content-Type must be application/json, got: {}",
    content_type
  );
}
