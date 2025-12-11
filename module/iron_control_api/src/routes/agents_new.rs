use std::{collections::HashMap, hash::Hash, str::FromStr, sync::Arc};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use chrono::{DateTime, Utc};
use iron_token_manager::agent_service::AgentService;
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite, SqlitePool};

use crate::{jwt_auth::AuthenticatedUser, rbac::{Permission, PermissionChecker, Role}, routes::auth::AuthState};

/// State for agent management routes
#[ derive( Clone ) ]
pub struct AgentManagementState
{
  pub db_pool: Pool< Sqlite >,
  pub permission_checker: Arc< PermissionChecker >,
}

impl AgentManagementState
{
  pub fn new( db_pool: Pool< Sqlite >, permission_checker: Arc< PermissionChecker > ) -> Self
  {
    Self {
      db_pool,
      permission_checker,
    }
  }
}

/// Request payload for creating a new agent
/// 
/// 
// Example:
/// 
/// {
///   "name": "Production Agent 1",
///   "budget": 100.00,
///   "providers": ["ip_openai_001", "ip_anthropic_001"],
///   "description": "Main production agent for customer requests",
///   "tags": ["production", "customer-facing"]
/// }
/// 
/// Validation rules:
/// 
/// | Field       | Type   | Required | Constraints              | Description                                      |
/// |-------------|--------|----------|--------------------------|--------------------------------------------------|
/// | name        | string | Yes      | 1-100 chars              | Human-readable agent name                        |
/// | budget      | number | Yes      | >= 0.01                  | Initial agent budget in USD (decimal, 2 places)  |
/// | providers   | array  | No       | Max unlimited            | Provider IDs agent can use (optional)            |
/// | description | string | No       | Max 500 chars            | Optional agent description                       |
/// | tags        | array  | No       | Max 20 tags, 50 chars ea | Optional tags for organization                   |
/// 
#[derive( Debug, Serialize, Deserialize, sqlx::FromRow )]
struct CreateAgentRequest 
{
    name: String,
    budget: f64,
    providers: Option< Vec< String > >,
    description: Option< String >,
    tags: Option< Vec< String > >,
}

/// Represents a single validation error for a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationProblem {
    pub field: String,
    pub message: String,
}

impl CreateAgentRequest {
    /// Maximum name length for DoS protection
    const MAX_NAME_LENGTH: usize = 100;

    /// Minimum budget
    const MIN_BUDGET: f64 = 0.01;

    /// Maximum description length
    const MAX_DESCRIPTION_LENGTH: usize = 500;

    /// Maximum number of tags
    const MAX_TAGS: usize = 20;

    /// Maximum tag length
    const MAX_TAG_LENGTH: usize = 50;

    /// Validates all fields and returns a vector of all validation problems.
    /// Returns an empty vector if all validations pass.
    fn validate(&self) -> Vec<ValidationProblem> {
        let mut problems = Vec::new();

        // Validate name
        if self.name.trim().is_empty() {
            problems.push(ValidationProblem {
                field: "name".to_string(),
                message: "Name cannot be empty.".to_string(),
            });
        } else if self.name.len() > Self::MAX_NAME_LENGTH {
            problems.push(ValidationProblem {
                field: "name".to_string(),
                message: "Name cannot exceed 100 characters.".to_string(),
            });
        }

        // Validate budget
        if self.budget < Self::MIN_BUDGET {
            problems.push(ValidationProblem {
                field: "budget".to_string(),
                message: "Budget must be at least 0.01 USD.".to_string(),
            });
        }

        // Validate description
        if let Some(desc) = &self.description {
            if desc.len() > Self::MAX_DESCRIPTION_LENGTH {
                problems.push(ValidationProblem {
                    field: "description".to_string(),
                    message: "Description cannot exceed 500 characters.".to_string(),
                });
            }
        }

        // Validate tags
        if let Some(tags) = &self.tags {
            if tags.len() > Self::MAX_TAGS {
                problems.push(ValidationProblem {
                    field: "tags".to_string(),
                    message: "Cannot have more than 20 tags.".to_string(),
                });
            }
            for (i, tag) in tags.iter().enumerate() {
                if tag.len() > Self::MAX_TAG_LENGTH {
                    problems.push(ValidationProblem {
                        field: format!("tags[{}]", i),
                        message: "Tag cannot exceed 50 characters.".to_string(),
                    });
                }
            }
        }

        problems
    }
}

/// IC Token details (shown ONLY on creation)
#[ derive( Debug, Serialize, Deserialize, sqlx::FromRow ) ]
struct ICToken {
    id: String,
    token: String,
    created_at: DateTime< Utc >,
}

