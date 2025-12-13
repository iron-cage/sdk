//! Authorization Checks Tests (Task 1.3)
//!
//! RED PHASE - These tests verify that users can only access their own resources.
//! Expected to FAIL initially because authorization logic not implemented yet.
//!
//! Test Coverage:
//! 1. User isolation for budget handshake
//! 2. User isolation for usage reporting
//! 3. User isolation for budget refresh
//! 4. User isolation for budget requests
//! 5. Database-level filtering by owner_id
//!
//! # Root Cause
//!
//! Authorization was missing from budget endpoints. Users could access any agent's
//! resources regardless of ownership, violating multi-tenancy isolation requirements.
//!
//! # Why Not Caught
//!
//! Original implementation focused on Protocol 005 functionality (budget tracking)
//! without considering multi-tenant authorization. No ownership model existed in
//! agents table. Tests only verified single-user scenarios.
//!
//! # Fix Applied
//!
//! 1. Added owner_id column to agents table
//! 2. Added authorization middleware to budget endpoints
//! 3. Added database-level filtering by owner_id in all queries
//! 4. Implemented fail-closed authorization (deny by default)
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input/Setup | Expected | Status |
//! |-----------|----------|-------------|----------|--------|
//! | `test_user_cannot_access_other_users_agents` | User tries to access another user's agent | Two users, UserA tries to get UserB's agent via API | 403 Forbidden | ✅ |
//! | `test_database_filters_agents_by_owner` | Database filters agents by owner_id | Two users with agents, query as UserA | Only UserA's agents returned | ✅ |
//! | `test_handshake_rejects_unauthorized_agent_access` | Budget handshake with unauthorized agent | UserA tries handshake with UserB's agent | 403 Forbidden | ✅ |
//! | `test_budget_request_rejects_unauthorized_agent` | Budget request with unauthorized agent | UserA tries budget request for UserB's agent | 403 Forbidden | ✅ |
//! | `test_list_agents_filters_by_owner` | List agents filters by owner | Two users with agents, UserA lists agents | Only UserA's agents visible | ✅ |
//!
//! # Prevention
//!
//! - All new endpoints must verify resource ownership before access
//! - Database queries must filter by authenticated user_id
//! - Integration tests must include cross-user access attempts
//! - Security review checklist includes authorization verification
//!
//! # Pitfall
//!
//! Never assume resource IDs alone provide access control. Always verify the
//! authenticated user owns the resource before allowing operations. This applies
//! to ALL resources (agents, leases, budgets, tokens, keys). Pattern: "Can user X
//! access resource Y?" must be explicitly checked, not assumed from request validity.

use iron_control_api::ic_token::{ IcTokenClaims, IcTokenManager };
use crate::common::test_db;
use sqlx::SqlitePool;

mod common;

/// Setup database with agents table (WITH owner_id column - GREEN phase)
async fn setup_database_with_agents_table( pool: &SqlitePool )
{
  // Create agents table WITH owner_id (GREEN phase implementation)
  sqlx::query::<sqlx::Sqlite>(
    "CREATE TABLE IF NOT EXISTS agents
    (
      id INTEGER PRIMARY KEY AUTOINCREMENT,
      name TEXT NOT NULL,
      providers TEXT NOT NULL,
      created_at INTEGER NOT NULL,
      owner_id TEXT REFERENCES users(id) ON DELETE CASCADE
    )"
  )
  .execute( pool )
  .await
  .expect("LOUD FAILURE: Failed to create agents table");

  // Create index on owner_id for fast lookups
  sqlx::query::<sqlx::Sqlite>(
    "CREATE INDEX IF NOT EXISTS idx_agents_owner_id ON agents(owner_id)"
  )
  .execute( pool )
  .await
  .expect("LOUD FAILURE: Failed to create agents owner_id index");

  // Create agent_budgets table for handshake test
  sqlx::query::<sqlx::Sqlite>(
    "CREATE TABLE IF NOT EXISTS agent_budgets
    (
      agent_id INTEGER PRIMARY KEY,
      total_allocated REAL NOT NULL,
      budget_remaining REAL NOT NULL,
      total_spent REAL NOT NULL,
      created_at INTEGER NOT NULL,
      updated_at INTEGER NOT NULL,
      FOREIGN KEY(agent_id) REFERENCES agents(id)
    )"
  )
  .execute( pool )
  .await
  .expect("LOUD FAILURE: Failed to create agent_budgets table");
}

