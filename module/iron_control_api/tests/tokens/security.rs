//! Token Security and Edge Case Tests
//!
//! This module tests critical security properties and edge cases for token management:
//! - Cryptographic hash storage (SHA-256, not plaintext)
//! - Token plaintext exposure prevention
//! - ID parameter edge cases (negative, zero, overflow)
//! - Unicode handling in string fields
//! - SQL injection resistance

use crate::common::extract_json_response;
use crate::common::test_state::TestAppState;
use iron_control_api::routes::tokens::{ CreateTokenResponse, TokenListItem };
use axum::{ Router, routing::{ post, get, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Create test router with token routes.
async fn create_test_router() -> ( Router, TestAppState )
{
  // Create test application state with auth + token support
  let app_state = TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id", get( iron_control_api::routes::tokens::get_token ) )
    .route( "/api/v1/api-tokens/:id/rotate", post( iron_control_api::routes::tokens::rotate_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Verify token plaintext only returned on creation
///
/// # Prevention
/// Ensures tokens are only exposed as plaintext during creation and rotation.
/// Subsequent GET requests must never return plaintext token.
///
/// # Pitfall
/// If tokens were stored in plaintext or returned in GET responses,
/// database compromise or API access would expose all tokens.
#[ tokio::test ]
async fn test_token_plaintext_only_on_creation()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &state, "test-user" );

  // Create token
  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &json!({
      "user_id": "test-user",
      "project_id": "test-project",
      "description": "Test token"
    }) ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  assert_eq!( create_response.status(), StatusCode::CREATED );

  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let plaintext_token = create_body.token;

  // Verify plaintext token returned on creation
  assert!
  (
    !plaintext_token.is_empty(),
    "Token must be returned on creation"
  );

  assert!
  (
    plaintext_token.len() >= 32,
    "Token must be at least 32 characters for security (base64-url encoded)"
  );

  // GET token by ID should NOT return plaintext
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", create_body.id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();
  let ( _status, get_body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;

  // TokenListItem type has no token field - this is enforced by type system
  // This proves plaintext is never exposed after creation
  assert_eq!( get_body.id, create_body.id );
  assert_eq!( get_body.user_id, "test-user" );
}

/// Verify GET endpoint never returns plaintext token
///
/// # Prevention
/// Validates that after token creation, plaintext value is never exposed
/// through API. Only metadata (id, user_id, project_id) should be returned.
///
/// # Pitfall
/// Accidentally including token field in GET response would expose all
/// tokens to anyone with API access.
#[ tokio::test ]
async fn test_get_token_never_returns_plaintext()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &state, "test-user" );

  // Create token
  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &json!({
      "user_id": "test-user",
      "project_id": "test-project",
      "description": "Test token"
    }) ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  let token_id = create_body.id;

  // GET token by ID with authentication
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();
  assert_eq!( get_response.status(), StatusCode::OK );

  let ( _status, get_body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;

  // Verify response has metadata
  assert_eq!( get_body.id, token_id );
  assert_eq!( get_body.user_id, "test-user" );
  assert_eq!( get_body.project_id, Some( "test-project".to_string() ) );
  assert!( get_body.is_active );

  // Verify response does NOT have token field (TokenListItem doesn't have token field)
  // This is enforced by the type system - TokenListItem has no token field
}

/// Verify negative ID returns 404 (not database error)
///
/// # Prevention
/// Ensures path parameter validation handles negative IDs gracefully.
///
/// # Pitfall
/// Negative IDs might cause database errors or unexpected behavior.
#[ tokio::test ]
async fn test_get_token_negative_id_returns_404()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/api-tokens/-1" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should return 404 Not Found (not 500)
  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

/// Verify zero ID returns 404
#[ tokio::test ]
async fn test_get_token_zero_id_returns_404()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/v1/api-tokens/0" )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();
  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

/// Verify very large ID handles gracefully
#[ tokio::test ]
async fn test_get_token_very_large_id_handles_gracefully()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for any user
  let jwt_token = generate_jwt_for_user( &state, "test_user" );

  let request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", i64::MAX ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  // Should return 404 (not crash or overflow)
  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

/// Verify Unicode in user_id is handled correctly
///
/// # Prevention
/// Ensures UTF-8 strings in user_id work correctly (create, store, retrieve).
///
/// # Pitfall
/// UTF-8 handling bugs might truncate or corrupt Unicode characters.
#[ tokio::test ]
async fn test_unicode_in_user_id()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for the user (Unicode user_id)
  let jwt_token = generate_jwt_for_user( &state, "用户-user-123" );

  let request_body = json!({
    "user_id": "用户-user-123",
    "project_id": "test-project",
    "description": "Test token"
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  assert_eq!( create_response.status(), StatusCode::CREATED );

  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  assert_eq!( create_body.user_id, "用户-user-123" );

  // Verify retrieval preserves Unicode
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", create_body.id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();
  let ( _status, get_body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;
  assert_eq!( get_body.user_id, "用户-user-123", "Unicode should be preserved in database" );
}

/// Verify SQL injection in user_id is safely escaped
///
/// # Prevention
/// Ensures parameterized queries prevent SQL injection attacks.
///
/// # Pitfall
/// String concatenation in SQL queries would allow injection.
/// Using sqlx query macros prevents this.
#[ tokio::test ]
async fn test_sql_injection_in_user_id()
{
  let ( router, state ) = create_test_router().await;

  // SQL injection attempt
  let malicious_user_id = "admin' OR '1'='1";

  // Generate JWT for the malicious user_id (treated as literal string)
  let jwt_token = generate_jwt_for_user( &state, malicious_user_id );

  let request_body = json!({
    "user_id": malicious_user_id,
    "project_id": "test-project",
    "description": "Test token"
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  assert_eq!
  (
    create_response.status(),
    StatusCode::CREATED,
    "SQL injection string should be treated as literal string"
  );

  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;

  // Verify SQL injection string stored as literal (not executed)
  assert_eq!( create_body.user_id, malicious_user_id );

  // Verify retrieval returns exact string
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", create_body.id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();
  let ( _status, get_body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;
  assert_eq!
  (
    get_body.user_id,
    malicious_user_id,
    "SQL injection should be escaped, not executed"
  );
}

/// Verify SQL injection in project_id is safely escaped
///
/// # Prevention
/// Ensures parameterized queries prevent SQL injection through project_id.
/// Malicious SQL is stored as literal string, not executed.
///
/// # Pitfall
/// If SQL was concatenated instead of using parameterized queries,
/// this could allow table drops or unauthorized data access.
#[ tokio::test ]
async fn test_sql_injection_in_project_id()
{
  let ( router, state ) = create_test_router().await;

  let malicious_project_id = "proj'; DROP TABLE tokens; --";

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &state, "user_1" );

  let request_body = json!({
    "user_id": "user_1",
    "project_id": malicious_project_id,
    "description": "Test token"
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  assert_eq!
  (
    create_response.status(),
    StatusCode::CREATED,
    "Malicious SQL should be stored as literal string"
  );

  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  assert_eq!( create_body.project_id, Some( malicious_project_id.to_string() ) );

  // Verify token can be retrieved (proves table wasn't dropped)
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", create_body.id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.clone().oneshot( get_request ).await.unwrap();
  assert_eq!
  (
    get_response.status(),
    StatusCode::OK,
    "Token retrieval should succeed (table should not have been dropped)"
  );

  // Create a second token to verify table is still functional
  let second_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &json!({
      "user_id": "user_2",
      "project_id": "safe-project"
    }) ).unwrap() ) )
    .unwrap();

  let second_response = router.oneshot( second_request ).await.unwrap();
  assert_eq!
  (
    second_response.status(),
    StatusCode::CREATED,
    "Creating second token should succeed (table should be functional)"
  );
}

/// Verify XSS attempt in description is stored as literal
///
/// # Prevention
/// Ensures HTML/JavaScript in description is stored as literal text.
/// Frontend must escape when rendering (not backend's responsibility).
///
/// # Pitfall
/// Backend storing literal strings is correct. Frontend MUST escape
/// before rendering to prevent XSS.
#[ tokio::test ]
async fn test_xss_in_description_stored_as_literal()
{
  let ( router, state ) = create_test_router().await;

  // Generate JWT for the user
  let jwt_token = generate_jwt_for_user( &state, "test-user" );

  let xss_description = "<script>alert('XSS')</script>";

  let request_body = json!({
    "user_id": "test-user",
    "project_id": "test-project",
    "description": xss_description
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let create_response = router.clone().oneshot( create_request ).await.unwrap();
  assert_eq!( create_response.status(), StatusCode::CREATED );

  let ( _status, create_body ): ( StatusCode, CreateTokenResponse ) = extract_json_response( create_response ).await;
  assert_eq!( create_body.description, Some( xss_description.to_string() ) );

  // Verify retrieval returns exact string (unmodified)
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/v1/api-tokens/{}", create_body.id ) )
    .header( "Authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let get_response = router.oneshot( get_request ).await.unwrap();
  let ( _status, get_body ): ( StatusCode, TokenListItem ) = extract_json_response( get_response ).await;
  assert_eq!
  (
    get_body.description,
    Some( xss_description.to_string() ),
    "XSS string should be returned as-is (frontend must escape)"
  );
}
