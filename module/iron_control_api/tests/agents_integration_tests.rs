//! Integration tests for Agents API endpoints
//!
//! Tests cover:
//! - Agent CRUD operations (create, list, get, update, delete)
//! - Role-based access control (admin vs regular user)
//! - Get agent tokens endpoint
//! - Error cases (401, 403, 404)

mod common;

use common::{ create_test_user, create_test_admin, create_test_access_token, test_state::TestAppState };
use axum::{
  Router,
  routing::{ get, post, put, delete as delete_route },
  http::{ StatusCode, Request, Method },
  body::Body,
};
use tower::ServiceExt;
use serde_json::json;
use sqlx::SqlitePool;

/// Test schema for agents integration tests
const AGENTS_SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS agents (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL,
  providers TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS api_tokens (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  token_hash TEXT NOT NULL UNIQUE,
  user_id TEXT NOT NULL,
  agent_id INTEGER,
  provider TEXT,
  name TEXT,
  is_active INTEGER NOT NULL DEFAULT 1,
  created_at INTEGER NOT NULL,
  last_used_at INTEGER,
  FOREIGN KEY(agent_id) REFERENCES agents(id) ON DELETE CASCADE
);
"#;

/// Helper to create test router with agents endpoints
async fn create_agents_router() -> ( Router, SqlitePool, String, String )
{
  // Create TestAppState with auth support
  let app_state = TestAppState::new().await;

  // Add agents schema to database
  sqlx::raw_sql( AGENTS_SCHEMA )
    .execute( &app_state.database )
    .await
    .expect( "LOUD FAILURE: Failed to apply agents schema" );

  // Create admin and regular user
  let ( admin_id, _ ) = create_test_admin( &app_state.database ).await;
  let ( user_id, _ ) = create_test_user( &app_state.database, "regular_user@mail.com" ).await;

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
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token )
}

// ============================================================================
// Agent Creation Tests
// ============================================================================

#[ tokio::test ]
async fn test_create_agent_as_admin_success()
{
  let ( app, _pool, admin_token, _user_token ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai", "anthropic"]
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
  let ( app, _pool, _admin_token, user_token ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai"]
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
  let ( app, _pool, _admin_token, _user_token ) = create_agents_router().await;

  let request_body = json!({
    "name": "Test Agent",
    "providers": ["openai"]
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
  let ( app, pool, admin_token, _user_token ) = create_agents_router().await;

  // Create test agents
  let now = chrono::Utc::now().timestamp_millis();
  sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Agent 1" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Agent 2" )
    .bind( "[\"anthropic\"]" )
    .bind( now )
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

  assert_eq!( agents.len(), 2, "Admin should see all agents" );
}

#[ tokio::test ]
async fn test_list_agents_as_user_sees_only_accessible()
{
  let ( app, pool, _admin_token, user_token ) = create_agents_router().await;

  // Create agents
  let now = chrono::Utc::now().timestamp_millis();
  sqlx::query( "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( 1 )
    .bind( "Agent 1" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  sqlx::query( "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( 2 )
    .bind( "Agent 2" )
    .bind( "[\"anthropic\"]" )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create token for user only on Agent 1
  sqlx::query(
    "INSERT INTO api_tokens (token_hash, user_id, agent_id, provider, created_at) VALUES (?, ?, ?, ?, ?)"
  )
  .bind( "hash123" )
  .bind( "user_123" )
  .bind( 1 )
  .bind( "openai" )
  .bind( now )
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

  assert_eq!( agents.len(), 1, "User should only see agents they have tokens for" );
  assert_eq!( agents[ 0 ][ "name" ].as_str().unwrap(), "Agent 1" );
}

// ============================================================================
// Get Agent Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
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

  assert_eq!( response.status(), StatusCode::OK );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let agent: serde_json::Value = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agent[ "name" ].as_str().unwrap(), "Test Agent" );
}

#[ tokio::test ]
async fn test_get_agent_as_user_without_access_forbidden()
{
  let ( app, pool, _admin_token, user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
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
  let ( app, _pool, admin_token, _user_token ) = create_agents_router().await;

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
  let ( app, pool, admin_token, _user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Old Name" )
    .bind( "[\"openai\"]" )
    .bind( now )
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
  let ( app, pool, _admin_token, user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
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
  let ( app, pool, admin_token, _user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

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
  let ( app, pool, _admin_token, user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at) VALUES (?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
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
  let ( app, pool, admin_token, _user_token ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at) VALUES (?, ?, ?, ?)" )
    .bind( 1 )
    .bind( "Test Agent" )
    .bind( "[\"openai\"]" )
    .bind( now )
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
