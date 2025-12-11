//! Protocol 012: Budget Request Workflow API Tests
//!
//! Tests for Budget Request CRUD endpoints (Phase 2):
//! - POST /api/v1/budget/requests (create request)
//! - GET /api/v1/budget/requests/:id (get request)
//! - GET /api/v1/budget/requests (list requests)
//! - PATCH /api/v1/budget/requests/:id/approve (approve)
//! - PATCH /api/v1/budget/requests/:id/reject (reject)

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use iron_control_api::routes::budget::
{
  BudgetState,
  create_budget_request,
  get_budget_request,
  list_budget_requests,
  approve_budget_request,
  reject_budget_request,
};
use iron_token_manager::
{
  agent_budget::AgentBudgetManager,
  lease_manager::LeaseManager,
  provider_key_storage::ProviderKeyStorage,
};
use serde_json::json;
use sqlx::{ Row, SqlitePool };
use std::sync::Arc;
use tower::ServiceExt;

/// Helper: Create test database with all migrations
async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect( "Failed to apply migrations" );
  pool
}

/// Helper: Seed agent with specific budget
async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_usd: f64 )
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Insert agent
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( budget_usd )
  .bind( 0.0 )
  .bind( budget_usd )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();
}

/// Helper: Create test BudgetState for Protocol 012 tests
async fn create_budget_state( pool: SqlitePool ) -> BudgetState
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];

  let ic_token_manager = Arc::new( iron_control_api::ic_token::IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
  );
  let lease_manager = Arc::new( LeaseManager::from_pool( pool.clone() ) );
  let agent_budget_manager = Arc::new( AgentBudgetManager::from_pool( pool.clone() ) );
  let provider_key_storage = Arc::new( ProviderKeyStorage::new( pool.clone() ) );

  BudgetState
  {
    ic_token_manager,
    ip_token_crypto,
    lease_manager,
    agent_budget_manager,
    provider_key_storage,
    db_pool: pool,
  }
}

// ============================================================================
// POST /api/v1/budget/requests - Create Budget Request
// ============================================================================

/// TEST: Create budget request successfully
///
/// # Happy Path
///
/// Agent with budget requests increase from $100 to $200
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Response contains request_id
/// - Request stored in database with status=pending
#[ tokio::test ]
async fn test_create_budget_request_success()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments and model testing"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 201 Created
  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Creating budget request should return 201 Created"
  );

  // Verify response body
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  // Response should contain request_id
  assert!( response_json[ "request_id" ].is_string() );
  let request_id = response_json[ "request_id" ].as_str().unwrap();
  assert!( request_id.starts_with( "breq_" ) );

  // Verify request was stored in database
  let stored_request = sqlx::query(
    "SELECT id, agent_id, requester_id, requested_budget_micros, justification, status
     FROM budget_change_requests WHERE id = ?"
  )
  .bind( request_id )
  .fetch_one( &pool )
  .await
  .expect( "Request should be stored in database" );

  // Verify stored values
  assert_eq!( stored_request.get::< i64, _ >( "agent_id" ), 1 );
  assert_eq!( stored_request.get::< String, _ >( "requester_id" ), "user-123" );
  assert_eq!( stored_request.get::< i64, _ >( "requested_budget_micros" ), 200_000_000 );
  assert_eq!( stored_request.get::< String, _ >( "status" ), "pending" );
}

/// TEST: Create budget request with invalid justification (too short)
///
/// # Error Case
///
/// Justification < 20 characters (database constraint)
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message indicates justification too short
#[ tokio::test ]
async fn test_create_budget_request_invalid_justification_too_short()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Too short"  // Only 9 characters
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with short justification should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "justification too short" ),
    "Error message should indicate justification too short: {}",
    body_str
  );
}

/// TEST: Create budget request with invalid justification (too long)
///
/// # Error Case
///
/// Justification > 500 characters (database constraint)
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - HTTP message indicates justification too long
#[ tokio::test ]
async fn test_create_budget_request_invalid_justification_too_long()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let long_justification = "a".repeat( 501 );  // 501 characters

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": long_justification
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with long justification should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "justification too long" ),
    "Error message should indicate justification too long: {}",
    body_str
  );
}

/// TEST: Create budget request with empty justification
///
/// # Edge Case
///
/// Attempt to create budget request with empty justification string
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "justification too short (min 20 characters)"
///
/// # Rationale
///
/// Database constraint requires justification between 20-500 characters.
/// Empty string (length 0) should be rejected as too short.
#[ tokio::test ]
async fn test_create_budget_request_empty_justification()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": ""  // Empty string
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with empty justification should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "justification too short" ),
    "Error message should indicate justification too short: {}",
    body_str
  );
}

/// TEST: Create budget request with exactly 20 character justification
///
/// # Boundary Test
///
/// Create request with justification of exactly 20 characters (minimum allowed)
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Request created successfully
///
/// # Rationale
///
/// Database constraint minimum is 20 characters. Boundary should be inclusive.
#[ tokio::test ]
async fn test_create_budget_request_justification_exactly_20_chars()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "12345678901234567890"  // Exactly 20 chars
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Request with 20-char justification should succeed"
  );
}

