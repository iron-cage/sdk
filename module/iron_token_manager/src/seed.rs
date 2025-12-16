//! Database seeding utilities for development and testing
//!
//! Provides functions to populate database with realistic sample data for manual testing.
//! Seeding is controlled by the `ENABLE_DEMO_SEED` environment variable:
//!
//! - **Default** (no env var): No seeding - database remains empty
//! - **Demo Mode** (`ENABLE_DEMO_SEED=true`): Seeds `admin@ironcage.ai` with `IronDemo2025!`
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
//! - Seeds only to empty tables (fails if data exists)
//! - Uses predictable test data (not secure for real production)
//! - Provider keys use placeholder encryption

use sqlx::SqlitePool;
use crate::error::Result;

/// Check if demo seed mode is enabled via environment variable
///
/// Returns true if `ENABLE_DEMO_SEED=true` is set, false otherwise.
/// When enabled, seeds database with demo accounts for production demos.
/// When disabled (default), database remains empty - no auto-seeding.
#[must_use]
pub fn is_demo_seed_enabled() -> bool
{
  std::env::var( "ENABLE_DEMO_SEED" )
    .map( |v| v.to_lowercase() == "true" || v == "1" )
    .unwrap_or( false )
}

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
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM api_call_traces" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM audit_log" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM project_provider_key_assignments" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM token_blacklist" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM user_audit_log" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  // Parent tables (referenced by foreign keys)
  sqlx::query( "DELETE FROM api_tokens" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM ai_provider_keys" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM usage_limits" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM users" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

  sqlx::query( "DELETE FROM agents" )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;

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

/// User seed data structure
struct SeedUser
{
  id: &'static str,
  username: &'static str,
  email: &'static str,
  role: &'static str,
  is_active: bool,
  days_ago: i64,
}

/// Get demo seed users for production demo deployments
///
/// Password: `IronDemo2025!`
fn demo_users() -> Vec< SeedUser >
{
  vec![
    SeedUser { id: "user_admin", username: "admin", email: "admin@ironcage.ai", role: "admin", is_active: true, days_ago: 0 },
    SeedUser { id: "user_demo", username: "demo", email: "demo@ironcage.ai", role: "user", is_active: true, days_ago: 0 },
    SeedUser { id: "user_viewer", username: "viewer", email: "viewer@ironcage.ai", role: "user", is_active: true, days_ago: 0 },
    SeedUser { id: "user_tester", username: "tester", email: "tester@ironcage.ai", role: "user", is_active: true, days_ago: 7 },
    SeedUser { id: "user_guest", username: "guest", email: "guest@ironcage.ai", role: "user", is_active: true, days_ago: 0 },
  ]
}

/// Seed users table with demo accounts
///
/// Creates 5 demo users (only when `ENABLE_DEMO_SEED=true`):
/// - `admin@ironcage.ai` (admin, active)
/// - `demo@ironcage.ai` (user, active)
/// - `viewer@ironcage.ai` (user, active)
/// - `tester@ironcage.ai` (user, active)
/// - `guest@ironcage.ai` (user, active)
/// - Password: `IronDemo2025!`
///
/// # Errors
///
/// Returns error if database insertion fails
pub async fn seed_users( pool: &SqlitePool ) -> Result< () >
{
  let now_ms = crate::storage::current_time_ms();
  let day_ms: i64 = 24 * 60 * 60 * 1000;

  // Bcrypt hash of "IronDemo2025!" (cost = 12)
  // Generated with: cargo run --package iron_token_manager --example gen_password_hash
  let password_hash = "$2b$12$AJbkR5cbO1NDN8vXQ2FSr.02E7lvpf6X7fp7yfBkppqHWtHF8vh86";

  for user in demo_users()
  {
    let created_at = now_ms - ( user.days_ago * day_ms );

    sqlx::query(
      "INSERT INTO users (id, username, password_hash, email, role, is_active, created_at) \
       VALUES ($1, $2, $3, $4, $5, $6, $7)"
    )
    .bind( user.id )
    .bind( user.username )
    .bind( password_hash )
    .bind( user.email )
    .bind( user.role )
    .bind( i32::from( user.is_active ) )
    .bind( created_at )
    .execute( pool )
    .await
    .map_err( |_| crate::error::TokenError::Generic )?;
  }

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
  .bind( "user_admin" )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .bind( "user_admin" )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .bind( "user_admin" )
  .bind::< Option< &str > >( None )
  .bind( "Admin Master Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind::< Option< i64 > >( None )  // Never expires
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 2: Developer token (expires in 30 days)
  let token_hash_2 = "dev_token_hash_placeholder_bbb222";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_2 )
  .bind( "user_demo" )
  .bind::< Option< &str > >( None )
  .bind( "Developer Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind( now_ms + ( 30 * day_ms ) )  // Expires in 30 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 3: Project token
  let token_hash_3 = "project_token_hash_placeholder_ccc333";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_3 )
  .bind( "user_demo" )
  .bind( "project_alpha" )
  .bind( "Project Alpha Token" )
  .bind( 1 )
  .bind( now_ms )
  .bind::< Option< i64 > >( None )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 4: Inactive token
  let token_hash_4 = "inactive_token_hash_placeholder_ddd444";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_4 )
  .bind( "user_viewer" )
  .bind::< Option< &str > >( None )
  .bind( "Inactive Token" )
  .bind( 0 )  // Inactive
  .bind( now_ms )
  .bind::< Option< i64 > >( None )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 5: Expired token
  let token_hash_5 = "expired_token_hash_placeholder_eee555";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_5 )
  .bind( "user_demo" )
  .bind::< Option< &str > >( None )
  .bind( "Expired Token" )
  .bind( 1 )
  .bind( now_ms - ( 60 * day_ms ) )  // Created 60 days ago
  .bind( now_ms - ( 30 * day_ms ) )  // Expired 30 days ago
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 6: Expiring soon (within 7 days)
  let token_hash_6 = "expiring_soon_token_hash_placeholder_fff666";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_6 )
  .bind( "user_demo" )
  .bind( "project_beta" )
  .bind( "Expiring Soon Token" )
  .bind( 1 )
  .bind( now_ms - ( 23 * day_ms ) )  // Created 23 days ago
  .bind( now_ms + ( 7 * day_ms ) )  // Expires in 7 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 7: Tester token (unlimited user, short expiry)
  let token_hash_7 = "tester_token_hash_placeholder_ggg777";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_7 )
  .bind( "user_tester" )
  .bind::< Option< &str > >( None )
  .bind( "Tester Token" )
  .bind( 1 )
  .bind( now_ms - ( 7 * day_ms ) )  // Created when user was created
  .bind( now_ms + ( 14 * day_ms ) )  // Expires in 14 days
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Token 8: Second tester token (for rotation testing)
  let token_hash_8 = "tester_token_2_hash_placeholder_hhh888";
  sqlx::query(
    "INSERT INTO api_tokens \
     (token_hash, user_id, project_id, name, is_active, created_at, expires_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7)"
  )
  .bind( token_hash_8 )
  .bind( "user_tester" )
  .bind( "project_alpha" )
  .bind( "Tester Token 2" )
  .bind( 1 )
  .bind( now_ms - ( 2 * day_ms ) )  // Created 2 days ago
  .bind::< Option< i64 > >( None )  // Never expires
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

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
      max_cost_microdollars_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_microdollars_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "user_admin" )
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
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Limit 2: Demo standard tier
  sqlx::query(
    "INSERT INTO usage_limits \
     (user_id, project_id, max_tokens_per_day, max_requests_per_minute, \
      max_cost_microdollars_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_microdollars_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "user_demo" )
  .bind::< Option< &str > >( None )
  .bind( 1_000_000 )  // 1M tokens/day
  .bind( 60 )  // 60 requests/minute
  .bind( 50_000_000_i64 )  // $50/month in microdollars
  .bind( 250_000 )  // Current: 250k tokens used today
  .bind( 15 )  // Current: 15 requests this minute
  .bind( 12_500_000_i64 )  // Current: $12.50 this month in microdollars
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Limit 3: Viewer free tier
  sqlx::query(
    "INSERT INTO usage_limits \
     (user_id, project_id, max_tokens_per_day, max_requests_per_minute, \
      max_cost_microdollars_per_month, current_tokens_today, current_requests_this_minute, \
      current_cost_microdollars_this_month, created_at, updated_at) \
     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)"
  )
  .bind( "user_viewer" )
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
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .map_err( |_| crate::error::TokenError::Generic )?;

  // Get Anthropic key ID
  let anthropic_key_id: i64 = sqlx::query_scalar(
    "SELECT id FROM ai_provider_keys WHERE provider = 'anthropic' LIMIT 1"
  )
  .fetch_one( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .map_err( |_| crate::error::TokenError::Generic )?;

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
  .map_err( |_| crate::error::TokenError::Generic )?;

  let dev_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'dev_token_hash_placeholder_bbb222' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  let project_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'project_token_hash_placeholder_ccc333' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

  let tester_token_id: Option< i64 > = sqlx::query_scalar(
    "SELECT id FROM api_tokens WHERE token_hash = 'tester_token_hash_placeholder_ggg777' LIMIT 1"
  )
  .fetch_optional( pool )
  .await
  .map_err( |_| crate::error::TokenError::Generic )?;

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
      .map_err( |_| crate::error::TokenError::Generic )?;
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
      .map_err( |_| crate::error::TokenError::Generic )?;
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
    .map_err( |_| crate::error::TokenError::Generic )?;

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
    .map_err( |_| crate::error::TokenError::Generic )?;
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
    .map_err( |_| crate::error::TokenError::Generic )?;

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
    .map_err( |_| crate::error::TokenError::Generic )?;
  }

  // Note: Some tokens deliberately have ZERO usage (newly created tokens, inactive tokens, etc.)
  // This is important for testing edge cases in usage reporting and quota enforcement

  Ok( () )
}

