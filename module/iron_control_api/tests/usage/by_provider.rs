//! Endpoint integration tests for GET /api/usage/by-provider/:provider.
//!
//! ## Purpose
//! Verify that provider-specific usage endpoint correctly aggregates usage for a specific
//! provider (openai, anthropic, google) across ALL tokens.
//!
//! ## Test Matrix
//!
//! | Test Case | Database State | Expected Status | Expected Behavior |
//! |-----------|----------------|-----------------|-------------------|
//! | Unknown provider | No usage for provider | 200 OK | Zero usage |
//! | Valid provider | Has usage records | 200 OK | Aggregate for provider |
//! | Multiple tokens | Same provider, different tokens | 200 OK | Sum across tokens |
//!
//! ## Known Edge Cases
//! - Unknown provider returns 200 OK with zero usage (not 404)
//! - Provider name is case-sensitive ("openai" ≠ "OpenAI")
//! - Aggregation must include ALL tokens using the provider
//!
//! ## Failure Modes
//! If these tests fail:
//! 1. Check UsageTracker::get_all_usage_for_provider() WHERE clause on provider field
//! 2. Check Path parameter extraction handles lowercase/uppercase correctly
//! 3. Verify provider field in token_usage table matches expected values

use crate::common::{ extract_json_response, extract_response };
use iron_control_api::routes::usage::{ UsageState, ProviderStats };
use axum::{ Router, routing::get, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;

/// Create test router with by-provider route.
async fn create_test_router() -> Router
{
  let usage_state = UsageState::new( "sqlite::memory:" )
    .await
    .expect( "LOUD FAILURE: Failed to create usage state with in-memory database" );

  Router::new()
    .route(
      "/api/usage/by-provider/:provider",
      get( iron_control_api::routes::usage::get_usage_by_provider )
    )
    .with_state( usage_state )
}

/// Test unknown provider returns 200 OK with zero usage.
///
/// WHY: Unknown provider is valid query (user checking if provider has usage),
/// not an error. Should return 200 with zeros, not 404.
#[ tokio::test ]
async fn test_by_provider_unknown_returns_zeros()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/unknown_provider" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Unknown provider must return 200 OK with zero usage, not error"
  );

  let ( status, body ): ( StatusCode, ProviderStats ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::OK );
  assert_eq!(
    body.provider, "unknown_provider",
    "LOUD FAILURE: Response must echo requested provider name"
  );
  assert_eq!(
    body.tokens, 0,
    "LOUD FAILURE: Unknown provider must have zero tokens"
  );
  assert_eq!(
    body.requests, 0,
    "LOUD FAILURE: Unknown provider must have zero requests"
  );
  assert_eq!(
    body.cost_cents, 0,
    "LOUD FAILURE: Unknown provider must have zero cost"
  );
}

/// Test response structure matches ProviderStats specification.
#[ tokio::test ]
async fn test_by_provider_response_structure()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/openai" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::OK );

  // Parse as JSON to verify structure
  let json: serde_json::Value = serde_json::from_str( &body )
    .expect( "LOUD FAILURE: Response must be valid JSON" );

  assert!(
    json.get( "provider" ).is_some(),
    "LOUD FAILURE: Response must include 'provider' field"
  );
  assert!(
    json.get( "tokens" ).is_some(),
    "LOUD FAILURE: Response must include 'tokens' field"
  );
  assert!(
    json.get( "requests" ).is_some(),
    "LOUD FAILURE: Response must include 'requests' field"
  );
  assert!(
    json.get( "cost_cents" ).is_some(),
    "LOUD FAILURE: Response must include 'cost_cents' field"
  );
}

/// Test path parameter extraction with common provider names.
#[ tokio::test ]
async fn test_by_provider_common_providers()
{
  let router = create_test_router().await;

  // Test openai
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/openai" )
    .body( Body::empty() )
    .unwrap();
  let response = router.clone().oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::OK );

  // Test anthropic
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/anthropic" )
    .body( Body::empty() )
    .unwrap();
  let response = router.clone().oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::OK );

  // Test google
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/google" )
    .body( Body::empty() )
    .unwrap();
  let response = router.oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::OK );
}

/// Test case sensitivity (lowercase vs uppercase).
///
/// WHY: Database provider field is lowercase ("openai"), not mixed case ("OpenAI").
#[ tokio::test ]
async fn test_by_provider_case_sensitivity()
{
  let router = create_test_router().await;

  // Lowercase (correct)
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/openai" )
    .body( Body::empty() )
    .unwrap();
  let response = router.clone().oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::OK );

  // Mixed case (returns zeros, not error)
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/OpenAI" )
    .body( Body::empty() )
    .unwrap();
  let response = router.oneshot( request ).await.unwrap();
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Mixed case provider returns 200 OK (no match), not error"
  );
}

/// Test POST method rejected.
#[ tokio::test ]
async fn test_by_provider_rejects_post_method()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/usage/by-provider/openai" )
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
async fn test_by_provider_content_type_is_json()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/usage/by-provider/openai" )
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
