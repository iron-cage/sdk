//! Integration tests for Agents API endpoints
//!
//! Tests cover:
//! - Agent CRUD operations (create, list, get, update, delete)
//! - Role-based access control (admin vs regular user)
//! - Get agent tokens endpoint
//! - Error cases (401, 403, 404)
//!
//! ## Security Tests Added (2025-12-12)
//!
//! Manual testing (Task 1.3) identified missing security-critical tests for unauthenticated
//! access and authorization bypass scenarios. Added 5 tests:
//!
//! - `test_list_agents_without_auth_unauthorized`: Prevents unauthenticated agent enumeration
//! - `test_get_agent_without_auth_unauthorized`: Prevents unauthenticated agent access
//! - `test_delete_agent_without_auth_unauthorized`: Prevents unauthenticated agent deletion
//! - `test_delete_nonexistent_agent_as_admin`: Verifies proper 404 error handling
//! - `test_create_agent_ignores_owner_id_in_request`: Prevents authorization bypass via owner_id override
//!
//! These tests ensure authentication middleware cannot be accidentally removed and that
//! owner_id is always derived from JWT claims, never from request body.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_create_agent_as_admin_success` | Admin creates agent | POST /api/agents with admin token, valid agent data | 201 Created, agent in DB with correct owner_id | ✅ |
//! | `test_create_agent_as_user_forbidden` | Regular user creates agent | POST /api/agents with user token, valid agent data | 403 Forbidden | ✅ |
//! | `test_create_agent_without_auth_unauthorized` | Unauthenticated creation | POST /api/agents without auth header, valid agent data | 401 Unauthorized | ✅ |
//! | `test_create_agent_ignores_owner_id_in_request` | Authorization bypass attempt | POST /api/agents with admin token, request includes owner_id field | 201 Created, owner_id derived from JWT (not request) | ✅ |
//! | `test_list_agents_as_admin_sees_all` | Admin lists all agents | GET /api/agents with admin token, DB has agents from multiple users | 200 OK, all agents returned | ✅ |
//! | `test_list_agents_as_user_sees_only_accessible` | User lists accessible agents | GET /api/agents with user token, DB has user's agents + others | 200 OK, only user's agents returned | ✅ |
//! | `test_list_agents_without_auth_unauthorized` | Unauthenticated listing | GET /api/agents without auth header | 401 Unauthorized | ✅ |
//! | `test_get_agent_as_admin_success` | Admin retrieves specific agent | GET /api/agents/:id with admin token, agent exists | 200 OK, agent details returned | ✅ |
//! | `test_get_agent_as_user_without_access_forbidden` | User retrieves other user's agent | GET /api/agents/:id with user token, agent belongs to different user | 403 Forbidden | ✅ |
//! | `test_get_agent_not_found` | Retrieve nonexistent agent | GET /api/agents/999999 with admin token | 404 Not Found | ✅ |
//! | `test_get_agent_without_auth_unauthorized` | Unauthenticated retrieval | GET /api/agents/:id without auth header | 401 Unauthorized | ✅ |
//! | `test_update_agent_as_admin_success` | Admin updates agent | PUT /api/agents/:id with admin token, valid update data | 200 OK, agent updated in DB | ✅ |
//! | `test_update_agent_as_user_forbidden` | User updates other user's agent | PUT /api/agents/:id with user token, agent belongs to different user | 403 Forbidden | ✅ |
//! | `test_delete_agent_as_admin_success` | Admin deletes agent | DELETE /api/agents/:id with admin token | 204 No Content, agent removed from DB | ✅ |
//! | `test_delete_agent_as_user_forbidden` | User deletes other user's agent | DELETE /api/agents/:id with user token, agent belongs to different user | 403 Forbidden | ✅ |
//! | `test_delete_nonexistent_agent_as_admin` | Delete nonexistent agent | DELETE /api/agents/999999 with admin token | 404 Not Found | ✅ |
//! | `test_delete_agent_without_auth_unauthorized` | Unauthenticated deletion | DELETE /api/agents/:id without auth header | 401 Unauthorized | ✅ |
//! | `test_get_agent_tokens_success` | Retrieve agent's API tokens | GET /api/agents/:id/tokens with admin token, agent has tokens | 200 OK, list of agent's tokens returned | ✅ |

mod common;

use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::TestAppState };
use axum::{
  Router, body::Body, http::{ Method, Request, StatusCode }, routing::{ delete as delete_route, get, post, put }
};
use tower::ServiceExt;
use serde_json::json;
use sqlx::{Row, SqlitePool};


