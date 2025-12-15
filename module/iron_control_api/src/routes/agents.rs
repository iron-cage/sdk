//! Agent management API routes
//!
//! Provides REST endpoints for managing AI agents within the Iron Runtime platform.
//! Agents are the primary control mechanism for budget management and LLM access.
//!
//! **Endpoints:**
//! - `GET /agents` - List agents (filtered by user role)
//! - `GET /agents/:id` - Get single agent (with access control)
//! - `POST /agents` - Create agent (admin only)
//! - `PUT /agents/:id` - Update agent (admin only)
//! - `DELETE /agents/:id` - Delete agent (admin only)
//!
//! **Access Control:**
//! - Admins: Full access to all agents
//! - Regular users: Can only view agents they own

use std::sync::Arc;

use axum::{
    extract::{FromRef, Path, State},
    http::StatusCode,
    response::Json,
};
use iron_token_manager::{agent_budget::AgentBudgetManager, storage::TokenStorage};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};
use tokio::sync::Mutex;
use crate::{ic_token::{IcTokenClaims, IcTokenManager}, jwt_auth::JwtSecret, routes::auth::AuthState};

use crate::jwt_auth::AuthenticatedUser;

#[derive(Clone)]
pub struct AgentState {
    pub pool: Pool< Sqlite >,
    pub agent_budget_manager: Arc<AgentBudgetManager>,
    pub token_storage: Arc<TokenStorage>,
    pub ic_token_manager: Arc<IcTokenManager>,
    pub jwt_secret: Arc<JwtSecret>,
}

impl AgentState {
    pub async fn new(database_url: &str, ic_token_secret: &str, jwt_secret_key: &str) -> Result< Self, Box< dyn std::error::Error > > {
        let pool = Pool::<Sqlite>::connect(database_url).await?;
        let token_storage = Arc::new(TokenStorage::from_pool(pool.clone()));
        let agent_budget_manager = Arc::new(AgentBudgetManager::from_pool(pool.clone()));
        let ic_token_manager = Arc::new(IcTokenManager::new(ic_token_secret.to_string()));
        let jwt_secret = Arc::new(JwtSecret::new(jwt_secret_key.to_string()));

        Ok(Self {
            pool,
            token_storage,
            agent_budget_manager,
            ic_token_manager,
            jwt_secret,
        })
    }
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Agent {
    pub id: i64,
    pub name: String,
    #[sqlx(skip)]
    pub providers: Vec<String>,
    #[serde(skip)]
    providers_json: Option<String>,
    pub created_at: i64,
    pub owner_id: String,
    pub ic_token: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub providers: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub providers: Option<Vec<String>>,
}


impl FromRef< AgentState > for AuthState
{
  fn from_ref( state: &AgentState ) -> Self
  {
    AuthState
    {
      jwt_secret: state.jwt_secret.clone(),
      db_pool: state.pool.clone(),
      rate_limiter: crate::rate_limiter::LoginRateLimiter::new(),
    }
  }
}

impl FromRef< AgentState > for sqlx::SqlitePool
{
  fn from_ref( state: &AgentState ) -> Self
  {
    state.pool.clone()
  }
}

/// List all agents (filtered by user role)
pub async fn list_agents(
    State(pool): State<SqlitePool>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<Agent>>, (StatusCode, String)> {
    let mut agents = if user.0.role == "admin" {
        // Admin sees all agents
        sqlx::query_as::<_, Agent>(
            r#"
            SELECT
                id,
                name,
                providers as providers_json,
                created_at,
                owner_id,
                ic_token
            FROM agents
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
    } else {
        // Regular users only see agents they own
        sqlx::query_as::<_, Agent>(
            r#"
            SELECT
                id,
                name,
                providers as providers_json,
                created_at,
                owner_id,
                ic_token
            FROM agents
            WHERE owner_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(&user.0.sub)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
    };

    // Parse providers JSON
    for agent in &mut agents {
        if let Some(ref json) = agent.providers_json {
            agent.providers = serde_json::from_str(json).unwrap_or_else(|_| vec![]);
        }
    }

    Ok(Json(agents))
}

/// Get a single agent
pub async fn get_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    user: AuthenticatedUser,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let mut agent = sqlx::query_as::<_, Agent>(
        r#"
        SELECT
            id,
            name,
            providers as providers_json,
            created_at,
            owner_id,
            ic_token
        FROM agents
        WHERE id = ?
        "#
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    // Check if user has access (admin or owns the agent)
    if user.0.role != "admin" && agent.owner_id != user.0.sub {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this agent".to_string(),
        ));
    }

    // Parse providers JSON
    if let Some(ref json) = agent.providers_json {
        agent.providers = serde_json::from_str(json).unwrap_or_else(|_| vec![]);
    }

    Ok(Json(agent))
}

/// Create a new agent (admin only)
pub async fn create_agent(
    State(state): State<AgentState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, String)> {
    // Only admins can create agents
    let AgentState { pool, token_storage, agent_budget_manager: _, ic_token_manager, jwt_secret: _ } = state;
    let CreateAgentRequest { name, providers } = req;
    
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can create agents".to_string(),
        ));
    }

    let providers_json = serde_json::to_string(&providers).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON error: {}", e),
        )
    })?;
    let created_at = chrono::Utc::now().timestamp_millis();
    let owner_id = user.0.sub.clone();

    let result = sqlx::query(
        r#"
        INSERT INTO agents (name, providers, created_at, owner_id)
        VALUES (?, ?, ?, ?)
        "#
    )
    .bind(&name)
    .bind(&providers_json)
    .bind(created_at)
    .bind(&owner_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let agent_id = result.last_insert_rowid();

    let claims = IcTokenClaims::new(agent_id.to_string(), agent_id.to_string(), vec![], None);
    let plaintext_token = ic_token_manager.generate_token(&claims).unwrap();

    token_storage.create_token(&plaintext_token, &owner_id, None, Some(&name), Some(agent_id), None).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create token: {}", e),
        )
    })?;

    let agent = Agent {
        id: agent_id,
        name,
        providers: providers,
        providers_json: Some(providers_json),
        created_at,
        owner_id,
        ic_token: plaintext_token,
    };

    Ok((StatusCode::CREATED, Json(agent)))
}

