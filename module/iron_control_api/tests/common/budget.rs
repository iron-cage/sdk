//! Budget test infrastructure for Protocol 005 testing.
//!
//! Provides shared utilities for budget corner case, security, and concurrency tests:
//! - In-memory database setup with token_manager migrations
//! - BudgetState builder
//! - Agent/budget seeding
//! - IC Token generation
//! - Budget router creation
//!
//! # Authority
//! organizational_principles.rulebook.md § Anti-Duplication Principle

use axum::Router;
use iron_control_api::
{
  ic_token::{ IcTokenClaims, IcTokenManager },
  routes::budget::{ BudgetState, handshake, report_usage, refresh_budget, return_budget },
};
use iron_token_manager::lease_manager::LeaseManager;
use sqlx::SqlitePool;
use std::sync::Arc;

/// Helper: Create test database with all migrations
///
/// Creates in-memory SQLite database and applies all token_manager migrations
/// required for Protocol 005 budget testing.
#[ allow( dead_code ) ]
pub async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect("LOUD FAILURE: Failed to apply migrations");
  pool
}

/// Helper: Create test BudgetState
///
/// Builds BudgetState with all required managers for budget endpoint testing:
/// - IC Token manager (JWT generation/validation)
/// - IP Token crypto (lease encryption)
/// - Provider key crypto (provider key decryption)
/// - Lease manager (budget lease tracking)
/// - Agent budget manager (budget accounting)
/// - Provider key storage (API key management)
/// - JWT secret (user authentication)
#[ allow( dead_code ) ]
pub async fn create_test_budget_state( pool: SqlitePool ) -> BudgetState
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];
  let provider_key_master : [ u8; 32 ] = [ 42u8; 32 ]; // Test master key for provider keys

  let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
  );
  let provider_key_crypto = Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master ).unwrap()
  );
  let crypto_service = Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master ).unwrap()
  );
  let lease_manager = Arc::new( LeaseManager::from_pool( pool.clone() ) );
  let agent_budget_manager = Arc::new(
    iron_token_manager::agent_budget::AgentBudgetManager::from_pool( pool.clone() )
  );
  let provider_key_storage = Arc::new(
    iron_token_manager::provider_key_storage::ProviderKeyStorage::new( pool.clone() )
  );
  let jwt_secret = Arc::new( iron_control_api::jwt_auth::JwtSecret::new( "test_jwt_secret".to_string() ) );

  BudgetState
  {
    ic_token_manager,
    ip_token_crypto,
    lease_manager,
    agent_budget_manager,
    provider_key_storage,
    provider_key_crypto,
    db_pool: pool,
    jwt_secret,
    crypto_service: Some( crypto_service ),
  }
}

/// Helper: Create test budget state without crypto service
///
/// Use this for testing crypto unavailable scenarios.
#[ allow( dead_code ) ]
pub async fn create_test_budget_state_no_crypto( pool: SqlitePool ) -> BudgetState
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];
  let provider_key_master : [ u8; 32 ] = [ 42u8; 32 ];

  let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
  );
  let provider_key_crypto = Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master ).unwrap()
  );
  let lease_manager = Arc::new( LeaseManager::from_pool( pool.clone() ) );
  let agent_budget_manager = Arc::new(
    iron_token_manager::agent_budget::AgentBudgetManager::from_pool( pool.clone() )
  );
  let provider_key_storage = Arc::new(
    iron_token_manager::provider_key_storage::ProviderKeyStorage::new( pool.clone() )
  );
  let jwt_secret = Arc::new( iron_control_api::jwt_auth::JwtSecret::new( "test_jwt_secret".to_string() ) );

  BudgetState
  {
    ic_token_manager,
    ip_token_crypto,
    lease_manager,
    agent_budget_manager,
    provider_key_storage,
    provider_key_crypto,
    db_pool: pool,
    jwt_secret,
    crypto_service: None,
  }
}

/// Helper: Generate IC Token for test agent
///
/// Creates IC Token with standard claims for testing.
#[ allow( dead_code ) ]
pub fn create_ic_token( agent_id: i64, manager: &IcTokenManager ) -> String
{
  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    None,
  );

  manager.generate_token( &claims ).expect("LOUD FAILURE: Should generate IC Token")
}

/// Helper: Seed agent with budget and provider key
///
/// Creates test data for budget testing:
/// - Test user (required for owner_id foreign key)
/// - Agent record
/// - Agent budget allocation
/// - Provider API key
/// - Usage limits
///
/// # Fix(issue-concurrency-001)
/// Root cause: Hardcoded agent_id=1 and provider_key id=1 conflicted with migration 017 seeded data
/// Pitfall: Always use unique IDs for test data; use agent_id > 100 and provider_key id = agent_id * 1000 to avoid conflicts
#[ allow( dead_code ) ]
pub async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_microdollars: i64 )
{
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user if it doesn't exist (required for owner_id foreign key)
  sqlx::query(
    "INSERT OR IGNORE INTO users (id, username, password_hash, email, role, is_active, created_at)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( "test_username" )
  .bind( "$2b$12$test_password_hash" )
  .bind( "test@example.com" )
  .bind( "admin" )
  .bind( 1 )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent with owner_id
  sqlx::query(
    "INSERT OR IGNORE INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent budget (using microdollars)
  sqlx::query(
    "INSERT OR IGNORE INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( budget_microdollars )
  .bind( budget_microdollars )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert provider key
  // Use unique provider key ID based on agent_id to avoid conflicts between tests
  // Create real encrypted provider key for testing
  let test_provider_key = format!( "sk-test_key_for_agent_{}", agent_id );
  let provider_key_master : [ u8; 32 ] = [ 42u8; 32 ]; // Test master key (must match create_test_budget_state)
  let crypto_service = iron_secrets::crypto::CryptoService::new( &provider_key_master )
    .expect( "LOUD FAILURE: Should create crypto service" );
  let encrypted = crypto_service.encrypt( &test_provider_key )
    .expect( "LOUD FAILURE: Should encrypt provider key" );

  sqlx::query(
    "INSERT OR IGNORE INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 )
  .bind( "openai" )
  .bind( encrypted.ciphertext_base64() )
  .bind( encrypted.nonce_base64() )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert usage_limits for test_user (required for budget validation)
  sqlx::query(
    "INSERT OR IGNORE INTO usage_limits (user_id, max_cost_microdollars_per_month, current_cost_microdollars_this_month, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "test_user" )
  .bind( 10_000_000_000_i64 )  // $10,000 USD limit (in microdollars)
  .bind( 0i64 )                 // No spending yet
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();
}

/// Helper: Create router for budget endpoints
///
/// Builds Axum router with all budget endpoints mounted for testing.
#[ allow( dead_code ) ]
pub async fn create_budget_router( state: BudgetState ) -> Router
{
  use iron_control_api::routes::budget::request_workflow::{
    approve_budget_request,
    reject_budget_request,
  };

  Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .route( "/api/budget/return", axum::routing::post( return_budget ) )
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( approve_budget_request ) )
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( reject_budget_request ) )
    .with_state( state )
}