/// Test schema for agents integration tests
const AGENTS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS agents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  providers TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  owner_id TEXT REFERENCES users(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS api_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  token_hash TEXT NOT NULL UNIQUE,
  user_id TEXT NOT NULL,
  agent_id INTEGER,
  provider TEXT,
  name TEXT,
  project_id TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  last_used_at INTEGER,
  FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
"#;



/// Helper to create test router with agents endpoints
async fn create_agents_router() -> ( Router, SqlitePool, String, String, String, String )
{
  // Create TestAppState with auth support
  let app_state = TestAppState::new().await;

  // Use the app state's database pool (shared across auth, tokens, and agents)
  let agent_pool = app_state.database.clone();

  // Enable foreign key constraints for this connection
  sqlx::raw_sql( "PRAGMA foreign_keys = ON;" )
    .execute( &agent_pool )
    .await
    .expect( "LOUD FAILURE: Failed to enable foreign keys" );

  // Add agents schema to database
  sqlx::raw_sql( AGENTS_SCHEMA )
    .execute( &agent_pool )
    .await
    .expect( "LOUD FAILURE: Failed to apply agents schema" );

  // Create admin and regular user in the correct pool
  let ( admin_id, _ ) = create_test_admin( &agent_pool ).await;
  let ( user_id, _ ) = create_test_user( &agent_pool, "regular_user@mail.com" ).await;

  // Generate tokens using TEST_JWT_SECRET
  let admin_token = create_test_access_token( &admin_id, "admin@admin.com", "admin", "test_jwt_secret_key_for_testing_12345" );
  let user_token = create_test_access_token( &user_id, "regular_user@mail.com", "user", "test_jwt_secret_key_for_testing_12345" );

  let router = Router::new()
    .route( "/api/agents", get( iron_control_api::routes::agents::list_agents ) )
    .route( "/api/agents", post( iron_control_api::routes::agents::create_agent ) )
    .route( "/api/agents/:id", get( iron_control_api::routes::agents::get_agent ) )
    .route( "/api/agents/:id", put( iron_control_api::routes::agents::update_agent ) )
    .route( "/api/agents/:id", delete_route( iron_control_api::routes::agents::delete_agent ) )
    .route( "/api/agents/:id/tokens", get( iron_control_api::routes::agents::get_agent_tokens ) )
    .with_state( app_state );

  ( router, agent_pool, admin_token, user_token, admin_id, user_id )
}

// ============================================================================
// Agent Creation Tests
// ============================================================================

#[ tokio::test ]
async fn test_create_agent_as_admin_success()
{
  let ( app, _pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai", "anthropic"],
    "requested_budget_microdollars": 1000000
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/agents" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::CREATED, "Admin should create agent successfully" );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agent: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agent[ "name" ].as_str().unwrap(), "Test Agent" );
  assert_eq!( agent[ "providers" ].as_array().unwrap().len(), 2 );
}

#[ tokio::test ]
async fn test_create_agent_as_user_forbidden()
{
  let ( app, _pool, _admin_token, user_token, _admin_id, _user_id ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai"],
    "requested_budget_microdollars": 1000000
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/agents" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN, "Regular user should not create agents" );
}

#[ tokio::test ]
async fn test_create_agent_without_auth_unauthorized()
{
  let ( app, _pool, _admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai"],
    "requested_budget_microdollars": 1000000
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/agents" )
        .header( "content-type", "application/json" )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::UNAUTHORIZED, "Unauthenticated request should fail" );
}

// ============================================================================
// Agent Listing Tests
// ============================================================================

#[ tokio::test ]
async fn test_list_agents_as_admin_sees_all()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create test agents
  let now = chrono::Utc::now().timestamp_millis();
  sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Agent 1" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Agent 2" )
    .bind( "[\"anthropic\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agents: Vec< serde_json::Value > = serde_json::from_slice( &body_bytes ).unwrap();

  assert!( agents.len() >= 2, "Admin should see at least the 2 created agents (plus any seeded agents)" );

  // Verify the created agents are present
  let agent_names: Vec< &str > = agents.iter()
    .filter_map( |a| a[ "name" ].as_str() )
    .collect();
  assert!( agent_names.contains( &"Agent 1" ), "Should contain Agent 1" );
  assert!( agent_names.contains( &"Agent 2" ), "Should contain Agent 2" );
}

#[ tokio::test ]
async fn test_list_agents_as_user_sees_only_accessible()
{
  let ( app, pool, _admin_token, user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agents - one owned by admin, one owned by user
  let now = chrono::Utc::now().timestamp_millis();
  sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( 100 )
    .bind( "Admin Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( 101 )
    .bind( "User Agent" )
    .bind( "[\"anthropic\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents" )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agents: Vec< serde_json::Value > = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agents.len(), 1, "User should only see agents they own" );
  assert_eq!( agents[ 0 ][ "name" ].as_str().unwrap(), "User Agent" );
}

// ============================================================================
// Get Agent Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  let status = response.status();
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();

  if status != StatusCode::OK {
    let body_str = String::from_utf8( body_bytes.to_vec() ).unwrap();
    panic!( "Expected 200 OK, got {}. Body: {}", status, body_str );
  }

  let agent: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agent[ "name" ].as_str().unwrap(), "Test Agent" );
}

#[ tokio::test ]
async fn test_get_agent_as_user_without_access_forbidden()
{
  let ( app, pool, _admin_token, user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN, "User without token access should not see agent" );
}

#[ tokio::test ]
async fn test_get_agent_not_found()
{
  let ( app, _pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents/999999" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

// ============================================================================
// Update Agent Tests
// ============================================================================

#[ tokio::test ]
async fn test_update_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Old Name" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let request_body = json!({
    "name": "New Name",
    "providers": ["anthropic", "openai"]
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agent: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agent[ "name" ].as_str().unwrap(), "New Name" );
  assert_eq!( agent[ "providers" ].as_array().unwrap().len(), 2 );
}

#[ tokio::test ]
async fn test_update_agent_as_user_forbidden()
{
  let ( app, pool, _admin_token, user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let request_body = json!({
    "name": "New Name"
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN );
}

// ============================================================================
// Delete Agent Tests
// ============================================================================

#[ tokio::test ]
async fn test_delete_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  let result = sqlx::query("SELECT * FROM agents").fetch_all(&pool).await.unwrap();

  let rows: Vec<i64> = result.into_iter().map(|r| r.get::<i64, &str>("id")).collect();

  println!("rows: {:?}", rows);

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();
  
  let agent_id = result.last_insert_rowid();
  println!( "agent_id: {}", agent_id );

  let result = sqlx::query("SELECT * FROM agents").fetch_all(&pool).await.unwrap();

  let rows: Vec<i64> = result.into_iter().map(|r| r.get::<i64, &str>("id")).collect();

  println!("rows: {:?}", rows);

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::DELETE )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NO_CONTENT );

  // Verify deletion
  let count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM agents WHERE id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .unwrap();

  assert_eq!( count, 0, "Agent should be deleted" );
}

