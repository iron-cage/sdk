//! Audit logging tests for token lifecycle events
//!
//! Tests that token creation, revocation, and other lifecycle events are logged
//! to the audit_log table for compliance and security monitoring.
//!
//! ## Test Matrix
//!
//! | Test Case | Operation | Expected Audit Log | Status |
//! |-----------|-----------|-------------------|--------|
//! | `test_create_token_logs_creation` | POST /api/v1/api-tokens | audit_log entry with action='created' | ❌ |
//! | `test_revoke_token_logs_revocation` | DELETE /api/v1/api-tokens/:id | audit_log entry with action='revoked' | ❌ |
//! | `test_audit_log_excludes_token_value` | POST /api/v1/api-tokens | changes field does NOT contain plaintext token | ❌ |
//!
//! ## Audit Log Schema (migration 001)
//!
//! ```sql
//! CREATE TABLE audit_log (
//!   id INTEGER PRIMARY KEY AUTOINCREMENT,
//!   entity_type TEXT NOT NULL,  -- "token", "limit", "usage"
//!   entity_id INTEGER NOT NULL,  -- token_id
//!   action TEXT NOT NULL,  -- "created", "updated", "deleted", "activated", "deactivated"
//!   actor_user_id TEXT NOT NULL,  -- user who performed action
//!   changes TEXT,  -- JSON object with before/after values (excludes plaintext token)
//!   logged_at INTEGER NOT NULL,  -- milliseconds since epoch
//!   ip_address TEXT,  -- optional
//!   user_agent TEXT  -- optional
//! );
//! ```

use iron_control_api::routes::tokens::CreateTokenResponse;
use axum::{ Router, routing::{ post, delete }, http::{ Request, StatusCode } };
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;

/// Helper: Generate JWT token for a given user_id
fn generate_jwt_for_user( app_state: &crate::common::test_state::TestAppState, user_id: &str ) -> String
{
  app_state.auth.jwt_secret
    .generate_access_token( user_id, &format!( "{}@test.com", user_id ), "user", &format!( "token_{}", user_id ) )
    .expect( "LOUD FAILURE: Failed to generate JWT token" )
}

/// Create test router with token routes
async fn create_test_router() -> ( Router, crate::common::test_state::TestAppState )
{
  let app_state = crate::common::test_state::TestAppState::new().await;

  let router = Router::new()
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .with_state( app_state.clone() );

  ( router, app_state )
}

