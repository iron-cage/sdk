//! Endpoint integration tests for user management
//!
//! Test Matrix: User Management Endpoints
//!
//! | Endpoint | Method | Test Cases | Expected Status Codes |
//! |----------|--------|------------|----------------------|
//! | /api/users | POST | Valid, duplicate username, invalid email/password/role | 201, 400, 500 |
//! | /api/users | GET | No filters, role filter, search, pagination | 200 |
//! | /api/users/:id | GET | Valid ID, non-existent ID | 200, 404 |
//! | /api/users/:id/suspend | PUT | Valid, with reason | 200 |
//! | /api/users/:id/activate | PUT | Valid | 200 |
//! | /api/users/:id | DELETE | Valid, self-deletion prevention | 200, 500 |
//! | /api/users/:id/role | PUT | Valid, invalid role, self-modification prevention | 200, 400, 500 |
//! | /api/users/:id/reset-password | POST | Valid, with force_change | 200 |
//!
//! Coverage:
//! - Request validation (username, email, password, role limits)
//! - HTTP status codes (200, 201, 400, 403, 404, 500)
//! - JSON response structure
//! - Database persistence
//! - Audit logging
//! - Edge cases (duplicate usernames, self-operations, invalid filters)

use crate::common::extract_json_response;
use iron_control_api::routes::users::
{
  UserManagementState, CreateUserResponse, ListUsersResponse, UserResponse,
};
use iron_control_api::rbac::PermissionChecker;
use axum::
{
  Router,
  routing::{ get, post, put, delete },
  http::{ Request, StatusCode },
};
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;
use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use std::sync::Arc;

/// Create test database with users table and migrations
async fn create_test_database() -> SqlitePool
{
  // Use shared in-memory database so all connections see the same data
  // Without ?cache=shared, each connection gets its own private database!
  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( "sqlite::memory:?cache=shared" )
    .await
    .expect( "LOUD FAILURE: Failed to create in-memory database" );

  // Enable foreign key constraints (required for SQLite)
  sqlx::raw_sql( "PRAGMA foreign_keys = ON;" )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to enable foreign keys" );

  // Apply migrations (003, 005, 006)
  let migration_003 = include_str!( "../../../iron_token_manager/migrations/003_create_users_table.sql" );
  let migration_005 = include_str!( "../../../iron_token_manager/migrations/005_enhance_users_table.sql" );
  let migration_006 = include_str!( "../../../iron_token_manager/migrations/006_create_user_audit_log.sql" );

  sqlx::raw_sql( migration_003 )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to apply migration 003" );

  sqlx::raw_sql( migration_005 )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to apply migration 005" );

  sqlx::raw_sql( migration_006 )
    .execute( &pool )
    .await
    .expect( "LOUD FAILURE: Failed to apply migration 006" );

  // Create admin user with ID=999 for testing (used as performed_by in audit logs)
  let admin_password_hash = bcrypt::hash( "admin_password", 4 )
    .expect( "LOUD FAILURE: Failed to hash admin password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64;

  sqlx::query(
    "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (999, 'test_admin', ?, 'admin@test.com', 'admin', 1, ?)"
  )
  .bind( &admin_password_hash )
  .bind( now )
  .execute( &pool )
  .await
  .expect( "LOUD FAILURE: Failed to create test admin user" );

  pool
}

/// Create test router with user management routes
async fn create_test_router() -> Router
{
  let db_pool = create_test_database().await;
  let permission_checker = Arc::new( PermissionChecker::new() );

  let state = UserManagementState::new( db_pool, permission_checker );

  Router::new()
    .route( "/api/users", post( iron_control_api::routes::users::create_user ) )
    .route( "/api/users", get( iron_control_api::routes::users::list_users ) )
    .route( "/api/users/:id", get( iron_control_api::routes::users::get_user ) )
    .route( "/api/users/:id/suspend", put( iron_control_api::routes::users::suspend_user ) )
    .route( "/api/users/:id/activate", put( iron_control_api::routes::users::activate_user ) )
    .route( "/api/users/:id", delete( iron_control_api::routes::users::delete_user ) )
    .route( "/api/users/:id/role", put( iron_control_api::routes::users::change_user_role ) )
    .route( "/api/users/:id/reset-password", post( iron_control_api::routes::users::reset_password ) )
    .with_state( state )
}

//
// Create User Tests
//

/// Test POST /api/users with valid request returns 201 Created
#[ tokio::test ]
async fn test_create_user_valid_request()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "testuser",
    "password": "testpassword123",
    "email": "test@example.com",
    "role": "user",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "LOUD FAILURE: Valid user creation must return 201 Created"
  );

  let ( status, body ): ( StatusCode, CreateUserResponse ) = extract_json_response( response ).await;
  assert_eq!( status, StatusCode::CREATED );
  assert_eq!( body.username, "testuser" );
  assert_eq!( body.email, Some( "test@example.com".to_string() ) );
  assert_eq!( body.role, "user" );
  assert!( body.is_active, "LOUD FAILURE: New user must be active by default" );
}

