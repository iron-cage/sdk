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
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use iron_token_manager::{agent_service::{
    Agent as ServiceAgent, AgentService, AgentSortField, AgentTokenItem as ServiceAgentTokenItem, CreateAgentParams, ICToken as ServiceICToken, ListAgentsFilters, SortDirection, UpdateAgentParams
}, token_generator::TokenGenerator};

use crate::jwt_auth::AuthenticatedUser;
use std::sync::Arc;
use iron_token_manager::storage::TokenStorage;
use crate::{ jwt_auth::JwtSecret, routes::auth::AuthState };

#[derive(Clone)]
pub struct AgentState {
    pub agent_service: Arc<AgentService>,
    pub token_storage: Arc<TokenStorage>,
    pub db_pool: SqlitePool,
    pub jwt_secret: Arc< JwtSecret >,
}

impl AgentState {
    pub fn new(pool: SqlitePool, token_storage: Arc<TokenStorage>, jwt_secret: Arc< JwtSecret >) -> Self {
        Self {
            agent_service: Arc::new(AgentService::new(pool.clone())),
            token_storage,
            db_pool: pool,
            jwt_secret,
        }
    }
}

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


#[derive(Debug, Deserialize, Serialize)]
pub struct CreateAgentRequest {
    pub name: String,
    pub budget: f64,
    pub providers: Option<Vec<String>>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub project_id: Option<String>,
    pub owner_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAgentRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
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
#[derive(Debug, Serialize, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

/// Agent list item with computed fields
#[derive(Debug, Serialize, Deserialize)]
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
            owner_id: agent.user_id,
            project_id: agent.project_id,
            status: agent.status,
            created_at: agent.created_at,
            updated_at: agent.updated_at,
        }
    }
}

