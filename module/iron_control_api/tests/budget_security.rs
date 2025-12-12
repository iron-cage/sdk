//! Protocol 005 security corner case tests
//!
//! Tests security-critical scenarios for Protocol 005 (Budget Control Protocol):
//! - SQL injection protection in provider names, model names, reason fields
//! - Authorization enforcement (IC Token ownership validation)
//! - IP Token replay attack prevention
//! - IC Token signature validation
//! - Provider key authorization and access control
//! - Lease revocation enforcement
//!
//! # Corner Case Coverage
//!
//! Tests address the following critical security gaps from gap analysis:
//! 12. SQL injection in provider name (CRITICAL - security)
//! 13. IC Token from different agent / authorization (CRITICAL - authorization)
//! 14. IP Token replay attack (MEDIUM - cryptographic property)
//! 15. IC Token tampering / invalid signature (CRITICAL - authentication)
//! 16. Provider key mismatch (CRITICAL - authorization)
//! 17. Disabled provider key access (HIGH - access control)
//! 18. Revoked lease usage reporting (HIGH - enforcement)
//! 19. SQL injection in model name (CRITICAL - security)
//! 20. SQL injection in reason field (CRITICAL - security)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_sql_injection_in_provider_name` | SQL injection attack via provider field | POST /api/budget/handshake with provider="openai'; DROP TABLE agents; --" | 400/404 Bad Request, agents table intact | ✅ |
//! | `test_ic_token_authorization_enforcement` | Authorization bypass attempt with different agent's IC Token | Create lease for agent_1, attempt refresh with agent_2's IC Token | 403 Forbidden | ✅ |
//! | `test_ip_token_replay_prevention` | Replay attack prevention | Two handshakes with same IC Token | Each handshake produces unique IP Token (different nonces) | ✅ |
//! | `test_ic_token_invalid_signature` | IC Token signature tampering | Modify IC Token payload, keep signature unchanged | 400/403 Invalid token signature | ✅ |
//! | `test_provider_key_mismatch` | Provider key for wrong provider | Request openai provider with anthropic provider_key_id | 400/403 Provider mismatch | ✅ |
//! | `test_disabled_provider_key_access` | Disabled provider key access | Request with provider_key_id where is_enabled=0 | 403 Provider key disabled | ✅ |
//! | `test_revoked_lease_usage_reporting` | Revoked lease usage reporting | Revoke lease, attempt report usage | 403 Lease revoked | ✅ |
//! | `test_sql_injection_in_model_name` | SQL injection attack via model field | POST /api/budget/report with model="gpt-4'; DROP TABLE budget_leases; --" | 400/200 with SQL injection prevented | ✅ |
//! | `test_sql_injection_in_reason_field` | SQL injection attack via reason field | POST /api/budget/refresh with reason="Need more'; DROP TABLE agents; --" | 400/200 with SQL injection prevented | ✅ |

use axum::
{
  body::Body,
  http::{ Request, StatusCode },
  Router,
};
use base64::Engine;
use iron_control_api::
{
  ic_token::{ IcTokenClaims, IcTokenManager },
  routes::budget::{ BudgetState, handshake, refresh_budget, report_usage },
};
use iron_token_manager::lease_manager::LeaseManager;
use serde_json::json;
use sqlx::SqlitePool;
use std::sync::Arc;
use tower::ServiceExt;

/// Helper: Create test database with all migrations
async fn setup_test_db() -> SqlitePool
{
  let pool = SqlitePool::connect( "sqlite::memory:" ).await.unwrap();
  iron_token_manager::migrations::apply_all_migrations( &pool )
    .await
    .expect("LOUD FAILURE: Failed to apply migrations");
  pool
}

/// Helper: Create test BudgetState
async fn create_test_budget_state( pool: SqlitePool ) -> BudgetState
{
  let ic_token_secret = "test_secret_key_12345".to_string();
  let ip_token_key : [ u8; 32 ] = [ 0u8; 32 ];

  let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
  let ip_token_crypto = Arc::new(
    iron_control_api::ip_token::IpTokenCrypto::new( &ip_token_key ).unwrap()
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
    db_pool: pool,
    jwt_secret,
  }
}