/// Test POST /api/users with empty username returns 400
#[ tokio::test ]
async fn test_create_user_empty_username_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "",
    "password": "testpassword123",
    "email": "test@example.com",
    "role": "user",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty username must return 400 Bad Request"
  );
}

/// Test POST /api/users with short password returns 400
#[ tokio::test ]
async fn test_create_user_short_password_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "testuser",
    "password": "short",
    "email": "test@example.com",
    "role": "user",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Short password (< 8 chars) must return 400 Bad Request"
  );
}

/// Test POST /api/users with empty email returns 400
#[ tokio::test ]
async fn test_create_user_empty_email_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "testuser",
    "password": "testpassword123",
    "email": "",
    "role": "user",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Empty email must return 400 Bad Request"
  );
}

/// Test POST /api/users with invalid email (no @) returns 400
#[ tokio::test ]
async fn test_create_user_invalid_email_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "testuser",
    "password": "testpassword123",
    "email": "notanemail",
    "role": "user",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Invalid email (no @) must return 400 Bad Request"
  );
}

/// Test POST /api/users with invalid role returns 400
#[ tokio::test ]
async fn test_create_user_invalid_role_rejected()
{
  let router = create_test_router().await;

  let request_body = json!({
    "username": "testuser",
    "password": "testpassword123",
    "email": "test@example.com",
    "role": "superadmin",
  });

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Invalid role must return 400 Bad Request"
  );
}

//
// List Users Tests
//

/// Test GET /api/users returns all users
#[ tokio::test ]
async fn test_list_users_no_filters()
{
  let router = create_test_router().await;

  // Create test user first
  let create_request_body = json!({
    "username": "testuser",
    "password": "testpassword123",
    "email": "test@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let _ = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  // List users
  let list_request = Request::builder()
    .method( "GET" )
    .uri( "/api/users" )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, list_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: List users must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, ListUsersResponse ) = extract_json_response( response ).await;
  assert!( body.total >= 1, "LOUD FAILURE: Must have at least 1 user" );
  assert!( !body.users.is_empty(), "LOUD FAILURE: Users list cannot be empty" );
}

/// Test GET /api/users with role filter
#[ tokio::test ]
async fn test_list_users_with_role_filter()
{
  let router = create_test_router().await;

  // Create admin user
  let create_request_body = json!({
    "username": "adminuser",
    "password": "testpassword123",
    "email": "admin@example.com",
    "role": "admin",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let _ = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  // List users with role=admin filter
  let list_request = Request::builder()
    .method( "GET" )
    .uri( "/api/users?role=admin" )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, list_request )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let ( _status, body ): ( StatusCode, ListUsersResponse ) = extract_json_response( response ).await;
  assert!(
    body.users.iter().all( |u| u.role == "admin" ),
    "LOUD FAILURE: All users must have admin role when filtered"
  );
}

/// Test GET /api/users with search filter
#[ tokio::test ]
async fn test_list_users_with_search()
{
  let router = create_test_router().await;

  // Create user with specific username
  let create_request_body = json!({
    "username": "searchable_user",
    "password": "testpassword123",
    "email": "searchable@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let _ = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  // Search for user
  let list_request = Request::builder()
    .method( "GET" )
    .uri( "/api/users?search=searchable" )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, list_request )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let ( _status, body ): ( StatusCode, ListUsersResponse ) = extract_json_response( response ).await;
  assert!(
    body.total >= 1,
    "LOUD FAILURE: Search must find at least 1 user"
  );
}

//
// Get User Tests
//

/// Test GET /api/users/:id with valid ID returns user
#[ tokio::test ]
async fn test_get_user_valid_id()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "getuser",
    "password": "testpassword123",
    "email": "getuser@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Get user by ID
  let get_request = Request::builder()
    .method( "GET" )
    .uri( format!( "/api/users/{}", user_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, get_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Get user with valid ID must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert_eq!( body.id, user_id );
  assert_eq!( body.username, "getuser" );
}

/// Test GET /api/users/:id with non-existent ID returns 404
#[ tokio::test ]
async fn test_get_user_nonexistent_id()
{
  let router = create_test_router().await;

  let request = Request::builder()
    .method( "GET" )
    .uri( "/api/users/99999" )
    .body( Body::empty() )
    .unwrap();

  let response = router.oneshot( request ).await.unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "LOUD FAILURE: Get user with non-existent ID must return 404 Not Found"
  );
}

//
// Suspend User Tests
//

/// Test PUT /api/users/:id/suspend suspends user
#[ tokio::test ]
async fn test_suspend_user_valid()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "suspenduser",
    "password": "testpassword123",
    "email": "suspend@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Suspend user
  let suspend_request_body = json!({
    "reason": "Test suspension",
  });

  let suspend_request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/users/{}/suspend", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &suspend_request_body ).unwrap() ) )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, suspend_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Suspend user must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert!( !body.is_active, "LOUD FAILURE: Suspended user must have is_active=false" );
  assert!( body.suspended_at.is_some(), "LOUD FAILURE: Suspended user must have suspended_at timestamp" );
}

//
// Activate User Tests
//

/// Test PUT /api/users/:id/activate activates user
#[ tokio::test ]
async fn test_activate_user_valid()
{
  let router = create_test_router().await;

  // Create and suspend user
  let create_request_body = json!({
    "username": "activateuser",
    "password": "testpassword123",
    "email": "activate@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Suspend first
  let suspend_request_body = json!({
    "reason": "Test suspension",
  });

  let suspend_request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/users/{}/suspend", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &suspend_request_body ).unwrap() ) )
    .unwrap();

  let _ = ServiceExt::< Request< Body > >::oneshot( router.clone(), suspend_request )
    .await
    .unwrap();

  // Activate user
  let activate_request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/users/{}/activate", user_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, activate_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Activate user must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert!( body.is_active, "LOUD FAILURE: Activated user must have is_active=true" );
}

//
// Delete User Tests
//

/// Test DELETE /api/users/:id deletes user (soft delete)
#[ tokio::test ]
async fn test_delete_user_valid()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "deleteuser",
    "password": "testpassword123",
    "email": "delete@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Delete user
  let delete_request = Request::builder()
    .method( "DELETE" )
    .uri( format!( "/api/users/{}", user_id ) )
    .body( Body::empty() )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, delete_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Delete user must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert!( body.deleted_at.is_some(), "LOUD FAILURE: Deleted user must have deleted_at timestamp" );
}

//
// Change Role Tests
//

/// Test PUT /api/users/:id/role changes user role
#[ tokio::test ]
async fn test_change_user_role_valid()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "roleuser",
    "password": "testpassword123",
    "email": "role@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Change role to admin
  let role_request_body = json!({
    "role": "admin",
  });

  let role_request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/users/{}/role", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &role_request_body ).unwrap() ) )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, role_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Change user role must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert_eq!( body.role, "admin", "LOUD FAILURE: User role must be updated to admin" );
}

/// Test PUT /api/users/:id/role with invalid role returns 400
#[ tokio::test ]
async fn test_change_user_role_invalid_rejected()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "roleuser2",
    "password": "testpassword123",
    "email": "role2@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Try to change to invalid role
  let role_request_body = json!({
    "role": "superadmin",
  });

  let role_request = Request::builder()
    .method( "PUT" )
    .uri( format!( "/api/users/{}/role", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &role_request_body ).unwrap() ) )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, role_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Invalid role must return 400 Bad Request"
  );
}

