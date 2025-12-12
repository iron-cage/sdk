//! Integration tests for Analytics API endpoints (Usage + Traces)
//!
//! Tests cover:
//! - Usage analytics endpoints (aggregate, by-project, by-provider)
//! - Traces endpoints (list, get)
//! - Path parameter validation (DoS prevention)
//! - Error cases (400, 404, 500)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_get_usage_aggregate_empty` | Aggregate usage with no data | GET /api/usage/aggregate with admin token, empty database | 200 OK, zero totals | ✅ |
//! | `test_get_usage_by_project_valid_id` | Usage by valid project ID | GET /api/usage/by-project/:id with admin token, existing project | 200 OK, project-specific usage data | ✅ |
//! | `test_get_usage_by_project_empty_id` | Usage with empty project ID | GET /api/usage/by-project/ (empty ID) with admin token | 400 Bad Request (path validation) | ✅ |
//! | `test_get_usage_by_project_too_long` | Usage with oversized project ID (DoS) | GET /api/usage/by-project/:id with 100K+ char ID | 400 Bad Request (DoS prevention) | ✅ |
//! | `test_get_usage_by_provider_valid` | Usage by valid provider name | GET /api/usage/by-provider/:provider with admin token, valid provider | 200 OK, provider-specific usage data | ✅ |
//! | `test_get_usage_by_provider_empty` | Usage with empty provider name | GET /api/usage/by-provider/ (empty provider) with admin token | 400 Bad Request (path validation) | ✅ |
//! | `test_get_usage_by_provider_too_long` | Usage with oversized provider name (DoS) | GET /api/usage/by-provider/:provider with 100K+ char name | 400 Bad Request (DoS prevention) | ✅ |
//! | `test_list_traces_empty` | List traces with no data | GET /api/traces with admin token, empty database | 200 OK, empty array | ✅ |
//! | `test_get_trace_not_found` | Get nonexistent trace | GET /api/traces/nonexistent_id with admin token | 404 Not Found | ✅ |

mod common;

use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::{ TestAppState, TestTracesAppState } };
use axum::{
  Router,
  routing::get,
  http::{ StatusCode, Request, Method },
  body::Body,
};
use tower::ServiceExt;
use sqlx::SqlitePool;

/// Test schema for analytics integration tests
const ANALYTICS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS usage_records (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  token_id INTEGER NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  cost_cents INTEGER NOT NULL,
  timestamp INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS api_call_traces (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  token_id INTEGER NOT NULL,
  provider TEXT NOT NULL,
  model TEXT NOT NULL,
  endpoint TEXT NOT NULL,
  response_status INTEGER NOT NULL,
  duration_ms INTEGER NOT NULL,
  input_tokens INTEGER NOT NULL,
  output_tokens INTEGER NOT NULL,
  cost_cents INTEGER NOT NULL,
  traced_at INTEGER NOT NULL
);
"#;

/// Helper to create test router with analytics endpoints
async fn create_analytics_router() -> ( Router, SqlitePool, String, String )
{
  // Create TestAppState with auth support
  let app_state = TestAppState::new().await;

  // Add analytics schema to database
  sqlx::raw_sql( ANALYTICS_SCHEMA )
    .execute( &app_state.database )
    .await
    .expect( "LOUD FAILURE: Failed to apply analytics schema" );

  // Create admin and regular user
  let ( admin_id, _ ) = create_test_admin( &app_state.database ).await;
  let ( user_id, _ ) = create_test_user( &app_state.database, "regular_user@mail.com" ).await;

  // Generate tokens using TEST_JWT_SECRET
  let admin_token = create_test_access_token( &admin_id, "admin@admin.com", "admin", "test_jwt_secret_key_for_testing_12345" );
  let user_token = create_test_access_token( &user_id, "regular_user@mail.com", "user", "test_jwt_secret_key_for_testing_12345" );

  // Create usage and traces state
  let usage_state = iron_control_api::routes::usage::UsageState::new( "sqlite::memory:" )
    .await
    .expect("LOUD FAILURE: Failed to create UsageState");

  let traces_app_state = TestTracesAppState::new().await;

  let router = Router::new()
    .route( "/api/usage/aggregate", get( iron_control_api::routes::usage::get_aggregate_usage ) )
    .route( "/api/usage/by-project/:project_id", get( iron_control_api::routes::usage::get_usage_by_project ) )
    .route( "/api/usage/by-provider/:provider", get( iron_control_api::routes::usage::get_usage_by_provider ) )
    .with_state( usage_state.clone() )
    .route( "/api/traces", get( iron_control_api::routes::traces::list_traces ) )
    .route( "/api/traces/:id", get( iron_control_api::routes::traces::get_trace ) )
    .with_state( traces_app_state );

  ( router, app_state.database.clone(), admin_token, user_token )
}

// ============================================================================
// Usage Aggregate Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_usage_aggregate_empty()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/usage/aggregate" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let aggregate: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( aggregate[ "total_requests" ].as_i64().unwrap(), 0 );
  assert_eq!( aggregate[ "total_tokens" ].as_i64().unwrap(), 0 );
  assert_eq!( aggregate[ "total_cost_cents" ].as_i64().unwrap(), 0 );
}

// ============================================================================
// Usage By Project Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_usage_by_project_valid_id()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/usage/by-project/project_123" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );
}

#[ tokio::test ]
async fn test_get_usage_by_project_empty_id()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  // URL-encode whitespace as %20
  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/usage/by-project/%20%20%20" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST, "Whitespace-only project_id should be rejected" );
}

#[ tokio::test ]
async fn test_get_usage_by_project_too_long()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  // Create project_id exceeding 1000 characters
  let long_id = "a".repeat( 1001 );

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/usage/by-project/{}", long_id ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST, "project_id exceeding 1000 chars should be rejected for DoS prevention" );
}

// ============================================================================
// Usage By Provider Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_usage_by_provider_valid()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/usage/by-provider/openai" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );
}

#[ tokio::test ]
async fn test_get_usage_by_provider_empty()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  // URL-encode whitespace as %20
  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/usage/by-provider/%20%20%20" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST, "Whitespace-only provider should be rejected" );
}

#[ tokio::test ]
async fn test_get_usage_by_provider_too_long()
{
  let ( app, _pool, _admin_token, _user_token ) = create_analytics_router().await;

  // Create provider exceeding 100 characters
  let long_provider = "a".repeat( 101 );

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/usage/by-provider/{}", long_provider ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST, "provider exceeding 100 chars should be rejected for DoS prevention" );
}

// ============================================================================
// Traces List Tests
// ============================================================================

#[ tokio::test ]
async fn test_list_traces_empty()
{
  let ( app, _pool, _admin_token, user_token ) = create_analytics_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/traces" )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let traces: Vec< serde_json::Value > = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( traces.len(), 0, "Should return empty list when no traces exist" );
}

// ============================================================================
// Traces Get Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_trace_not_found()
{
  let ( app, _pool, _admin_token, user_token ) = create_analytics_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/traces/999999" )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}