/// Helper: Generate IC Token for test agent
fn create_ic_token( agent_id: i64, manager: &IcTokenManager ) -> String
{
  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    None,
  );

  manager.generate_token( &claims ).expect("LOUD FAILURE: Should generate IC Token")
}

/// Helper: Seed agent with specific budget and provider key
///
/// # Fix(issue-database-state-unique-001)
/// Root cause: Hardcoded agent_id=1 and provider_key id=1 conflicted with migration 017 seeded data
/// Pitfall: Always use unique IDs for test data; use agent_id > 100 and provider_key id = agent_id * 1000 to avoid conflicts
async fn seed_agent_with_budget( pool: &SqlitePool, agent_id: i64, budget_usd: f64 )
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
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();

  // Insert agent budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0.0, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( budget_usd )
  .bind( budget_usd )
  .bind( now_ms )
  .bind( now_ms )
  .execute( pool )
  .await
  .unwrap();

  // Insert into ai_provider_keys (actual table name from migration 004)
  // Use unique provider key ID based on agent_id to avoid conflicts between tests
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 )  // Unique provider key ID per test (e.g., agent 114 → key 114000)
  .bind( "openai" )
  .bind( "encrypted_test_key_base64" )
  .bind( "test_nonce_base64" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( pool )
  .await
  .unwrap();
}

/// Helper: Create router for budget endpoints
async fn create_budget_router( state: BudgetState ) -> Router
{
  Router::new()
    .route( "/api/budget/handshake", axum::routing::post( handshake ) )
    .route( "/api/budget/report", axum::routing::post( report_usage ) )
    .route( "/api/budget/refresh", axum::routing::post( refresh_budget ) )
    .with_state( state )
}

/// Test 12: SQL injection protection in provider name
///
/// # Corner Case
/// Malicious provider name attempting SQL injection: `"openai'; DROP TABLE agents; --"`
///
/// # Expected Behavior
/// Parameterized queries prevent injection, return validation error instead of executing SQL
///
/// # Priority
/// CRITICAL - Security vulnerability prevention
#[ tokio::test ]
async fn test_sql_injection_in_provider_name()
{
  let pool = setup_test_db().await;
  let agent_id = 201;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Attempt SQL injection via provider field
  let malicious_provider = "openai'; DROP TABLE agents; --";

  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": malicious_provider
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return validation error (400) not execute SQL
  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::NOT_FOUND,
    "SQL injection should be prevented, got status: {}", response.status()
  );

  // Verify agents table still exists (injection failed)
  let agent_count : i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM agents" )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: agents table should still exist");

  // Expect 2 agents: migration 017 seeds agent_id=1, test seeds agent_id=201
  assert_eq!( agent_count, 2, "agents table should be intact (SQL injection prevented)" );
}

/// Test 13: Authorization enforcement - IC Token from different agent
///
/// # Corner Case
/// IC Token contains agent_id=123, but refresh request is for lease owned by agent_id=456
///
/// # Expected Behavior
/// HTTP 403 Forbidden "Unauthorized - lease belongs to different agent"
///
/// # Priority
/// CRITICAL - Authorization bypass prevention
#[ tokio::test ]
async fn test_ic_token_authorization_enforcement()
{
  let pool = setup_test_db().await;
  let agent_1 = 202;  // Use ID > 100 to avoid migration 017 conflict
  let agent_2 = 203;  // Use ID > 100 to avoid migration 017 conflict

  // Seed both agents with budgets
  seed_agent_with_budget( &pool, agent_1, 100.0 ).await;
  seed_agent_with_budget( &pool, agent_2, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Create IC Token for agent 1
  let ic_token_agent_1 = create_ic_token( agent_1, &state.ic_token_manager );

  // Create lease for agent 1
  let app = create_budget_router( state.clone() ).await;
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token_agent_1.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Attempt to refresh agent 1's lease using agent 2's IC Token (authorization violation)
  let ic_token_agent_2 = create_ic_token( agent_2, &state.ic_token_manager );

  let app2 = create_budget_router( state ).await;
  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token_agent_2,
        "current_lease_id": lease_id
      }).to_string()
    ))
    .unwrap();

  let refresh_response = app2.oneshot( refresh_request ).await.unwrap();

  // Should return 403 Forbidden (authorization violation)
  assert_eq!(
    refresh_response.status(), StatusCode::FORBIDDEN,
    "Should reject refresh from different agent's IC Token"
  );
}