/// Agent status
#[ derive( Debug, Serialize, Deserialize ) ]
enum AgentStatus 
{
    #[ serde( rename = "active" ) ]
    Active,
    #[ serde( rename = "inactive" ) ]
    Inactive,
    #[ serde( rename = "exhausted" ) ]
    Exhausted,
}

/// Error codes for agent operations
#[ derive( Debug, Serialize, Deserialize ) ]
enum AgentErrorCode 
{
    #[ serde( rename = "VALIDATION_ERROR" ) ]
    ValidationError,
    #[ serde( rename = "FORBIDDEN" ) ]
    Forbidden,
    #[ serde( rename = "NOT_FOUND" ) ]
    NotFound,
}


/// Response payload for creating a new agent
/// 
/// Example:
/// 
/// {
///   "id": "agent_abc123",
///   "name": "Production Agent 1",
///   "budget": 100.00,
///   "providers": ["ip_openai_001", "ip_anthropic_001"],
///   "description": "Main production agent for customer requests",
///   "tags": ["production", "customer-facing"],
///   "owner_id": "user_xyz789",
///   "project_id": "proj_master",
///   "ic_token": {
///     "id": "ic_def456ghi789",
///     "token": "ic_xyz789abc123def456...",
///     "created_at": "2025-12-10T10:30:45Z"
///   },
///   "status": "active",
///   "created_at": "2025-12-10T10:30:45Z",
///   "updated_at": "2025-12-10T10:30:45Z"
/// }

#[ derive( Debug, Serialize, Deserialize ) ]
struct CreateAgentResponse
{
    /// Unique agent identifier (agent- prefix)
    id: String,
    /// Agent name
    name: String,
    /// Current agent budget in USD
    budget: f64,
    /// Provider IDs assigned to agent
    providers: Vec< String >,
    /// Agent description (omitted if empty)
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    description: Option< String >,
    /// Agent tags (omitted if empty)
    tags: Vec< String >,
    /// User ID who created the agent (inferred from auth token)
    owner_id: String,
    /// Project ID (defaults to Master Project in Pilot)
    project_id: String,
    /// IC Token details (shown ONLY on creation)
    ic_token: ICToken,
    /// Agent
    status: AgentStatus,
    /// ISO 8601 timestamp
    created_at: DateTime< Utc >,
    /// ISO 8601 timestamp
    updated_at: DateTime< Utc >,
}

/// Error response payload
#[ derive( Debug, Serialize, Deserialize ) ]
struct ErrorResponse
{
    code: AgentErrorCode,
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    message: Option< String >,
    #[ serde( skip_serializing_if = "Option::is_none" ) ]
    fields: Option< HashMap< String, String > >,
}


/// POST /api/v1/agents
/// 
/// Creates a new agent with specified budget and provider assignments. Automatically generates an IC Token for agent authentication.
/// 
/// # Arguments
/// 
/// * `state` - Authentication state (JWT secret)
/// * `request` - Agent creation parameters
/// 
/// # Returns
/// 
/// - 201 Created with agent details if successful
/// - 400 Bad Request if validation fails
/// - 401 Unauthorized if authentication fails
/// - 500 Internal Server Error if agent creation fails
pub async fn create_agent(
    State( state ): State< AuthState >,
    AuthenticatedUser( claims ): AuthenticatedUser,
    Json( request ): Json< CreateAgentRequest >,
) -> impl IntoResponse
{
    // Validate request
    let validation_problems = request.validate();
    if !validation_problems.is_empty() {
        let validation_map: HashMap< String, String > = validation_problems
            .into_iter()
            .map( |problem| ( problem.field, problem.message ) )
            .collect();        

        let error_response = ErrorResponse {
            code: AgentErrorCode::ValidationError,
            message: None,
            fields: Some ( validation_map ),
        };
        return (StatusCode::BAD_REQUEST, Json(error_response));
    }

    let user_id = claims.sub;
    let role = Role::from_str( &claims.role ).unwrap_or( Role::User );
    // Check RBAC permission

    if !state.permission_checker.has_permission( role, Permission::CreateAgent ) {
        let error_response = serde_json::jsonErrorResponse {
            code: AgentErrorCode::Forbidden,
            message: Some( "Insufficient permissions".to_string() ),
            fields: None,
        };
        return ( StatusCode::FORBIDDEN, Json( error_response ) );
    }
    
    // Create agent service
    let agent_service = AgentService::new( state.db_pool.clone() );
    
    agent_service.create_agent(
        &request.name,
        &claims.sub,
        request.budget,
        &request.status,
        request.providers,
        request.description,
        request.tags,
    ).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json( serde_json::json!({ "error": format!( "Failed to create agent: {}", e ) }) ),
        )
    })?;
    

}