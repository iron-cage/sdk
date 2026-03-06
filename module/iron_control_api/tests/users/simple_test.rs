//! Simplified test to isolate the issue
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_direct_handler_call` | Direct UserService integration test | Create user via UserService, suspend via same instance, verify audit log | User suspended (is_active=false), audit log entry created | ✅ |

use std::sync::Arc;

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use tracing::{debug, info};

use crate::common::test_db;
use iron_control_api::{
  rbac::PermissionChecker,
  routes::users::{CreateUserRequest, SuspendUserRequest, UserManagementState},
};

/// Create test database

#[tokio::test]
async fn test_direct_handler_call() {
  let db = test_db::create_test_db().await;
  let pool = db.pool();
  let user_service = iron_token_manager::user_service::UserService::new(pool.clone());

  // Create user through UserService
  let params = iron_token_manager::user_service::CreateUserParams {
    username: "testuser".to_string(),
    password: "testpass123".to_string(),
    email: "test@test.com".to_string(),
    role: "developer".to_string(),
  };

  let user = user_service
    .create_user(params, 999)
    .await
    .expect("LOUD FAILURE: Create user failed");
  debug!("Created user: {}", user.id);

  // Now try to suspend through the same UserService instance
  let result = user_service
    .suspend_user(user.id, 999, Some("Test".to_string()))
    .await;

  match result {
    Ok(suspended) => {
      info!("Suspend succeeded!");
      info!("  is_active: {}", suspended.is_active);
      info!("  suspended_at: {:?}", suspended.suspended_at);
      assert!(!suspended.is_active, "User should be suspended");
    }
    Err(e) => {
      panic!("Suspend failed: {:?}", e);
    }
  }

  // Verify audit log entry exists
  let audit_count: i64 =
    sqlx::query_scalar("SELECT COUNT(*) FROM user_audit_log WHERE operation = 'suspend'")
      .fetch_one(&pool)
      .await
      .expect("LOUD FAILURE: Audit query failed");

  debug!("Audit log entries for suspend: {}", audit_count);
  assert_eq!(audit_count, 1, "Should have exactly 1 suspend audit entry");
}