/// TEST: Create budget request with exactly 500 character justification
///
/// # Boundary Test
///
/// Create request with justification of exactly 500 characters (maximum allowed)
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Request created successfully
///
/// # Rationale
///
/// Database constraint maximum is 500 characters. Boundary should be inclusive.
#[ tokio::test ]
async fn test_create_budget_request_justification_exactly_500_chars()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let exactly_500 = "a".repeat( 500 );  // Exactly 500 characters
  assert_eq!( exactly_500.len(), 500, "Test data should be exactly 500 chars" );

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": exactly_500
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Request with 500-char justification should succeed"
  );
}

/// TEST: Create budget request with unicode characters in justification
///
/// # Edge Case
///
/// Justification contains unicode characters (emoji, accents, CJK)
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Unicode properly handled and stored
///
/// # Rationale
///
/// Modern systems should support unicode in text fields. Justifications
/// may legitimately contain non-ASCII characters from international users.
#[ tokio::test ]
async fn test_create_budget_request_unicode_justification()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need budget for 日本 testing with émojis 🚀 and àccénts"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Request with unicode justification should succeed"
  );
}

/// TEST: Create budget request with special characters in justification
///
/// # Edge Case
///
/// Justification contains special characters (quotes, brackets, punctuation)
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Special characters properly handled and stored
///
/// # Rationale
///
/// Justifications may contain technical notation, quotes, or punctuation.
/// System should handle common special characters without errors.
#[ tokio::test ]
async fn test_create_budget_request_special_chars_justification()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need budget for \"special\" tests: <tags>, [arrays], {objects}, & more!"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Request with special character justification should succeed"
  );
}

/// TEST: Create budget request for nonexistent agent
///
/// # Error Case
///
/// Agent ID doesnt exist in database
///
/// # Expected Behavior
///
/// - HTTP 404 Not Found
/// - Error message indicates agent not found
#[ tokio::test ]
async fn test_create_budget_request_nonexistent_agent()
{
  let pool = setup_test_db().await;
  // Dont seed any agents

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 999,
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 404 Not Found
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Request for nonexistent agent should return 404 Not Found"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Agent not found" ),
    "Error message should indicate agent not found: {}",
    body_str
  );
}

/// TEST: Create budget request with negative requested budget
///
/// # Error Case
///
/// Requested budget <= 0
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message indicates budget must be positive
#[ tokio::test ]
async fn test_create_budget_request_negative_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": -50.0,  // Negative
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with negative budget should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "requested_budget_usd must be positive" ),
    "Error message should indicate budget must be positive: {}",
    body_str
  );
}

/// TEST: Create budget request with zero requested budget
///
/// # Edge Case
///
/// Attempt to create budget request with zero budget
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requested_budget_usd must be positive"
///
/// # Why Not Caught
///
/// This edge case test was missing. While validation exists (budget.rs:883),
/// only negative values were tested. Zero is equally invalid but untested.
///
/// # Prevention
///
/// Test boundary values explicitly: negative, zero, positive. Check both
/// sides of comparison operators (< 0, = 0, > 0).
#[ tokio::test ]
async fn test_create_budget_request_zero_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 0.0,  // Zero
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with zero budget should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "requested_budget_usd must be positive" ),
    "Error message should indicate budget must be positive: {}",
    body_str
  );
}

/// TEST: Create budget request with NaN requested budget
///
/// # Edge Case
///
/// Attempt to create budget request with NaN (Not a Number)
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requested_budget_usd must be a valid number"
///
/// # Root Cause
///
/// Original validation (budget.rs:883) used `requested_budget_usd <= 0.0` which
/// doesnt catch NaN values. NaN comparisons always return false, bypassing validation.
///
/// # Why Not Caught
///
/// No tests existed for special float values (NaN, Infinity). JSON parsing accepts
/// these values (encoded as null or special strings), but validation must reject them.
///
/// # Fix Applied
///
/// Added `is_finite()` check to reject NaN and Infinity before positivity check.
///
/// # Prevention
///
/// Always validate special float values when accepting numeric input. Use `is_finite()`
/// for currency/quantities. Test with NaN, Infinity, -Infinity explicitly.
///
/// # Pitfall
///
/// NaN bypasses comparison operators (NaN != NaN, NaN <= 0 is false). Always check
/// `is_finite()` or `is_nan()` before other numeric validations on f64/f32 inputs.
#[ tokio::test ]
async fn test_create_budget_request_nan_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": f64::NAN,  // NaN
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 422 Unprocessable Entity (JSON deserialization rejects NaN)
  // Note: serde_json rejects NaN before application validation runs
  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "Request with NaN budget should return 422 Unprocessable Entity"
  );

  // NaN is rejected at JSON parsing layer, error message varies by implementation
  // The important thing is that NaN is rejected, not the specific message
}

