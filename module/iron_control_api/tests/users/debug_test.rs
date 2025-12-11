//! Debug test to see actual error messages

use iron_control_api::routes::users::UserManagementState;
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
    Ok( _ ) => info!( "Admin user created successfully with ID=999" ),
    Err( e ) => info!( "Failed to create admin user: {:?}", e ),
  }

  // Verify admin user exists
  let admin_check: Result< i64, _ > = sqlx::query_scalar(
    "SELECT COUNT(*) FROM users WHERE id = 999"
  )
  .fetch_one( &pool )
  .await;

  match admin_check {
    Ok( count ) => info!( "Admin user count: {}", count ),
    Err( e ) => info!( "Failed to check admin user: {:?}", e ),
  }

  // Check if user_audit_log table exists
  let audit_table_check: Result< i64, _ > = sqlx::query_scalar(
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='user_audit_log'"
  )
  .fetch_one( &pool )
  .await;

  match audit_table_check {
    Ok( count ) => info!( "user_audit_log table exists: {}", count == 1 ),
    Err( e ) => info!( "Failed to check audit table: {:?}", e ),
  }

  pool
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
