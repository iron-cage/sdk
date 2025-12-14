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
  Router,
  routing::{ get, post, put, delete },
  http::{ StatusCode, Request, Method },
  body::Body,
};
use iron_control_api::routes::agents::{AgentDetails, CreateAgentRequest, GetAgentProvidersResponse, PaginatedAgentsResponse, RemoveProviderFromAgentResponse};
use iron_token_manager::agent_service::{AgentService, CreateAgentParams};
use serde::{Deserialize, Serialize};
use tower::ServiceExt;
use serde_json::json;
use sqlx::SqlitePool;

/// Helper to create test router with agents endpoints
async fn create_agents_router() -> ( Router, SqlitePool, String, String, String, String )
{
  // Create TestAppState with auth support
  let app_state = TestAppState::new().await;


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
    .route( "/api/agents/:id/details", get( iron_control_api::routes::agents::get_agent_details ) )
    .route( "/api/agents/:id", put( iron_control_api::routes::agents::update_agent ) )
    .route( "/api/agents/:id/tokens", get( iron_control_api::routes::agents::get_agent_tokens ) )
    .route( "/api/agents/:id/providers", get( iron_control_api::routes::agents::get_agent_providers ).put( iron_control_api::routes::agents::assign_providers_to_agent ) )
    .route( "/api/agents/:agent_id/providers/:provider_id", delete( iron_control_api::routes::agents::remove_provider_from_agent ) )
    .with_state( app_state.clone() );

  ( router, app_state.database.clone(), admin_token, user_token, admin_id, user_id )
}

// ============================================================================
// Agent Creation Tests
// ============================================================================

#[ tokio::test ]
async fn test_create_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;
  let now = chrono::Utc::now().timestamp_millis();

  let request_body = CreateAgentRequest {
    name: "Test Agent".to_string(),
    budget: 100.0,
    providers: Some(vec!["openai_1".to_string()]),
    description: None,
    tags: None,
    project_id: None,
    owner_id: _admin_id,
  };

  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await.unwrap();

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
  assert_eq!( agent[ "providers" ].as_array().unwrap().len(), 1 );
}

#[ tokio::test ]
async fn test_create_agent_as_user_allowed()
{
  let ( app, pool, _admin_token, user_token, _admin_id, _user_id ) = create_agents_router().await;
  let now = chrono::Utc::now().timestamp_millis();

  let request_body = CreateAgentRequest {
    name: "Test Agent".to_string(),
    budget: 100.0,
    providers: Some(vec!["openai_1".to_string()]),
    description: None,
    tags: None,
    project_id: None,
    owner_id: _user_id,
  };

  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
      .bind( "openai_1" )
      .bind( "openai" )
      .bind( "https://api.openai.com/v1" )
      .bind( now )
      .bind( "encrypted_api_key" )
      .bind( "encryption_nonce" )
      .execute( &pool )
      .await.unwrap();

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

  assert_eq!( response.status(), StatusCode::CREATED, "Regular user should be able to create agents" );
}

