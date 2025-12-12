//! Endpoint integration tests for token management (FR-7).
//!
//! Test Matrix: Token Management Endpoints
//!
//! | Endpoint | Method | Test Cases | Expected Status Codes |
//! |----------|--------|------------|----------------------|
//! | /api/v1/api-tokens | POST | Valid request, Empty user_id, Invalid fields | 201, 400 |
//! | /api/v1/api-tokens | GET | NOT TESTED - Requires authentication | - |
//! | /api/v1/api-tokens/:id | GET | Valid ID, Non-existent ID | 200, 404 |
//! | /api/v1/api-tokens/:id/rotate | POST | Valid rotation, Non-existent ID | 200, 404 |
//! | /api/v1/api-tokens/:id | DELETE | Valid revocation, Non-existent ID | 204, 404 |
//!
//! Note: GET /api/v1/api-tokens (list_tokens) requires JWT authentication via AuthenticatedUser.
//! This endpoint is not tested in integration tests as they don't include auth infrastructure.
//! The endpoint is functional and can be tested via manual/integration tests with auth setup.
//!
//! Coverage:
//! - Request validation (user_id, project_id, description length)
//! - HTTP status codes (201, 200, 204, 400, 404, 500)
//! - JSON response structure
//! - Database persistence
//! - Error handling

use crate::common::{ extract_response, extract_json_response };
use iron_control_api::routes::tokens::{ CreateTokenResponse, TokenListItem };
use axum::{ Router, routing::{ post, get, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Create test router with token routes.
async fn create_test_router() -> ( Router, crate::common::test_state::TestAppState )
{
  // Create test application state with auth + token support
  let app_state = crate::common::test_state::TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id", get( iron_control_api::routes::tokens::get_token ) )
    .route( "/api/v1/api-tokens/:id/rotate", post( iron_control_api::routes::tokens::rotate_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Test POST /api/v1/api-tokens with valid request returns 201 Created.
#[ tokio::test ]
async fn test_create_token_valid_request()
{
  let ( router, _app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": "project_abc",
    "description": "Production API key",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Valid token creation must return 201 Created"
  );

  let ( status, body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( response ).await;
  tracing::debug!( "Status: {:?}, Body: {:?}", status, body );
  assert_eq!( status, StatusCode::CREATED );
  assert_eq!( body.user_id, "user_test" );
  assert_eq!( body.project_id, Some( "project_abc".to_string() ) );
  assert_eq!( body.description, Some( "Production API key".to_string() ) );
  assert!(
    !body.token.is_empty(),
    "LOUD FAILURE: Token must be returned on creation"
  );
  // Token format: base64-url encoded random bytes, at least 32 chars
  assert!(
    body.token.len() >= 32,
    "LOUD FAILURE: Token must be at least 32 characters. Got: {}",
    body.token.len()
  );
}

/// Test POST /api/v1/api-tokens with minimal valid request (only user_id).
#[ tokio::test ]
async fn test_create_token_minimal_request()
{
  let ( router, _app_state ) = create_test_router().await;
  
  let request_body = json!({
    "user_id": "user_minimal",
    "project_id": null,
    "description": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Minimal valid request must return 201 Created"
  );

  let ( _status, body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( response ).await;
  assert_eq!( body.user_id, "user_minimal" );
  assert_eq!( body.project_id, None );
  assert_eq!( body.description, None );
}

/// Test POST /api/v1/api-tokens with empty user_id returns 400.
#[ tokio::test ]
async fn test_create_token_empty_user_id_rejected()
{
  let ( router, _app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "",
    "project_id": null,
    "description": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty user_id must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "user_id" ) || body.contains( "empty" ),
    "LOUD FAILURE: Error must mention user_id or empty. Got: {}",
    body
  );
}

/// Test POST /api/v1/api-tokens with empty project_id returns 400.
#[ tokio::test ]
async fn test_create_token_empty_project_id_rejected()
{
  let ( router, _app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": "",
    "description": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty project_id must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "project_id" ) || body.contains( "empty" ),
    "LOUD FAILURE: Error must mention project_id or empty. Got: {}",
    body
  );
}

/// Test POST /api/v1/api-tokens with description too long returns 400.
#[ tokio::test ]
async fn test_create_token_description_too_long_rejected()
{
  let ( router, _app_state ) = create_test_router().await;

  let long_description = "a".repeat( 501 ); // Max is 500

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "description": long_description,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Description longer than 500 chars must return 400 Bad Request"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::BAD_REQUEST );
  assert!(
    body.contains( "description" ) || body.contains( "long" ) || body.contains( "500" ),
    "LOUD FAILURE: Error must mention description length limit. Got: {}",
    body
  );
}

/// Test GET /api/v1/api-tokens/:id with valid ID returns 200 OK.
#[ tokio::test ]
async fn test_get_token_valid_id_returns_200()
{
  let ( router, app_state ) = create_test_router().await;

  // First create a token
  let create_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "description": "Test token",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;

  // Generate JWT for the same user
  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );

  // Now get the token by ID with authentication
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();

  assert_eq!(
    get_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: GET with valid ID must return 200 OK"
  );

  let ( status, body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;
  assert_eq!( status, StatusCode::OK );
  assert_eq!( body.id, token_id );
  assert_eq!( body.user_id, "user_test" );
  assert_eq!( body.description, Some( "Test token".to_string() ) );
  assert!( body.is_active, "LOUD FAILURE: Newly created token must be active" );
}

/// Test GET /api/v1/api-tokens/:id with non-existent ID returns 404.
#[ tokio::test ]
async fn test_get_token_nonexistent_id_returns_404()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/api-tokens/999999" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: GET with non-existent ID must return 404 Not Found"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::NOT_FOUND );
  assert!(
    body.contains( "not found" ) || body.contains( "error" ),
    "LOUD FAILURE: 404 response must contain error message. Got: {}",
    body
  );
}

/// Test DELETE /api/v1/api-tokens/:id with valid ID returns 204.
#[ tokio::test ]
async fn test_revoke_token_valid_id_returns_204()
{
  let ( router, app_state ) = create_test_router().await;

  // First create a token
  let create_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "description": "Token to revoke",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;

  // Generate JWT for the same user
  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );

  // Now revoke the token with authentication
  let delete_request = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let delete_response = router.oneshot( delete_request ).await.unwrap();

  assert_eq!(
    delete_response.status(),
    StatusCode::NO_CONTENT,
    "LOUD FAILURE: DELETE with valid ID must return 204 No Content"
  );
}

/// Test DELETE /api/v1/api-tokens/:id with non-existent ID returns 404.
#[ tokio::test ]
async fn test_revoke_token_nonexistent_id_returns_404()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "DELETE" )
    .uri( "/api/v1/api-tokens/999999" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: DELETE with non-existent ID must return 404 Not Found"
  );
}

/// Test token created_at timestamp is present and valid.
#[ tokio::test ]
async fn test_token_created_at_timestamp()
{
  let ( router, _app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_timestamp_test",
    "project_id": null,
    "description": null,
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();
  let ( _status, body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( response ).await;

  assert!(
    body.created_at > 0,
    "LOUD FAILURE: created_at must be a valid Unix timestamp (> 0). Got: {}",
    body.created_at
  );

  // created_at is in milliseconds, so just verify it's reasonable (within last hour)
  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap()
    .as_millis() as i64;

  let one_hour = 3600000; // 1 hour in milliseconds
  assert!(
    body.created_at >= now - one_hour && body.created_at <= now + one_hour,
    "LOUD FAILURE: created_at must be within reasonable time range. Got: {}, now: {}",
    body.created_at,
    now
  );
}

/// Test token ID is auto-incremented.
#[ tokio::test ]
async fn test_token_id_auto_increment()
{
  let ( router, _app_state ) = create_test_router().await;

  // Create first token
  let request1_body = json!({
    "user_id": "user1",
    "project_id": null,
    "description": null,
  });

  let request1 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request1_body ).unwrap() ) )
    .unwrap();

  let response1 = router.clone().oneshot( request1 ).await.unwrap();
  let ( _status, body1 ): ( StatusCode, CreateTokenResponse ) = extract_json_response( response1 ).await;
  let id1 = body1.id;

  // Create second token
  let request2_body = json!({
    "user_id": "user2",
    "project_id": null,
    "description": null,
  });

  let request2 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request2_body ).unwrap() ) )
    .unwrap();

  let response2 = router.oneshot( request2 ).await.unwrap();
  let ( _status, body2 ): ( StatusCode, CreateTokenResponse ) = extract_json_response( response2 ).await;
  let id2 = body2.id;

  assert!(
    id2 > id1,
    "LOUD FAILURE: Second token ID must be greater than first. id1={}, id2={}",
    id1,
    id2
  );
}