/// Update an agent (admin only)
pub async fn update_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    user: AuthenticatedUser,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<Agent>, (StatusCode, String)> {
    // Only admins can update agents
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can update agents".to_string(),
        ));
    }

    // Check if agent exists
    let _existing: Option<i64> = sqlx::query_scalar("SELECT id FROM agents WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    if _existing.is_none() {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    // Update fields if provided
    if let Some(name) = &req.name {
        sqlx::query("UPDATE agents SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    if let Some(providers) = &req.providers {
        let providers_json = serde_json::to_string(providers).map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("JSON error: {}", e),
            )
        })?;
        sqlx::query("UPDATE agents SET providers = ? WHERE id = ?")
            .bind(&providers_json)
            .bind(id)
            .execute(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    // Fetch updated agent
    let mut agent = sqlx::query_as::<_, Agent>(
        r#"
        SELECT
            id,
            name,
            providers as providers_json,
            created_at,
            owner_id,
            ic_token
        FROM agents
        WHERE id = ?
        "#
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Parse providers JSON
    if let Some(ref json) = agent.providers_json {
        agent.providers = serde_json::from_str(json).unwrap_or_else(|_| vec![]);
    }

    Ok(Json(agent))
}

/// Delete an agent (admin only)
pub async fn delete_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    user: AuthenticatedUser,
) -> Result<StatusCode, (StatusCode, String)> {
    // Only admins can delete agents
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can delete agents".to_string(),
        ));
    }

    let result = sqlx::query("DELETE FROM agents WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Token list item for agent tokens endpoint
#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct AgentTokenItem {
    pub id: i64,
    pub user_id: String,
    pub provider: Option<String>,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
}

/// Get all tokens for an agent (filtered by user role)
pub async fn get_agent_tokens(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AgentTokenItem>>, (StatusCode, String)> {
    // Check if agent exists and get owner_id for authorization
    let agent_owner: Option<String> = sqlx::query_scalar("SELECT owner_id FROM agents WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    let owner_id = match agent_owner {
        Some(owner) => owner,
        None => {
            return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
        }
    };

    // Check if user has access (admin or owns the agent)
    if user.0.role != "admin" && owner_id != user.0.sub {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this agent".to_string(),
        ));
    }

    // Get tokens based on user role
    let rows = if user.0.role == "admin" {
        // Admin sees all tokens for this agent
        sqlx::query(
            r#"
            SELECT 
                id,
                user_id,
                provider,
                name,
                created_at,
                last_used_at,
                is_active
            FROM api_tokens
            WHERE agent_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(id)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
    } else {
        // Regular users only see their own tokens for this agent
        sqlx::query(
            r#"
            SELECT 
                id,
                user_id,
                provider,
                name,
                created_at,
                last_used_at,
                is_active
            FROM api_tokens
            WHERE agent_id = ? AND user_id = ?
            ORDER BY created_at DESC
            "#
        )
        .bind(id)
        .bind(&user.0.sub)
        .fetch_all(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
    };

    let tokens: Vec<AgentTokenItem> = rows
        .iter()
        .map(|row| AgentTokenItem {
            id: row.get("id"),
            user_id: row.get("user_id"),
            provider: row.get("provider"),
            name: row.get("name"),
            created_at: row.get("created_at"),
            last_used_at: row.get("last_used_at"),
            is_active: row.get("is_active"),
        })
        .collect();

    Ok(Json(tokens))
}