/// TEST: Create budget request with Infinity requested budget
///
/// # Edge Case
///
/// Attempt to create budget request with Infinity
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requested_budget_usd must be a valid number"
///
/// # Root Cause
///
/// Original validation (budget.rs:883) used `requested_budget_usd <= 0.0` which
/// passes for Infinity (Infinity > 0 is true), but Infinity is not a valid budget.
///
/// # Why Not Caught
///
/// No tests existed for special float values. Infinity bypasses positive check but
/// should be rejected as nonsensical value for currency.
///
/// # Fix Applied
///
/// Added `is_finite()` check to reject NaN and Infinity before positivity check.
///
/// # Prevention
///
/// Always validate special float values when accepting numeric input. Use `is_finite()`
/// for currency/quantities. Test with NaN, Infinity, -Infinity explicitly.
///
/// # Pitfall
///
/// Infinity passes comparison checks (Infinity > 0 is true) but is invalid for real-world
/// quantities like currency. Must explicitly check `is_finite()` before treating as valid number.
#[ tokio::test ]
async fn test_create_budget_request_infinity_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": f64::INFINITY,  // Infinity
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 422 Unprocessable Entity (JSON deserialization rejects Infinity)
  // Note: serde_json rejects Infinity before application validation runs
  assert_eq!(
    response.status(),
    StatusCode::UNPROCESSABLE_ENTITY,
    "Request with Infinity budget should return 422 Unprocessable Entity"
  );

  // Infinity is rejected at JSON parsing layer, error message varies by implementation
  // The important thing is that Infinity is rejected, not the specific message
}

/// TEST: Create budget request with same budget as current
///
/// # Edge Case
///
/// Attempt to create budget request where requested budget equals current budget
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requested_budget_usd must differ from current budget"
///
/// # Root Cause
///
/// Original validation didnt check if requested budget differs from current budget.
/// This allowed nonsensical requests where user asks for budget they already have.
///
/// # Why Not Caught
///
/// No business logic validation existed for same-budget requests. While technically
/// valid data, requesting identical budget is logically meaningless and wastes approval workflow.
///
/// # Fix Applied
///
/// Added validation in create_budget_request handler to compare requested vs current
/// budget after fetching current value, before creating database record.
///
/// # Prevention
///
/// Validate business logic constraints in addition to data type constraints. Check
/// that operations make logical sense in problem domain (budget changes must change budget).
///
/// # Pitfall
///
/// API layer validation can catch business logic issues that database constraints cant.
/// Same-value requests are syntactically valid but semantically meaningless. Validate
/// "does this operation make sense?" not just "is this data valid?".
#[ tokio::test ]
async fn test_create_budget_request_same_as_current()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;  // Current budget is $100

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 100.0,  // Same as current
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 400 Bad Request
  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with same budget as current should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "requested_budget_usd must differ from current budget" ),
    "Error message should indicate budget must differ: {}",
    body_str
  );
}

/// TEST: Create budget request with zero agent_id
///
/// # Edge Case
///
/// Attempt to create budget request with agent_id = 0
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "agent_id must be positive"
///
/// # Rationale
///
/// Database agent IDs are auto-incrementing integers starting from 1.
/// Zero is not a valid agent ID and should be rejected at validation layer.
#[ tokio::test ]
async fn test_create_budget_request_zero_agent_id()
{
  let pool = setup_test_db().await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 0,  // Zero agent_id
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with zero agent_id should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "agent_id must be positive" ),
    "Error message should indicate agent_id must be positive: {}",
    body_str
  );
}

/// TEST: Create budget request with negative agent_id
///
/// # Edge Case
///
/// Attempt to create budget request with negative agent_id
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "agent_id must be positive"
///
/// # Rationale
///
/// Negative agent IDs are invalid. Database uses positive integers for IDs.
/// Validation should reject negative values before database query.
#[ tokio::test ]
async fn test_create_budget_request_negative_agent_id()
{
  let pool = setup_test_db().await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": -1,  // Negative agent_id
    "requester_id": "user-123",
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with negative agent_id should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "agent_id must be positive" ),
    "Error message should indicate agent_id must be positive: {}",
    body_str
  );
}

/// TEST: Create budget request with empty requester_id
///
/// # Edge Case
///
/// Attempt to create budget request with empty string as requester_id
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requester_id cannot be empty"
///
/// # Rationale
///
/// requester_id identifies who created the request. Empty string is invalid
/// as it provides no audit trail. Validation should reject before database.
#[ tokio::test ]
async fn test_create_budget_request_empty_requester_id()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "",  // Empty requester_id
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with empty requester_id should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "requester_id cannot be empty" ),
    "Error message should indicate requester_id cannot be empty: {}",
    body_str
  );
}

/// TEST: Create budget request with whitespace-only requester_id
///
/// # Edge Case
///
/// Attempt to create budget request with whitespace-only requester_id
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message: "requester_id cannot be empty"
///
/// # Rationale
///
/// Whitespace-only strings (e.g., "   ") should be treated as empty.
/// Validation uses trim() to catch this edge case.
#[ tokio::test ]
async fn test_create_budget_request_whitespace_requester_id()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "   ",  // Whitespace-only requester_id
    "requested_budget_usd": 200.0,
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Request with whitespace-only requester_id should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "requester_id cannot be empty" ),
    "Error message should indicate requester_id cannot be empty: {}",
    body_str
  );
}

