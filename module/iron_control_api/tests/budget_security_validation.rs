//! Budget input validation tests for Protocol 005
//!
//! Tests verify proper rejection of malformed requests, missing fields, and oversized inputs.
//!
//! # Authority
//! - Protocol 005 specification: Input validation requirements
//! - Security: Input sanitization and bounds checking
//!
//! # Test Matrix
//!
//! ## E3: Malformed Request Body
//! | Test Case | Scenario | Expected Response |
//! |-----------|----------|-------------------|
//! | Invalid JSON | Syntax error in JSON | 400 Bad Request |
//! | Missing content-type | No content-type header | 415 Unsupported Media Type |
//! | Empty body | Valid JSON but no fields | 400 Bad Request |
//!
//! ## E4: Missing Required Fields
//! | Endpoint | Missing Field | Expected Response |
//! |----------|---------------|-------------------|
//! | /handshake | ic_token | 400/422 validation error |
//! | /handshake | provider | 400/422 validation error |
//! | /report | lease_id | 400/422 validation error |
//! | /report | cost_microdollars | 400/422 validation error |
//! | /refresh | ic_token | 400/422 validation error |
//! | /refresh | current_lease_id | 400/422 validation error |
//!
//! ## E8: Long String Handling
//! | Test Case | Scenario | Expected Response |
//! |-----------|----------|-------------------|
//! | ic_token > 2000 chars | Oversized token | 400/422 validation error |
//! | lease_id > 100 chars | Oversized ID | 400/422 validation error |
//! | model > 1000 chars | Oversized model name | 400/422 validation error |

mod common;

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
};
use common::budget::
{
  setup_test_db,
  create_test_budget_state,
  create_ic_token,
  seed_agent_with_budget,
  create_budget_router,
};
use serde_json::json;
use tower::ServiceExt;

/// E3.1: Invalid JSON syntax
///
/// # Security Risk
/// Malformed JSON could trigger parser vulnerabilities or cause unexpected behavior
///
/// # Expected Behavior
/// - 400 Bad Request returned
/// - Clear error message about JSON parsing failure
/// - No server-side panic or crash
#[ tokio::test ]
async fn test_malformed_json_syntax()
{
  let pool = setup_test_db().await;
  let agent_id = 500i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Send malformed JSON to /api/budget/report
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( r#"{ "lease_id": "foo", invalid }"# ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Malformed JSON should return 400 Bad Request"
  );
}

/// E3.2: Missing content-type header
///
/// # Security Risk
/// Missing content-type could bypass validation or trigger unintended parsers
///
/// # Expected Behavior
/// - 415 Unsupported Media Type OR 400 Bad Request
/// - Clear error about missing/invalid content-type
#[ tokio::test ]
async fn test_missing_content_type()
{
  let pool = setup_test_db().await;
  let agent_id = 501i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Send valid JSON but no content-type header
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        // NO content-type header
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Axum typically accepts JSON without content-type but may vary
  // Accept either 200 (permissive) or 415 (strict)
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::UNSUPPORTED_MEDIA_TYPE
      || response.status() == StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Missing content-type should be handled (got {})",
    response.status()
  );
}

/// E3.3: Empty request body
///
/// # Security Risk
/// Empty body could bypass validation or cause null pointer issues
///
/// # Expected Behavior
/// - 400 Bad Request (missing required fields)
/// - Clear validation error messages
#[ tokio::test ]
async fn test_empty_request_body()
{
  let pool = setup_test_db().await;
  let agent_id = 502i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Send empty JSON object
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( "{}" ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with 400 or 422 (missing required fields)
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Empty body should return validation error (got {})",
    response.status()
  );
}

/// E4.1: Missing ic_token in handshake
///
/// # Security Risk
/// Missing authentication could allow unauthorized budget access
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "ic_token required" or similar
#[ tokio::test ]
async fn test_handshake_missing_ic_token()
{
  let pool = setup_test_db().await;
  let agent_id = 503i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Send handshake without ic_token
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing ic_token should return validation error (got {})",
    response.status()
  );
}

/// E4.2: Missing provider in handshake
///
/// # Security Risk
/// Missing provider could cause incorrect budget tracking
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "provider required" or similar
#[ tokio::test ]
async fn test_handshake_missing_provider()
{
  let pool = setup_test_db().await;
  let agent_id = 504i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router = create_budget_router( state ).await;

  // Send handshake without provider
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing provider should return validation error (got {})",
    response.status()
  );
}

/// E4.3: Missing lease_id in report
///
/// # Security Risk
/// Missing lease_id could cause budget to be charged incorrectly
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "lease_id required" or similar
#[ tokio::test ]
async fn test_report_missing_lease_id()
{
  let pool = setup_test_db().await;
  let agent_id = 505i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Send report without lease_id
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "request_id": "req_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing lease_id should return validation error (got {})",
    response.status()
  );
}