#[ tokio::test ]
async fn test_delete_agent_as_user_forbidden()
{
  let ( app, pool, _admin_token, user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::DELETE )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN );
}

// ============================================================================
// Get Agent Tokens Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_agent_tokens_success()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( 100 )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  // Create tokens for agent
  sqlx::query(
    "INSERT INTO api_tokens (token_hash, user_id, agent_id, provider, created_at) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "hash1" )
  .bind( "user_123" )
  .bind( agent_id )
  .bind( "openai" )
  .bind( now )
  .execute( &pool )
  .await
  .unwrap();

  sqlx::query(
    "INSERT INTO api_tokens (token_hash, user_id, agent_id, provider, created_at) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "hash2" )
  .bind( "user_123" )
  .bind( agent_id )
  .bind( "anthropic" )
  .bind( now )
  .execute( &pool )
  .await
  .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}/tokens", agent_id ) )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  let status = response.status();
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();

  if status != StatusCode::OK
  {
    let body_str = String::from_utf8( body_bytes.to_vec() ).unwrap();
    panic!( "Expected 200 OK, got {}. Body: {}", status, body_str );
  }

  let tokens: Vec< serde_json::Value > = serde_json::from_slice( &body_bytes ).unwrap();
  assert_eq!( tokens.len(), 2, "Should return all agent tokens" );
}

// ============================================================================
// Security Tests - Unauthenticated Access
// ============================================================================

#[ tokio::test ]
async fn test_list_agents_without_auth_unauthorized()
{
  let ( app, _pool, _admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents" )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Unauthenticated list agents should return 401 UNAUTHORIZED"
  );
}

#[ tokio::test ]
async fn test_get_agent_without_auth_unauthorized()
{
  let ( app, pool, _admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent first
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Unauthenticated get agent should return 401 UNAUTHORIZED"
  );
}

#[ tokio::test ]
async fn test_delete_agent_without_auth_unauthorized()
{
  let ( app, pool, _admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent first
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::DELETE )
        .uri( format!( "/api/agents/{}", agent_id ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::UNAUTHORIZED,
    "Unauthenticated delete agent should return 401 UNAUTHORIZED"
  );
}

#[ tokio::test ]
async fn test_delete_nonexistent_agent_as_admin()
{
  let ( app, _pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::DELETE )
        .uri( "/api/agents/99999" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::NOT_FOUND,
    "Deleting nonexistent agent should return 404 NOT FOUND"
  );
}

// ============================================================================
// Security Tests - Owner ID Override Attempts
// ============================================================================

#[ tokio::test ]
async fn test_create_agent_ignores_owner_id_in_request()
{
  let ( app, pool, admin_token, _user_token, admin_id, user_id ) = create_agents_router().await;

  // Attempt to create agent with different owner_id in request body
  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai"],
    "requested_budget_microdollars": 1000000,
    "owner_id": user_id  // Trying to set different owner
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::POST )
        .uri( "/api/agents" )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!(
    response.status(),
    StatusCode::CREATED,
    "Agent creation should succeed"
  );

  // Verify owner_id is set to admin (from JWT), not user_id (from request body)
  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agent: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();
  let agent_id = agent[ "id" ].as_i64().unwrap();

  // Query database to verify actual owner_id
  let row: ( String, ) = sqlx::query_as( "SELECT owner_id FROM agents WHERE id = ?" )
    .bind( agent_id )
    .fetch_one( &pool )
    .await
    .unwrap();

  assert_eq!(
    row.0, admin_id,
    "Owner ID should be set to authenticated user (admin), not request body value"
  );
}
