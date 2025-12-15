//! Seed Data Validation Tests
//!
//! These tests validate that the seed data matches the documentation in
//! `tests/fixtures/seed_data_reference.md`. This prevents documentation drift.
//!
//! **Purpose:** Ensure developers can trust the seed data documentation.
//!
//! **Validation Strategy:**
//! - Test each seeded entity count
//! - Verify specific entity properties (usernames, roles, balances, etc.)
//! - Validate foreign key relationships
//! - Check seed data consistency across multiple runs

mod common;

use common::create_test_db_with_seed;

#[ tokio::test ]
async fn validate_seeded_users_count()
{
  let db = create_test_db_with_seed().await;

  let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count users" );

  assert_eq!(
    user_count, 5,
    "LOUD FAILURE: Seed should create exactly 5 users (admin, demo, viewer, tester, guest)"
  );
}

#[ tokio::test ]
async fn validate_admin_user_properties()
{
  let db = create_test_db_with_seed().await;

  // Verify admin user exists with correct properties
  let admin: ( String, String, i64 ) = sqlx::query_as(
    "SELECT username, role, is_active FROM users WHERE username = 'admin'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Admin user should exist" );

  assert_eq!( admin.0, "admin", "Username should be 'admin'" );
  assert_eq!( admin.1, "admin", "Role should be 'admin'" );
  assert_eq!( admin.2, 1, "Admin should be active" );
}

#[ tokio::test ]
async fn validate_demo_user_properties()
{
  let db = create_test_db_with_seed().await;

  let demo: ( String, String, i64 ) = sqlx::query_as(
    "SELECT username, role, is_active FROM users WHERE username = 'demo'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Demo user should exist" );

  assert_eq!( demo.0, "demo", "Username should be 'demo'" );
  assert_eq!( demo.1, "user", "Role should be 'user' (not admin)" );
  assert_eq!( demo.2, 1, "Demo user should be active" );
}

#[ tokio::test ]
async fn validate_viewer_user_inactive()
{
  let db = create_test_db_with_seed().await;

  let viewer: ( String, String, i64 ) = sqlx::query_as(
    "SELECT username, role, is_active FROM users WHERE username = 'viewer'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Viewer user should exist" );

  assert_eq!( viewer.0, "viewer", "Username should be 'viewer'" );
  assert_eq!( viewer.1, "user", "Role should be 'user'" );
  assert_eq!( viewer.2, 0, "Viewer should be INACTIVE (for revocation testing)" );
}

#[ tokio::test ]
async fn validate_provider_keys_count()
{
  let db = create_test_db_with_seed().await;

  let key_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM ai_provider_keys" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count provider keys" );

  assert_eq!(
    key_count, 2,
    "LOUD FAILURE: Seed should create exactly 2 provider keys (OpenAI, Anthropic)"
  );
}