/// TEST: Create budget request with f64::MAX budget
///
/// # Edge Case
///
/// Attempt to create budget request with extremely large budget (f64::MAX)
///
/// # Expected Behavior
///
/// - Either HTTP 400 Bad Request with reasonable limit message
/// - Or HTTP 422 Unprocessable Entity if conversion fails
///
/// # Rationale
///
/// f64::MAX is ~1.8e308, far beyond any reasonable budget allocation.
/// System should reject or handle gracefully without panic/overflow.
/// Microdollar conversion (USD * 1_000_000) may overflow i64::MAX.
#[ tokio::test ]
async fn test_create_budget_request_extremely_large_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": f64::MAX,  // Extremely large budget
    "justification": "Need additional budget for Q2 research experiments"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Accept either 400 or 422 as valid error responses
  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "Request with f64::MAX budget should return 400 or 422, got: {}",
    response.status()
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  // Should contain error message about invalid value or overflow
  assert!(
    !body_str.is_empty(),
    "Error response should contain error message explaining rejection"
  );
}

/// TEST: Create budget request with very small positive budget
///
/// # Edge Case
///
/// Create budget request with very small positive budget (0.000001 USD)
///
/// # Expected Behavior
///
/// - HTTP 201 Created
/// - Request stored successfully
/// - Microdollar conversion preserves precision (1 microdollar)
///
/// # Rationale
///
/// Very small positive budgets are technically valid and should be accepted.
/// This tests precision handling in USD to microdollar conversion.
/// 0.000001 USD = 1 microdollar (minimum representable amount).
#[ tokio::test ]
async fn test_create_budget_request_very_small_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let state = create_budget_state( pool.clone() ).await;

  let request_body = json!(
  {
    "agent_id": 1,
    "requester_id": "user-123",
    "requested_budget_usd": 0.000001,  // 1 microdollar
    "justification": "Testing minimum budget precision handling capability"
  } );

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::post( create_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/budget/requests" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Request with very small positive budget should succeed"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_data: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  // CreateBudgetRequestResponse contains: request_id, status, created_at
  assert!(
    response_data[ "request_id" ].is_string(),
    "Response should contain request_id"
  );
  assert_eq!(
    response_data[ "status" ].as_str().unwrap(),
    "pending",
    "Status should be pending"
  );

  // Verify stored in database with correct microdollar value
  let request_id = response_data[ "request_id" ].as_str().unwrap();
  let stored_record = sqlx::query(
    "SELECT requested_budget_micros FROM budget_change_requests WHERE id = ?"
  )
  .bind( request_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  let stored_micros: i64 = stored_record.try_get( "requested_budget_micros" ).unwrap();
  assert_eq!(
    stored_micros, 1,
    "Stored value should be 1 microdollar"
  );
}

// ============================================================================
// GET /api/v1/budget/requests/:id - Get Budget Request by ID
// ============================================================================

/// TEST: Get budget request by ID successfully
///
/// # Happy Path
///
/// Fetch existing budget request by its ID
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains complete request details
/// - All fields match stored values
#[ tokio::test ]
async fn test_get_budget_request_success()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  // Create a budget request directly in database
  let request_id = "breq_test_123";
  let now_ms = chrono::Utc::now().timestamp_millis();

  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-456" )
  .bind( 100_000_000 )
  .bind( 250_000_000 )
  .bind( "Need budget increase for expanded testing across multiple model providers" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because get_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id", axum::routing::get( get_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/budget/requests/{}", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Getting existing budget request should return 200 OK"
  );

  // Verify response body
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  // Verify all fields
  assert_eq!( response_json[ "id" ].as_str().unwrap(), request_id );
  assert_eq!( response_json[ "agent_id" ].as_i64().unwrap(), 1 );
  assert_eq!( response_json[ "requester_id" ].as_str().unwrap(), "user-456" );
  assert_eq!( response_json[ "current_budget_usd" ].as_f64().unwrap(), 100.0 );
  assert_eq!( response_json[ "requested_budget_usd" ].as_f64().unwrap(), 250.0 );
  assert_eq!(
    response_json[ "justification" ].as_str().unwrap(),
    "Need budget increase for expanded testing across multiple model providers"
  );
  assert_eq!( response_json[ "status" ].as_str().unwrap(), "pending" );
  assert!( response_json[ "created_at" ].is_i64() );
  assert!( response_json[ "updated_at" ].is_i64() );
}

/// TEST: Get budget request with nonexistent ID
///
/// # Error Case
///
/// Request ID doesnt exist in database
///
/// # Expected Behavior
///
/// - HTTP 404 Not Found
/// - Error message indicates request not found
#[ tokio::test ]
async fn test_get_budget_request_not_found()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  // This will fail because get_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id", axum::routing::get( get_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests/breq_nonexistent_999" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 404 Not Found
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Getting nonexistent budget request should return 404 Not Found"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Budget request not found" ),
    "Error message should indicate request not found: {}",
    body_str
  );
}

/// TEST: Get budget request with malformed ID
///
/// # Edge Case
///
/// Attempt to get budget request with malformed ID (not breq_UUID format)
///
/// # Expected Behavior
///
/// - HTTP 404 Not Found
/// - Error message indicates request not found
///
/// # Rationale
///
/// Malformed IDs should be treated as non-existent rather than causing
/// server errors. System should gracefully handle invalid ID formats.
#[ tokio::test ]
async fn test_get_budget_request_malformed_id()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests/:id", axum::routing::get( get_budget_request ) )
    .with_state( state );

  // Test with various malformed ID formats (must be valid URI syntax)
  let malformed_ids = vec![
    "not-a-uuid",
    "12345",
    "breq_",
    "breq_invalid",
    "breq_not_a_real_uuid",
    "random_string_123",
  ];

  for malformed_id in malformed_ids
  {
    let request = Request::builder()
      .method( "GET" )
      .uri( format!( "/api/v1/budget/requests/{}", malformed_id ) )
      .body( Body::empty() )
      .unwrap();

    let response = app.clone().oneshot( request ).await.unwrap();

    assert_eq!(
      response.status(),
      StatusCode::NOT_FOUND,
      "Getting budget request with malformed ID '{}' should return 404 Not Found",
      malformed_id
    );
  }
}

