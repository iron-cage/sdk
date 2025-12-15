//! Version endpoint integration tests
//!
//! ## Test Purpose
//!
//! Validates /api/v1/version endpoint exists and returns
//! correct build metadata from build system.
//!
//! ## What Must Pass
//!
//! - GET /api/v1/version returns 200
//! - Response has: current_version, supported_versions,
//!   deprecated_versions, latest_endpoint, build
//! - build.commit is a real git hash (7+ hex chars)
//! - build.timestamp is static (build-time, not runtime)
//! - GET /api/health has NO version field
//!
//! ## Migration Context
//!
//! This test enforces the migration from health.version to
//! dedicated /api/v1/version endpoint. Old pattern (version in
//! health response) must not exist after implementation.

use axum::{ http::StatusCode, body::Body, Router };
use tower::ServiceExt;
use http_body_util::BodyExt;
use serde_json::Value;

/// Test that /api/v1/version endpoint returns valid build metadata
///
/// This test enforces the new pattern: version info comes from
/// dedicated endpoint, not embedded in health response.
#[ tokio::test ]
async fn test_version_endpoint_returns_valid_metadata()
{
  let app = build_test_app().await;

  let request = axum::http::Request::builder()
    .uri( "/api/v1/version" )
    .body( Body::empty() )
    .unwrap();

  let response = app.clone().oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: /api/v1/version should return 200"
  );

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: Value = serde_json::from_slice( &body )
    .expect( "LOUD FAILURE: Version response should be valid JSON" );

  // Verify all required fields present
  assert!(
    json.get( "current_version" ).is_some(),
    "LOUD FAILURE: Missing current_version field"
  );
  assert!(
    json.get( "supported_versions" ).is_some(),
    "LOUD FAILURE: Missing supported_versions field"
  );
  assert!(
    json.get( "deprecated_versions" ).is_some(),
    "LOUD FAILURE: Missing deprecated_versions field"
  );
  assert!(
    json.get( "latest_endpoint" ).is_some(),
    "LOUD FAILURE: Missing latest_endpoint field"
  );
  assert!(
    json.get( "build" ).is_some(),
    "LOUD FAILURE: Missing build metadata"
  );

  // Verify build metadata is real (not hardcoded)
  let build = &json[ "build" ];
  assert!(
    build.get( "commit" ).is_some(),
    "LOUD FAILURE: Missing build.commit"
  );
  assert!(
    build.get( "timestamp" ).is_some(),
    "LOUD FAILURE: Missing build.timestamp"
  );
  assert!(
    build.get( "environment" ).is_some(),
    "LOUD FAILURE: Missing build.environment"
  );

  // Verify commit is a real git hash (7+ hex characters)
  let commit = build[ "commit" ].as_str()
    .expect( "LOUD FAILURE: build.commit should be string" );
  assert!(
    commit.len() >= 7,
    "LOUD FAILURE: build.commit too short ({}), expected ≥7 chars",
    commit.len()
  );
  assert!(
    commit.chars().all( |c| c.is_ascii_hexdigit() ),
    "LOUD FAILURE: build.commit '{}' contains non-hex characters",
    commit
  );
}

/// Test that build.timestamp is static (build-time, not runtime)
///
/// This ensures timestamp comes from build script (env!),
/// not runtime computation. Two requests should return
/// identical timestamp.
#[ tokio::test ]
async fn test_version_timestamp_is_static()
{
  let app = build_test_app().await;

  // First request
  let request1 = axum::http::Request::builder()
    .uri( "/api/v1/version" )
    .body( Body::empty() )
    .unwrap();

  let response1 = app.clone().oneshot( request1 ).await.unwrap();
  let body1 = response1.into_body().collect().await.unwrap().to_bytes();
  let json1: Value = serde_json::from_slice( &body1 )
    .expect( "LOUD FAILURE: First response should be JSON" );
  let timestamp1 = json1[ "build" ][ "timestamp" ].as_str().unwrap();

  // Wait 100ms
  tokio::time::sleep( tokio::time::Duration::from_millis( 100 ) ).await;

  // Second request
  let request2 = axum::http::Request::builder()
    .uri( "/api/v1/version" )
    .body( Body::empty() )
    .unwrap();

  let response2 = app.oneshot( request2 ).await.unwrap();
  let body2 = response2.into_body().collect().await.unwrap().to_bytes();
  let json2: Value = serde_json::from_slice( &body2 )
    .expect( "LOUD FAILURE: Second response should be JSON" );
  let timestamp2 = json2[ "build" ][ "timestamp" ].as_str().unwrap();

  // Timestamps must be identical (build-time, not runtime)
  assert_eq!(
    timestamp1,
    timestamp2,
    "LOUD FAILURE: Timestamp changed between requests (runtime, not build-time)"
  );
}

/// Test that health endpoint has NO version field
///
/// This enforces old pattern removal. Health endpoint should
/// only return status and timestamp, NOT version.
#[ tokio::test ]
async fn test_health_has_no_version_field()
{
  let app = build_test_app().await;

  let request = axum::http::Request::builder()
    .uri( "/api/health" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: /api/health should return 200"
  );

  let body = response.into_body().collect().await.unwrap().to_bytes();
  let json: Value = serde_json::from_slice( &body )
    .expect( "LOUD FAILURE: Health response should be valid JSON" );

  // version field must NOT exist
  assert!(
    json.get( "version" ).is_none(),
    "LOUD FAILURE: health endpoint still has version field (old pattern not removed)"
  );

  // Verify health only has expected fields
  let obj = json.as_object()
    .expect( "LOUD FAILURE: Health response should be object" );
  assert!(
    obj.contains_key( "status" ),
    "LOUD FAILURE: Health missing status field"
  );
  assert!(
    obj.contains_key( "timestamp" ),
    "LOUD FAILURE: Health missing timestamp field"
  );

  // Should have exactly 2 fields (status, timestamp)
  assert_eq!(
    obj.len(),
    2,
    "LOUD FAILURE: Health has {} fields, expected 2 (status, timestamp)",
    obj.len()
  );
}

/// Helper function to build test app
///
/// This builds a minimal router with health and version endpoints
/// for testing purposes.
async fn build_test_app() -> Router
{
  use axum::routing::get;

  Router::new()
    .route( "/api/health", get( iron_control_api::routes::health::health_check ) )
    .route( "/api/v1/version", get( iron_control_api::routes::version::get_version ) )
}
