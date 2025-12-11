//! Database seeding utilities for development and testing
//!
//! Provides functions to populate database with realistic sample data for manual testing.
//! Designed for use in development and test environments only.
//!
//! # Usage
//!
//! ```rust,ignore
//! use iron_token_manager::seed::seed_all;
//!
//! let pool = SqlitePool::connect("sqlite:///dev.db").await?;
//! seed_all(&pool).await?;
//! ```
//!
//! # Safety
//!
//! - NEVER use in production
//! - Seeds only to empty tables (fails if data exists)
//! - Uses predictable test data (not secure)
//! - Provider keys use placeholder encryption

use sqlx::SqlitePool;
use crate::error::Result;

/// Wipe all data from database tables
///
/// Deletes all rows from all tables in reverse dependency order to respect
/// foreign key constraints. Does NOT drop tables or schema.
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Ok if all tables wiped successfully
///
/// # Errors
///
/// Returns error if any delete fails
///
/// # Safety
///
/// This is destructive! Use only in development/test environments.
pub async fn wipe_database( pool: &SqlitePool ) -> Result< () >
{
  // Delete in reverse dependency order to respect foreign keys

  // Child tables first (foreign key references)
  sqlx::query( "DELETE FROM token_usage" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM api_call_traces" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM audit_log" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM project_provider_key_assignments" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM token_blacklist" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM user_audit_log" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  // Parent tables (referenced by foreign keys)
  sqlx::query( "DELETE FROM api_tokens" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM ai_provider_keys" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM usage_limits" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM users" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  sqlx::query( "DELETE FROM agents" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

  Ok( () )
}

/// Seed all tables with sample data
///
/// Populates database with realistic development data:
/// - 5 sample users (admin, developer, viewer, tester, guest)
/// - 2 AI provider keys (`OpenAI`, `Anthropic`)
/// - 8 API tokens (various scopes and states)
/// - 3 usage limits (different tiers, some users have no limits)
/// - 2 project-provider assignments
/// - 10+ usage records (various patterns)
///
/// Edge cases covered:
/// - Expired tokens, expiring soon tokens, never-expiring tokens
/// - Active and inactive tokens
/// - Users with no tokens (guest)
/// - Users with no limits (tester)
/// - Tokens near usage limits
/// - Tokens with zero usage
///
/// # Arguments
///
/// * `pool` - Database connection pool
///
/// # Returns
///
/// Ok if all seeding succeeds
///
/// # Errors
///
/// Returns error if any insert fails
pub async fn seed_all( pool: &SqlitePool ) -> Result< () >
{
  seed_users( pool ).await?;
  seed_provider_keys( pool ).await?;
  seed_api_tokens( pool ).await?;
  seed_usage_limits( pool ).await?;
  seed_project_assignments( pool ).await?;
  seed_token_usage( pool ).await?;

  Ok( () )
}

/// Seed users table with sample accounts
///
/// Creates 5 users with various edge cases:
/// - `admin` (role: admin, active)
/// - `developer` (role: user, active)
/// - `viewer` (role: user, inactive)
/// - `tester` (role: user, active, no usage limits)
/// - `guest` (role: user, active, no tokens)
///
/// Passwords are bcrypt hashed "password123" (DO NOT use in production!)
async fn seed_users( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();
  let day_ms = 24 * 60 * 60 * 1000;

  // Bcrypt hash of "password123" (cost = 4, for fast dev testing)
  let password_hash = "$2b$04$xQa5kFZZhNGwPDXqJvw9XuXUdQEPAqXwNMOqQcU6MqWxPLxOVyJqO";

  // Admin user
  sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5)"
  )
  .bind( "admin" )
  .bind( password_hash )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Developer user
  sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5)"
  )
  .bind( "developer" )
  .bind( password_hash )
  .bind( "user" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Inactive viewer user
  sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5)"
  )
  .bind( "viewer" )
  .bind( password_hash )
  .bind( "user" )
  .bind( 0 )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Tester user (no usage limits - unlimited testing)
  sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5)"
  )
  .bind( "tester" )
  .bind( password_hash )
  .bind( "user" )
  .bind( 1 )
  .bind( now_ms - ( 7 * day_ms ) )  // Created a week ago
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Guest user (no tokens - just registered)
  sqlx::query(
    "INSERT INTO users (username, password_hash, role, is_active, created_at) \
     VALUES ($1, $2, $3, $4, $5)"
  )
  .bind( "guest" )
  .bind( password_hash )
  .bind( "user" )
  .bind( 1 )
  .bind( now_ms - ( 60 * 60 * 1000 ) )  // Created 1 hour ago
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  Ok( () )
}

