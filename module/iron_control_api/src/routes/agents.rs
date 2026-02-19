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

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

use crate::jwt_auth::AuthenticatedUser;

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
    pub provider_key_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub providers: Vec<String>,
    pub provider_key_id: i64,
    pub initial_budget_microdollars: i64,
    /// Optional owner_id - admins can assign agents to other users.
    /// If not provided, defaults to the authenticated user.
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub providers: Option<Vec<String>>,
    pub provider_key_id: Option<Option<i64>>, // Some(Some(id)) sets; Some(None) clears
    /// Optional owner_id - only admins can reassign agents to other users.
    pub owner_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentBudgetRequest {
    pub total_allocated_microdollars: i64,
}

#[derive(Debug, Serialize)]
pub struct AgentBudgetResponse {
    pub agent_id: i64,
    pub total_allocated: i64,
    pub total_spent: i64,
    pub budget_remaining: i64,
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
                provider_key_id
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
                provider_key_id
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
            provider_key_id
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
    State(pool): State<SqlitePool>,
    user: AuthenticatedUser,
    Json(req): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, String)> {
    // Only admins can create agents
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can create agents".to_string(),
        ));
    }

    if req.initial_budget_microdollars <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "initial_budget_microdollars must be positive".to_string(),
        ));
    }

    // Validate provider key exists and fetch provider name
    let provider_row = sqlx::query(
        r#"SELECT provider FROM ai_provider_keys WHERE id = ? AND is_enabled = 1"#
    )
    .bind(req.provider_key_id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let provider_name: String = match provider_row {
        Some(row) => row.get::<String, _>("provider"),
        None => {
            return Err((StatusCode::NOT_FOUND, "Provider key not found or disabled".to_string()));
        }
    };

    // Normalize providers to match provider key
    let provider_list = vec![provider_name];
    let providers_json = serde_json::to_string(&provider_list).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON error: {}", e),
        )
    })?;

    let created_at = chrono::Utc::now().timestamp_millis();
    let is_admin = user.0.role == "admin";

    // Only admins can assign agents to other users
    if req.owner_id.is_some() && !is_admin {
        return Err((
            StatusCode::FORBIDDEN,
            "Only admins can assign agents to other users".to_string(),
        ));
    }

    // Determine owner_id: admins can specify, others default to self
    // Note: owner_id references users.id (e.g., "user_demo"), not users.username
    let owner_id = if let Some(ref specified_owner) = req.owner_id {
        // Validate that specified user exists (check users.id column)
        let user_exists: Option<String> = sqlx::query_scalar(
            "SELECT id FROM users WHERE id = ?"
        )
        .bind(specified_owner)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        if user_exists.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Specified owner_id '{}' does not exist", specified_owner),
            ));
        }

        specified_owner.clone()
    } else {
        user.0.sub.clone()
    };

    let result = sqlx::query(
        r#"
        INSERT INTO agents (name, providers, created_at, owner_id, provider_key_id)
        VALUES (?, ?, ?, ?, ?)
        "#
    )
    .bind(&req.name)
    .bind(&providers_json)
    .bind(created_at)
    .bind(&owner_id)
    .bind(req.provider_key_id)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let agent_id = result.last_insert_rowid();

    // Create required initial agent budget
    sqlx::query(
        r#"
        INSERT INTO agent_budgets
          (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
        VALUES (?, ?, 0, ?, ?, ?)
        "#
    )
    .bind(agent_id)
    .bind(req.initial_budget_microdollars)
    .bind(req.initial_budget_microdollars)
    .bind(created_at)
    .bind(created_at)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to create agent budget: {}", e),
        )
    })?;

    let agent = Agent {
        id: agent_id,
        name: req.name,
        providers: provider_list,
        providers_json: Some(providers_json),
        created_at,
        owner_id,
        provider_key_id: Some(req.provider_key_id),
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

    // Update provider_key_id if provided (Some(Some(id)) sets; Some(None) clears)
    if let Some(provider_key_id_opt) = req.provider_key_id {
        if let Some(key_id) = provider_key_id_opt {
            let provider_row = sqlx::query(
                r#"SELECT provider FROM ai_provider_keys WHERE id = ? AND is_enabled = 1"#
            )
            .bind(key_id)
            .fetch_optional(&pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
            let provider_name: String = match provider_row {
                Some(row) => row.get::<String, _>("provider"),
                None => {
                    return Err((StatusCode::NOT_FOUND, "Provider key not found or disabled".to_string()));
                }
            };

            // Align providers list with provider key
            let providers_json = serde_json::to_string(&vec![provider_name]).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("JSON error: {}", e),
                )
            })?;

            sqlx::query("UPDATE agents SET provider_key_id = ?, providers = ? WHERE id = ?")
                .bind(Some(key_id))
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
        } else {
            // Clearing provider key also clears providers list
            let providers_json = serde_json::to_string(&Vec::<String>::new()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("JSON error: {}", e),
                )
            })?;

            sqlx::query("UPDATE agents SET provider_key_id = NULL, providers = ? WHERE id = ?")
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

    }

    // Update owner_id if provided (admin only - already checked above)
    // Note: owner_id references users.id (e.g., "user_demo"), not users.username
    if let Some(ref new_owner) = req.owner_id {
        // Validate that the new owner exists (check users.id column)
        let user_exists: Option<String> = sqlx::query_scalar(
            "SELECT id FROM users WHERE id = ?"
        )
        .bind(new_owner)
        .fetch_optional(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        if user_exists.is_none() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Specified owner_id '{}' does not exist", new_owner),
            ));
        }

        sqlx::query("UPDATE agents SET owner_id = ? WHERE id = ?")
            .bind(new_owner)
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
            provider_key_id
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

