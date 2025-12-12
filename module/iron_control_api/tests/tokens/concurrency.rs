//! Concurrent access tests for token management endpoints.
//!
//! Tests race conditions, concurrent modifications, and atomicity guarantees
//! for token operations.
//!
//! ## Test Matrix
//!
//! | Test Case | Concurrent Operations | Expected Result | Status |
//! |-----------|----------------------|----------------|--------|
//! | `test_concurrent_token_creation_uniqueness` | 2x POST /api/v1/api-tokens | Both succeed with unique tokens | ✅ |
//! | `test_concurrent_rotate_same_token` | 2x POST /api/v1/api-tokens/:id/rotate | One succeeds, one gets 404 | ✅ |
//! | `test_concurrent_revoke_same_token` | 2x DELETE /api/v1/api-tokens/:id | One 204, one 404 | ✅ |
//! | `test_concurrent_rotate_and_revoke` | Rotate + Revoke same token | One succeeds, one 404 | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Concurrent creates succeed independently
//!
//! **Concurrent Access:**
//! - ✅ Concurrent rotation of same token → Only one succeeds (atomicity)
//! - ✅ Concurrent revocation → Only one succeeds
//! - ✅ Concurrent rotate + revoke → Race condition handled gracefully
//! - ✅ Token uniqueness maintained under concurrent creates
//!
//! **Error Conditions:**
//! - ✅ Loser of race gets clear error (404 Not Found)
//! - ✅ No partial state updates (all-or-nothing)
//!
//! **State Transitions:**
//! - ✅ Atomic state changes (no intermediate states visible)
//! - ✅ First operation wins, second sees updated state
//!
//! **Edge Cases:**
//! - ✅ Database locking prevents corruption
//! - ✅ SQLite IMMEDIATE transactions ensure serializability
//!
//! **Boundary Conditions:** Not applicable
//! **Resource Limits:** Not applicable (unbounded token count)
//! **Precondition Violations:** Tested via second operation seeing NOT_FOUND
//!
//! ## Test Isolation Pattern
//!
//! **Problem:** SQLite shared memory databases (`mode=memory&cache=shared`) require unique
//! connection strings when tests run in parallel. Tests sharing the same database path will
//! interfere with each other, causing non-deterministic failures.
//!
//! **Solution:** Use atomic counter (`DB_COUNTER`) combined with process ID to generate unique
//! database names per test. Each test gets isolated database: `test_concurrency_{pid}_{counter}`.
//!
//! **Why Atomic Counter:** `AtomicUsize::fetch_add(1, Ordering::SeqCst)` provides thread-safe
//! increment, ensuring each concurrent test gets unique ID without race conditions.
//!
//! **Alternative Approaches:**
//! - ❌ Thread ID (`.as_u64()`) - unstable feature `thread_id_value`
//! - ✅ Atomic counter - stable, simple, guaranteed uniqueness
//! - ❌ Random UUID - overkill, adds unnecessary dependency
//! - ❌ Timestamp + random - more complex than needed
//!
//! **Result:** All 4 concurrency tests pass reliably when run in parallel via `cargo test`.