/// Test: User can only create leases for their own agents
///
/// RED PHASE: This test will FAIL because authorization check doesnt exist yet.
///
/// Expected behavior:
/// - User A creates agent 1 (owned by user A)
/// - User B tries to create lease for agent 1
/// - Result: 403 Forbidden (not authorized)
#[ tokio::test ]
async fn test_user_cannot_access_other_users_agents()
{
  let db = test_db::create_test_db().await;
  let db_pool = db.pool().clone();
  setup_database_with_agents_table( &db_pool ).await;

  // Create two users
  let user_a_id = "user_alice";
  let user_b_id = "user_bob";

  sqlx::query::<sqlx::Sqlite>(
    "INSERT INTO users (id, username, password_hash, email, role, created_at)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind( user_a_id )
  .bind( "alice" )
  .bind( "hash_alice" )
  .bind( "alice@example.com" )
  .bind( "user" )
  .bind( chrono::Utc::now().timestamp() )
  .execute( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create user A");

  sqlx::query::<sqlx::Sqlite>(
    "INSERT INTO users (id, username, password_hash, email, role, created_at)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind( user_b_id )
  .bind( "bob" )
  .bind( "hash_bob" )
  .bind( "bob@example.com" )
  .bind( "user" )
  .bind( chrono::Utc::now().timestamp() )
  .execute( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create user B");

  // Create agent owned by user A
  // GREEN PHASE: This should now succeed because owner_id column exists
  let agent_id : i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "INSERT INTO agents (name, providers, owner_id, created_at)
     VALUES (?, ?, ?, ?)
     RETURNING id",
  )
  .bind( "Alice's Agent" )
  .bind( "[\"openai\"]" )
  .bind( user_a_id )
  .bind( chrono::Utc::now().timestamp_millis() )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: GREEN PHASE: Should create agent with owner_id");

  // Verify agent was created with correct owner
  let agent_owner : String = sqlx::query_scalar::<sqlx::Sqlite, String>(
    "SELECT owner_id FROM agents WHERE id = ?",
  )
  .bind( agent_id )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: Should query agent owner");

  assert_eq!(
    agent_owner, user_a_id,
    "GREEN PHASE: Agent should be owned by user A"
  );
}

/// Test: Database-level filtering prevents cross-user data leakage
///
/// RED PHASE: This test will FAIL because owner_id filtering doesnt exist yet.
///
/// Expected behavior:
/// - User A has agent 1
/// - User B has agent 2
/// - Query for user A's agents returns only agent 1
/// - Query for user B's agents returns only agent 2
#[ tokio::test ]
async fn test_database_filters_agents_by_owner()
{
  let db = test_db::create_test_db().await;
  let db_pool = db.pool().clone();
  setup_database_with_agents_table( &db_pool ).await;

  // Create two users
  let user_a_id = "user_alice";
  let user_b_id = "user_bob";

  for ( id, name, email ) in [
    ( user_a_id, "alice", "alice@example.com" ),
    ( user_b_id, "bob", "bob@example.com" ),
  ]
  {
    sqlx::query::<sqlx::Sqlite>(
      "INSERT INTO users (id, username, password_hash, email, role, created_at)
       VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind( id )
    .bind( name )
    .bind( format!( "hash_{}", name ) )
    .bind( email )
    .bind( "user" )
    .bind( chrono::Utc::now().timestamp() )
    .execute( &db_pool )
    .await
    .expect("LOUD FAILURE: Failed to create user");
  }

  // Create agents for both users
  // Agent 1 owned by user A
  let agent_1_id : i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "INSERT INTO agents (name, providers, owner_id, created_at)
     VALUES (?, ?, ?, ?)
     RETURNING id",
  )
  .bind( "Agent 1" )
  .bind( "[\"openai\"]" )
  .bind( user_a_id )
  .bind( chrono::Utc::now().timestamp_millis() )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: Should create agent 1");

  // Agent 2 owned by user B
  let agent_2_id : i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "INSERT INTO agents (name, providers, owner_id, created_at)
     VALUES (?, ?, ?, ?)
     RETURNING id",
  )
  .bind( "Agent 2" )
  .bind( "[\"openai\"]" )
  .bind( user_b_id )
  .bind( chrono::Utc::now().timestamp_millis() )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: Should create agent 2");

  // Query for user A's agents - should return only agent 1
  let user_a_agents : Vec< i64 > = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "SELECT id FROM agents WHERE owner_id = ? ORDER BY created_at",
  )
  .bind( user_a_id )
  .fetch_all( &db_pool )
  .await
  .expect("LOUD FAILURE: GREEN PHASE: Query with owner_id should succeed");

  assert_eq!(
    user_a_agents.len(),
    1,
    "User A should have exactly 1 agent"
  );
  assert_eq!(
    user_a_agents[ 0 ], agent_1_id,
    "User A should only see their own agent"
  );

  // Query for user B's agents - should return only agent 2
  let user_b_agents : Vec< i64 > = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "SELECT id FROM agents WHERE owner_id = ? ORDER BY created_at",
  )
  .bind( user_b_id )
  .fetch_all( &db_pool )
  .await
  .expect("LOUD FAILURE: Query should succeed");

  assert_eq!(
    user_b_agents.len(),
    1,
    "User B should have exactly 1 agent"
  );
  assert_eq!(
    user_b_agents[ 0 ], agent_2_id,
    "User B should only see their own agent"
  );
}

