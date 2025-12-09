use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

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
                created_at
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
        // Regular users only see agents where they have tokens
        sqlx::query_as::<_, Agent>(
            r#"
            SELECT DISTINCT
                a.id,
                a.name,
                a.providers as providers_json,
                a.created_at
            FROM agents a
            INNER JOIN api_tokens t ON t.agent_id = a.id
            WHERE t.user_id = ?
            ORDER BY a.created_at DESC
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
            created_at
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

    // Check if user has access (admin or has tokens for this agent)
    if user.0.role != "admin" {
        let has_access: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) as count
            FROM api_tokens
            WHERE agent_id = ? AND user_id = ?
            "#
        )
        .bind(id)
        .bind(&user.0.sub)
        .fetch_one(&pool)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

        if has_access == 0 {
            return Err((
                StatusCode::FORBIDDEN,
                "You don't have access to this agent".to_string(),
            ));
        }
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

    let providers_json = serde_json::to_string(&req.providers).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JSON error: {}", e),
        )
    })?;
    let created_at = chrono::Utc::now().timestamp();

    let result = sqlx::query(
        r#"
        INSERT INTO agents (name, providers, created_at)
        VALUES (?, ?, ?)
        "#
    )
    .bind(&req.name)
    .bind(&providers_json)
    .bind(created_at)
    .execute(&pool)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let agent = Agent {
        id: result.last_insert_rowid(),
        name: req.name,
        providers: req.providers,
        providers_json: Some(providers_json),
        created_at,
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
            created_at
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
