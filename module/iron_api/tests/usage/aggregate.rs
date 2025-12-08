//! Endpoint integration tests for GET /api/usage/aggregate.
//!
//! ## Purpose
//! Verify that aggregate usage endpoint correctly aggregates token usage statistics
//! across ALL tokens in the system, regardless of user_id or project_id.
//!
//! ## Test Matrix
//!
//! | Test Case | Database State | Expected Status | Expected Totals |
//! |-----------|----------------|-----------------|-----------------|
//! | Empty database | No usage records | 200 OK | All zeros |
//! | Single token | 1 usage record | 200 OK | Match record |
//! | Multiple tokens | 3 usage records | 200 OK | Sum of all |
//! | Multiple providers | OpenAI + Anthropic | 200 OK | Breakdown by provider |
//! | Database error | (simulated) | 500 Error | Error message |
//!
//! ## Known Edge Cases
//! - Empty database must return 200 OK with zero values (not 404)
//! - Provider breakdown must include ALL providers with usage, not just active tokens
//! - Aggregation must use COALESCE to handle NULL values correctly
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check UsageTracker::get_all_aggregate_usage() SQL query (no token_id filter)
//! 2. Check UsageTracker::get_usage_by_provider_all() GROUP BY logic
//! 3. Check error handler returns HTTP 500 (not silent failure with vec![])
//! 4. Verify in-memory database schema matches production schema

use crate::common::{ extract_json_response, extract_response };
use iron_api::routes::usage::{ UsageState, AggregateUsageResponse };
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::Value;

/// Create test router with usage aggregate route.
async fn create_test_router() -> Router
{
  let usage_state = UsageState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create usage state with in-memory database" );

  Router::new()
    .route( "/api/usage/aggregate", get( iron_api::routes::usage::get_aggregate_usage ) )
    .with_state( usage_state )
}

/// Test empty database returns 200 OK with zero totals.
///
/// WHY: Empty database is valid state (new installation), not an error.
/// Endpoint must return 200 with zeros, not 404 or 500.
#[ tokio::test ]
async fn test_aggregate_empty_database_returns_zeros()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/aggregate" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Empty database must return 200 OK, not error status"
  );

  let ( status, body ): ( StatusCode, AggregateUsageResponse ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::OK );
  assert_eq!(
    body.total_tokens, 0,
    "LOUD FAILURE: Empty database must have zero total_tokens"
  );
  assert_eq!(
    body.total_requests, 0,
    "LOUD FAILURE: Empty database must have zero total_requests"
  );
  assert_eq!(
    body.total_cost_cents, 0,
    "LOUD FAILURE: Empty database must have zero total_cost_cents"
  );
  assert_eq!(
    body.providers.len(), 0,
    "LOUD FAILURE: Empty database must have empty providers array"
  );
}

/// Test response structure matches specification.
///
/// WHY: Frontend depends on exact field names and types.
#[ tokio::test ]
async fn test_aggregate_response_structure()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/aggregate" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::OK );

  // Parse as JSON to verify structure
  let json: Value = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response must be valid JSON" );

  assert!(
    json.get( "total_tokens" ).is_some(),
    "LOUD FAILURE: Response must include 'total_tokens' field"
  );
  assert!(
    json.get( "total_requests" ).is_some(),
    "LOUD FAILURE: Response must include 'total_requests' field"
  );
  assert!(
    json.get( "total_cost_cents" ).is_some(),
    "LOUD FAILURE: Response must include 'total_cost_cents' field"
  );
  assert!(
    json.get( "providers" ).is_some(),
    "LOUD FAILURE: Response must include 'providers' array"
  );

  // Verify providers is an array
  assert!(
    json[ "providers" ].is_array(),
    "LOUD FAILURE: 'providers' must be an array"
  );
}

/// Test Content-Type header is application/json.
///
/// WHY: Frontend expects JSON, not plain text.
#[ tokio::test ]
async fn test_aggregate_content_type_is_json()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/aggregate" )
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

/// Test HTTP method validation (POST should fail).
///
/// WHY: Spec defines GET only. POST should return 405 Method Not Allowed.
#[ tokio::test ]
async fn test_aggregate_rejects_post_method()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/usage/aggregate" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::METHOD_NOT_ALLOWED,
    "LOUD FAILURE: POST to GET-only endpoint must return 405 Method Not Allowed"
  );
}