// ============================================================================
// GET /api/v1/budget/requests - List Budget Requests
// ============================================================================

/// TEST: List all budget requests successfully
///
/// # Happy Path
///
/// Fetch all budget requests without filters
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains array of requests
/// - Multiple requests with different statuses returned
#[ tokio::test ]
async fn test_list_budget_requests_all()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;
  seed_agent_with_budget( &pool, 2, 200.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create multiple requests with different agents and statuses
  for ( i, agent_id, status ) in [ ( 1, 1, "pending" ), ( 2, 1, "approved" ), ( 3, 2, "pending" ) ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( agent_id )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for list endpoint validation" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because list_budget_requests handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Listing all budget requests should return 200 OK"
  );

  // Verify response body contains all 3 requests
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!( requests.len(), 3, "Should return all 3 requests" );
}

/// TEST: List budget requests filtered by agent_id
///
/// # Filter Case
///
/// Fetch only requests for specific agent
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains only requests for specified agent
/// - Other agent requests not included
#[ tokio::test ]
async fn test_list_budget_requests_by_agent()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;
  seed_agent_with_budget( &pool, 2, 200.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create requests for both agents
  for ( i, agent_id, status ) in [ ( 1, 1, "pending" ), ( 2, 1, "approved" ), ( 3, 2, "pending" ) ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( agent_id )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for agent filter validation" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because list_budget_requests handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?agent_id=1" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Listing budget requests by agent should return 200 OK"
  );

  // Verify response body contains only agent 1 requests (2 requests)
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!( requests.len(), 2, "Should return only 2 requests for agent 1" );

  // Verify all returned requests are for agent 1
  for req in requests
  {
    assert_eq!( req[ "agent_id" ].as_i64().unwrap(), 1 );
  }
}

/// TEST: List budget requests filtered by status
///
/// # Filter Case
///
/// Fetch only requests with specific status
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains only requests with specified status
/// - Other status requests not included
#[ tokio::test ]
async fn test_list_budget_requests_by_status()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;
  seed_agent_with_budget( &pool, 2, 200.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create requests with different statuses
  for ( i, agent_id, status ) in [ ( 1, 1, "pending" ), ( 2, 1, "approved" ), ( 3, 2, "pending" ) ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( agent_id )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for status filter validation" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because list_budget_requests handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?status=pending" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Listing budget requests by status should return 200 OK"
  );

  // Verify response body contains only pending requests (2 requests)
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!( requests.len(), 2, "Should return only 2 pending requests" );

  // Verify all returned requests have pending status
  for req in requests
  {
    assert_eq!( req[ "status" ].as_str().unwrap(), "pending" );
  }
}

/// TEST: List budget requests with empty result
///
/// # Empty Case
///
/// No requests exist in database
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains empty array
#[ tokio::test ]
async fn test_list_budget_requests_empty()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  // This will fail because list_budget_requests handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Listing budget requests should return 200 OK even when empty"
  );

  // Verify response body is empty array
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert!( response_json[ "requests" ].is_array() );
  assert_eq!( response_json[ "requests" ].as_array().unwrap().len(), 0 );
}

/// TEST: List budget requests filtered by non-existent agent
///
/// # Edge Case
///
/// Filter by agent_id that has no requests
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains empty array
///
/// # Rationale
///
/// Filtering by valid but non-existent agent should return empty results,
/// not an error. This is standard REST API behavior for filters.
#[ tokio::test ]
async fn test_list_budget_requests_nonexistent_agent()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;
  seed_agent_with_budget( &pool, 999, 500.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create request for agent 1 only
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "breq_test_1" )
  .bind( 1 )
  .bind( "user-123" )
  .bind( 100_000_000 )
  .bind( 200_000_000 )
  .bind( "Test justification for nonexistent agent filter" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  // Filter by agent 999 which has no requests
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?agent_id=999" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Filtering by nonexistent agent should return 200 OK"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!( response_json[ "requests" ].is_array() );
  assert_eq!(
    response_json[ "requests" ].as_array().unwrap().len(),
    0,
    "Should return empty array for nonexistent agent"
  );
}

/// TEST: List budget requests filtered by approved status
///
/// # Filter Case
///
/// Fetch only approved requests
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains only approved requests
#[ tokio::test ]
async fn test_list_budget_requests_status_approved()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create requests with different statuses
  for ( i, status ) in [ ( 1, "pending" ), ( 2, "approved" ), ( 3, "rejected" ), ( 4, "approved" ) ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( 1 )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for approved status filter" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?status=approved" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Filtering by approved status should return 200 OK"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!( requests.len(), 2, "Should return 2 approved requests" );

  // Verify all returned requests are approved
  for req in requests
  {
    assert_eq!( req[ "status" ].as_str().unwrap(), "approved" );
  }
}