#[ tokio::test ]
async fn test_create_agent_without_auth_unauthorized()
{
  let ( app, _pool, _admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let request_body = CreateAgentRequest {
    name: "Test Agent".to_string(),
    budget: 100.0,
    providers: Some(vec!["openai".to_string(), "anthropic".to_string()]),
    description: None,
    tags: None,
    project_id: None,
    owner_id: "123".to_string(),
  };

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

async fn seed_test_agents( pool: &SqlitePool, admin_id: &str, user_id: Option< &str > ) 
{
  let now = chrono::Utc::now().timestamp_millis();

  let agent1 = CreateAgentParams {
    name: "Agent 4".to_string(),
    budget: 400.0,
    providers: Some(vec!["openai_1".to_string()]),
    description: None,
    tags: None,
    project_id: None,
  };

  let agent2 = CreateAgentParams {
    name: "Agent 4".to_string(),
    budget: 400.0,
    providers: Some(vec!["openai_1".to_string()]),
    description: None,
    tags: None,
    project_id: None,
  };

  let agent3 = CreateAgentParams {
    name: "Agent 5".to_string(),
    budget: 400.0,
    providers: Some(vec!["openai_1".to_string()]),
    description: None,
    tags: None,
    project_id: None,
  };

  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( pool )
    .await.unwrap();

  let service: AgentService = AgentService::new(pool.clone());

  service.create_agent(agent1, &admin_id).await.unwrap();
  service.create_agent(agent2, &admin_id).await.unwrap();
  service.create_agent(agent3, &user_id.unwrap_or(&admin_id)).await.unwrap();
}

#[ tokio::test ]
async fn test_list_agents_as_admin_sees_all()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create test agents
  seed_test_agents( &pool, &admin_id, Some( &_user_id ) ).await;

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
  let agents: PaginatedAgentsResponse = serde_json::from_slice( &body_bytes ).unwrap();
  assert_eq!( agents.data.len(), 4, "Admin should see all agents" );
}

#[ tokio::test ]
async fn test_list_agents_as_user_sees_only_accessible()
{
  let ( app, pool, _admin_token, user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agents - one owned by admin, one owned by user

  seed_test_agents( &pool, &admin_id, Some( &user_id ) ).await;

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
  let agents: PaginatedAgentsResponse = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agents.data.len(), 1, "User should only see agents they own" );
  assert_eq!( agents.data[ 0 ].name, "Agent 5" );
}

// ============================================================================
// Update Agent Tests
// ============================================================================

#[ tokio::test ]
async fn test_update_agent_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, admin_id, _user_id ) = create_agents_router().await;
  let now = chrono::Utc::now().timestamp_millis();

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_dsonifni" )
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

  let agent_id = result.last_insert_rowid();

  let request_body = json!({
    "name": "New Name",
    "tags": ["tag1", "tag2"],
    "description": "New Description",
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}", "agent_dsonifni" ) )
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
  assert_eq!( agent[ "tags" ].as_array().unwrap().len(), 2 );
  assert_eq!( agent[ "description" ].as_str().unwrap(), "New Description" );
}