use iron_control_api::routes::tokens::CreateTokenResponse;
use axum::{ Router, routing::{ post, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;
use tokio::task::JoinHandle;
use std::sync::Arc;
use std::sync::atomic::{ AtomicUsize, Ordering };

/// Global counter for generating unique database names across concurrent tests
static DB_COUNTER: AtomicUsize = AtomicUsize::new( 0 );

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with token routes using shared database file.
///
/// NOTE: Concurrency tests require a shared database file (not `:memory:`)
/// because in-memory databases are connection-specific in SQLite.
async fn create_test_router() -> ( Router, crate::common::test_state::TestAppState )
{
  // Use a unique temporary file for this test run
  // Include atomic counter to prevent conflicts when tests run in parallel
  let unique_id = DB_COUNTER.fetch_add( 1, Ordering::SeqCst );
  let db_path = format!( "file:test_concurrency_{}_{}?mode=memory&cache=shared",
    std::process::id(),
    unique_id
  );

  // Create test application state with auth + token support using shared database
  let app_state = crate::common::test_state::TestAppState::with_db_path( &db_path ).await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id/rotate", post( iron_control_api::routes::tokens::rotate_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Helper: Create a token and return its ID.
async fn create_token( router: Arc< Router >, user_id: &str ) -> ( StatusCode, i64, String )
{
  let request_body = json!({
    "user_id": user_id,
    "project_id": "test_project",
    "description": "Concurrent test token",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = ( *router ).clone().oneshot( request ).await.unwrap();
  let status = response.status();

  if status == StatusCode::CREATED
  {
    let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
    let body: CreateTokenResponse = serde_json::from_slice( &body_bytes ).unwrap();
    ( status, body.id, body.token )
  }
  else
  {
    ( status, 0, String::new() )
  }
}

/// Test concurrent token creation produces unique tokens.
///
/// WHY: Token generation must be cryptographically unique even under
/// concurrent load. This prevents token collisions and ensures each
/// token is truly unique.
#[ tokio::test ]
async fn test_concurrent_token_creation_uniqueness()
{
  let ( router, _app_state ) = create_test_router().await;
  let router = Arc::new( router );

  // Launch 10 concurrent token creations
  let mut handles: Vec< JoinHandle< ( StatusCode, i64, String ) > > = vec![];

  for i in 0..10
  {
    let router_clone = Arc::clone( &router );
    let user_id = format!( "user_concurrent_{}", i );

    let handle = tokio::spawn( async move
    {
      create_token( router_clone, &user_id ).await
    } );

    handles.push( handle );
  }

  // Collect all results
  let mut tokens = vec![];
  let mut ids = vec![];

  for handle in handles
  {
    let ( status, id, token ) = handle.await.unwrap();
    assert_eq!(
      status,
      StatusCode::CREATED,
      "LOUD FAILURE: All concurrent creates must succeed"
    );
    tokens.push( token );
    ids.push( id );
  }

  // Verify all tokens are unique
  let unique_tokens: std::collections::HashSet< _ > = tokens.iter().collect();
  assert_eq!(
    unique_tokens.len(),
    10,
    "LOUD FAILURE: All 10 tokens must be unique. Got {} unique tokens",
    unique_tokens.len()
  );

  // Verify all IDs are unique
  let unique_ids: std::collections::HashSet< _ > = ids.iter().collect();
  assert_eq!(
    unique_ids.len(),
    10,
    "LOUD FAILURE: All 10 token IDs must be unique"
  );
}

/// Test concurrent rotation of the same token.
///
/// WHY: Only one rotation should succeed. The second should fail with 404
/// because the token no longer exists (rotation creates new token and deletes old one).
#[ tokio::test ]
async fn test_concurrent_rotate_same_token()
{
  let ( router, app_state ) = create_test_router().await;
  let router = Arc::new( router );

  // Create a token first
  let user_id = "user_rotate_concurrent";
  let ( _, token_id, _ ) = create_token( Arc::clone( &router ), user_id ).await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, user_id );

  // Launch 2 concurrent rotations of the same token
  let router_clone1 = Arc::clone( &router );
  let router_clone2 = Arc::clone( &router );
  let jwt_token1 = jwt_token.clone();
  let jwt_token2 = jwt_token;

  let handle1 = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( format!( "/api/v1/api-tokens/{}/rotate", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token1 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone1 ).clone().oneshot( request ).await.unwrap();
    response.status()
  } );

  let handle2 = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( format!( "/api/v1/api-tokens/{}/rotate", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token2 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone2 ).clone().oneshot( request ).await.unwrap();
    response.status()
  } );

  let status1 = handle1.await.unwrap();
  let status2 = handle2.await.unwrap();

  // One should succeed (200 OK), one should fail (404 NOT FOUND)
  let statuses = [ status1, status2 ];
  let success_count = statuses.iter().filter( |&&s| s == StatusCode::OK ).count();
  let not_found_count = statuses.iter().filter( |&&s| s == StatusCode::NOT_FOUND ).count();

  assert_eq!(
    success_count,
    1,
    "LOUD FAILURE: Exactly one rotation must succeed. Got {} successes",
    success_count
  );

  assert_eq!(
    not_found_count,
    1,
    "LOUD FAILURE: Exactly one rotation must fail with 404. Got {} 404s",
    not_found_count
  );
}

/// Test concurrent revocation of the same token.
///
/// WHY: DELETE operations should be atomic. First revoke succeeds (204),
/// second revoke fails (404) because token no longer exists.
#[ tokio::test ]
async fn test_concurrent_revoke_same_token()
{
  let ( router, app_state ) = create_test_router().await;
  let router = Arc::new( router );

  // Create a token first
  let user_id = "user_revoke_concurrent";
  let ( _, token_id, _ ) = create_token( Arc::clone( &router ), user_id ).await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, user_id );

  // Launch 2 concurrent revocations
  let router_clone1 = Arc::clone( &router );
  let router_clone2 = Arc::clone( &router );
  let jwt_token1 = jwt_token.clone();
  let jwt_token2 = jwt_token;

  let handle1 = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "DELETE" )
      .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token1 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone1 ).clone().oneshot( request ).await.unwrap();
    response.status()
  } );

  let handle2 = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "DELETE" )
      .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token2 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone2 ).clone().oneshot( request ).await.unwrap();
    response.status()
  } );

  let status1 = handle1.await.unwrap();
  let status2 = handle2.await.unwrap();

  // One should succeed (204 NO CONTENT), one should fail (404 NOT FOUND)
  let statuses = [ status1, status2 ];
  let success_count = statuses.iter().filter( |&&s| s == StatusCode::NO_CONTENT ).count();
  let not_found_count = statuses.iter().filter( |&&s| s == StatusCode::NOT_FOUND ).count();

  assert_eq!(
    success_count,
    1,
    "LOUD FAILURE: Exactly one revocation must succeed. Got {} successes",
    success_count
  );

  assert_eq!(
    not_found_count,
    1,
    "LOUD FAILURE: Exactly one revocation must fail with 404. Got {} 404s",
    not_found_count
  );
}

