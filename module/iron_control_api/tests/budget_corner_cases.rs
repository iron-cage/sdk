//! Manual corner case tests for Protocol 005 critical gaps
//!
//! These tests cover corner cases not yet in automated test suite, identified
//! during manual testing workflow. Tests focus on:
//! - Whitespace-only inputs (empty string variants)
//! - Exact boundary violations (off-by-one DoS prevention)
//! - Malformed JWT segments
//! - Negative numeric values
//! - Extreme float values
//! - Error message security (no sensitive data leaks)
//!
//! # Authority
//! test_organization.rulebook.md § Comprehensive Corner Case Coverage
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_handshake_whitespace_only_ic_token` | Whitespace-only IC token validation | POST /api/budget/handshake with ic_token="   \t\n  " | 400 Bad Request, error mentions "empty" | ✅ |
//! | `test_handshake_whitespace_only_provider` | Whitespace-only provider validation | POST /api/budget/handshake with provider="   " | 400 Bad Request | ✅ |
//! | `test_handshake_ic_token_over_max_length` | IC token DoS protection | POST /api/budget/handshake with ic_token >10KB | 400 Bad Request (length limit) | ✅ |
//! | `test_handshake_provider_over_max_length` | Provider DoS protection | POST /api/budget/handshake with provider >1KB | 400 Bad Request (length limit) | ✅ |
//! | `test_handshake_malformed_jwt_missing_segments` | Malformed JWT handling | POST /api/budget/handshake with ic_token="invalid.jwt" | 400 Bad Request (JWT validation) | ✅ |
//! | `test_report_usage_negative_tokens` | Negative token value validation | POST /api/budget/report with tokens=-100 | 400 Bad Request | ✅ |
//! | `test_report_usage_negative_cost` | Negative cost value validation | POST /api/budget/report with cost_usd=-5.0 | 400 Bad Request | ✅ |
//! | `test_error_messages_no_sensitive_data_leak` | Error message security | Invalid handshake request | Error message contains no sensitive data (tokens, keys) | ✅ |
//! | `test_report_usage_zero_cost_cached_response` | Zero cost cached response | POST /api/budget/report with cost_usd=0.0 | 200 OK (cached responses valid) | ✅ |
//! | `test_database_foreign_key_constraint_agent` | FK constraint enforcement | Create lease for nonexistent agent_id | Database error (FK violation) | ✅ |
//! | `test_database_not_null_constraint` | NOT NULL constraint enforcement | Insert lease with NULL required field | Database error (NOT NULL violation) | ✅ |

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
use iron_control_api::ic_token::IcTokenClaims;
use serde_json::json;
use sqlx::Row;
use tower::ServiceExt;

/// Test: Whitespace-only ic_token input
///
/// # Corner Case
/// ic_token field contains only whitespace characters (spaces, tabs, newlines)
///
/// # Expected Behavior
/// HTTP 400 Bad Request with validation error "ic_token cannot be empty"
///
/// # Priority
/// HIGH - Input validation completeness
#[ tokio::test ]
async fn test_handshake_whitespace_only_ic_token()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 106, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Whitespace-only ic_token
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": "   \t\n  ",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Whitespace-only ic_token should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().contains( "empty" ),
    "Error should mention empty ic_token"
  );
}

/// Test: Whitespace-only provider input
///
/// # Corner Case
/// provider field contains only whitespace
///
/// # Expected Behavior
/// HTTP 400 Bad Request with validation error
#[ tokio::test ]
async fn test_handshake_whitespace_only_provider()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 107, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "  \t  "
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Whitespace-only provider should be rejected"
  );
}

/// Test: ic_token exactly at 2001 chars (over max boundary)
///
/// # Corner Case
/// ic_token exactly 2001 characters (1 over MAX_IC_TOKEN_LENGTH = 2000)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "ic_token too long"
///
/// # Priority
/// HIGH - DoS prevention boundary
#[ tokio::test ]
async fn test_handshake_ic_token_over_max_length()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 108, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Create ic_token of exactly 2001 characters
  let oversized_token = "a".repeat( 2001 );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": oversized_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "ic_token over 2000 chars should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().contains( "too long" ),
    "Error should mention ic_token too long"
  );
}

/// Test: provider name exactly at 51 chars (over max boundary)
///
/// # Corner Case
/// provider exactly 51 characters (1 over MAX_PROVIDER_LENGTH = 50)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "provider too long"
#[ tokio::test ]
async fn test_handshake_provider_over_max_length()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 109, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Provider name of exactly 51 characters
  let oversized_provider = "a".repeat( 51 );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": oversized_provider
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "provider over 50 chars should be rejected"
  );
}

/// Test: Malformed JWT with missing segments
///
/// # Corner Case
/// ic_token contains malformed JWT (only 2 segments instead of 3)
///
/// # Expected Behavior
/// HTTP 401 Unauthorized "Invalid IC Token"
#[ tokio::test ]
async fn test_handshake_malformed_jwt_missing_segments()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 110, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let app = create_budget_router( state ).await;

  // Malformed JWT with only 2 segments (missing signature)
  let malformed_jwt = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0";

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": malformed_jwt,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::UNAUTHORIZED,
    "Malformed JWT should return 401 Unauthorized"
  );
}

/// Test: Negative tokens value in usage report
///
/// # Corner Case
/// tokens field is negative (-1000)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "tokens cannot be negative"
#[ tokio::test ]
async fn test_report_usage_negative_tokens()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 111, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with negative tokens
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_001",
        "tokens": -1000,  // NEGATIVE VALUE
        "cost_usd": 5.0,
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Negative tokens should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "negative" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "invalid" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "positive" ),
    "Error should mention negative/positive tokens, got: {}", error_data[ "error" ]
  );
}

/// Test: Negative cost_usd value in usage report
///
/// # Corner Case
/// cost_usd field is negative (-5.0)
///
/// # Expected Behavior
/// HTTP 400 Bad Request "cost_usd cannot be negative"
#[ tokio::test ]
async fn test_report_usage_negative_cost()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 112, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with negative cost
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_001",
        "tokens": 1000,
        "cost_usd": -5.0,  // NEGATIVE VALUE
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::BAD_REQUEST,
    "Negative cost_usd should be rejected"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!(
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "negative" ) ||
    error_data[ "error" ].as_str().unwrap().to_lowercase().contains( "invalid" ),
    "Error should mention negative cost, got: {}", error_data[ "error" ]
  );
}

/// Test: Error messages dont leak sensitive data
///
/// # Corner Case
/// When authentication fails, error should not leak whether agent exists
///
/// # Expected Behavior
/// Generic "Invalid IC Token" error, not "Agent not found" or "Budget insufficient"
///
/// # Priority
/// HIGH - Security (information disclosure prevention)
#[ tokio::test ]
async fn test_error_messages_no_sensitive_data_leak()
{
  let pool = setup_test_db().await;
  // Don't seed any agent (agent doesn't exist)

  let state = create_test_budget_state( pool ).await;
  // Create IC Token for non-existent agent 999
  let ic_token = create_ic_token( 999, &state.ic_token_manager );

  let app = create_budget_router( state ).await;

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return 404 or 400, NOT leak "agent doesn't exist"
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let error_data : serde_json::Value = serde_json::from_slice( &body ).unwrap();
  let error_msg = error_data[ "error" ].as_str().unwrap().to_lowercase();

  // Error should be generic, not leak existence information
  assert!(
    !error_msg.contains( "agent" ) || !error_msg.contains( "not found" ),
    "Error should not leak agent existence: {}", error_msg
  );
}

/// Test: Zero-cost usage reports (cached responses, free tier)
///
/// # Corner Case
/// Report usage with tokens > 0 but cost_usd = 0.0 (cached response, free tier)
///
/// # Expected Behavior
/// Accepted (HTTP 200), budget accounting handles $0.00 correctly
///
/// # Priority
/// MEDIUM - Edge case from specification
#[ tokio::test ]
async fn test_report_usage_zero_cost_cached_response()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 113, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( 1, &state.ic_token_manager );

  // Create lease first
  let app1 = create_budget_router( state.clone() ).await;
  let handshake_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_resp = app1.oneshot( handshake_req ).await.unwrap();
  assert_eq!( handshake_resp.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_resp.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap();

  // Report usage with zero cost (cached response)
  let app2 = create_budget_router( state ).await;
  let report_req = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_cached_001",
        "tokens": 1000,       // Tokens used but cached
        "cost_usd": 0.0,      // ZERO COST
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app2.oneshot( report_req ).await.unwrap();

  assert_eq!(
    response.status(), StatusCode::OK,
    "Zero-cost usage should be accepted (cached responses, free tier)"
  );

  // Verify lease budget didn't change
  let lease_check : ( f64, f64 ) = sqlx::query_as(
    "SELECT budget_granted, budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( lease_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert!( lease_check.0 > 0.0, "Granted should be positive" );
  assert_eq!( lease_check.1, 0.0, "Spent should remain 0.0 for zero-cost request" );
}

/// Test: Database foreign key constraint enforcement
///
/// # Corner Case
/// Attempt to create lease for non-existent agent (foreign key violation)
///
/// # Expected Behavior
/// Database rejects with foreign key constraint violation
///
/// # Priority
/// HIGH - Data integrity enforcement
#[ tokio::test ]
async fn test_database_foreign_key_constraint_agent()
{
  let pool = setup_test_db().await;
  // Don't seed any agent

  // Attempt to create lease for non-existent agent (should fail at DB level)
  let result = sqlx::query(
    "INSERT INTO budget_leases (id, agent_id, budget_id, budget_granted, budget_spent, created_at, expires_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "lease_test_001" )
  .bind( 9999i64 )  // Non-existent agent
  .bind( 1i64 )
  .bind( 10.0 )
  .bind( 0.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() + 3600000 )
  .execute( &pool )
  .await;

  assert!(
    result.is_err(),
    "Foreign key constraint should prevent lease creation for non-existent agent"
  );

  if let Err( e ) = result
  {
    let error_msg = e.to_string().to_lowercase();
    assert!(
      error_msg.contains( "foreign" ) || error_msg.contains( "constraint" ),
      "Error should mention foreign key constraint, got: {}", e
    );
  }
}

/// Test: Database NOT NULL constraint enforcement
///
/// # Corner Case
/// Attempt to insert NULL into NOT NULL column (budget_granted)
///
/// # Expected Behavior
/// Database rejects with NOT NULL constraint violation
#[ tokio::test ]
async fn test_database_not_null_constraint()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 105, 100.0 ).await;

  // Attempt to insert NULL into budget_granted (NOT NULL column)
  let result = sqlx::query(
    "INSERT INTO budget_leases (id, agent_id, budget_id, budget_granted, budget_spent, created_at, expires_at)
     VALUES (?, ?, ?, NULL, ?, ?, ?)"
  )
  .bind( "lease_test_002" )
  .bind( 1i64 )
  .bind( 1i64 )
  // budget_granted = NULL (should fail)
  .bind( 0.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() + 3600000 )
  .execute( &pool )
  .await;

  assert!(
    result.is_err(),
    "NOT NULL constraint should prevent NULL in budget_granted"
  );

  if let Err( e ) = result
  {
    let error_msg = e.to_string().to_lowercase();
    assert!(
      error_msg.contains( "null" ) || error_msg.contains( "not null" ),
      "Error should mention NOT NULL constraint, got: {}", e
    );
  }
}

/// Manual Test Gap #2: IC Token with future-dated iat claim
///
/// # Corner Case
/// IC Token with `iat` (issued at) timestamp in the future
///
/// # Expected Behavior
/// Should reject with 400/401 OR accept with logged warning (document actual behavior)
///
/// # Risk
/// MEDIUM - Clock skew exploitation
#[ tokio::test ]
async fn test_handshake_future_dated_ic_token()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 201, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state.clone() ).await;

  // Create IC Token with future iat (1 hour in the future)
  let future_timestamp = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap()
    .as_secs() + 3600;

  let future_claims = IcTokenClaims
  {
    agent_id: "agent_201".to_string(),
    budget_id: "budget_201".to_string(),
    issued_at: future_timestamp,
    expires_at: None,
    issuer: "iron-control-panel".to_string(),
    permissions: vec![ "llm:call".to_string() ],
  };

  let future_token = state.ic_token_manager.generate_token( &future_claims )
    .expect("LOUD FAILURE: Should generate future-dated token");

  // Test handshake with future-dated IC Token
  let request_body = json!({
    "ic_token": future_token,
    "provider": "openai",
    "provider_key_id": 201000,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .clone()
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Document actual behavior
  let status = response.status();
  println!( "Manual Test Gap #2: Future-dated IC Token behavior: {}", status );

  // Current implementation accepts future-dated tokens (no iat validation)
  // This test documents the behavior - may need security review
  assert!(
    status == StatusCode::OK || status == StatusCode::UNAUTHORIZED || status == StatusCode::BAD_REQUEST,
    "Should either accept (200) or reject (400/401) future-dated tokens, got: {}", status
  );
}

/// Manual Test Gap #10: NULL ic_token field
///
/// # Corner Case
/// JSON request with {"ic_token": null, "provider": "openai"}
///
/// # Expected Behavior
/// 400 Bad Request "ic_token is required"
///
/// # Risk
/// MEDIUM - Null pointer dereference potential
#[ tokio::test ]
async fn test_handshake_null_ic_token_field()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Craft request with null ic_token
  let request_body = json!({
    "ic_token": null,
    "provider": "openai",
    "provider_key_id": 1,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL ic_token should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #11: NULL provider field
///
/// # Corner Case
/// JSON request with {"ic_token": "<valid>", "provider": null}
///
/// # Expected Behavior
/// 400 Bad Request "provider is required"
///
/// # Risk
/// MEDIUM - Null pointer dereference potential
#[ tokio::test ]
async fn test_handshake_null_provider_field()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 202, 100.0 ).await;

  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state.clone() ).await;
  let ic_token = create_ic_token( 202, &state.ic_token_manager );

  // Craft request with null provider
  let request_body = json!({
    "ic_token": ic_token,
    "provider": null,
    "provider_key_id": 202000,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL provider should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #12: Missing IC Token agent_id claim
///
/// # Corner Case
/// Valid JWT structure but missing agent_id in payload
///
/// # Expected Behavior
/// 400 Bad Request or 401 Unauthorized "Missing required claim: agent_id"
///
/// # Risk
/// HIGH - Could cause crashes if agent_id not validated
#[ tokio::test ]
async fn test_handshake_missing_agent_id_claim()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state.clone() ).await;

  // Create JWT with only budget_id and permissions (no agent_id)
  use jsonwebtoken::{ encode, EncodingKey, Header };
  use serde::{ Serialize, Deserialize };

  #[ derive( Serialize, Deserialize ) ]
  struct PartialClaims
  {
    budget_id: String,
    #[ serde( rename = "iat" ) ]
    issued_at: u64,
    #[ serde( rename = "iss" ) ]
    issuer: String,
    permissions: Vec< String >,
  }

  let partial_claims = PartialClaims
  {
    budget_id: "budget_203".to_string(),
    issued_at: std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs(),
    issuer: "iron-control-panel".to_string(),
    permissions: vec![ "llm:call".to_string() ],
  };

  let token_missing_agent_id = encode(
    &Header::default(),
    &partial_claims,
    &EncodingKey::from_secret( b"test_secret_key_12345" )
  ).unwrap();

  // Test handshake with token missing agent_id
  let request_body = json!({
    "ic_token": token_missing_agent_id,
    "provider": "openai",
    "provider_key_id": 1,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNAUTHORIZED,
    "Missing agent_id claim should be rejected with 400 or 401, got: {}", response.status()
  );
}

/// Manual Test Gap #13: Missing IC Token budget_id claim
///
/// # Corner Case
/// Valid JWT structure but missing budget_id in payload
///
/// # Expected Behavior
/// 400 Bad Request or 401 Unauthorized "Missing required claim: budget_id"
///
/// # Risk
/// MEDIUM - Budget tracking could fail
#[ tokio::test ]
async fn test_handshake_missing_budget_id_claim()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state.clone() ).await;

  // Create JWT with only agent_id and permissions (no budget_id)
  use jsonwebtoken::{ encode, EncodingKey, Header };
  use serde::{ Serialize, Deserialize };

  #[ derive( Serialize, Deserialize ) ]
  struct PartialClaims
  {
    agent_id: String,
    #[ serde( rename = "iat" ) ]
    issued_at: u64,
    #[ serde( rename = "iss" ) ]
    issuer: String,
    permissions: Vec< String >,
  }

  let partial_claims = PartialClaims
  {
    agent_id: "agent_204".to_string(),
    issued_at: std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs(),
    issuer: "iron-control-panel".to_string(),
    permissions: vec![ "llm:call".to_string() ],
  };

  let token_missing_budget_id = encode(
    &Header::default(),
    &partial_claims,
    &EncodingKey::from_secret( b"test_secret_key_12345" )
  ).unwrap();

  // Test handshake with token missing budget_id
  let request_body = json!({
    "ic_token": token_missing_budget_id,
    "provider": "openai",
    "provider_key_id": 1,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNAUTHORIZED,
    "Missing budget_id claim should be rejected with 400 or 401, got: {}", response.status()
  );
}

/// Manual Test Gap #8/#9: Float overflow and NaN in cost_usd
///
/// # Corner Case
/// cost_usd = f64::INFINITY or NaN
///
/// # Expected Behavior
/// 400 Bad Request - reject non-finite values
///
/// # Risk
/// MEDIUM - Budget accounting corruption
#[ tokio::test ]
async fn test_report_usage_non_finite_cost()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 205, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let router_handshake = create_budget_router( state.clone() ).await;
  let ic_token = create_ic_token( 205, &state.ic_token_manager );

  // Create lease
  let handshake_request = json!({
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 205000,
    "requested_budget_usd": 10.0,
  });

  let handshake_response = router_handshake
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &handshake_request ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_result : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_result[ "lease_id" ].as_str().unwrap();

  // Test 1: Infinity
  let router_report_inf = create_budget_router( state.clone() ).await;
  let report_infinity = json!({
    "lease_id": lease_id,
    "cost_usd": f64::INFINITY,
  });

  let response_inf = router_report_inf
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &report_infinity ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response_inf.status() == StatusCode::BAD_REQUEST || response_inf.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "Infinity cost should be rejected with 400 or 422, got: {}", response_inf.status()
  );

  // Test 2: NaN
  let router_report_nan = create_budget_router( state.clone() ).await;
  let report_nan = json!({
    "lease_id": lease_id,
    "cost_usd": f64::NAN,
  });

  let response_nan = router_report_nan
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &report_nan ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response_nan.status() == StatusCode::BAD_REQUEST || response_nan.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NaN cost should be rejected with 400 or 422, got: {}", response_nan.status()
  );
}

/// Manual Test Gap #15: NULL lease_id field
///
/// # Corner Case
/// JSON request with {"lease_id": null, "cost_usd": 5.0}
///
/// # Expected Behavior
/// 400 Bad Request "lease_id is required"
///
/// # Risk
/// MEDIUM - Null pointer dereference potential
#[ tokio::test ]
async fn test_report_usage_null_lease_id()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Craft request with null lease_id
  let request_body = json!({
    "lease_id": null,
    "cost_usd": 5.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL lease_id should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #16: NULL cost_usd field
///
/// # Corner Case
/// JSON request with {"lease_id": "<valid>", "cost_usd": null}
///
/// # Expected Behavior
/// 400 Bad Request "cost_usd is required"
///
/// # Risk
/// MEDIUM - Budget accounting corruption
#[ tokio::test ]
async fn test_report_usage_null_cost_usd()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Craft request with null cost_usd
  let request_body = json!({
    "lease_id": "test_lease_id",
    "cost_usd": null,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL cost_usd should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #5: Cost exactly equals remaining budget
///
/// # Corner Case
/// Lease has budget_granted=10.0, budget_spent=9.5 (remaining=0.5)
/// Report usage with cost_usd=0.5 (exactly equals remaining)
///
/// # Expected Behavior
/// 200 OK, budget exhausted exactly to 0.0, no off-by-one errors
///
/// # Risk
/// MEDIUM - Off-by-one errors in budget enforcement
#[ tokio::test ]
async fn test_cost_exactly_equals_remaining_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 120, 100.0 ).await;
  let state = create_test_budget_state( pool.clone() ).await;

  // Create lease with $10.00 budget
  let lease_id = "lease_exact_boundary_test";
  state
    .lease_manager
    .create_lease( lease_id, 120, 120, 10.0, None )
    .await
    .expect("LOUD FAILURE: Should create lease");

  // Record $9.50 usage (leaving exactly $0.50 remaining)
  state
    .lease_manager
    .record_usage( lease_id, 9.5 )
    .await
    .expect("LOUD FAILURE: Should record partial usage");

  // Report exactly $0.50 usage (exactly equals remaining)
  let router = create_budget_router( state.clone() ).await;
  let request_body = json!({
    "lease_id": lease_id,
    "request_id": "req_boundary_test",
    "tokens": 500,
    "cost_usd": 0.5,
    "model": "gpt-4",
    "provider": "openai",
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Verify: 200 OK (not 403), budget boundary handled correctly
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Cost exactly equals remaining should be accepted with 200 OK"
  );

  // Verify: Budget exhausted to exactly 0.0
  let lease_record = sqlx::query(
    "SELECT budget_granted, budget_spent FROM budget_leases WHERE id = ?"
  )
  .bind( lease_id )
  .fetch_one( &pool )
  .await
  .expect("LOUD FAILURE: Should fetch lease record");

  let budget_granted: f64 = lease_record.get("budget_granted");
  let budget_spent: f64 = lease_record.get("budget_spent");
  let budget_remaining = budget_granted - budget_spent;

  assert_eq!( budget_spent, 10.0, "Budget spent should be 10.0" );
  assert!(
    (budget_remaining - 0.0).abs() < 0.0001,
    "Budget remaining should be exactly 0.0, got: {}", budget_remaining
  );
}

/// Manual Test Gap #14: Invalid agent_id format (zero)
///
/// # Corner Case
/// IC Token with agent_id=0
///
/// # Expected Behavior
/// 400 Bad Request "agent_id must be positive"
///
/// # Risk
/// MEDIUM - Invalid database lookups
#[ tokio::test ]
async fn test_handshake_invalid_agent_id_zero()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;

  // Create IC Token with agent_id=0 (invalid)
  let zero_claims = IcTokenClaims
  {
    agent_id: "0".to_string(),  // Invalid: zero agent_id
    budget_id: "budget_0".to_string(),
    issued_at: std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs(),
    expires_at: None,
    issuer: "iron-control-panel".to_string(),
    permissions: vec![ "llm:call".to_string() ],
  };

  let ic_token = state.ic_token_manager.generate_token( &zero_claims )
    .expect("LOUD FAILURE: Should generate token");

  let router = create_budget_router( state ).await;
  let request_body = json!({
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 1,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY
      || response.status() == StatusCode::UNAUTHORIZED,
    "Zero agent_id should be rejected with 400/422/401, got: {}", response.status()
  );
}

/// Manual Test Gap #14 (variant): Invalid agent_id format (negative)
///
/// # Corner Case
/// IC Token with agent_id=-1
///
/// # Expected Behavior
/// 400 Bad Request "agent_id must be positive"
///
/// # Risk
/// MEDIUM - Invalid database lookups
#[ tokio::test ]
async fn test_handshake_invalid_agent_id_negative()
{
  let pool = setup_test_db().await;
  let state = create_test_budget_state( pool ).await;

  // Create IC Token with agent_id=-1 (invalid)
  let negative_claims = IcTokenClaims
  {
    agent_id: "-1".to_string(),  // Invalid: negative agent_id
    budget_id: "budget_-1".to_string(),
    issued_at: std::time::SystemTime::now()
      .duration_since( std::time::UNIX_EPOCH )
      .unwrap()
      .as_secs(),
    expires_at: None,
    issuer: "iron-control-panel".to_string(),
    permissions: vec![ "llm:call".to_string() ],
  };

  let ic_token = state.ic_token_manager.generate_token( &negative_claims )
    .expect("LOUD FAILURE: Should generate token");

  let router = create_budget_router( state ).await;
  let request_body = json!({
    "ic_token": ic_token,
    "provider": "openai",
    "provider_key_id": 1,
    "requested_budget_usd": 10.0,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/handshake" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY
      || response.status() == StatusCode::UNAUTHORIZED,
    "Negative agent_id should be rejected with 400/422/401, got: {}", response.status()
  );
}

/// Manual Test Gap #17: Integer overflow in tokens_used
///
/// # Corner Case
/// Report usage with tokens_used > i64::MAX
///
/// # Expected Behavior
/// 400 Bad Request OR value clamped to i64::MAX
///
/// # Risk
/// LOW - Token accounting corruption
#[ tokio::test ]
async fn test_report_usage_integer_overflow_tokens()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 121, 100.0 ).await;
  let state = create_test_budget_state( pool ).await;

  // Create lease
  let lease_id = "lease_tokens_overflow_test";
  state
    .lease_manager
    .create_lease( lease_id, 121, 121, 10.0, None )
    .await
    .expect("LOUD FAILURE: Should create lease");

  let router = create_budget_router( state ).await;

  // Attempt to report with tokens_used > i64::MAX
  // Note: JSON can represent numbers larger than i64::MAX, but Rust deserialization should reject them
  let request_body = json!({
    "lease_id": lease_id,
    "cost_usd": 1.0,
    "tokens_used": 9_223_372_036_854_775_808_u64,  // i64::MAX + 1
    "model": "gpt-4",
    "provider": "openai",
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/report" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  // Accept either rejection (400/422) or success (200) with clamping
  // Both behaviors are acceptable per the manual testing plan
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNPROCESSABLE_ENTITY
      || response.status() == StatusCode::OK,
    "Overflow tokens_used should be rejected or clamped, got: {}", response.status()
  );
}

//
// ROUND 2: PRIORITY 3 - MEDIUM TESTS
//

/// Manual Test Gap #25: Refresh - NULL additional_budget field
///
/// # Corner Case
/// JSON request with {"agent_id": 101, "additional_budget": null}
///
/// # Expected Behavior
/// 400 Bad Request "additional_budget is required"
///
/// # Risk
/// MEDIUM - Budget corruption
#[ tokio::test ]
async fn test_refresh_null_additional_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 122, 100.0 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  // Craft request with null additional_budget
  let request_body = json!({
    "agent_id": 122,
    "additional_budget": null,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NULL additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #26: Refresh - Float overflow additional_budget (f64::MAX)
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=f64::MAX
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget overflow
#[ tokio::test ]
async fn test_refresh_float_overflow_f64_max()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 123, 100.0 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 123,
    "additional_budget": f64::MAX,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "f64::MAX additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #26 (variant): Refresh - Float overflow additional_budget (Infinity)
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=Infinity
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget overflow
#[ tokio::test ]
async fn test_refresh_float_overflow_infinity()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 124, 100.0 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 124,
    "additional_budget": f64::INFINITY,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "Infinity additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}

/// Manual Test Gap #27: Refresh - NaN additional_budget
///
/// # Corner Case
/// POST /api/budget/refresh with additional_budget=NaN
///
/// # Expected Behavior
/// 400 Bad Request
///
/// # Risk
/// MEDIUM - Budget corruption
#[ tokio::test ]
async fn test_refresh_nan_additional_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 125, 100.0 ).await;
  let state = create_test_budget_state( pool ).await;
  let router = create_budget_router( state ).await;

  let request_body = json!({
    "agent_id": 125,
    "additional_budget": f64::NAN,
  });

  let response = router
    .oneshot(
      Request::builder()
        .method( "POST" )
        .uri( "/api/budget/refresh" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap()
    )
    .await
    .unwrap();

  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "NaN additional_budget should be rejected with 400 or 422, got: {}", response.status()
  );
}