/// Test that creating a token logs the creation to audit_log
///
/// WHY: Protocol 014 requires audit logging for all token lifecycle events for
/// compliance and security monitoring. Token creation must be logged with
/// action='created', entity_type='token', and the user who created it.
///
/// CRITICAL: Audit log must NOT contain plaintext token value (security requirement).
#[ tokio::test ]
async fn test_create_token_logs_creation()
{
  let ( router, app_state ) = create_test_router().await;
  let user_id = "user_audit_test";

  // Generate JWT token for authenticated request
  let jwt_token = generate_jwt_for_user( &app_state, user_id );

  let request_body = json!({
    "name": "Test Audit Token",
    "description": "Token to test audit logging",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/api-tokens" )
    .header( "content-type", "application/json" )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();
  let status = response.status();

  assert_eq!(
    status,
    StatusCode::CREATED,
    "LOUD FAILURE: Token creation must succeed for audit log test"
  );

  // Extract created token ID from response
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let body: CreateTokenResponse = serde_json::from_slice( &body_bytes ).unwrap();
  let token_id = body.id;

  // Verify audit log entry was created
  let audit_log_entry: Option< ( i64, String, i64, String, String, String, i64 ) > = sqlx::query_as(
    "SELECT id, entity_type, entity_id, action, actor_user_id, changes, logged_at
     FROM audit_log
     WHERE entity_type = 'token' AND entity_id = ? AND action = 'created'"
  )
  .bind( token_id )
  .fetch_optional( app_state.tokens.storage.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to query audit_log table" );

  assert!(
    audit_log_entry.is_some(),
    "LOUD FAILURE: Audit log entry must exist for token creation (token_id={})",
    token_id
  );

  let ( _audit_id, entity_type, entity_id, action, actor_user_id, changes, logged_at ) = audit_log_entry.unwrap();

  // Verify audit log fields
  assert_eq!(
    entity_type, "token",
    "LOUD FAILURE: entity_type must be 'token'"
  );

  assert_eq!(
    entity_id, token_id,
    "LOUD FAILURE: entity_id must match created token ID"
  );

  assert_eq!(
    action, "created",
    "LOUD FAILURE: action must be 'created' for token creation"
  );

  assert_eq!(
    actor_user_id, user_id,
    "LOUD FAILURE: actor_user_id must match authenticated user"
  );

  // Verify timestamp is recent (within last 60 seconds)
  let now_millis = chrono::Utc::now().timestamp_millis();
  assert!(
    logged_at > now_millis - 60_000 && logged_at <= now_millis,
    "LOUD FAILURE: logged_at timestamp must be recent (got {}, expected near {})",
    logged_at,
    now_millis
  );

  // CRITICAL: Verify changes JSON does NOT contain plaintext token
  // Security requirement: Never log plaintext tokens
  assert!(
    !changes.contains( &body.token ),
    "LOUD FAILURE: Audit log must NOT contain plaintext token value (security violation)"
  );

  // Verify changes JSON contains token metadata (but not plaintext value)
  let changes_json: serde_json::Value = serde_json::from_str( &changes )
    .expect( "LOUD FAILURE: changes field must be valid JSON" );

  assert!(
    changes_json.get( "name" ).is_some(),
    "LOUD FAILURE: Audit log changes must include token name"
  );
}

/// Test that revoking a token logs the revocation to audit_log
///
/// WHY: Protocol 014 requires audit logging for all token lifecycle events.
/// Token revocation must be logged with action='revoked', entity_type='token',
/// and the user who revoked it.
#[ tokio::test ]
async fn test_revoke_token_logs_revocation()
{
  let ( router, app_state ) = create_test_router().await;
  let user_id = "user_revoke_audit";

  // Generate JWT token for authenticated request
  let jwt_token = generate_jwt_for_user( &app_state, user_id );

  // First, create a token to revoke
  let request_body = json!({
    "name": "Token to Revoke",
    "description": "Test revocation audit logging",
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

  let body_bytes = axum::body::to_bytes( create_response.into_body(), usize::MAX ).await.unwrap();
  let body: CreateTokenResponse = serde_json::from_slice( &body_bytes ).unwrap();
  let token_id = body.id;

  // Now revoke the token
  let revoke_request = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/v1/api-tokens/{}", token_id ) )
    .header( "authorization", format!( "Bearer {}", jwt_token ) )
    .body( Body::empty() )
    .unwrap();

  let revoke_response = router.oneshot( revoke_request ).await.unwrap();

  assert_eq!(
    revoke_response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Token revocation must succeed for audit log test"
  );

  // Verify audit log entry was created for revocation
  let audit_log_entry: Option< ( String, i64, String, String ) > = sqlx::query_as(
    "SELECT entity_type, entity_id, action, actor_user_id
     FROM audit_log
     WHERE entity_type = 'token' AND entity_id = ? AND action = 'revoked'"
  )
  .bind( token_id )
  .fetch_optional( app_state.tokens.storage.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to query audit_log table" );

  assert!(
    audit_log_entry.is_some(),
    "LOUD FAILURE: Audit log entry must exist for token revocation (token_id={})",
    token_id
  );

  let ( entity_type, entity_id, action, actor_user_id ) = audit_log_entry.unwrap();

  assert_eq!(
    entity_type, "token",
    "LOUD FAILURE: entity_type must be 'token'"
  );

  assert_eq!(
    entity_id, token_id,
    "LOUD FAILURE: entity_id must match revoked token ID"
  );

  assert_eq!(
    action, "revoked",
    "LOUD FAILURE: action must be 'revoked' for token revocation"
  );

  assert_eq!(
    actor_user_id, user_id,
    "LOUD FAILURE: actor_user_id must match authenticated user"
  );
}
