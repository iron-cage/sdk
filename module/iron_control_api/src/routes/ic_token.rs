//! IC Token management REST API endpoints
//!
//! Provides endpoints for generating, managing, and revoking IC (Iron Cage) tokens
//! for agent authentication with the budget runtime.
//!
//! # Endpoints
//!
//! - POST /api/v1/agents/:id/ic-token - Generate IC token for agent
//! - GET /api/v1/agents/:id/ic-token - Get IC token status (not the actual token)
//! - POST /api/v1/agents/:id/ic-token/regenerate - Regenerate IC token (invalidates old)
//! - DELETE /api/v1/agents/:id/ic-token - Revoke IC token
//!
//! # Security
//!
//! - IC tokens are shown only once on creation (like API tokens)
//! - Only the SHA-256 hash is stored in the database
//! - Only agent owner or admin can manage IC tokens

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Serialize;
use sha2::{Sha256, Digest};
use sqlx::SqlitePool;
use std::sync::Arc;

use crate::ic_token::{IcTokenClaims, IcTokenManager};
use crate::jwt_auth::AuthenticatedUser;

/// IC Token route state
#[derive(Clone)]
pub struct IcTokenState {
    pub pool: SqlitePool,
    pub ic_token_manager: Arc<IcTokenManager>,
}

/// Response for IC token generation (includes actual token - shown only once)
#[derive(Debug, Serialize)]
pub struct IcTokenResponse {
    pub agent_id: i64,
    pub ic_token: String,
    pub created_at: i64,
    pub warning: String,
}

/// Response for IC token status (does NOT include actual token)
#[derive(Debug, Serialize)]
pub struct IcTokenStatusResponse {
    pub agent_id: i64,
    pub has_ic_token: bool,
    pub created_at: Option<i64>,
}

/// Helper to compute SHA-256 hash of a token
fn sha256_hash(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Check if user has access to agent (owner or admin)
async fn check_agent_access(
    pool: &SqlitePool,
    agent_id: i64,
    user_id: &str,
    user_role: &str,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    // Admin can access any agent
    if user_role == "admin" {
        return Ok(());
    }

    // Check if user owns the agent
    let owner_id: Option<String> = sqlx::query_scalar(
        "SELECT owner_id FROM agents WHERE id = ?"
    )
    .bind(agent_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": format!("Database error: {}", e)})),
    ))?;

    match owner_id {
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found", "code": "AGENT_NOT_FOUND"})),
        )),
        Some(owner) if owner == user_id => Ok(()),
        Some(_) => Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied", "code": "ACCESS_DENIED"})),
        )),
    }
}

/// POST /api/v1/agents/:id/ic-token
///
/// Generate a new IC token for an agent.
/// Returns 409 Conflict if agent already has an IC token.
pub async fn generate_ic_token(
    State(state): State<IcTokenState>,
    Path(agent_id): Path<i64>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
    // Check access
    if let Err(e) = check_agent_access(&state.pool, agent_id, &claims.sub, &claims.role).await {
        return e.into_response();
    }

    // Check if agent already has IC token
    let existing_hash: Option<Option<String>> = sqlx::query_scalar(
        "SELECT ic_token_hash FROM agents WHERE id = ?"
    )
    .bind(agent_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": format!("Database error: {}", e)})),
    ))
    .unwrap_or(None);

    if existing_hash.flatten().is_some() {
        return (
            StatusCode::CONFLICT,
            Json(serde_json::json!({
                "error": "Agent already has IC token. Use regenerate endpoint to replace it.",
                "code": "IC_TOKEN_EXISTS"
            })),
        ).into_response();
    }

    // Generate IC token
    let ic_claims = IcTokenClaims::new(
        format!("agent_{}", agent_id),
        format!("budget_{}", agent_id),  // Legacy field, kept for compatibility
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None,  // Long-lived, no expiration
    );

    let ic_token = match state.ic_token_manager.generate_token(&ic_claims) {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to generate IC token: {}", e)})),
            ).into_response();
        }
    };

    // Store hash in database
    let token_hash = sha256_hash(&ic_token);
    let created_at = chrono::Utc::now().timestamp();

    if let Err(e) = sqlx::query(
        "UPDATE agents SET ic_token_hash = ?, ic_token_created_at = ? WHERE id = ?"
    )
    .bind(&token_hash)
    .bind(created_at)
    .bind(agent_id)
    .execute(&state.pool)
    .await
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save IC token: {}", e)})),
        ).into_response();
    }

    // Return token (one-time display)
    (
        StatusCode::CREATED,
        Json(IcTokenResponse {
            agent_id,
            ic_token,
            created_at,
            warning: "Store this token securely. It will not be shown again.".to_string(),
        }),
    ).into_response()
}