/// Test 14: IP Token replay attack prevention
///
/// # Corner Case
/// Same IP Token used multiple times (replay attack)
///
/// # Expected Behavior
/// Each handshake produces unique IP Token (different nonce), preventing replay
///
/// # Priority
/// MEDIUM - Cryptographic security property
#[ tokio::test ]
async fn test_ip_token_replay_prevention()
{
  let pool = setup_test_db().await;
  let agent_id = 204;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with sufficient budget for multiple handshakes
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Perform first handshake
  let app1 = create_budget_router( state.clone() ).await;
  let request1 = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response1 = app1.oneshot( request1 ).await.unwrap();
  assert_eq!( response1.status(), StatusCode::OK );

  let body1 = axum::body::to_bytes( response1.into_body(), usize::MAX ).await.unwrap();
  let data1 : serde_json::Value = serde_json::from_slice( &body1 ).unwrap();
  let ip_token_1 = data1[ "ip_token" ].as_str().unwrap();

  // Perform second handshake with same IC Token
  let app2 = create_budget_router( state ).await;
  let request2 = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response2 = app2.oneshot( request2 ).await.unwrap();
  assert_eq!( response2.status(), StatusCode::OK );

  let body2 = axum::body::to_bytes( response2.into_body(), usize::MAX ).await.unwrap();
  let data2 : serde_json::Value = serde_json::from_slice( &body2 ).unwrap();
  let ip_token_2 = data2[ "ip_token" ].as_str().unwrap();

  // IP Tokens should be different (unique nonce per encryption)
  assert_ne!(
    ip_token_1, ip_token_2,
    "Each handshake should produce unique IP Token (prevents replay attacks)"
  );
}

/// Test 15: IC Token signature tampering
///
/// # Corner Case
/// Malicious IC Token with modified payload but unchanged signature
///
/// # Expected Behavior
/// JWT signature validation fails, return 400/403 Invalid token
///
/// # Priority
/// CRITICAL - Authentication bypass prevention
#[ tokio::test ]
async fn test_ic_token_invalid_signature()
{
  let pool = setup_test_db().await;
  let agent_id = 205;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;

  // Create valid IC Token
  let valid_ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Tamper with token by modifying payload (change agent_id claim)
  // JWT format: header.payload.signature
  let parts : Vec<&str> = valid_ic_token.split( '.' ).collect();
  assert_eq!( parts.len(), 3, "JWT should have 3 parts" );

  // Create tampered token: modify payload (decode, change, encode), keep original signature
  let tampered_payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
    .encode( b"{\"agent_id\":\"agent_999\",\"budget_id\":\"budget_999\"}" );
  let tampered_token = format!( "{}.{}.{}", parts[ 0 ], tampered_payload, parts[ 2 ] );

  let app = create_budget_router( state ).await;

  // Attempt handshake with tampered IC Token
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": tampered_token,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return authentication error (400/401/403)
  assert!(
    response.status() == StatusCode::BAD_REQUEST
      || response.status() == StatusCode::UNAUTHORIZED
      || response.status() == StatusCode::FORBIDDEN,
    "Tampered IC Token should be rejected, got status: {}", response.status()
  );
}

