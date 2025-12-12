use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use iron_token_manager::agent_service::{
    AgentService, Agent as ServiceAgent, CreateAgentParams, UpdateAgentParams,
    ListAgentsFilters, AgentTokenItem as ServiceAgentTokenItem,
    ICToken as ServiceICToken, ListAgentsResult, AgentSortField, SortDirection,
};

use crate::jwt_auth::AuthenticatedUser;

#[derive(Debug, Serialize, Deserialize)]
pub struct Agent {
    pub id: String,
    pub name: String,
    pub budget: f64,
    pub providers: Vec<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub owner_id: String,
    pub project_id: Option<String>,
    pub ic_token: Option<ICToken>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ICToken {
    pub id: String,
    pub token: String,
    pub created_at: String,
}

impl From<ServiceICToken> for ICToken {
    fn from(token: ServiceICToken) -> Self {
        Self {
            id: token.id,
            token: token.token,
            created_at: token.created_at,
        }
    }
}

impl From<ServiceAgent> for Agent {
    fn from(agent: ServiceAgent) -> Self {
        Self {
            id: agent.id,
            name: agent.name,
            budget: agent.budget,
            providers: agent.providers,
            description: agent.description,
            tags: agent.tags,
            owner_id: agent.owner_id,
            project_id: agent.project_id,
            ic_token: agent.ic_token.map(ICToken::from),
            status: agent.status,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub budget: f64,
    pub providers: Option<Vec<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub project_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub budget: Option<f64>,
    pub providers: Option<Vec<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub status: Option<String>,
}

/// Query parameters for listing agents
#[derive(Debug, Deserialize)]
pub struct ListAgentsQuery {
    /// Page number (1-indexed, default: 1)
    #[serde(default = "default_page")]
    pub page: u32,
    /// Results per page (default: 50, max: 100)
    #[serde(default = "default_per_page")]
    pub per_page: u32,
    /// Filter by name (partial match, case-insensitive)
    pub name: Option<String>,
    /// Filter by status: "active", "exhausted", "inactive"
    pub status: Option<String>,
    /// Sort field: name, budget, created_at (prefix - for desc, default: -created_at)
    #[serde(default = "default_sort")]
    pub sort: String,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 50 }
fn default_sort() -> String { "-created_at".to_string() }

/// Pagination metadata
#[derive(Debug, Serialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

/// Agent list item with computed fields
#[derive(Debug, Serialize)]
pub struct AgentListItem {
    pub id: String,
    pub name: String,
    pub budget: f64,
    pub spent: f64,
    pub remaining: f64,
    pub providers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub owner_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

impl From<ServiceAgent> for AgentListItem {
    fn from(agent: ServiceAgent) -> Self {
        // TODO: spent will be computed from usage data later
        let spent = 0.0;
        let remaining = (agent.budget - spent).max(0.0);
        Self {
            id: agent.id,
            name: agent.name,
            budget: agent.budget,
            spent,
            remaining,
            providers: agent.providers,
            description: agent.description,
            tags: agent.tags,
            owner_id: agent.owner_id,
            project_id: agent.project_id,
            status: agent.status,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }
}

/// Paginated response for agent listing
#[derive(Debug, Serialize)]
pub struct PaginatedAgentsResponse {
    pub data: Vec<AgentListItem>,
    pub pagination: Pagination,
}

/// Parse sort string into field and direction
/// Format: "field" for ascending, "-field" for descending
fn parse_sort(sort: &str) -> (AgentSortField, SortDirection) {
    let (desc, field_str) = if let Some(stripped) = sort.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, sort)
    };

    let field = match field_str {
        "name" => AgentSortField::Name,
        "budget" => AgentSortField::Budget,
        "created_at" | _ => AgentSortField::CreatedAt,
    };

    let direction = if desc { SortDirection::Desc } else { SortDirection::Asc };

    (field, direction)
}

/// List all agents (filtered by user role) with pagination and sorting
pub async fn list_agents(
    State(pool): State<SqlitePool>,
    Query(query): Query<ListAgentsQuery>,
    user: AuthenticatedUser,
) -> Result<Json<PaginatedAgentsResponse>, (StatusCode, String)> {
    let service = AgentService::new(pool);

    // Parse sort parameter
    let (sort_field, sort_direction) = parse_sort(&query.sort);

    // Clamp per_page to max 100
    let per_page = query.per_page.min(100);

    let filters = ListAgentsFilters {
        owner_id: if user.0.role == "admin" {
            None // Admin sees all agents
        } else {
            Some(user.0.sub.clone()) // Regular users only see agents they own
        },
        name: query.name,
        status: query.status,
        page: Some(query.page),
        per_page: Some(per_page),
        sort_field: Some(sort_field),
        sort_direction: Some(sort_direction),
    };

    let result = service.list_agents(filters).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let total_pages = if result.total == 0 {
        1
    } else {
        ((result.total as f64) / (per_page as f64)).ceil() as u32
    };

    let response = PaginatedAgentsResponse {
        data: result.agents.into_iter().map(AgentListItem::from).collect(),
        pagination: Pagination {
            page: query.page,
            per_page,
            total: result.total,
            total_pages,
        },
    };

    Ok(Json(response))
}

/// Get a single agent
pub async fn get_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let service = AgentService::new(pool);

    let agent = service.get_agent(&id).await.map_err(|e| {
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

    Ok(Json(Agent::from(agent)))
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

    let service = AgentService::new(pool);

    let params = CreateAgentParams {
        name: req.name,
        budget: req.budget,
        providers: req.providers,
        description: req.description,
        tags: req.tags,
        project_id: None, // leave empty for now as project is not supported yet
    };

    let agent = service.create_agent(params, &user.0.sub).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok((StatusCode::CREATED, Json(Agent::from(agent))))
}

/// Update an agent (admin only)
pub async fn update_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
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

    let service = AgentService::new(pool);

    let params = UpdateAgentParams {
        name: req.name,
        budget: req.budget,
        providers: req.providers,
        description: req.description,
        tags: req.tags,
        status: req.status,
    };

    let agent = service.update_agent(&id, params).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    Ok(Json(Agent::from(agent)))
}

/// Delete an agent (admin only)
pub async fn delete_agent(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<StatusCode, (StatusCode, String)> {
    // Only admins can delete agents
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can delete agents".to_string(),
        ));
    }

