//! Debug test to see actual error messages

use crate::common::{extract_json_response, extract_response};
use hyper::StatusCode;
use iron_control_api::routes::{auth::LoginResponse, users::UserManagementState};
use iron_control_api::rbac::PermissionChecker;
use axum::
{
  Router,
  routing::{ post, put },
  http::Request,
};
use axum::body::Body;
use tower::ServiceExt;
use serde_json::json;
use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use std::sync::Arc;
use axum::extract::FromRef;
use iron_control_api::routes::auth::AuthState;
use iron_control_api::jwt_auth::JwtSecret;

#[derive(Clone)]
struct TestAppState {
    auth: AuthState,
    users: UserManagementState,
}

impl FromRef<TestAppState> for AuthState {
    fn from_ref(state: &TestAppState) -> Self {
        state.auth.clone()
    }
}

impl FromRef<TestAppState> for UserManagementState {
    fn from_ref(state: &TestAppState) -> Self {
        state.users.clone()
    }
}

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

  // Create admin user with ID=999 for testing
  let admin_password_hash = bcrypt::hash( "admin_password", 4 )
    .expect( "LOUD FAILURE: Failed to hash admin password" );

  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time went backwards" )
    .as_millis() as i64;

  let admin_result = sqlx::query(
    "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (999, 'test_admin', ?, 'admin@test.com', 'admin', 1, ?)"
  )
  .bind( &admin_password_hash )
  .bind( now )
  .execute( &pool )
  .await;

  match admin_result {
    Ok( _ ) => println!( "Admin user created successfully with ID=999" ),
    Err( e ) => println!( "Failed to create admin user: {:?}", e ),
  }

  // Verify admin user exists
  let admin_check: Result< i64, _ > = sqlx::query_scalar(
    "SELECT COUNT(*) FROM users WHERE id = 999"
  )
  .fetch_one( &pool )
  .await;

  match admin_check {
    Ok( count ) => println!( "Admin user count: {}", count ),
    Err( e ) => println!( "Failed to check admin user: {:?}", e ),
  }

  // Check if user_audit_log table exists
  let audit_table_check: Result< i64, _ > = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='user_audit_log'"
  )
  .fetch_one( &pool )
  .await;

  match audit_table_check {
    Ok( count ) => println!( "user_audit_log table exists: {}", count == 1 ),
    Err( e ) => println!( "Failed to check audit table: {:?}", e ),
  }

  pool
}

/// Create test router with user management routes
async fn create_test_router() -> Router
{
  let db_pool = create_test_database().await;
  let permission_checker = Arc::new( PermissionChecker::new() );

  let auth_state = AuthState {
    db_pool: db_pool.clone(),
    jwt_secret: Arc::new(JwtSecret::new("test_secret".to_string())),
  };

  let user_state = UserManagementState::new( db_pool, permission_checker );

  let state = TestAppState {
    auth: auth_state,
    users: user_state,
  };

  Router::new()
    .route( "/api/users", post( iron_control_api::routes::users::create_user ) )
    .route( "/api/users/:id/suspend", put( iron_control_api::routes::users::suspend_user ) )
    .with_state( state )
}

/// Get Admin Authentication Bearer Token
async fn get_admin_bearer_token(router: &Router) -> String
{
  let admin_login_body = json!({
    "username": "test_admin",
    "password": "admin_password",
  });

  let admin_login_request = Request::builder()
    .method( "POST" )
    .uri( "/api/auth/login" )
    .header( "content-type", "application/json" )
    .body( Body::from( serde_json::to_string( &admin_login_body ).unwrap() ) )
    .unwrap();

  let admin_login_response = router.clone().oneshot( admin_login_request ).await.unwrap();

  let ( _status, admin_login_body ): ( StatusCode, LoginResponse ) = extract_json_response( admin_login_response ).await;
  admin_login_body.access_token
}

#[ tokio::test ]
async fn debug_direct_suspend()
{
  let pool = create_test_database().await;
  let user_service = iron_token_manager::user_service::UserService::new( pool.clone() );

  // Create user directly
  let params = iron_token_manager::user_service::CreateUserParams
  {
    username: "directuser".to_string(),
    password: "testpass123".to_string(),
    email: "direct@test.com".to_string(),
    role: "user".to_string(),
  };

  let create_result = user_service.create_user( params, 999 ).await;
  match &create_result {
    Ok( user ) => println!( "Created user ID: {}", user.id ),
    Err( e ) => println!( "Failed to create user: {:?}", e ),
  }

  if let Ok( user ) = create_result {
    // Try to suspend
    println!( "Attempting to suspend user {}...", user.id );
    let suspend_result = user_service.suspend_user( user.id, 999, Some( "Test".to_string() ) ).await;
    match suspend_result {
      Ok( _suspended_user ) => println!( "Successfully suspended user" ),
      Err( e ) => println!( "Failed to suspend user: {:?}", e ),
    }
  }
}