/// Paginated response for agent listing
#[derive(Debug, Serialize, Deserialize)]
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
    State(state): State<AgentState>,
    Query(query): Query<ListAgentsQuery>,
    user: AuthenticatedUser,
) -> Result<Json<PaginatedAgentsResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Validation
    let mut validation_errors = std::collections::HashMap::new();
    if query.page < 1 {
        validation_errors.insert("page".to_string(), "Must be >= 1".to_string());
    }
    if query.per_page < 1 || query.per_page > 100 {
        validation_errors.insert("per_page".to_string(), "Must be between 1 and 100".to_string());
    }
    
    let allowed_sort_fields = ["name", "budget", "created_at"];
    let sort_field_str = query.sort.strip_prefix('-').unwrap_or(&query.sort);
    if !allowed_sort_fields.contains(&sort_field_str) {
         validation_errors.insert("sort".to_string(), "Invalid sort field (allowed: name, budget, created_at)".to_string());
    }

    if !validation_errors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: None,
                    fields: Some(validation_errors),
                },
            }),
        ));
    }

    let service = &state.agent_service;

    // Parse sort parameter
    let (sort_field, sort_direction) = parse_sort(&query.sort);

    // Clamp per_page to max 100
    let per_page = query.per_page.min(100);

    let filters = ListAgentsFilters {
        user_id: if user.0.role == "admin" {
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
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
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
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<Agent>, (StatusCode, String)> {
    let service = &state.agent_service;

    let agent = service.get_agent(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?
    .ok_or((StatusCode::NOT_FOUND, "Agent not found".to_string()))?;

    // Check if user has access (admin or owns the agent)
    if user.0.role != "admin" && agent.user_id != user.0.sub {
        return Err((
            StatusCode::FORBIDDEN,
            "You don't have access to this agent".to_string(),
        ));
    }

    Ok(Json(Agent {
        id: agent.id,
        name: agent.name,
        budget: agent.budget,
        providers: agent.providers,
        description: agent.description,
        tags: agent.tags,
        owner_id: agent.user_id,
        project_id: agent.project_id,
        status: agent.status,
        created_at: agent.created_at,
        updated_at: agent.updated_at,
        ic_token: None,
    }))
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

/// Create a new agent (admin only)
pub async fn create_agent(
    State(state): State<AgentState>,
    user: AuthenticatedUser,
    Json(req): Json<CreateAgentRequest>,
) -> Result<(StatusCode, Json<Agent>), (StatusCode, Json<ErrorResponse>)> {
    // Validation
    let mut validation_errors = std::collections::HashMap::new();
    if req.budget < 0.01 {
        validation_errors.insert("budget".to_string(), "Must be >= 0.01".to_string());
    }
    if req.name.trim().is_empty() {
        validation_errors.insert("name".to_string(), "Required field".to_string());
    }

    if !validation_errors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: None,
                    fields: Some(validation_errors),
                },
            }),
        ));
    }

    println!("role {}", user.0.role);

    // Permission check
    if user.0.role != "admin" && user.0.sub != req.owner_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    // Provider existence check
    if let Some(providers) = &req.providers {
        for provider_id in providers {
            let exists: Option<String> = sqlx::query_scalar("SELECT id FROM ai_provider_keys WHERE id = ?")
                .bind(provider_id)
                .fetch_optional(state.token_storage.pool())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ErrorResponse {
                            error: ErrorDetail {
                                code: "INTERNAL_ERROR".to_string(),
                                message: Some(format!("Database error: {}", e)),
                                fields: None,
                            },
                        }),
                    )
                })?;

            if exists.is_none() {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ErrorResponse {
                        error: ErrorDetail {
                            code: "PROVIDER_NOT_FOUND".to_string(),
                            message: Some(format!("Provider '{}' does not exist", provider_id)),
                            fields: None,
                        },
                    }),
                ));
            }
        }
    }
    
    let service = &state.agent_service;
    let generator = TokenGenerator::new();
    let token_service = &state.token_storage;

    let plaintext_token = generator.generate();
    let providers_string = serde_json::to_string(&req.providers).map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INVALID_PROVIDERS_FORMAT".to_string(),
                    message: Some(format!("Failed to serialize providers: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;
        

    let params = CreateAgentParams {
        name: req.name,
        budget: req.budget,
        providers: req.providers,
        description: req.description,
        tags: req.tags,
        project_id: None, // leave empty for now as project is not supported yet
    };

    let agent = service.create_agent(params, &req.owner_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;

    let token = token_service.create_token(&plaintext_token, &req.owner_id, None, None, Some(&agent.id), Some(&providers_string)).await.map_err(|e| {
           (StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Failed to create agent token: {}", e)),
                    fields: None,
                },
            }))
        })?;

    Ok((StatusCode::CREATED, Json(Agent {
        id: agent.id,
        name: agent.name,
        budget: agent.budget,
        providers: agent.providers,
        description: agent.description,
        tags: agent.tags,
        owner_id: agent.user_id,
        project_id: agent.project_id,
        status: agent.status,
        created_at: agent.created_at,
        updated_at: agent.updated_at,
        ic_token: Some(ICToken { 
            id: token.id.to_string(), 
            token: token.token, 
            created_at: chrono::DateTime::from_timestamp(token.created_at / 1000, ((token.created_at % 1000) * 1_000_000) as u32)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default()
        }),
    })))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProviderListItem {
    id: String,
    name: String,
    endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentDetails {
    pub id: String,
    pub name: String,
    pub budget: f64,
    pub spent: f64,
    pub remaining: f64,
    pub percent_used: f64,
    pub providers: Vec<ProviderListItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    pub owner_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_id: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String
}

/// Get agent details
pub async fn get_agent_details(
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<AgentDetails>, (StatusCode, Json<ErrorResponse>)> {
    let service = &state.agent_service;

    let agent = service.get_agent_details(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;

    let agent = match agent {
        Some(a) => a,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "AGENT_NOT_FOUND".to_string(),
                    message: Some(format!("Agent '{}' does not exist", id)),
                    fields: None,
                },
            }),
        )),
    };

    if user.0.sub != agent.user_id && user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    Ok(Json(AgentDetails {
        id: agent.id,
        name: agent.name,
        budget: agent.budget,
        spent: agent.spent,
        remaining: agent.remaining,
        percent_used: agent.percent_used,
        providers: agent.providers.into_iter().map(|p| ProviderListItem {
            id: p.id,
            name: p.name,
            endpoint: p.endpoint,
        }).collect(),
        description: agent.description,
        tags: agent.tags,
        owner_id: agent.user_id,
        project_id: agent.project_id,
        status: agent.status,
        created_at: agent.created_at.to_string(),
        updated_at: agent.updated_at.to_string(),
    }))
}