/// PUT /api/v1/agents/:id/budget
///
/// Update an agent's total allocated budget (microdollars).
/// Recomputes budget_remaining = total_allocated - total_spent.
pub async fn update_agent_budget(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    user: AuthenticatedUser,
    Json(req): Json<UpdateAgentBudgetRequest>,
) -> Result<Json<AgentBudgetResponse>, (StatusCode, String)> {
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can update agent budgets".to_string(),
        ));
    }

    if req.total_allocated_microdollars <= 0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "total_allocated_microdollars must be positive".to_string(),
        ));
    }

    // Ensure agent exists and get owner_id (for potential future checks)
    let agent_exists: Option<String> = sqlx::query_scalar("SELECT owner_id FROM agents WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    if agent_exists.is_none() {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    // Get current spent (create row if missing)
    let budget_row: Option<(i64,)> = sqlx::query_as(
        "SELECT total_spent FROM agent_budgets WHERE agent_id = ?"
    )
    .bind(id)
    .fetch_optional(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let total_spent = budget_row.map(|r| r.0).unwrap_or(0);

    if req.total_allocated_microdollars < total_spent {
        return Err((
            StatusCode::BAD_REQUEST,
            "total_allocated_microdollars cannot be less than total_spent".to_string(),
        ));
    }

    let budget_remaining = req.total_allocated_microdollars - total_spent;
    let now_ms = chrono::Utc::now().timestamp_millis();

    // Upsert budget row
    sqlx::query(
        r#"
        INSERT INTO agent_budgets (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
        ON CONFLICT(agent_id) DO UPDATE SET
          total_allocated = excluded.total_allocated,
          total_spent = agent_budgets.total_spent,
          budget_remaining = excluded.total_allocated - agent_budgets.total_spent,
          updated_at = excluded.updated_at
        "#
    )
    .bind(id)
    .bind(req.total_allocated_microdollars)
    .bind(total_spent)
    .bind(budget_remaining)
    .bind(now_ms)
    .bind(now_ms)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    Ok(Json(AgentBudgetResponse {
        agent_id: id,
        total_allocated: req.total_allocated_microdollars,
        total_spent,
        budget_remaining,
    }))
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