/// Seed AI provider keys with sample data
///
/// Creates 2 provider keys:
/// - `OpenAI` key (enabled, with balance)
/// - `Anthropic` key (enabled, with balance)
///
/// Keys use placeholder encryption (NOT secure for production!)
async fn seed_provider_keys( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();

  // Placeholder encrypted keys (base64 of "fake_encrypted_key_openai")
  let openai_encrypted = "ZmFrZV9lbmNyeXB0ZWRfa2V5X29wZW5haQ==";
  let openai_nonce = "YWFhYWFhYWFhYWFh";  // base64 of "aaaaaaaaaaaa"

  sqlx::query(
    "INSERT INTO ai_provider_keys \
     (provider, encrypted_api_key, encryption_nonce, description, is_enabled, \
      created_at, balance_cents, balance_updated_at, user_id) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
  )
  .bind( "openai" )
  .bind( openai_encrypted )
  .bind( openai_nonce )
  .bind( "Development OpenAI key" )
  .bind( 1 )
  .bind( now_ms )
  .bind( 5000 )  // $50.00 balance
  .bind( now_ms )
  .bind( "admin" )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Anthropic key
  let anthropic_encrypted = "ZmFrZV9lbmNyeXB0ZWRfa2V5X2FudGhyb3BpYw==";
  let anthropic_nonce = "YmJiYmJiYmJiYmJi";  // base64 of "bbbbbbbbbbbb"

  sqlx::query(
    "INSERT INTO ai_provider_keys \
     (provider, encrypted_api_key, encryption_nonce, description, is_enabled, \
      created_at, balance_cents, balance_updated_at, user_id) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
  )
  .bind( "anthropic" )
  .bind( anthropic_encrypted )
  .bind( anthropic_nonce )
  .bind( "Development Anthropic key" )
  .bind( 1 )
  .bind( now_ms )
  .bind( 10000 )  // $100.00 balance
  .bind( now_ms )
  .bind( "admin" )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  Ok( () )
}