/// Update an agent (admin only)
pub async fn update_agent(
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
    Json(req): Json<UpdateAgentRequest>,
) -> Result<Json<Agent>, (StatusCode, Json<ErrorResponse>)> {
    let service = &state.agent_service;

    // Fetch agent first to check existence and permissions
    let agent = service.get_agent(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;

    let agent = match agent {
        Some(a) => a,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "AGENT_NOT_FOUND".to_string(),
                    message: Some(format!("Agent '{}' does not exist", id)),
                    fields: None,
                },
            }),
        )),
    };

    // Permission check
    if user.0.role != "admin" && user.0.sub != agent.user_id {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    // Validation
    let mut validation_errors = std::collections::HashMap::new();
    if let Some(name) = &req.name {
        if name.len() < 1 || name.len() > 100 {
             validation_errors.insert("name".to_string(), "Must be between 1 and 100 characters".to_string());
        }
    }
    if let Some(tags) = &req.tags {
        if tags.len() > 20 {
             validation_errors.insert("tags".to_string(), "Maximum 20 tags allowed".to_string());
        }
    }

    if !validation_errors.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: None,
                    fields: Some(validation_errors),
                },
            }),
        ));
    }

    // Check if any fields provided
    if req.name.is_none() && req.description.is_none() && req.tags.is_none() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "NO_FIELDS_PROVIDED".to_string(),
                    message: Some("At least one field must be updated".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    let params = UpdateAgentParams {
        name: req.name,
        description: req.description,
        tags: req.tags,
    };

    let updated_agent = service.update_agent(&id, params).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?
    .ok_or((
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: ErrorDetail {
                code: "AGENT_NOT_FOUND".to_string(),
                message: Some(format!("Agent '{}' does not exist", id)),
                fields: None,
            },
        }),
    ))?;
    
    Ok(Json(Agent {
        id: updated_agent.id,
        name: updated_agent.name,
        budget: updated_agent.budget,
        providers: updated_agent.providers,
        description: updated_agent.description,
        tags: updated_agent.tags,
        owner_id: updated_agent.user_id,
        project_id: updated_agent.project_id,
        status: updated_agent.status,
        created_at: updated_agent.created_at,
        updated_at: updated_agent.updated_at,
        ic_token: None,
    }))
}

/// Delete an agent (admin only)
pub async fn delete_agent(
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // Only admins can delete agents
    if user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            "Only administrators can delete agents".to_string(),
        ));
    }

    let service = &state.agent_service;

    let deleted = service.delete_agent(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    if !deleted {
        return Err((StatusCode::NOT_FOUND, "Agent not found".to_string()));
    }

    Ok((StatusCode::NO_CONTENT, "Agent deleted successfully".to_string()))
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
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<Vec<AgentTokenItem>>, (StatusCode, String)> {
    let service = &state.agent_service;

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


#[derive(Debug, Serialize, Deserialize)]
pub struct AgentProviderItem {
    pub id: String,
    pub name: String,
    pub endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AgentProviderItemExtended {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub models: Vec<String>,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetAgentProvidersResponse {
    pub agent_id: String,
    pub providers: Vec<AgentProviderItemExtended>,
    pub count: usize,
}

/// Get agent providers
pub async fn get_agent_providers(
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
) -> Result<Json<GetAgentProvidersResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = &state.agent_service;

    let agent = service.get_agent_details(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;

    let agent = match agent {
        Some(a) => a,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "AGENT_NOT_FOUND".to_string(),
                    message: Some(format!("Agent not found: {}", id)),
                    fields: None,
                },
            }),
        )),
    };

    if user.0.sub != agent.user_id && user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions: You can only view your own agents".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    let providers: Vec<AgentProviderItemExtended> = agent.providers.into_iter().map(|p| AgentProviderItemExtended {
        id: p.id,
        name: p.name,
        endpoint: p.endpoint,
        models: p.models,
        status: p.status,
    }).collect();

    Ok(Json(GetAgentProvidersResponse {
        agent_id: agent.id,
        count: providers.len(),
        providers,
    }))
}


#[derive(Debug, Serialize, Deserialize)]
pub struct AssignProvidersToAgentRequest {
    pub providers: Vec<String>,
}