/// Test 16: Provider key mismatch
///
/// # Corner Case
/// Request openai provider using anthropic provider_key_id
///
/// # Expected Behavior
/// HTTP 400/403 Provider key mismatch error
///
/// # Priority
/// CRITICAL - Authorization, prevents credential leakage across providers
#[ tokio::test ]
async fn test_provider_key_mismatch()
{
  let pool = setup_test_db().await;
  let agent_id = 206;  // Use ID > 100 to avoid migration 017 conflict
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  // Insert additional provider key for anthropic (ID = agent_id * 1000 + 1)
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 + 1 )
  .bind( "anthropic" )
  .bind( "encrypted_anthropic_key_base64" )
  .bind( "anthropic_nonce_base64" )
  .bind( 1 )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Attempt handshake: request openai provider with anthropic provider_key_id
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai",
        "provider_key_id": agent_id * 1000 + 1  // anthropic key ID
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return validation error (400/403)
  assert!(
    response.status() == StatusCode::BAD_REQUEST || response.status() == StatusCode::FORBIDDEN,
    "Provider key mismatch should be rejected, got status: {}", response.status()
  );
}

/// Test 17: Disabled provider key access
///
/// # Corner Case
/// Request with provider_key_id where is_enabled=0
///
/// # Expected Behavior
/// HTTP 403 Forbidden "Provider key is disabled"
///
/// # Priority
/// HIGH - Access control enforcement
#[ tokio::test ]
async fn test_disabled_provider_key_access()
{
  let pool = setup_test_db().await;
  let agent_id = 207;  // Use ID > 100 to avoid migration 017 conflict
  let now_ms = chrono::Utc::now().timestamp_millis();

  // Create test user
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
  .execute( &pool )
  .await
  .unwrap();

  // Insert agent
  sqlx::query(
    "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( format!( "test_agent_{}", agent_id ) )
  .bind( serde_json::to_string( &vec![ "openai" ] ).unwrap() )
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  // Insert agent budget
  sqlx::query(
    "INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
     VALUES (?, ?, 0.0, ?, ?, ?)"
  )
  .bind( agent_id )
  .bind( 100.0 )
  .bind( 100.0 )
  .bind( now_ms )
  .bind( now_ms )
  .execute( &pool )
  .await
  .unwrap();

  // Insert DISABLED provider key (is_enabled = 0)
  sqlx::query(
    "INSERT INTO ai_provider_keys (id, provider, encrypted_api_key, encryption_nonce, is_enabled, created_at, user_id)
     VALUES (?, ?, ?, ?, ?, ?, ?)"
  )
  .bind( agent_id * 1000 )
  .bind( "openai" )
  .bind( "encrypted_test_key_base64" )
  .bind( "test_nonce_base64" )
  .bind( 0 )  // DISABLED
  .bind( now_ms )
  .bind( "test_user" )
  .execute( &pool )
  .await
  .unwrap();

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );
  let app = create_budget_router( state ).await;

  // Attempt handshake with disabled provider key
  let request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "provider": "openai",
        "provider_key_id": agent_id * 1000
      }).to_string()
    ))
    .unwrap();

  let response = app.oneshot( request ).await.unwrap();

  // Should return 403 Forbidden
  assert_eq!(
    response.status(), StatusCode::FORBIDDEN,
    "Disabled provider key should be rejected"
  );
}

/// Test 18: Revoked lease usage reporting
///
/// # Corner Case
/// Attempt to report usage on revoked lease
///
/// # Expected Behavior
/// HTTP 403 Forbidden "Lease has been revoked"
///
/// # Priority
/// HIGH - Enforcement of lease lifecycle
#[ tokio::test ]
async fn test_revoked_lease_usage_reporting()
{
  let pool = setup_test_db().await;
  let agent_id = 208;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Create lease via handshake
  let app = create_budget_router( state.clone() ).await;
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Revoke the lease (set lease_status = 'revoked')
  sqlx::query( "UPDATE budget_leases SET lease_status = 'revoked' WHERE id = ?" )
    .bind( &lease_id )
    .execute( &pool )
    .await
    .unwrap();

  // Attempt to report usage on revoked lease
  let app2 = create_budget_router( state ).await;
  let report_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_001",
        "tokens": 150,
        "cost_usd": 0.05,
        "model": "gpt-4",
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let report_response = app2.oneshot( report_request ).await.unwrap();

  // Should return 403 Forbidden
  assert_eq!(
    report_response.status(), StatusCode::FORBIDDEN,
    "Revoked lease should reject usage reporting"
  );
}

