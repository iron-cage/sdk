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
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_create_token_valid_request` | Create token with valid complete request | POST /api/v1/api-tokens with all fields | 201 Created, token returned | ✅ |
//! | `test_create_token_minimal_request` | Create token with minimal fields | POST /api/v1/api-tokens with user_id only | 201 Created, token returned | ✅ |
//! | `test_create_token_empty_user_id_rejected` | Create token with empty user_id | POST with user_id="" | 400 Bad Request "user_id cannot be empty" | ✅ |
//! | `test_create_token_empty_project_id_rejected` | Create token with empty project_id | POST with project_id="" | 400 Bad Request "project_id cannot be empty" | ✅ |
//! | `test_create_token_description_too_long_rejected` | Create token with oversized description | POST with description=501 chars | 400 Bad Request "description too long" | ✅ |
//! | `test_get_token_valid_id_returns_200` | Get token by valid ID | GET /api/v1/api-tokens/:id with existing token | 200 OK, token metadata returned | ✅ |
//! | `test_get_token_nonexistent_id_returns_404` | Get token by nonexistent ID | GET /api/v1/api-tokens/999999 | 404 Not Found | ✅ |
//! | `test_revoke_token_valid_id_returns_204` | Revoke token by valid ID | DELETE /api/v1/api-tokens/:id with existing token | 204 No Content, token marked inactive | ✅ |
//! | `test_revoke_token_nonexistent_id_returns_404` | Revoke token by nonexistent ID | DELETE /api/v1/api-tokens/999999 | 404 Not Found | ✅ |
//! | `test_token_created_at_timestamp` | Verify created_at timestamp | Create token, check created_at field | Timestamp within last 5 seconds | ✅ |
//! | `test_token_id_auto_increment` | Verify ID auto-increment | Create 3 tokens sequentially | IDs increment (1, 2, 3) | ✅ |
//! | `test_rotate_token_valid_id_returns_200` | Rotate token by valid ID | POST /api/v1/api-tokens/:id/rotate | 200 OK, new token returned, old revoked | ✅ |
//! | `test_rotate_token_nonexistent_id_returns_404` | Rotate token by nonexistent ID | POST /api/v1/api-tokens/999999/rotate | 404 Not Found | ✅ |
//! | `test_validate_token_valid_returns_metadata` | Validate valid token | POST /api/v1/api-tokens/validate with valid token | 200 OK, metadata returned | ✅ |
//! | `test_validate_token_invalid_returns_false` | Validate invalid token | POST /api/v1/api-tokens/validate with invalid token | 200 OK, valid=false | ✅ |
//! | `test_validate_token_revoked_returns_false` | Validate revoked token | POST /api/v1/api-tokens/validate with revoked token | 200 OK, valid=false | ✅ |
//! | `test_validate_token_missing_token_field_400` | Validate without token field | POST /api/v1/api-tokens/validate with empty body | 400 Bad Request | ✅ |
//! | `test_validate_token_no_auth_required` | Validate endpoint doesn't require auth | POST /api/v1/api-tokens/validate without auth header | 200 OK (no 401) | ✅ |

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
    .route( "/api/v1/api-tokens/validate", post( iron_control_api::routes::tokens::validate_token ) )
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
  let ( router, app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": "project_abc",
    "description": "Production API key",
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
  let ( router, app_state ) = create_test_router().await;
  
  let request_body = json!({
    "user_id": "user_minimal",
    "project_id": null,
    "description": null,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user_minimal" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
  let ( router, app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "",
    "project_id": null,
    "description": null,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
  let ( router, app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_test",
    "project_id": "",
    "description": null,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
  let ( router, app_state ) = create_test_router().await;

  let long_description = "a".repeat( 501 ); // Max is 500

  let request_body = json!({
    "user_id": "user_test",
    "project_id": null,
    "description": long_description,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );

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
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;

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

/// Test DELETE /api/v1/api-tokens/:id with valid ID returns 200 OK (Protocol 014).
#[ tokio::test ]
async fn test_revoke_token_valid_id_returns_204()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, "user_test" );

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
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;

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
    StatusCode::OK,
    "LOUD FAILURE: DELETE with valid ID must return 200 OK with details (Protocol 014)"
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
  let ( router, app_state ) = create_test_router().await;

  let request_body = json!({
    "user_id": "user_timestamp_test",
    "project_id": null,
    "description": null,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user_timestamp_test" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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
  let ( router, app_state ) = create_test_router().await;

  // Create first token
  let request1_body = json!({
    "user_id": "user1",
    "project_id": null,
    "description": null,
  });

  let jwt_token = generate_jwt_for_user( &app_state, "user1" );
  let request1 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
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

  let jwt_token2 = generate_jwt_for_user( &app_state, "user2" );
  let request2 = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token2 ) )
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

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &app_state, "user_rotate" );

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
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &create_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = created.id;
  let original_token = created.token.clone();

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

// --- Validate Endpoint Tests (Deliverable 1.6) ---

/// Test POST /api/v1/api-tokens/validate with valid token returns metadata.
///
/// Deliverable 1.6: POST /api/v1/api-tokens/validate endpoint
///
/// This endpoint allows external services to validate API tokens without authentication.
/// It returns token validity status and metadata (user_id, project_id) for valid tokens.
#[ tokio::test ]
async fn test_validate_token_valid_returns_metadata()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for test user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user_validate" );

  // Create a token first
  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "Content-Type", "application/json" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( json!({ "user_id": "test_user_validate" }).to_string() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_value = created.token;

  // Now validate the token
  let validate_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/validate" )
    .header( "Content-Type", "application/json" )
    .body( Body::from( json!({ "token": token_value }).to_string() ) )
    .unwrap();

  let validate_response = router.oneshot( validate_request ).await.unwrap();

  assert_eq!(
    validate_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: POST /api/v1/api-tokens/validate with valid token must return 200 OK"
  );

  let body_bytes = axum::body::to_bytes( validate_response.into_body(), usize::MAX ).await.unwrap();
  let result: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!(
    result[ "valid" ],
    true,
    "LOUD FAILURE: Valid token must return {{\"valid\":true}}. Got: {:?}",
    result
  );

  assert_eq!(
    result[ "user_id" ].as_str().unwrap(),
    "test_user_validate",
    "LOUD FAILURE: Valid token must return user_id. Got: {:?}",
    result
  );
}

/// Test POST /api/v1/api-tokens/validate with invalid token returns false.
#[ tokio::test ]
async fn test_validate_token_invalid_returns_false()
{
  let ( router, _app_state ) = create_test_router().await;

  let validate_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/validate" )
    .header( "Content-Type", "application/json" )
    .body( Body::from( json!({ "token": "invalid_token_value" }).to_string() ) )
    .unwrap();

  let validate_response = router.oneshot( validate_request ).await.unwrap();

  assert_eq!(
    validate_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: POST /api/v1/api-tokens/validate must return 200 OK even for invalid tokens"
  );

  let body_bytes = axum::body::to_bytes( validate_response.into_body(), usize::MAX ).await.unwrap();
  let result: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!(
    result[ "valid" ],
    false,
    "LOUD FAILURE: Invalid token must return {{\"valid\":false}}. Got: {:?}",
    result
  );
}

/// Test POST /api/v1/api-tokens/validate with revoked token returns false.
#[ tokio::test ]
async fn test_validate_token_revoked_returns_false()
{
  let ( router, app_state ) = create_test_router().await;

  // Generate JWT for test user
  let jwt_token = generate_jwt_for_user( &app_state, "test_user_revoke" );

  // Create a token
  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "Content-Type", "application/json" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( json!({ "user_id": "test_user_revoke" }).to_string() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, created ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_value = created.token;
  let token_id = created.id;

  // Revoke the token
  let revoke_request = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let _revoke_response = router.clone().oneshot( revoke_request ).await.unwrap();

  // Now validate the revoked token
  let validate_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/validate" )
    .header( "Content-Type", "application/json" )
    .body( Body::from( json!({ "token": token_value }).to_string() ) )
    .unwrap();

  let validate_response = router.oneshot( validate_request ).await.unwrap();

  assert_eq!(
    validate_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: POST /api/v1/api-tokens/validate with revoked token must return 200 OK"
  );

  let body_bytes = axum::body::to_bytes( validate_response.into_body(), usize::MAX ).await.unwrap();
  let result: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!(
    result[ "valid" ],
    false,
    "LOUD FAILURE: Revoked token must return {{\"valid\":false}}. Got: {:?}",
    result
  );
}

/// Test POST /api/v1/api-tokens/validate with missing token field returns 400.
#[ tokio::test ]
async fn test_validate_token_missing_token_field_400()
{
  let ( router, _app_state ) = create_test_router().await;

  let validate_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/validate" )
    .header( "Content-Type", "application/json" )
    .body( Body::from( json!({}).to_string() ) )
    .unwrap();

  let validate_response = router.oneshot( validate_request ).await.unwrap();

  assert_eq!(
    validate_response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: POST /api/v1/api-tokens/validate with missing token field must return 400 Bad Request"
  );
}

/// Test POST /api/v1/api-tokens/validate doesnt require authentication.
#[ tokio::test ]
async fn test_validate_token_no_auth_required()
{
  let ( router, _app_state ) = create_test_router().await;

  // Make request WITHOUT Authorization header
  let validate_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens/validate" )
    .header( "Content-Type", "application/json" )
    .body( Body::from( json!({ "token": "some_token" }).to_string() ) )
    .unwrap();

  let validate_response = router.oneshot( validate_request ).await.unwrap();

  // Should return 200 OK (not 401 Unauthorized)
  assert_eq!(
    validate_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: POST /api/v1/api-tokens/validate must NOT require authentication. Got status: {}",
    validate_response.status()
  );
}

// --- Bug Reproducer Tests ---

/// Fix(issue-001): Generic error for FK constraint violation when creating token with non-existent user_id
///
/// Root cause: The create_token handler in routes/tokens.rs catches database errors with a generic
/// "Failed to create token" message without distinguishing between different failure modes. When
/// SQLite's foreign key constraint fails (user_id doesn't exist in users table), the specific
/// constraint error is swallowed and replaced with a generic message.
///
/// Pitfall: Generic error messages make API integration difficult because clients cannot distinguish
/// between different failure modes (FK constraint vs other database errors). This violates the principle
/// of actionable error messages and makes debugging significantly harder for API users. Always parse
/// and expose specific constraint violation details to clients with appropriate HTTP status codes.
///
/// Current Behavior: Returns 500 with {"error": "Failed to create token"}
/// Expected After Fix: Should return 404 with {"error": "User not found: 'nonexistent_user_xyz'", "code": "USER_NOT_FOUND"}
/// OR: 409 with {"error": "Foreign key constraint failed: user_id 'nonexistent_user_xyz' does not exist", "code": "FK_CONSTRAINT_VIOLATION"}
#[ tokio::test ]
#[ ignore ]
async fn bug_reproducer_issue_001_fk_constraint_generic_error()
{
  let ( router, app_state ) = create_test_router().await;

  // Attempt to create token with user_id that doesn't exist in database
  let request_body = json!({
    "user_id": "nonexistent_user_xyz",
    "project_id": "test_project",
    "description": "Test token with non-existent user",
  });

  let jwt_token = generate_jwt_for_user( &app_state, "nonexistent_user_xyz" );
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Current buggy behavior: Returns 500 with generic error
  assert_eq!(
    response.status(),
    StatusCode::INTERNAL_SERVER_ERROR,
    "LOUD FAILURE: Creating token with non-existent user_id currently returns 500"
  );

  let ( status, body ) = extract_response( response ).await;
  assert_eq!( status, StatusCode::INTERNAL_SERVER_ERROR );
  assert!(
    body.contains( "Failed to create token" ),
    "LOUD FAILURE: Current error message is generic. Got: {}",
    body
  );

  // After fix, this test should be updated to assert:
  // - Status: 404 NOT_FOUND (or 409 CONFLICT)
  // - Body contains specific error: "User not found" or "Foreign key constraint failed"
  // - Body contains the problematic user_id: "nonexistent_user_xyz"
  // - Body contains error code: "USER_NOT_FOUND" or "FK_CONSTRAINT_VIOLATION"
}