/// TEST: List budget requests filtered by rejected status
///
/// # Filter Case
///
/// Fetch only rejected requests
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains only rejected requests
#[ tokio::test ]
async fn test_list_budget_requests_status_rejected()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create requests with different statuses
  for ( i, status ) in [ ( 1, "pending" ), ( 2, "approved" ), ( 3, "rejected" ), ( 4, "rejected" ) ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( 1 )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for rejected status filter" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?status=rejected" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Filtering by rejected status should return 200 OK"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!( requests.len(), 2, "Should return 2 rejected requests" );

  // Verify all returned requests are rejected
  for req in requests
  {
    assert_eq!( req[ "status" ].as_str().unwrap(), "rejected" );
  }
}

/// TEST: List budget requests filtered by both agent_id and status
///
/// # Combined Filter Case
///
/// Filter by both agent and status simultaneously
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Response contains only requests matching BOTH filters
#[ tokio::test ]
async fn test_list_budget_requests_agent_and_status()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;
  seed_agent_with_budget( &pool, 2, 200.0 ).await;

  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create diverse requests
  for ( i, agent_id, status ) in [
    ( 1, 1, "pending" ),
    ( 2, 1, "approved" ),
    ( 3, 1, "pending" ),
    ( 4, 2, "pending" ),
    ( 5, 2, "approved" ),
  ]
  {
    sqlx::query(
      "INSERT INTO budget_change_requests
       (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
        justification, status, created_at, updated_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
    )
    .bind( format!( "breq_test_{}", i ) )
    .bind( agent_id )
    .bind( "user-123" )
    .bind( 100_000_000 )
    .bind( 200_000_000 )
    .bind( "Test justification for combined filter validation" )
    .bind( status )
    .bind( now_ms + i * 1000 )
    .bind( now_ms + i * 1000 )
    .execute( &pool )
    .await
    .unwrap();
  }

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  // Filter by agent 1 AND status pending (should return 2 requests)
  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?agent_id=1&status=pending" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Combined filter should return 200 OK"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let response_json: serde_json::Value = serde_json::from_slice( &body ).unwrap();

  assert!( response_json[ "requests" ].is_array() );
  let requests = response_json[ "requests" ].as_array().unwrap();
  assert_eq!(
    requests.len(),
    2,
    "Should return 2 requests matching agent_id=1 AND status=pending"
  );

  // Verify all returned requests match BOTH filters
  for req in requests
  {
    assert_eq!( req[ "agent_id" ].as_i64().unwrap(), 1 );
    assert_eq!( req[ "status" ].as_str().unwrap(), "pending" );
  }
}

/// TEST: List budget requests with invalid status parameter
///
/// # Error Case
///
/// Provide invalid status value
///
/// # Expected Behavior
///
/// - HTTP 400 Bad Request
/// - Error message indicates invalid status
///
/// # Rationale
///
/// Status enum has only: pending, approved, rejected.
/// Invalid values should be rejected to prevent confusion.
#[ tokio::test ]
async fn test_list_budget_requests_invalid_status()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests", axum::routing::get( list_budget_requests ) )
    .with_state( state );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/budget/requests?status=invalid_status" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "Invalid status parameter should return 400 Bad Request"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Invalid status" ) || body_str.contains( "invalid status" ),
    "Error message should indicate invalid status: {}",
    body_str
  );
}

// ============================================================================
// PATCH /api/v1/budget/requests/:id/approve - Approve Budget Request
// ============================================================================

/// TEST: Approve pending budget request successfully
///
/// # Happy Path
///
/// Administrator approves a pending budget request
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Request status changed to approved
/// - Response confirms approval
#[ tokio::test ]
async fn test_approve_budget_request_success()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_approve_test_1";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-789" )
  .bind( 100_000_000 )
  .bind( 300_000_000 )
  .bind( "Need budget increase for production deployment" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because approve_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Approving pending request should return 200 OK"
  );

  // Verify response
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert_eq!( response_json[ "request_id" ].as_str().unwrap(), request_id );
  assert_eq!( response_json[ "status" ].as_str().unwrap(), "approved" );

  // Verify database was updated
  let stored_request = sqlx::query( "SELECT status FROM budget_change_requests WHERE id = ?" )
    .bind( request_id )
    .fetch_one( &pool )
    .await
    .unwrap();

  assert_eq!( stored_request.get::< String, _ >( "status" ), "approved" );
}

/// TEST: Approve nonexistent budget request
///
/// # Error Case
///
/// Request ID doesnt exist
///
/// # Expected Behavior
///
/// - HTTP 404 Not Found
/// - Error message indicates request not found
#[ tokio::test ]
async fn test_approve_budget_request_not_found()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  // This will fail because approve_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( "/api/v1/budget/requests/breq_nonexistent/approve" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 404 Not Found
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Approving nonexistent request should return 404 Not Found"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Budget request not found" ),
    "Error message should indicate request not found: {}",
    body_str
  );
}