/// Test 19: SQL injection in model name
///
/// # Corner Case
/// Malicious model name attempting SQL injection: `"gpt-4'; DROP TABLE budget_leases; --"`
///
/// # Expected Behavior
/// Parameterized queries prevent injection, usage recorded correctly or validation error
///
/// # Priority
/// CRITICAL - Security vulnerability prevention
#[ tokio::test ]
async fn test_sql_injection_in_model_name()
{
  let pool = setup_test_db().await;
  let agent_id = 209;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Create lease via handshake
  let app = create_budget_router( state.clone() ).await;
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Attempt SQL injection via model field
  let malicious_model = "gpt-4'; DROP TABLE budget_leases; --";

  let app2 = create_budget_router( state ).await;
  let report_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/report" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "lease_id": lease_id,
        "request_id": "req_test_sql_injection",
        "tokens": 150,
        "cost_usd": 0.05,
        "model": malicious_model,
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let report_response = app2.oneshot( report_request ).await.unwrap();

  // Should accept (200) or reject with validation error (400/422), NOT execute SQL
  assert!(
    report_response.status() == StatusCode::OK
      || report_response.status() == StatusCode::BAD_REQUEST
      || report_response.status() == StatusCode::UNPROCESSABLE_ENTITY,
    "SQL injection should be prevented, got status: {}", report_response.status()
  );

  // Verify budget_leases table still exists (injection failed)
  let lease_count : i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM budget_leases" )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: budget_leases table should still exist");

  assert!(
    lease_count >= 1,
    "budget_leases table should be intact (SQL injection prevented)"
  );
}

/// Test 20: SQL injection in reason field
///
/// # Corner Case
/// Malicious reason field attempting SQL injection: `"Need more'; DROP TABLE agents; --"`
///
/// # Expected Behavior
/// Parameterized queries prevent injection, refresh succeeds or validation error
///
/// # Priority
/// CRITICAL - Security vulnerability prevention
#[ tokio::test ]
async fn test_sql_injection_in_reason_field()
{
  let pool = setup_test_db().await;
  let agent_id = 210;  // Use ID > 100 to avoid migration 017 conflict

  // Seed agent with budget
  seed_agent_with_budget( &pool, agent_id, 100.0 ).await;

  let state = create_test_budget_state( pool.clone() ).await;
  let ic_token = create_ic_token( agent_id, &state.ic_token_manager );

  // Create lease via handshake
  let app = create_budget_router( state.clone() ).await;
  let handshake_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/handshake" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token.clone(),
        "provider": "openai"
      }).to_string()
    ))
    .unwrap();

  let handshake_response = app.oneshot( handshake_request ).await.unwrap();
  assert_eq!( handshake_response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( handshake_response.into_body(), usize::MAX ).await.unwrap();
  let handshake_data : serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let lease_id = handshake_data[ "lease_id" ].as_str().unwrap().to_string();

  // Attempt SQL injection via reason field
  let malicious_reason = "Need more'; DROP TABLE agents; --";

  let app2 = create_budget_router( state ).await;
  let refresh_request = Request::builder()
    .method( "POST" )
    .uri( "/api/budget/refresh" )
    .header( "content-type", "application/json" )
    .body( Body::from(
      json!({
        "ic_token": ic_token,
        "current_lease_id": lease_id,
        "reason": malicious_reason
      }).to_string()
    ))
    .unwrap();

  let refresh_response = app2.oneshot( refresh_request ).await.unwrap();

  // Should accept (200) or reject with validation error (400), NOT execute SQL
  assert!(
    refresh_response.status() == StatusCode::OK || refresh_response.status() == StatusCode::BAD_REQUEST,
    "SQL injection should be prevented, got status: {}", refresh_response.status()
  );

  // Verify agents table still exists (injection failed)
  let agent_count : i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM agents" )
    .fetch_one( &pool )
    .await
    .expect("LOUD FAILURE: agents table should still exist");

  // Expect at least 2 agents: migration 017 seeds agent_id=1, test seeds agent_id=210
  assert!(
    agent_count >= 2,
    "agents table should be intact (SQL injection prevented)"
  );
}