#[ tokio::test ]
async fn validate_openai_provider_key()
{
  let db = create_test_db_with_seed().await;

  let openai: ( String, i64, i64 ) = sqlx::query_as(
    "SELECT provider, is_enabled, balance_cents
     FROM ai_provider_keys
     WHERE provider = 'openai'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: OpenAI provider key should exist" );

  assert_eq!( openai.0, "openai", "Provider should be 'openai'" );
  assert_eq!( openai.1, 1, "OpenAI key should be enabled" );
  assert_eq!( openai.2, 5000, "OpenAI balance should be $50.00 (5000 cents)" );
}

#[ tokio::test ]
async fn validate_anthropic_provider_key()
{
  let db = create_test_db_with_seed().await;

  let anthropic: ( String, i64, i64 ) = sqlx::query_as(
    "SELECT provider, is_enabled, balance_cents
     FROM ai_provider_keys
     WHERE provider = 'anthropic'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Anthropic provider key should exist" );

  assert_eq!( anthropic.0, "anthropic", "Provider should be 'anthropic'" );
  assert_eq!( anthropic.1, 1, "Anthropic key should be enabled" );
  assert_eq!( anthropic.2, 10000, "Anthropic balance should be $100.00 (10000 cents)" );
}

#[ tokio::test ]
async fn validate_api_tokens_count()
{
  let db = create_test_db_with_seed().await;

  let token_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM api_tokens" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count tokens" );

  assert_eq!(
    token_count, 8,
    "LOUD FAILURE: Seed should create exactly 8 tokens"
  );
}

#[ tokio::test ]
async fn validate_admin_token_never_expires()
{
  let db = create_test_db_with_seed().await;

  let admin_token: ( String, i64, Option< i64 > ) = sqlx::query_as(
    "SELECT name, is_active, expires_at
     FROM api_tokens
     WHERE name = 'Admin Master Token'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Admin Master Token should exist" );

  assert_eq!( admin_token.0, "Admin Master Token", "Name should match" );
  assert_eq!( admin_token.1, 1, "Admin token should be active" );
  assert_eq!( admin_token.2, None, "Admin token should NEVER expire (NULL expires_at)" );
}

#[ tokio::test ]
async fn validate_inactive_token_exists()
{
  let db = create_test_db_with_seed().await;

  let inactive_token: ( String, i64 ) = sqlx::query_as(
    "SELECT name, is_active
     FROM api_tokens
     WHERE name = 'Inactive Token'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Inactive Token should exist" );

  assert_eq!( inactive_token.0, "Inactive Token", "Name should match" );
  assert_eq!( inactive_token.1, 0, "Token should be INACTIVE (for revocation testing)" );
}

#[ tokio::test ]
async fn validate_expired_token_is_expired()
{
  let db = create_test_db_with_seed().await;

  #[ allow( clippy::cast_possible_truncation ) ]
  let now_ms = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .unwrap()
    .as_millis() as i64;

  let expired_token: ( String, i64, Option< i64 > ) = sqlx::query_as(
    "SELECT name, is_active, expires_at
     FROM api_tokens
     WHERE name = 'Expired Token'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Expired Token should exist" );

  assert_eq!( expired_token.0, "Expired Token", "Name should match" );
  assert_eq!( expired_token.1, 1, "Token should be marked active (but expired)" );

  let expires_at = expired_token.2.expect("LOUD FAILURE: Expired token should have expires_at timestamp");
  assert!(
    expires_at < now_ms,
    "LOUD FAILURE: Expired token should have expires_at in the past (doc says 30 days ago)"
  );
}

#[ tokio::test ]
async fn validate_usage_limits_count()
{
  let db = create_test_db_with_seed().await;

  let limit_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM usage_limits" )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Failed to count usage limits" );

  assert_eq!(
    limit_count, 3,
    "LOUD FAILURE: Seed should create exactly 3 usage limits"
  );
}

#[ tokio::test ]
async fn validate_admin_unlimited_limits()
{
  let db = create_test_db_with_seed().await;

  let admin_limit: ( String, Option< i64 >, Option< i64 >, Option< i64 > ) = sqlx::query_as(
    "SELECT user_id, max_tokens_per_day, max_requests_per_minute, max_cost_microdollars_per_month
     FROM usage_limits
     WHERE user_id = 'user_admin'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Admin usage limit should exist" );

  assert_eq!( admin_limit.0, "user_admin", "User ID should match" );
  assert_eq!( admin_limit.1, None, "Max tokens should be UNLIMITED (NULL)" );
  assert_eq!( admin_limit.2, None, "Max requests should be UNLIMITED (NULL)" );
  assert_eq!( admin_limit.3, None, "Max cost should be UNLIMITED (NULL)" );
}

#[ tokio::test ]
async fn validate_demo_standard_tier()
{
  let db = create_test_db_with_seed().await;

  let dev_limit: ( String, Option< i64 >, Option< i64 >, Option< i64 >, i64, i64, i64 ) =
    sqlx::query_as(
      "SELECT user_id, max_tokens_per_day, max_requests_per_minute, max_cost_microdollars_per_month,
              current_tokens_today, current_requests_this_minute, current_cost_microdollars_this_month
       FROM usage_limits
       WHERE user_id = 'user_demo'"
    )
    .fetch_one( db.pool() )
    .await
    .expect( "LOUD FAILURE: Demo usage limit should exist" );

  assert_eq!( dev_limit.0, "user_demo", "User ID should match" );
  assert_eq!( dev_limit.1, Some( 1_000_000 ), "Max tokens should be 1M/day" );
  assert_eq!( dev_limit.2, Some( 60 ), "Max requests should be 60/minute" );
  assert_eq!( dev_limit.3, Some( 50_000_000 ), "Max cost should be $50/month (50M microdollars)" );
  assert_eq!( dev_limit.4, 250_000, "Current tokens should be 250k (25% used)" );
  assert_eq!( dev_limit.5, 15, "Current requests should be 15 (25% used)" );
  assert_eq!( dev_limit.6, 12_500_000, "Current cost should be $12.50 (12.5M microdollars, 25% used)" );
}

#[ tokio::test ]
async fn validate_viewer_near_limit()
{
  let db = create_test_db_with_seed().await;

  let viewer_limit: ( String, Option< i64 >, i64 ) = sqlx::query_as(
    "SELECT user_id, max_tokens_per_day, current_tokens_today
     FROM usage_limits
     WHERE user_id = 'user_viewer'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Viewer usage limit should exist" );

  assert_eq!( viewer_limit.0, "user_viewer", "User ID should match" );
  assert_eq!( viewer_limit.1, Some( 100_000 ), "Max tokens should be 100k/day" );
  assert_eq!(
    viewer_limit.2, 95_000,
    "Current tokens should be 95k (95% used - NEAR LIMIT for testing)"
  );
}

#[ tokio::test ]
async fn validate_project_assignments_count()
{
  let db = create_test_db_with_seed().await;

  let assignment_count: i64 = sqlx::query_scalar(
    "SELECT COUNT(*) FROM project_provider_key_assignments"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to count project assignments" );

  assert_eq!(
    assignment_count, 2,
    "LOUD FAILURE: Seed should create exactly 2 project assignments"
  );
}

#[ tokio::test ]
async fn validate_project_alpha_has_both_providers()
{
  let db = create_test_db_with_seed().await;

  // Verify project_alpha has OpenAI assignment
  let openai_assignment: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM project_provider_key_assignments pa
     JOIN ai_provider_keys pk ON pa.provider_key_id = pk.id
     WHERE pa.project_id = 'project_alpha' AND pk.provider = 'openai'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to query OpenAI assignment" );

  assert_eq!(
    openai_assignment, 1,
    "project_alpha should have OpenAI provider assigned"
  );

  // Verify project_alpha has Anthropic assignment
  let anthropic_assignment: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM project_provider_key_assignments pa
     JOIN ai_provider_keys pk ON pa.provider_key_id = pk.id
     WHERE pa.project_id = 'project_alpha' AND pk.provider = 'anthropic'"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to query Anthropic assignment" );

  assert_eq!(
    anthropic_assignment, 1,
    "project_alpha should have Anthropic provider assigned"
  );
}

#[ tokio::test ]
async fn validate_seed_idempotency()
{
  // Verify that calling seed_all() on already-seeded database doesn't duplicate data
  let db = create_test_db_with_seed().await;

  // Attempt to seed again (should fail or be idempotent)
  // Current implementation will fail on duplicate insert, which is expected behavior
  let result = iron_token_manager::seed::seed_all( db.pool() ).await;

  assert!(
    result.is_err(),
    "LOUD FAILURE: Seeding twice should fail (prevents accidental data duplication)"
  );
}

#[ tokio::test ]
async fn validate_all_users_have_same_password_hash()
{
  let db = create_test_db_with_seed().await;

  // All users should have the same password hash (demo password)
  let password_hashes: Vec< ( String, ) > =
    sqlx::query_as( "SELECT DISTINCT password_hash FROM users" )
      .fetch_all( db.pool() )
      .await
      .expect( "LOUD FAILURE: Failed to query password hashes" );

  assert_eq!(
    password_hashes.len(),
    1,
    "LOUD FAILURE: All users should have the SAME password hash"
  );

  // Bcrypt hash of "IronDemo2025!" with cost=12 (matches module/iron_token_manager/src/seed.rs)
  let expected_hash = "$2b$12$AJbkR5cbO1NDN8vXQ2FSr.02E7lvpf6X7fp7yfBkppqHWtHF8vh86";
  assert_eq!(
    password_hashes[ 0 ].0, expected_hash,
    "Password hash should match documented bcrypt hash (cost=12)"
  );
}

#[ tokio::test ]
async fn validate_foreign_key_integrity()
{
  // Verify foreign key relationships are correct
  let db = create_test_db_with_seed().await;

  // All provider keys should reference valid users
  let orphaned_keys: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM ai_provider_keys pk
     WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = pk.user_id)"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to check orphaned provider keys" );

  assert_eq!( orphaned_keys, 0, "No provider keys should be orphaned" );

  // All tokens should reference valid users
  let orphaned_tokens: i64 = sqlx::query_scalar(
    "SELECT COUNT(*)
     FROM api_tokens t
     WHERE NOT EXISTS (SELECT 1 FROM users u WHERE u.id = t.user_id)"
  )
  .fetch_one( db.pool() )
  .await
  .expect( "LOUD FAILURE: Failed to check orphaned tokens" );

  assert_eq!( orphaned_tokens, 0, "No tokens should be orphaned" );
}