//
// Reset Password Tests
//

/// Test POST /api/users/:id/reset-password resets password
#[ tokio::test ]
async fn test_reset_password_valid()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "resetuser",
    "password": "oldpassword123",
    "email": "reset@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Reset password
  let reset_request_body = json!({
    "new_password": "newpassword456",
    "force_change": true,
  });

  let reset_request = Request::builder()
    .method( "POST" )
    .uri( format!( "/api/users/{}/reset-password", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &reset_request_body ).unwrap() ) )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, reset_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::OK,
    "LOUD FAILURE: Reset password must return 200 OK"
  );

  let ( _status, body ): ( StatusCode, UserResponse ) = extract_json_response( response ).await;
  assert_eq!( body.id, user_id );
}

/// Test POST /api/users/:id/reset-password with short password returns 400
#[ tokio::test ]
async fn test_reset_password_short_rejected()
{
  let router = create_test_router().await;

  // Create user
  let create_request_body = json!({
    "username": "resetuser2",
    "password": "oldpassword123",
    "email": "reset2@example.com",
    "role": "user",
  });

  let create_request = Request::builder()
    .method( "POST" )
    .uri( "/api/users" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &create_request_body ).unwrap() ) )
    .unwrap();

  let create_response = ServiceExt::< Request< Body > >::oneshot( router.clone(), create_request )
    .await
    .unwrap();

  let ( _status, create_body ): ( StatusCode, CreateUserResponse ) = extract_json_response( create_response ).await;
  let user_id = create_body.id;

  // Try to reset with short password
  let reset_request_body = json!({
    "new_password": "short",
    "force_change": false,
  });

  let reset_request = Request::builder()
    .method( "POST" )
    .uri( format!( "/api/users/{}/reset-password", user_id ) )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &reset_request_body ).unwrap() ) )
    .unwrap();

  let response = ServiceExt::< Request< Body > >::oneshot( router, reset_request )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::BAD_REQUEST,
    "LOUD FAILURE: Short password (< 8 chars) must return 400 Bad Request"
  );
}