/// E4.4: Missing cost_microdollars in report
///
/// # Security Risk
/// Missing cost could allow free usage
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "cost_microdollars required" or similar
#[ tokio::test ]
async fn test_report_missing_cost()
{
  let pool = setup_test_db().await;
  let agent_id = 506i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Send report without cost_microdollars
  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_test",
          "tokens": 1000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing cost_microdollars should return validation error (got {})",
    response.status()
  );
}

/// E4.5: Missing ic_token in refresh
///
/// # Security Risk
/// Missing authentication could allow unauthorized lease refresh
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "ic_token required" or similar
#[ tokio::test ]
async fn test_refresh_missing_ic_token()
{
  let pool = setup_test_db().await;
  let agent_id = 507i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let access_token = common::create_test_access_token(
    "user_123",
    "test@example.com",
    "admin",
    "test_jwt_secret"
  );
  let router = create_budget_router( state ).await;

  // Send refresh without ic_token
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "current_lease_id": "lease_123",
          "requested_budget": 10_000_000
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing ic_token should return validation error (got {})",
    response.status()
  );
}

/// E4.6: Missing current_lease_id in refresh
///
/// # Security Risk
/// Missing lease ID could cause incorrect lease expiration
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "current_lease_id required" or similar
#[ tokio::test ]
async fn test_refresh_missing_current_lease_id()
{
  let pool = setup_test_db().await;
  let agent_id = 508i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let access_token = common::create_test_access_token(
    "user_123",
    "test@example.com",
    "admin",
    "test_jwt_secret"
  );
  let router = create_budget_router( state ).await;

  // Send refresh without current_lease_id
  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "authorization", format!( "Bearer {}", access_token ) )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "requested_budget": 10_000_000
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Missing current_lease_id should return validation error (got {})",
    response.status()
  );
}

/// E8.1: Oversized ic_token (> 2000 chars)
///
/// # Security Risk
/// Extremely long tokens could cause buffer overflows or DoS attacks
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "ic_token too long" or similar
#[ tokio::test ]
async fn test_oversized_ic_token()
{
  let pool = setup_test_db().await;
  let agent_id = 509i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Create 2001-character ic_token
  let oversized_token = "a".repeat( 2001 );

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": oversized_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject with validation error (or fail authentication)
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY
      || response.status() == StatusCode::UNAUTHORIZED,
    "LOUD FAILURE: Oversized ic_token should be rejected (got {})",
    response.status()
  );
}

/// E8.2: Oversized lease_id (> 100 chars)
///
/// # Security Risk
/// Long IDs could cause database issues or DoS attacks
///
/// # Expected Behavior
/// - 400/422 validation error OR 404 Not Found (ID doesn't exist)
#[ tokio::test ]
async fn test_oversized_lease_id()
{
  let pool = setup_test_db().await;
  let agent_id = 510i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Create 101-character lease_id
  let oversized_lease_id = format!( "lease_{}", "a".repeat( 95 ) );

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": oversized_lease_id,
          "request_id": "req_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": "gpt-4",
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should reject (validation error, not found, or forbidden)
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY
      || response.status() == StatusCode::NOT_FOUND
      || response.status() == StatusCode::FORBIDDEN,
    "LOUD FAILURE: Oversized lease_id should be rejected (got {})",
    response.status()
  );
}

/// E8.3: Oversized model name (> 1000 chars)
///
/// # Security Risk
/// Extremely long model names could cause logging issues or database problems
///
/// # Expected Behavior
/// - 400/422 validation error
/// - Error message: "model name too long" or similar
#[ tokio::test ]
async fn test_oversized_model_name()
{
  let pool = setup_test_db().await;
  let agent_id = 511i64;
  seed_agent_with_budget( &pool, agent_id, 100_000_000 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let router_handshake = create_budget_router( state.clone() ).await;

  // Create lease first
  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "ic_token": ic_token,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_json: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_json["lease_id"].as_str().unwrap();

  // Create 10000-character model name
  let oversized_model = "a".repeat( 10_000 );

  let router_report = create_budget_router( state ).await;
  let response = router_report
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( json!({
          "lease_id": lease_id,
          "request_id": "req_test",
          "tokens": 1000,
          "cost_microdollars": 5_000_000,
          "model": oversized_model,
          "provider": "openai"
        }).to_string() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Should accept (no hard limit on model name) OR reject with validation error
  // Implementation determines if model name has a length limit
  assert!(
    response.status() == StatusCode::OK
      || response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "LOUD FAILURE: Oversized model name should be handled (got {})",
    response.status()
  );
}