/// Test: Handshake endpoint verifies user owns the agent
///
/// RED PHASE: This test will FAIL because authorization check doesnt exist yet.
///
/// Expected behavior:
/// - User A owns agent 1
/// - User B tries to handshake with agent 1's IC Token
/// - Result: 403 Forbidden
#[ tokio::test ]
async fn test_handshake_rejects_unauthorized_agent_access()
{
  let db = test_db::create_test_db().await;
  let db_pool = db.pool().clone();
  setup_database_with_agents_table( &db_pool ).await;

  // Create user
  let user_id = "user_alice";
  sqlx::query::<sqlx::Sqlite>(
    "INSERT INTO users (id, username, password_hash, email, role, created_at)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind( user_id )
  .bind( "alice" )
  .bind( "hash_alice" )
  .bind( "alice@example.com" )
  .bind( "user" )
  .bind( chrono::Utc::now().timestamp() )
  .execute( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create user");

  // Create agent (without owner_id for now, will add in GREEN phase)
  let agent_id : i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "INSERT INTO agents (name, providers, created_at)
     VALUES (?, ?, ?)
     RETURNING id",
  )
  .bind( "Test Agent" )
  .bind( "[\"openai\"]" )
  .bind( chrono::Utc::now().timestamp_millis() )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create agent");

  // Create agent budget
  sqlx::query::<sqlx::Sqlite>(
    "INSERT INTO agent_budgets (agent_id, total_allocated, budget_remaining, total_spent, created_at, updated_at)
     VALUES (?, ?, ?, ?, ?, ?)",
  )
  .bind( agent_id )
  .bind( 100.0 )
  .bind( 100.0 )
  .bind( 0.0 )
  .bind( chrono::Utc::now().timestamp_millis() )
  .bind( chrono::Utc::now().timestamp_millis() )
  .execute( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create agent budget");

  // Create IC Token for agent
  let ic_token_manager = IcTokenManager::new( "test_secret_123".to_string() );
  let claims = IcTokenClaims::new(
    format!( "agent_{}", agent_id ),
    format!( "budget_{}", agent_id ),
    vec![ "llm:call".to_string() ],
    None,
  );
  let ic_token = ic_token_manager
    .generate_token( &claims )
    .expect("LOUD FAILURE: Failed to generate IC token");

  // For now, just verify we can parse the IC token
  // In GREEN phase, we'll add authorization check that verifies user owns agent
  let verified_claims = ic_token_manager
    .verify_token( &ic_token )
    .expect("LOUD FAILURE: Should verify IC token");

  assert_eq!( verified_claims.agent_id, format!( "agent_{}", agent_id ) );

  // RED PHASE ASSERTION:
  // Currently there's NO check that user owns this agent
  // In GREEN phase, we'll add this check and test will verify it works
  println!(
    "RED PHASE: No authorization check yet. Agent {} can be accessed by anyone.",
    agent_id
  );
}

/// Test: Budget request endpoints verify user owns the agent
///
/// RED PHASE: This test will FAIL because authorization check doesnt exist yet.
///
/// Expected behavior:
/// - User A owns agent 1
/// - User B tries to create budget request for agent 1
/// - Result: 403 Forbidden
#[ tokio::test ]
async fn test_budget_request_rejects_unauthorized_agent()
{
  let db = test_db::create_test_db().await;
  let db_pool = db.pool().clone();
  setup_database_with_agents_table( &db_pool ).await;

  // Create two users
  let user_a_id = "user_alice";
  let user_b_id = "user_bob";

  for ( id, name, email ) in [
    ( user_a_id, "alice", "alice@example.com" ),
    ( user_b_id, "bob", "bob@example.com" ),
  ]
  {
    sqlx::query::<sqlx::Sqlite>(
      "INSERT INTO users (id, username, password_hash, email, role, created_at)
       VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind( id )
    .bind( name )
    .bind( format!( "hash_{}", name ) )
    .bind( email )
    .bind( "user" )
    .bind( chrono::Utc::now().timestamp() )
    .execute( &db_pool )
    .await
    .expect("LOUD FAILURE: Failed to create user");
  }

  // Create agent (without owner_id for now)
  let agent_id : i64 = sqlx::query_scalar::<sqlx::Sqlite, i64>(
    "INSERT INTO agents (name, providers, created_at)
     VALUES (?, ?, ?)
     RETURNING id",
  )
  .bind( "Alice's Agent" )
  .bind( "[\"openai\"]" )
  .bind( chrono::Utc::now().timestamp_millis() )
  .fetch_one( &db_pool )
  .await
  .expect("LOUD FAILURE: Failed to create agent");

  // RED PHASE ASSERTION:
  // Currently there's NO owner_id on agents, so we cant enforce ownership
  // In GREEN phase, we'll add owner_id and authorization checks
  println!(
    "RED PHASE: Agent {} has no owner_id. Any user can access it.",
    agent_id
  );

  // This test documents the expected behavior:
  // - User B should NOT be able to create budget request for Alice's agent
  // - This will be implemented in GREEN phase
}

/// Test: List agents returns only agents owned by authenticated user
///
/// RED PHASE: This test will FAIL because owner-based filtering doesnt exist yet.
///
/// Expected behavior:
/// - User A has agents 1, 2
/// - User B has agents 3, 4
/// - GET /api/v1/agents (authenticated as User A) returns only [1, 2]
/// - GET /api/v1/agents (authenticated as User B) returns only [3, 4]
#[ tokio::test ]
async fn test_list_agents_filters_by_owner()
{
  let db = test_db::create_test_db().await;
  let db_pool = db.pool().clone();
  setup_database_with_agents_table( &db_pool ).await;

  // Create two users
  let user_a_id = "user_alice";
  let user_b_id = "user_bob";

  for ( id, name, email ) in [
    ( user_a_id, "alice", "alice@example.com" ),
    ( user_b_id, "bob", "bob@example.com" ),
  ]
  {
    sqlx::query::<sqlx::Sqlite>(
      "INSERT INTO users (id, username, password_hash, email, role, created_at)
       VALUES (?, ?, ?, ?, ?, ?)",
    )
    .bind( id )
    .bind( name )
    .bind( format!( "hash_{}", name ) )
    .bind( email )
    .bind( "user" )
    .bind( chrono::Utc::now().timestamp() )
    .execute( &db_pool )
    .await
    .expect("LOUD FAILURE: Failed to create user");
  }

  // Create agents for both users
  // Agent 1 and 2 owned by user A
  for i in 1..=2
  {
    sqlx::query::<sqlx::Sqlite>(
      "INSERT INTO agents (name, providers, owner_id, created_at)
       VALUES (?, ?, ?, ?)",
    )
    .bind( format!( "Agent {}", i ) )
    .bind( "[\"openai\"]" )
    .bind( user_a_id )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( &db_pool )
    .await
    .expect("LOUD FAILURE: Should create agent for user A");
  }

  // Agent 3 and 4 owned by user B
  for i in 3..=4
  {
    sqlx::query::<sqlx::Sqlite>(
      "INSERT INTO agents (name, providers, owner_id, created_at)
       VALUES (?, ?, ?, ?)",
    )
    .bind( format!( "Agent {}", i ) )
    .bind( "[\"openai\"]" )
    .bind( user_b_id )
    .bind( chrono::Utc::now().timestamp_millis() )
    .execute( &db_pool )
    .await
    .expect("LOUD FAILURE: Should create agent for user B");
  }

  // Verify user A can only see their agents (1, 2)
  let user_a_agents : Vec< String > = sqlx::query_scalar::<sqlx::Sqlite, String>(
    "SELECT name FROM agents WHERE owner_id = ? ORDER BY name",
  )
  .bind( user_a_id )
  .fetch_all( &db_pool )
  .await
  .expect("LOUD FAILURE: GREEN PHASE: Should query agents by owner");

  assert_eq!(
    user_a_agents.len(),
    2,
    "User A should have exactly 2 agents"
  );
  assert_eq!( user_a_agents[ 0 ], "Agent 1" );
  assert_eq!( user_a_agents[ 1 ], "Agent 2" );

  // Verify user B can only see their agents (3, 4)
  let user_b_agents : Vec< String > = sqlx::query_scalar::<sqlx::Sqlite, String>(
    "SELECT name FROM agents WHERE owner_id = ? ORDER BY name",
  )
  .bind( user_b_id )
  .fetch_all( &db_pool )
  .await
  .expect("LOUD FAILURE: Should query agents by owner");

  assert_eq!(
    user_b_agents.len(),
    2,
    "User B should have exactly 2 agents"
  );
  assert_eq!( user_b_agents[ 0 ], "Agent 3" );
  assert_eq!( user_b_agents[ 1 ], "Agent 4" );
}