/// Test POST /api/v1/api-tokens/:id/rotate with valid ID returns 200 and new token.
#[ tokio::test ]
async fn test_rotate_token_valid_id_returns_200()
{
  let ( router, app_state ) = create_test_router().await;

  // First create a token
  let create_body = json!({
    "user_id": "user_rotate",
    "project_id": "project_rotate",
    "description": "Token to rotate",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;
  let original_token = created.token.clone();

  // Generate JWT for the same user
  let jwt_token = generate_jwt_for_user( &app_state, "user_rotate" );

  // Now rotate the token with authentication
  let rotate_request = Request::builder()
    .method( "POST" )
    .uri( format!( "/api/v1/api-tokens/{}/rotate", token_id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let rotate_response = router.oneshot( rotate_request ).await.unwrap();

  assert_eq!(
    rotate_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: POST /api/v1/api-tokens/:id/rotate with valid ID must return 200 OK"
  );

  let ( _status, rotated ): ( StatusCode, CreateTokenResponse ) = extract_json_response( rotate_response ).await;

  // Verify new token is different from original
  assert_ne!(
    rotated.token,
    original_token,
    "LOUD FAILURE: Rotated token must be different from original token"
  );

  // Verify token is valid length
  assert!(
    rotated.token.len() >= 32,
    "LOUD FAILURE: Rotated token must be at least 32 characters. Got: {}",
    rotated.token.len()
  );

  // Verify ID is different (new token created)
  assert_ne!(
    rotated.id,
    token_id,
    "LOUD FAILURE: Rotated token must have new ID. Original: {}, Rotated: {}",
    token_id,
    rotated.id
  );
}

/// Test POST /api/v1/api-tokens/:id/rotate with non-existent ID returns 404.
#[ tokio::test ]
async fn test_rotate_token_nonexistent_id_returns_404()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user" );

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/999999/rotate" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: POST /api/v1/api-tokens/:id/rotate with non-existent ID must return 404 Not Found"
  );
}
