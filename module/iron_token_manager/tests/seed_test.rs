#![allow(missing_docs)]

use sqlx::SqlitePool;

use iron_token_manager::{migrations, seed};

#[tokio::test]
async fn test_wipe_database() {
  let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
  migrations::apply_all_migrations(&pool).await.unwrap();

  // Seed database
  seed::seed_all(&pool).await.unwrap();

  // Verify data exists
  let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert!(user_count > 0, "Users should exist before wipe");

  // Wipe database
  seed::wipe_database(&pool).await.unwrap();

  // Verify all tables empty
  let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(user_count, 0, "Users table should be empty after wipe");

  let token_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_tokens")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(token_count, 0, "Tokens table should be empty after wipe");
}

#[tokio::test]
async fn test_seed_all() {
  let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
  migrations::apply_all_migrations(&pool).await.unwrap();

  seed::seed_all(&pool).await.unwrap();

  // Verify users created
  let user_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(
    user_count, 5,
    "Should create 5 users (admin, demo, viewer, tester, guest)"
  );

  // Verify provider keys created
  let provider_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM ai_provider_keys")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(provider_count, 2, "Should create 2 provider keys");

  // Verify tokens created
  let token_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM api_tokens")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(
    token_count, 8,
    "Should create 8 tokens (guest user has none)"
  );

  // Verify usage limits created
  let limit_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM usage_limits")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(limit_count, 3, "Should create 3 usage limits");

  // Verify project assignments created
  let assignment_count =
    sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM project_provider_key_assignments")
      .fetch_one(&pool)
      .await
      .unwrap();
  assert_eq!(assignment_count, 2, "Should create 2 project assignments");

  // Verify usage records created
  let usage_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM token_usage")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert!(
    usage_count >= 10,
    "Should create at least 10 usage records, got {usage_count}"
  );
}

#[tokio::test]
async fn test_seed_users_creates_correct_roles() {
  let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
  migrations::apply_all_migrations(&pool).await.unwrap();

  seed::seed_users(&pool).await.unwrap();

  // Verify admin role
  let admin_role =
    sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE username = 'admin'")
      .fetch_one(&pool)
      .await
      .unwrap();
  assert_eq!(admin_role, "admin", "Admin should have admin role");

  // Verify demo role
  let dev_role = sqlx::query_scalar::<_, String>("SELECT role FROM users WHERE username = 'demo'")
    .fetch_one(&pool)
    .await
    .unwrap();
  assert_eq!(dev_role, "manager", "Demo user should have manager role");

  // Verify inactive user
  let viewer_active =
    sqlx::query_scalar::<_, i64>("SELECT is_active FROM users WHERE username = 'viewer'")
      .fetch_one(&pool)
      .await
      .unwrap();
  assert_eq!(viewer_active, 0, "Viewer should be inactive");
}