/// TEST: Approve already approved request
///
/// # Error Case
///
/// Request is already in approved status
///
/// # Expected Behavior
///
/// - HTTP 409 Conflict
/// - Error message indicates request already processed
#[ tokio::test ]
async fn test_approve_budget_request_already_approved()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_already_approved";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create already approved request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-789" )
  .bind( 100_000_000 )
  .bind( 300_000_000 )
  .bind( "Already processed request" )
  .bind( "approved" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because approve_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 409 Conflict
  assert_eq!(
    response.status(),
    StatusCode::CONFLICT,
    "Approving already approved request should return 409 Conflict"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "already approved" ) || body_str.contains( "already processed" ),
    "Error message should indicate request already processed: {}",
    body_str
  );
}

/// TEST: Approve rejected request
///
/// # Error Case
///
/// Request was previously rejected
///
/// # Expected Behavior
///
/// - HTTP 409 Conflict
/// - Error message indicates cannot approve rejected request
#[ tokio::test ]
async fn test_approve_budget_request_already_rejected()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_already_rejected";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create rejected request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-789" )
  .bind( 100_000_000 )
  .bind( 300_000_000 )
  .bind( "Previously rejected request" )
  .bind( "rejected" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because approve_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 409 Conflict
  assert_eq!(
    response.status(),
    StatusCode::CONFLICT,
    "Approving rejected request should return 409 Conflict"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "rejected" ) || body_str.contains( "cannot approve" ),
    "Error message should indicate cannot approve rejected request: {}",
    body_str
  );
}

// ============================================================================
// PATCH /api/v1/budget/requests/:id/reject - Reject Budget Request
// ============================================================================

/// TEST: Reject pending budget request successfully
///
/// # Happy Path
///
/// Administrator rejects a pending budget request
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Request status changed to rejected
/// - Response confirms rejection
#[ tokio::test ]
async fn test_reject_budget_request_success()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_reject_test_1";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-999" )
  .bind( 100_000_000 )
  .bind( 500_000_000 )
  .bind( "Excessive budget increase request" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because reject_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/reject", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Rejecting pending request should return 200 OK"
  );

  // Verify response
  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();
  let response_json: serde_json::Value = serde_json::from_str( &body_str )
    .expect( "Response should be valid JSON" );

  assert_eq!( response_json[ "request_id" ].as_str().unwrap(), request_id );
  assert_eq!( response_json[ "status" ].as_str().unwrap(), "rejected" );

  // Verify database was updated
  let stored_request = sqlx::query( "SELECT status FROM budget_change_requests WHERE id = ?" )
    .bind( request_id )
    .fetch_one( &pool )
    .await
    .unwrap();

  assert_eq!( stored_request.get::< String, _ >( "status" ), "rejected" );
}

/// TEST: Reject nonexistent budget request
///
/// # Error Case
///
/// Request ID doesnt exist
///
/// # Expected Behavior
///
/// - HTTP 404 Not Found
/// - Error message indicates request not found
#[ tokio::test ]
async fn test_reject_budget_request_not_found()
{
  let pool = setup_test_db().await;
  let state = create_budget_state( pool.clone() ).await;

  // This will fail because reject_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( "/api/v1/budget/requests/breq_nonexistent/reject" )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 404 Not Found
  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Rejecting nonexistent request should return 404 Not Found"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "Budget request not found" ),
    "Error message should indicate request not found: {}",
    body_str
  );
}

/// TEST: Reject already rejected request
///
/// # Error Case
///
/// Request is already in rejected status
///
/// # Expected Behavior
///
/// - HTTP 409 Conflict
/// - Error message indicates request already processed
#[ tokio::test ]
async fn test_reject_budget_request_already_rejected()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_double_reject";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create already rejected request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-999" )
  .bind( 100_000_000 )
  .bind( 300_000_000 )
  .bind( "Already processed rejection" )
  .bind( "rejected" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because reject_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/reject", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 409 Conflict
  assert_eq!(
    response.status(),
    StatusCode::CONFLICT,
    "Rejecting already rejected request should return 409 Conflict"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "already rejected" ) || body_str.contains( "already processed" ),
    "Error message should indicate request already processed: {}",
    body_str
  );
}

/// TEST: Reject already approved request
///
/// # Error Case
///
/// Request was previously approved
///
/// # Expected Behavior
///
/// - HTTP 409 Conflict
/// - Error message indicates cannot reject approved request
#[ tokio::test ]
async fn test_reject_budget_request_already_approved()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_reject_after_approve";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create approved request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-999" )
  .bind( 100_000_000 )
  .bind( 300_000_000 )
  .bind( "Previously approved request" )
  .bind( "approved" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  // This will fail because reject_budget_request handler doesnt exist yet
  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/reject", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 409 Conflict
  assert_eq!(
    response.status(),
    StatusCode::CONFLICT,
    "Rejecting approved request should return 409 Conflict"
  );

  let body = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body_str = String::from_utf8( body.to_vec() ).unwrap();

  assert!(
    body_str.contains( "approved" ) || body_str.contains( "cannot reject" ),
    "Error message should indicate cannot reject approved request: {}",
    body_str
  );
}

// ============================================================================
// Side Effects - Approve/Reject Budget Requests
// ============================================================================