/// Assign providers to an agent
pub async fn assign_providers_to_agent(
    State(state): State<AgentState>,
    Path(id): Path<String>,
    user: AuthenticatedUser,
    Json(req): Json<AssignProvidersToAgentRequest>,
) -> Result<Json<Agent>, (StatusCode, Json<ErrorResponse>)> {
    let service = AgentService::new(state.db_pool.clone());

    // Validation
    if req.providers.is_empty() {
        let mut validation_errors = std::collections::HashMap::new();
        validation_errors.insert("providers".to_string(), "Required field, must be array of provider IDs".to_string());
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "VALIDATION_ERROR".to_string(),
                    message: Some("providers field is required".to_string()),
                    fields: Some(validation_errors),
                },
            }),
        ));
    }
    
    let agent = service.get_agent_details(&id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;


    let agent = match agent {
        Some(a) => a,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "AGENT_NOT_FOUND".to_string(),
                    message: Some(format!("Agent not found: {}", id)),
                    fields: None,
                },
            }),
        )),
    };

    if user.0.sub != agent.user_id && user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions: You can only modify providers for your own agents".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    // Check providers existence
    for provider_id in &req.providers {
        let exists: Option<String> = sqlx::query_scalar("SELECT id FROM ai_provider_keys WHERE id = ?")
            .bind(provider_id)
            .fetch_optional(&state.db_pool)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: ErrorDetail {
                            code: "INTERNAL_ERROR".to_string(),
                            message: Some(format!("Database error: {}", e)),
                            fields: None,
                        },
                    }),
                )
            })?;

        if exists.is_none() {
            let mut fields = std::collections::HashMap::new();
            fields.insert("providers".to_string(), "One or more provider IDs are invalid".to_string());
            return Err((
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: ErrorDetail {
                        code: "INVALID_PROVIDER_ID".to_string(),
                        message: Some(format!("Provider not found: {}", provider_id)),
                        fields: Some(fields),
                    },
                }),
            ));
        }
    }

    let agent = service.assign_providers_to_agent(&id, req.providers).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?
    .ok_or((
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: ErrorDetail {
                code: "INVALID_PROVIDER_ID".to_string(),
                message: Some("Providers not found".to_string()),
                fields: None,
            },
        }),
    ))?;

    Ok(Json(Agent {
        id: agent.id,
        name: agent.name,
        budget: agent.budget,
        providers: agent.providers,
        description: agent.description,
        tags: agent.tags,
        owner_id: agent.user_id,
        project_id: agent.project_id,
        status: agent.status,
        created_at: agent.created_at,
        updated_at: agent.updated_at,
        ic_token: None,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemainedProviderItem {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveProviderFromAgentResponse {
    pub agent_id: String,
    pub provider_id: String,
    pub removed: bool,
    pub remaining_providers: Vec< RemainedProviderItem >,
    pub count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub warning: Option<String>,
}

/// Remove provider from an agent
pub async fn remove_provider_from_agent(
    State(state): State<AgentState>,
    Path((agent_id, provider_id)): Path<(String, String)>,
    user: AuthenticatedUser,
) -> Result<Json<RemoveProviderFromAgentResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = state.agent_service;

    let agent = service.get_agent_details(&agent_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?;

    let agent = match agent {
        Some(a) => a,
        None => return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "AGENT_NOT_FOUND".to_string(),
                    message: Some(format!("Agent not found: {}", agent_id)),
                    fields: None,
                },
            }),
        )),
    };

    if user.0.sub != agent.user_id && user.0.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "FORBIDDEN".to_string(),
                    message: Some("Insufficient permissions: You can only modify your own agents".to_string()),
                    fields: None,
                },
            }),
        ));
    }

    // Check if provider is assigned
    let is_assigned = agent.providers.iter().any(|p| p.id == provider_id);
    if !is_assigned {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "PROVIDER_NOT_ASSIGNED".to_string(),
                    message: Some(format!("Provider {} is not assigned to agent {}", provider_id, agent_id)),
                    fields: None,
                },
            }),
        ));
    }

    let providers_list: Vec<RemainedProviderItem> = service.remove_provider_from_agent(&agent_id, &provider_id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: ErrorDetail {
                    code: "INTERNAL_ERROR".to_string(),
                    message: Some(format!("Database error: {}", e)),
                    fields: None,
                },
            }),
        )
    })?.into_iter().map(|p| RemainedProviderItem {
        id: p.id,
        name: p.name,
    }).collect();

    let count = providers_list.len();

    Ok(Json(RemoveProviderFromAgentResponse {
        agent_id: agent.id,
        provider_id,
        removed: true,
        remaining_providers: providers_list,
        count,
        warning: if count == 0 { Some("Agent has zero providers and cannot make inference requests until provider assigned".to_string()) } else { None },
    }))
}

// TODO: implement get_agent_status

// /// Get agent status
// pub async fn get_agent_status(
//     State(pool): State<SqlitePool>,
//     Path(id): Path<String>,
//     user: AuthenticatedUser,
// ) -> Result<Json<Agent>, (StatusCode, Json<ErrorResponse>)> {
//     let service = AgentService::new(pool);

//     let agent = service.get_agent_details(&id).await.map_err(|e| {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             Json(ErrorResponse {
//                 error: ErrorDetail {
//                     code: "INTERNAL_ERROR".to_string(),
//                     message: Some(format!("Database error: {}", e)),
//                     fields: None,
//                 },
//             }),
//         )
//     })?;

//     let agent = match agent {
//         Some(a) => a,
//         None => return Err((
//             StatusCode::NOT_FOUND,
//             Json(ErrorResponse {
//                 error: ErrorDetail {
//                     code: "AGENT_NOT_FOUND".to_string(),
//                     message: Some(format!("Agent '{}' does not exist", id)),
//                     fields: None,
//                 },
//             }),
//         )),
//     };

//     if user.0.sub != agent.user_id && user.0.role != "admin" {
//         return Err((
//             StatusCode::FORBIDDEN,
//             Json(ErrorResponse {
//                 error: ErrorDetail {
//                     code: "FORBIDDEN".to_string(),
//                     message: Some("Insufficient permissions".to_string()),
//                     fields: None,
//                 },
//             }),
//         ));
//     }

//     Ok(Json(Agent::from(agent)))
// }