    let service = AgentService::new(pool);

    let deleted = service.delete_agent(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

/// Token list item for agent tokens endpoint
#[derive(Debug, Serialize)]
pub struct AgentTokenItem {
    pub id: i64,
    pub user_id: String,
    pub provider: Option<String>,
    pub name: Option<String>,
    pub created_at: i64,
    pub last_used_at: Option<i64>,
    pub is_active: bool,
}

impl From<ServiceAgentTokenItem> for AgentTokenItem {
    fn from(token: ServiceAgentTokenItem) -> Self {
        Self {
            id: token.id,
            user_id: token.user_id,
            provider: token.provider,
            name: token.name,
            created_at: token.created_at,
            last_used_at: token.last_used_at,
            is_active: token.is_active,
        }
    }
}

/// Get all tokens for an agent (filtered by user role)
pub async fn get_agent_tokens(
    State(pool): State<SqlitePool>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AgentTokenItem>>, (StatusCode, String)> {
    let service = AgentService::new(pool);

    // Check if agent exists and get owner_id for authorization
    let owner_id = service.get_agent_owner(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    // Check if user has access (admin or owns the agent)
    if user.0.role != "admin" && owner_id != user.0.sub {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this agent".to_string(),
        ));
    }

    // Get tokens based on user role
    let user_filter = if user.0.role == "admin" {
        None // Admin sees all tokens
    } else {
        Some(user.0.sub.as_str()) // Regular users only see their own tokens
    };

    let tokens = service.get_agent_tokens(&id, user_filter).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(tokens.into_iter().map(AgentTokenItem::from).collect()))
}