/// Seed API tokens with sample data
///
/// Creates 5 tokens:
/// - Active admin token (never expires)
/// - Active dev token (expires in 30 days)
/// - Active project token
/// - Inactive token
/// - Expired token
#[ allow( clippy::too_many_lines ) ]
async fn seed_api_tokens( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();
  let day_ms = 24 * 60 * 60 * 1000;

  // Token 1: Admin token (never expires)
  let token_hash_1 = "admin_token_hash_placeholder_aaa111";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_1 )
  .bind( "admin" )
  .bind::< Option< &str > >( None )
  .bind( "Admin Master Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind::< Option< i64 > >( None )  // Never expires
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 2: Developer token (expires in 30 days)
  let token_hash_2 = "dev_token_hash_placeholder_bbb222";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_2 )
  .bind( "developer" )
  .bind::< Option< &str > >( None )
  .bind( "Developer Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind( now_ms + ( 30 * day_ms ) )  // Expires in 30 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 3: Project token
  let token_hash_3 = "project_token_hash_placeholder_ccc333";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_3 )
  .bind( "developer" )
  .bind( "project_alpha" )
  .bind( "Project Alpha Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind::< Option< i64 > >( None )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 4: Inactive token
  let token_hash_4 = "inactive_token_hash_placeholder_ddd444";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_4 )
  .bind( "viewer" )
  .bind::< Option< &str > >( None )
  .bind( "Inactive Token" )
  .bind( 0 )  // Inactive
  .bind( now_ms )
  .bind::< Option< i64 > >( None )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 5: Expired token
  let token_hash_5 = "expired_token_hash_placeholder_eee555";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_5 )
  .bind( "developer" )
  .bind::< Option< &str > >( None )
  .bind( "Expired Token" )
  .bind( 1 )
  .bind( now_ms - ( 60 * day_ms ) )  // Created 60 days ago
  .bind( now_ms - ( 30 * day_ms ) )  // Expired 30 days ago
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 6: Expiring soon (within 7 days)
  let token_hash_6 = "expiring_soon_token_hash_placeholder_fff666";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_6 )
  .bind( "developer" )
  .bind( "project_beta" )
  .bind( "Expiring Soon Token" )
  .bind( 1 )
  .bind( now_ms - ( 23 * day_ms ) )  // Created 23 days ago
  .bind( now_ms + ( 7 * day_ms ) )  // Expires in 7 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 7: Tester token (unlimited user, short expiry)
  let token_hash_7 = "tester_token_hash_placeholder_ggg777";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_7 )
  .bind( "tester" )
  .bind::< Option< &str > >( None )
  .bind( "Tester Token" )
  .bind( 1 )
  .bind( now_ms - ( 7 * day_ms ) )  // Created when user was created
  .bind( now_ms + ( 14 * day_ms ) )  // Expires in 14 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Token 8: Second tester token (for rotation testing)
  let token_hash_8 = "tester_token_2_hash_placeholder_hhh888";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_8 )
  .bind( "tester" )
  .bind( "project_alpha" )
  .bind( "Tester Token 2" )
  .bind( 1 )
  .bind( now_ms - ( 2 * day_ms ) )  // Created 2 days ago
  .bind::< Option< i64 > >( None )  // Never expires
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Note: guest user deliberately has NO tokens (edge case testing)

  Ok( () )
}