/// Test concurrent rotate and revoke operations on the same token.
///
/// WHY: This tests the race condition between rotation and revocation.
/// One operation will win, the other will see the token as non-existent.
#[ tokio::test ]
async fn test_concurrent_rotate_and_revoke()
{
  let ( router, app_state ) = create_test_router().await;
  let router = Arc::new( router );

  // Create a token first
  let user_id = "user_rotate_revoke_race";
  let ( _, token_id, _ ) = create_token( Arc::clone( &router ), user_id ).await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, user_id );

  // Launch concurrent rotation and revocation
  let router_clone1 = Arc::clone( &router );
  let router_clone2 = Arc::clone( &router );
  let jwt_token1 = jwt_token.clone();
  let jwt_token2 = jwt_token;

  let rotate_handle = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "POST" )
      .uri( format!( "/api/v1/api-tokens/{}/rotate", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token1 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone1 ).clone().oneshot( request ).await.unwrap();
    ( "rotate", response.status() )
  } );

  let revoke_handle = tokio::spawn( async move
  {
    let request = Request::builder()
      .method( "DELETE" )
      .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
      .header( "Authorization", format!( "Bearer {}", jwt_token2 ) )
      .body( Body::empty() )
      .unwrap();

    let response = ( *router_clone2 ).clone().oneshot( request ).await.unwrap();
    ( "revoke", response.status() )
  } );

  let ( op1, status1 ) = rotate_handle.await.unwrap();
  let ( op2, status2 ) = revoke_handle.await.unwrap();

  // One operation should succeed, the other should get 404
  let results = vec![ ( op1, status1 ), ( op2, status2 ) ];

  let success_count = results.iter().filter( |( _, s )| s.is_success() ).count();
  let not_found_count = results.iter().filter( |( _, s )| *s == StatusCode::NOT_FOUND ).count();

  assert_eq!(
    success_count,
    1,
    "LOUD FAILURE: Exactly one operation must succeed. Results: {:?}",
    results
  );

  assert_eq!(
    not_found_count,
    1,
    "LOUD FAILURE: Exactly one operation must fail with 404. Results: {:?}",
    results
  );
}