/// GET /api/v1/agents/:id/ic-token
///
/// Get IC token status for an agent.
/// Does NOT return the actual token (security).
pub async fn get_ic_token_status(
    State(state): State<IcTokenState>,
    Path(agent_id): Path<i64>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
    // Check access
    if let Err(e) = check_agent_access(&state.pool, agent_id, &claims.sub, &claims.role).await {
        return e.into_response();
    }

    // Get IC token info
    let row: Option<(Option<String>, Option<i64>)> = sqlx::query_as(
        "SELECT ic_token_hash, ic_token_created_at FROM agents WHERE id = ?"
    )
    .bind(agent_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({"error": format!("Database error: {}", e)})),
    ))
    .unwrap_or(None);

    match row {
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found", "code": "AGENT_NOT_FOUND"})),
        ).into_response(),
        Some((hash, created_at)) => (
            StatusCode::OK,
            Json(IcTokenStatusResponse {
                agent_id,
                has_ic_token: hash.is_some(),
                created_at,
            }),
        ).into_response(),
    }
}

/// POST /api/v1/agents/:id/ic-token/regenerate
///
/// Regenerate IC token for an agent.
/// Invalidates the old token immediately.
pub async fn regenerate_ic_token(
    State(state): State<IcTokenState>,
    Path(agent_id): Path<i64>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
    // Check access
    if let Err(e) = check_agent_access(&state.pool, agent_id, &claims.sub, &claims.role).await {
        return e.into_response();
    }

    // Generate new IC token
    let ic_claims = IcTokenClaims::new(
        format!("agent_{}", agent_id),
        format!("budget_{}", agent_id),
        vec!["llm:call".to_string(), "analytics:write".to_string()],
        None,
    );

    let ic_token = match state.ic_token_manager.generate_token(&ic_claims) {
        Ok(token) => token,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": format!("Failed to generate IC token: {}", e)})),
            ).into_response();
        }
    };

    // Store new hash (invalidates old token)
    let token_hash = sha256_hash(&ic_token);
    let created_at = chrono::Utc::now().timestamp();

    let result = sqlx::query(
        "UPDATE agents SET ic_token_hash = ?, ic_token_created_at = ? WHERE id = ?"
    )
    .bind(&token_hash)
    .bind(created_at)
    .bind(agent_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found", "code": "AGENT_NOT_FOUND"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to save IC token: {}", e)})),
        ).into_response(),
        Ok(_) => (
            StatusCode::OK,
            Json(serde_json::json!({
                "agent_id": agent_id,
                "ic_token": ic_token,
                "created_at": created_at,
                "old_token_invalidated": true,
                "warning": "Old IC token is now invalid. Update your agent configuration."
            })),
        ).into_response(),
    }
}

/// DELETE /api/v1/agents/:id/ic-token
///
/// Revoke IC token for an agent.
/// Agent will not be able to authenticate until a new token is generated.
pub async fn revoke_ic_token(
    State(state): State<IcTokenState>,
    Path(agent_id): Path<i64>,
    AuthenticatedUser(claims): AuthenticatedUser,
) -> impl IntoResponse {
    // Check access
    if let Err(e) = check_agent_access(&state.pool, agent_id, &claims.sub, &claims.role).await {
        return e.into_response();
    }

    // Clear IC token hash
    let result = sqlx::query(
        "UPDATE agents SET ic_token_hash = NULL, ic_token_created_at = NULL WHERE id = ?"
    )
    .bind(agent_id)
    .execute(&state.pool)
    .await;

    match result {
        Ok(r) if r.rows_affected() == 0 => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent not found", "code": "AGENT_NOT_FOUND"})),
        ).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": format!("Failed to revoke IC token: {}", e)})),
        ).into_response(),
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
    }
}