/// Seed usage limits with sample data
///
/// Creates 3 limit tiers:
/// - Admin unlimited
/// - Developer standard tier
/// - Free tier
async fn seed_usage_limits( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();

  // Limit 1: Admin unlimited
  sqlx::query(
    "INSERT INTO usage_limits \
     (user_id, project_id, max_tokens_per_day, max_requests_per_minute, \
      max_cost_cents_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_cents_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "admin" )
  .bind::< Option< &str > >( None )
  .bind::< Option< i64 > >( None )  // Unlimited tokens
  .bind::< Option< i64 > >( None )  // Unlimited requests
  .bind::< Option< i64 > >( None )  // Unlimited cost
  .bind( 0 )
  .bind( 0 )
  .bind( 0 )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Limit 2: Developer standard tier
  sqlx::query(
    "INSERT INTO usage_limits \
     (user_id, project_id, max_tokens_per_day, max_requests_per_minute, \
      max_cost_cents_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_cents_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "developer" )
  .bind::< Option< &str > >( None )
  .bind( 1_000_000 )  // 1M tokens/day
  .bind( 60 )  // 60 requests/minute
  .bind( 5000 )  // $50/month
  .bind( 250_000 )  // Current: 250k tokens used today
  .bind( 15 )  // Current: 15 requests this minute
  .bind( 1250 )  // Current: $12.50 this month
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Limit 3: Free tier
  sqlx::query(
    "INSERT INTO usage_limits \
     (user_id, project_id, max_tokens_per_day, max_requests_per_minute, \
      max_cost_cents_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_cents_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "viewer" )
  .bind::< Option< &str > >( None )
  .bind( 100_000 )  // 100k tokens/day
  .bind( 10 )  // 10 requests/minute
  .bind( 0 )  // Free tier
  .bind( 95_000 )  // Current: 95k tokens used (near limit!)
  .bind( 2 )  // Current: 2 requests this minute
  .bind( 0 )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  Ok( () )
}

/// Seed project-provider key assignments
///
/// Creates 2 assignments:
/// - `project_alpha` -> `OpenAI` key
/// - `project_alpha` -> `Anthropic` key
///
/// Note: Dynamically looks up provider key IDs to avoid AUTOINCREMENT issues
async fn seed_project_assignments( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();

  // Get OpenAI key ID
  let openai_key_id: i64 = sqlx::query_scalar(
    "SELECT id FROM ai_provider_keys WHERE provider = 'openai' LIMIT 1"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Get Anthropic key ID
  let anthropic_key_id: i64 = sqlx::query_scalar(
    "SELECT id FROM ai_provider_keys WHERE provider = 'anthropic' LIMIT 1"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Assignment 1: project_alpha -> OpenAI
  sqlx::query(
    "INSERT INTO project_provider_key_assignments \
     (project_id, provider_key_id, assigned_at) \
     VALUES ($1, $2, $3)"
  )
  .bind( "project_alpha" )
  .bind( openai_key_id )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Assignment 2: project_alpha -> Anthropic
  sqlx::query(
    "INSERT INTO project_provider_key_assignments \
     (project_id, provider_key_id, assigned_at) \
     VALUES ($1, $2, $3)"
  )
  .bind( "project_alpha" )
  .bind( anthropic_key_id )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  Ok( () )
}

/// Seed token usage records with realistic patterns
///
/// Creates diverse usage patterns for manual testing:
/// - High usage tokens (near limits)
/// - Low usage tokens
/// - Zero usage tokens (newly created)
/// - Usage across multiple days
/// - Different providers and models
///
/// This helps test quota enforcement, rate limiting, and usage reporting.
#[ allow( clippy::too_many_lines ) ]
async fn seed_token_usage( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();
  let day_ms = 24 * 60 * 60 * 1000;

  // Get token IDs
  let admin_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'admin_token_hash_placeholder_aaa111' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  let dev_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'dev_token_hash_placeholder_bbb222' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  let project_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'project_token_hash_placeholder_ccc333' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  let tester_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'tester_token_hash_placeholder_ggg777' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError )?;

  // Pattern 1: Admin token - moderate usage over 7 days
  if let Some( token_id ) = admin_token_id
  {
    for day_offset in 0..7
    {
      sqlx::query(
        "INSERT INTO token_usage \
         (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
          requests_count, cost_cents, recorded_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
      )
      .bind( token_id )
      .bind( "openai" )
      .bind( "gpt-4" )
      .bind( 500 + ( day_offset * 50 ) )  // Gradually increasing usage
      .bind( 300 + ( day_offset * 30 ) )
      .bind( 800 + ( day_offset * 80 ) )
      .bind( 10 + day_offset )
      .bind( 80 + ( day_offset * 8 ) )
      .bind( now_ms - ( day_offset * day_ms ) )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    }
  }

  // Pattern 2: Developer token - high usage (near limit testing)
  if let Some( token_id ) = dev_token_id
  {
    for day_offset in 0..5
    {
      sqlx::query(
        "INSERT INTO token_usage \
         (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
          requests_count, cost_cents, recorded_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
      )
      .bind( token_id )
      .bind( "anthropic" )
      .bind( "claude-3-5-sonnet" )
      .bind( 8000 + ( day_offset * 1000 ) )  // High usage
      .bind( 2000 + ( day_offset * 500 ) )
      .bind( 10000 + ( day_offset * 1500 ) )
      .bind( 50 + ( day_offset * 10 ) )
      .bind( 500 + ( day_offset * 75 ) )
      .bind( now_ms - ( day_offset * day_ms ) )
      .execute( pool )
      .await
      .map_err( |_| crate::error::TokenError )?;
    }
  }

  // Pattern 3: Project token - sporadic usage
  if let Some( token_id ) = project_token_id
  {
    // Day 0 - high usage
    sqlx::query(
      "INSERT INTO token_usage \
       (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
        requests_count, cost_cents, recorded_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind( token_id )
    .bind( "openai" )
    .bind( "gpt-3.5-turbo" )
    .bind( 3000 )
    .bind( 1000 )
    .bind( 4000 )
    .bind( 25 )
    .bind( 120 )
    .bind( now_ms )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Day 3 - low usage
    sqlx::query(
      "INSERT INTO token_usage \
       (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
        requests_count, cost_cents, recorded_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind( token_id )
    .bind( "openai" )
    .bind( "gpt-3.5-turbo" )
    .bind( 200 )
    .bind( 100 )
    .bind( 300 )
    .bind( 2 )
    .bind( 15 )
    .bind( now_ms - ( 3 * day_ms ) )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
  }

  // Pattern 4: Tester token - mixed provider usage
  if let Some( token_id ) = tester_token_id
  {
    // OpenAI usage
    sqlx::query(
      "INSERT INTO token_usage \
       (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
        requests_count, cost_cents, recorded_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind( token_id )
    .bind( "openai" )
    .bind( "gpt-4" )
    .bind( 1500 )
    .bind( 800 )
    .bind( 2300 )
    .bind( 15 )
    .bind( 230 )
    .bind( now_ms - ( 2 * day_ms ) )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;

    // Anthropic usage
    sqlx::query(
      "INSERT INTO token_usage \
       (token_id, provider, model, input_tokens, output_tokens, total_tokens, \
        requests_count, cost_cents, recorded_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"
    )
    .bind( token_id )
    .bind( "anthropic" )
    .bind( "claude-3-5-haiku" )
    .bind( 5000 )
    .bind( 1000 )
    .bind( 6000 )
    .bind( 30 )
    .bind( 180 )
    .bind( now_ms - day_ms )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError )?;
  }

  // Note: Some tokens deliberately have ZERO usage (newly created tokens, inactive tokens, etc.)
  // This is important for testing edge cases in usage reporting and quota enforcement

  Ok( () )
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use sqlx::SqlitePool;

  #[ tokio::test ]
  async fn test_wipe_database()
  {
    let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
    crate::migrations::apply_all_migrations( &pool ).await.unwrap();

    // Seed database
    seed_all( &pool ).await.unwrap();

    // Verify data exists
    let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert!( user_count > 0, "Users should exist before wipe" );

    // Wipe database
    wipe_database( &pool ).await.unwrap();

    // Verify all tables empty
    let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( user_count, 0, "Users table should be empty after wipe" );

    let token_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM api_tokens" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( token_count, 0, "Tokens table should be empty after wipe" );
  }

  #[ tokio::test ]
  async fn test_seed_all()
  {
    let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
    crate::migrations::apply_all_migrations( &pool ).await.unwrap();

    seed_all( &pool ).await.unwrap();

    // Verify users created
    let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( user_count, 5, "Should create 5 users (admin, developer, viewer, tester, guest)" );

    // Verify provider keys created
    let provider_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM ai_provider_keys" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( provider_count, 2, "Should create 2 provider keys" );

    // Verify tokens created
    let token_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM api_tokens" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( token_count, 8, "Should create 8 tokens (guest user has none)" );

    // Verify usage limits created
    let limit_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM usage_limits" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert_eq!( limit_count, 3, "Should create 3 usage limits" );

    // Verify project assignments created
    let assignment_count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM project_provider_key_assignments"
    )
    .fetch_one( &pool )
    .await
    .unwrap();
    assert_eq!( assignment_count, 2, "Should create 2 project assignments" );

    // Verify usage records created
    let usage_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM token_usage" )
      .fetch_one( &pool )
      .await
      .unwrap();
    assert!( usage_count >= 10, "Should create at least 10 usage records, got {}", usage_count );
  }

  #[ tokio::test ]
  async fn test_seed_users_creates_correct_roles()
  {
    let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
    crate::migrations::apply_all_migrations( &pool ).await.unwrap();

    seed_users( &pool ).await.unwrap();

    // Verify admin role
    let admin_role: String = sqlx::query_scalar(
      "SELECT role FROM users WHERE username = 'admin'"
    )
    .fetch_one( &pool )
    .await
    .unwrap();
    assert_eq!( admin_role, "admin", "Admin should have admin role" );

    // Verify developer role
    let dev_role: String = sqlx::query_scalar(
      "SELECT role FROM users WHERE username = 'developer'"
    )
    .fetch_one( &pool )
    .await
    .unwrap();
    assert_eq!( dev_role, "user", "Developer should have user role" );

    // Verify inactive user
    let viewer_active: i64 = sqlx::query_scalar(
      "SELECT is_active FROM users WHERE username = 'viewer'"
    )
    .fetch_one( &pool )
    .await
    .unwrap();
    assert_eq!( viewer_active, 0, "Viewer should be inactive" );
  }
}