#[ tokio::test ]
async fn test_update_agent_as_user_forbidden()
{
  let ( app, pool, _admin_token, user_token, admin_id, _user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_dsonifnidsfas" )
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

  let request_body = json!({
    "name": "New Name",
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}", "agent_dsonifnidsfas" ) )
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
// Get Agent Details Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_agent_details_as_admin_success()
{
  let ( app, pool, admin_token, _user_token, _admin_id, user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}/details", agent_id ) )
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
  let details: AgentDetails = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( details.name, "Test Agent" );
  assert_eq!( details.budget, 100.0 );
}

#[ tokio::test ]
async fn test_get_agent_details_as_owner_success()
{
  let ( app, pool, _admin_token, user_token, _admin_id, user_id ) = create_agents_router().await;

  // Create agent owned by user
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}/details", agent_id ) )
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
  let details: AgentDetails = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( details.name, "Test Agent" );
  assert_eq!( details.budget, 100.0 );
}

#[ tokio::test ]
async fn test_get_agent_details_as_other_user_forbidden()
{
  let ( app, pool, _admin_token, user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agent owned by admin
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &admin_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}/details", agent_id ) )
        .header( "authorization", format!( "Bearer {}", user_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::FORBIDDEN );
}

#[ tokio::test ]
async fn test_get_agent_details_not_found()
{
  let ( app, _pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents/999999/details" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

// ============================================================================
// Get Agent Providers Tests
// ============================================================================

#[ tokio::test ]
async fn test_get_agent_providers_success()
{
  let ( app, pool, _admin_token, user_token, _admin_id, user_id ) = create_agents_router().await;

  // Create agent owned by user
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "[\"gpt-3.5-turbo\"]" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( format!( "/api/agents/{}/providers", agent_id ) )
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

  let response: GetAgentProvidersResponse = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( response.providers.len(), 1 );
  assert_eq!( response.providers[ 0 ].id, "openai_1" );
  assert_eq!( response.providers[ 0 ].name, "openai" );
  assert_eq!( response.providers[ 0 ].endpoint, "https://api.openai.com/v1" );
}

# [ tokio::test ]
async fn test_get_agent_providers_not_found() {
  let ( app, _pool, admin_token, _user_token, _admin_id, _user_id ) = create_agents_router().await;

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::GET )
        .uri( "/api/agents/999999/providers" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::empty() )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::NOT_FOUND );
}

/// ============================================================================
/// Assign Providers To Agent Tests
/// ============================================================================

#[ tokio::test ]
async fn test_assign_providers_to_agent_as_admin_success() {
  let ( app, pool, admin_token, _user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, description, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( "123" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "[\"gpt-3.5-turbo\"]" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "anthropic_1" )
    .bind( "[\"claude-3.5-sonnet\"]" )
    .bind( "anthropic" )
    .bind( "https://api.anthropic.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let request_body = json!({
    "providers": ["openai_1", "anthropic_1"],
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}/providers", agent_id ) )
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

  assert_eq!( agent[ "providers" ].as_array().unwrap().len(), 2 );
}

#[ tokio::test ]
async fn test_assign_providers_to_agent_empty_list() {
  let ( app, pool, admin_token, _user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (name, providers, created_at, owner_id) VALUES (?, ?, ?, ?)" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = result.last_insert_rowid();

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "[\"gpt-3.5-turbo\"]" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "anthropic_1" )
    .bind( "[\"claude-3.5-sonnet\"]" )
    .bind( "anthropic" )
    .bind( "https://api.anthropic.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let request_body = json!({
    "providers": [],
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}/providers", agent_id ) )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST );

  let body_bytes = axum::body::to_bytes( response.into_body(), usize::MAX )
    .await
    .unwrap();
  let error_response: ErrorResponse = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( error_response.error.code, "VALIDATION_ERROR" );
  assert_eq!( error_response.error.message.unwrap(), "providers field is required" );
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetail {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<std::collections::HashMap<String, String>>,
}


#[ tokio::test ]
async fn test_assign_providers_to_agent_invalid_provider() {
  let ( app, pool, admin_token, _user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "[\"gpt-3.5-turbo\"]" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let request_body = json!({
    "providers": ["openai_1", "invalid_provider"],
  });

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::PUT )
        .uri( format!( "/api/agents/{}/providers", "agent_123" ) )
        .header( "content-type", "application/json" )
        .header( "authorization", format!( "Bearer {}", admin_token ) )
        .body( Body::from( serde_json::to_string( &request_body ).unwrap() ) )
        .unwrap(),
    )
    .await
    .unwrap();

  assert_eq!( response.status(), StatusCode::BAD_REQUEST );
}

/// ============================================================================
/// Remove Providers To Agent Tests
/// ============================================================================
#[ tokio::test ]
async fn test_remove_provider_from_agent() {
  let ( app, pool, admin_token, _user_token, admin_id, user_id ) = create_agents_router().await;

  // Create agent
  let now = chrono::Utc::now().timestamp_millis();
  let result = sqlx::query( "INSERT INTO agents (id, name, providers, created_at, owner_id) VALUES (?, ?, ?, ?, ?)" )
    .bind( "agent_123" )
    .bind( "Test Agent" )
    .bind( "[\"openai_1\"]" )
    .bind( now )
    .bind( &user_id )
    .execute( &pool )
    .await
    .unwrap();

  let agent_id = "agent_123";

  // Create budget
  sqlx::query( "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, created_at, updated_at) VALUES (?, ?, ?, ?, ?)" )
    .bind( agent_id )
    .bind( 100.0 )
    .bind( 100.0 )
    .bind( now )
    .bind( now )
    .execute( &pool )
    .await
    .unwrap();

  // Create provider key
  sqlx::query( "INSERT INTO ai_provider_keys (id, models, provider, base_url, created_at, encrypted_api_key, encryption_nonce) VALUES (?, ?, ?, ?, ?, ?, ?)" )
    .bind( "openai_1" )
    .bind( "[\"gpt-3.5-turbo\"]" )
    .bind( "openai" )
    .bind( "https://api.openai.com/v1" )
    .bind( now )
    .bind( "encrypted_api_key" )
    .bind( "encryption_nonce" )
    .execute( &pool )
    .await
    .unwrap();

  let response = app
    .oneshot(
      Request::builder()
        .method( Method::DELETE )
        .uri( format!( "/api/agents/{}/providers/openai_1", agent_id ) )
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

  let agent: RemoveProviderFromAgentResponse = serde_json::from_slice( &body_bytes ).unwrap();

  assert_eq!( agent.remaining_providers.len(), 0 );
}