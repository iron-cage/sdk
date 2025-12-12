//! Simplified test to isolate the issue
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_direct_handler_call` | Direct UserService integration test | Create user via UserService, suspend via same instance, verify audit log | User suspended (is_active=false), audit log entry created | ✅ |

use iron_control_api::routes::users::{ UserManagementState, CreateUserRequest, SuspendUserRequest };
use iron_control_api::rbac::PermissionChecker;
use sqlx::{ SqlitePool, sqlite::SqlitePoolOptions };
use std::sync::Arc;
use tracing::{debug, info};

/// Create test database
async fn create_test_database() -> SqlitePool
{
  // Use shared in-memory database so all connections see the same data
  let pool = SqlitePoolOptions::new()
    .max_connections( 5 )
    .connect( "sqlite::memory:?cache=shared" )
    .await
    .expect( "Failed to create database" );

  // Enable foreign key constraints (required for SQLite)
  sqlx::raw_sql( "PRAGMA foreign_keys = ON;" )
    .execute( &pool )
    .await
    .expect( "Failed to enable foreign keys" );

  // Apply migrations
  let migration_003 = include_str!( "../../../iron_token_manager/migrations/003_create_users_table.sql" );
  let migration_005 = include_str!( "../../../iron_token_manager/migrations/005_enhance_users_table.sql" );
  let migration_006 = include_str!( "../../../iron_token_manager/migrations/006_create_user_audit_log.sql" );

  sqlx::raw_sql( migration_003 ).execute( &pool ).await.expect( "Migration 003 failed" );
  sqlx::raw_sql( migration_005 ).execute( &pool ).await.expect( "Migration 005 failed" );
  sqlx::raw_sql( migration_006 ).execute( &pool ).await.expect( "Migration 006 failed" );

  // Create admin user
  let admin_hash = bcrypt::hash( "admin_password", 4 ).expect( "Hash failed" );
  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "Time error" )
    .as_millis() as i64;

  sqlx::query(
    "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (999, 'test_admin', ?, 'admin@test.com', 'admin', 1, ?)"
  )
  .bind( &admin_hash )
  .bind( now )
  .execute( &pool )
  .await
  .expect( "Admin creation failed" );

  pool
}

#[ tokio::test ]
async fn test_direct_handler_call()
{
  let pool = create_test_database().await;
  let user_service = iron_token_manager::user_service::UserService::new( pool.clone() );

  // Create user through UserService
  let params = iron_token_manager::user_service::CreateUserParams
  {
    username: "testuser".to_string(),
    password: "testpass123".to_string(),
    email: "test@test.com".to_string(),
    role: "user".to_string(),
  };

  let user = user_service.create_user( params, 999 ).await.expect( "Create user failed" );
  debug!( "Created user: {}", user.id );

  // Now try to suspend through the same UserService instance
  let result = user_service.suspend_user( user.id, 999, Some( "Test".to_string() ) ).await;

  match result {
    Ok( suspended ) => {
      info!( "Suspend succeeded!" );
      info!( "  is_active: {}", suspended.is_active );
      info!( "  suspended_at: {:?}", suspended.suspended_at );
      assert!( !suspended.is_active, "User should be suspended" );
    }
    Err( e ) => {
      panic!( "Suspend failed: {:?}", e );
    }
  }

  // Verify audit log entry exists
  let audit_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM user_audit_log WHERE operation = 'suspend'"
  )
  .fetch_one( &pool )
  .await
  .expect( "Audit query failed" );

  debug!( "Audit log entries for suspend: {}", audit_count );
  assert_eq!( audit_count, 1, "Should have exactly 1 suspend audit entry" );
}
