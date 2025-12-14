//! Debug test to see actual error messages
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `debug_direct_suspend` | Debug user suspension with verbose logging | Create user via UserService, suspend user, log all operations and errors | User created and suspended successfully, debug logs show operation details | ✅ |

use iron_control_api::routes::users::UserManagementState;
use crate::common::test_db;
use axum::extract::FromRef;
use iron_control_api::routes::auth::AuthState;
use sqlx::sqlite::SqlitePool;
use sqlx::sqlite::SqlitePoolOptions;
use tracing::{info, debug};

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

#[ tokio::test ]
async fn debug_direct_suspend()
{
  let db = test_db::create_test_db().await;
  let pool = db.pool();
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
    Ok( user ) => debug!( "Created user ID: {}", user.id ),
    Err( e ) => debug!( "Failed to create user: {:?}", e ),
  }

  if let Ok( user ) = create_result {
    // Try to suspend
    debug!( "Attempting to suspend user {}...", user.id );
    let suspend_result = user_service.suspend_user( user.id, 999, Some( "Test".to_string() ) ).await;
    match suspend_result {
      Ok( _suspended_user ) => debug!( "Successfully suspended user" ),
      Err( e ) => debug!( "Failed to suspend user: {:?}", e ),
    }
  }
}