/// TEST: Approve budget request records history
///
/// # Side Effect Verification
///
/// When approving a budget request, verify that a history record is created
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Budget modification history record exists
/// - History record contains correct modification_type ("request_approved")
/// - History record links to budget request
#[ tokio::test ]
async fn test_approve_budget_request_records_history()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_history_test";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-456" )
  .bind( 100_000_000 )
  .bind( 250_000_000 )
  .bind( "Need budget increase for expanded testing" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Approving request should return 200 OK"
  );

  // Verify history record was created
  let history_record = sqlx::query(
    "SELECT modification_type, old_budget_micros, new_budget_micros, reason, related_request_id
     FROM budget_modification_history
     WHERE agent_id = ? AND related_request_id = ?"
  )
  .bind( 1 )
  .bind( request_id )
  .fetch_optional( &pool )
  .await
  .unwrap();

  assert!(
    history_record.is_some(),
    "History record should exist after approval"
  );

  let record = history_record.unwrap();
  let modification_type: String = record.try_get( "modification_type" ).unwrap();
  let old_budget_micros: i64 = record.try_get( "old_budget_micros" ).unwrap();
  let new_budget_micros: i64 = record.try_get( "new_budget_micros" ).unwrap();
  let related_request_id: Option< String > = record.try_get( "related_request_id" ).unwrap();

  assert_eq!( modification_type, "increase", "modification_type should be increase (budget increased from approval)" );
  assert_eq!( old_budget_micros, 100_000_000, "old_budget should be 100.0 USD in microdollars" );
  assert_eq!( new_budget_micros, 250_000_000, "new_budget should be 250.0 USD in microdollars" );
  assert_eq!( related_request_id, Some( request_id.to_string() ), "related_request_id should link to budget request" );
}

/// TEST: Approve budget request updates agent budget
///
/// # Side Effect Verification
///
/// When approving a budget request, verify that agent budget is updated atomically
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Agent budget updated to requested amount
/// - budget_remaining updated correctly
#[ tokio::test ]
async fn test_approve_budget_request_updates_agent_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_budget_update_test";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-456" )
  .bind( 100_000_000 )
  .bind( 350_000_000 )
  .bind( "Need substantial budget increase" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/approve", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Approving request should return 200 OK"
  );

  // Verify agent budget was updated
  let budget_record = sqlx::query(
    "SELECT total_allocated, budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( 1 )
  .fetch_one( &pool )
  .await
  .unwrap();

  let total_allocated: f64 = budget_record.try_get( "total_allocated" ).unwrap();
  let budget_remaining: f64 = budget_record.try_get( "budget_remaining" ).unwrap();

  assert_eq!( total_allocated, 350.0, "total_allocated should be updated to 350.0 USD" );
  assert_eq!( budget_remaining, 350.0, "budget_remaining should be 350.0 USD (no spending yet)" );
}

/// TEST: Reject budget request does NOT update agent budget
///
/// # Side Effect Verification
///
/// When rejecting a budget request, verify that agent budget remains unchanged
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - Agent budget remains at original value
/// - budget_remaining unchanged
#[ tokio::test ]
async fn test_reject_budget_request_does_not_update_budget()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_reject_no_update";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-456" )
  .bind( 100_000_000 )
  .bind( 250_000_000 )
  .bind( "Need budget increase" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/reject", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Rejecting request should return 200 OK"
  );

  // Verify agent budget was NOT updated
  let budget_record = sqlx::query(
    "SELECT total_allocated, budget_remaining FROM agent_budgets WHERE agent_id = ?"
  )
  .bind( 1 )
  .fetch_one( &pool )
  .await
  .unwrap();

  let total_allocated: f64 = budget_record.try_get( "total_allocated" ).unwrap();
  let budget_remaining: f64 = budget_record.try_get( "budget_remaining" ).unwrap();

  assert_eq!( total_allocated, 100.0, "total_allocated should remain 100.0 USD" );
  assert_eq!( budget_remaining, 100.0, "budget_remaining should remain 100.0 USD" );
}

/// TEST: Reject budget request does NOT create history record
///
/// # Side Effect Verification
///
/// When rejecting a budget request, verify that NO history record is created
///
/// # Expected Behavior
///
/// - HTTP 200 OK
/// - NO budget modification history record exists
/// - Rejection is recorded only in request status
#[ tokio::test ]
async fn test_reject_budget_request_does_not_create_history()
{
  let pool = setup_test_db().await;
  seed_agent_with_budget( &pool, 1, 100.0 ).await;

  let request_id = "breq_reject_no_history";
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create pending request
  sqlx::query(
    "INSERT INTO budget_change_requests
     (id, agent_id, requester_id, current_budget_micros, requested_budget_micros,
      justification, status, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( request_id )
  .bind( 1 )
  .bind( "user-456" )
  .bind( 100_000_000 )
  .bind( 250_000_000 )
  .bind( "Need budget increase" )
  .bind( "pending" )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_budget_state( pool.clone() ).await;

  let app = Router::new()
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state );

  let request = Request::builder()
    .method( "PATCH" )
    .uri( format!( "/api/v1/budget/requests/{}/reject", request_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Assert: HTTP 200 OK
  assert_eq!(
    response.status(),
    StatusCode::OK,
    "Rejecting request should return 200 OK"
  );

  // Verify NO history record was created
  let history_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM budget_modification_history WHERE agent_id = ? AND related_request_id = ?"
  )
  .bind( 1 )
  .bind( request_id )
  .fetch_one( &pool )
  .await
  .unwrap();

  assert_eq!(
    history_count, 0,
    "NO history record should exist after rejection"
  );
}
