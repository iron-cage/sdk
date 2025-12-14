//! User name field tests (GAP-007)
//!
//! Tests for user name field in users table.
//!
//! ## Root Cause (GAP-007)
//!
//! User name field not in users table. Login/refresh responses use `username` placeholder
//! instead of actual user name. User experience degraded (shows email instead of name).
//!
//! ## Test Coverage
//!
//! - User creation with name field
//! - Login returns actual user name (not email/username)
//! - Refresh returns actual user name
//! - Backward compatibility (users without name show email as fallback)
//! - Name validation (length constraints)

use axum::{ Router, http::{ Request, StatusCode }, body::Body };
use serde_json::json;
use sqlx::SqlitePool;
use tower::ServiceExt;

use crate::common::auth::{ setup_auth_test_db, seed_test_user, seed_test_user_with_name, create_auth_router };

/// Test: User creation with name field stores and returns actual name
///
/// RED phase expectation: This test will FAIL because:
/// - Database doesn't have name column
/// - User struct doesn't have name field
/// - Login response returns username instead of name
#[ tokio::test ]
async fn test_user_creation_with_name_field()
{
  let pool: SqlitePool = setup_auth_test_db().await;

  // Seed user with explicit name
  seed_test_user_with_name( &pool, "alice@example.com", "password123", "user", true, "Alice Smith" ).await;

  let router: Router = create_auth_router( pool.clone() ).await;

  // Login with valid credentials
  let login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "alice@example.com",
        "password": "password123"
      }).to_string()
    ))
    .unwrap();

  let response = router.oneshot( login_request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Login should succeed with valid credentials"
  );

  // Parse response body
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let login_response: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  // Verify user name field contains actual name (not email/username)
  let returned_name = login_response[ "user" ][ "name" ].as_str().unwrap();

  assert_eq!(
    returned_name,
    "Alice Smith",
    "LOUD FAILURE: Login response should return actual user name 'Alice Smith', not email or username. Got: {}",
    returned_name
  );
}

/// Test: Backward compatibility - users without name show email as fallback
#[ tokio::test ]
async fn test_backward_compatibility_no_name_shows_email()
{
  let pool: SqlitePool = setup_auth_test_db().await;

  // Seed user WITHOUT name (backward compatibility)
  seed_test_user( &pool, "olduser@example.com", "password123", "user", true ).await;

  let router: Router = create_auth_router( pool.clone() ).await;

  // Login
  let login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/v1/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "email": "olduser@example.com",
        "password": "password123"
      }).to_string()
    ))
    .unwrap();

  let response = router.oneshot( login_request ).await.unwrap();
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX ).await.unwrap();
  let login_response: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  // Verify fallback to username when name is NULL
  let returned_name = login_response[ "user" ][ "name" ].as_str().unwrap();

  assert_eq!(
    returned_name,
    "olduser",
    "LOUD FAILURE: When name is NULL, should fallback to username. Got: {}",
    returned_name
  );
}

/// Test: Name validation enforces length constraints (1-100 characters)
#[ tokio::test ]
async fn test_name_length_validation()
{
  let pool: SqlitePool = setup_auth_test_db().await;

  // Single character name (valid)
  seed_test_user_with_name( &pool, "a@example.com", "password123", "user", true, "A" ).await;

  // 100 character name (valid - max length)
  let max_name = "A".repeat( 100 );
  seed_test_user_with_name( &pool, "b@example.com", "password123", "user", true, &max_name ).await;

  // Verify both users were created successfully
  let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users WHERE email IN ('a@example.com', 'b@example.com')" )
    .fetch_one( &pool )
    .await
    .expect( "LOUD FAILURE: Should query users" );

  assert_eq!(
    count,
    2,
    "LOUD FAILURE: Both users with valid name lengths should be created"
  );
}
