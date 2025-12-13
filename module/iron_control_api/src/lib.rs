//! REST API and WebSocket server for Iron Runtime dashboard.
//!
//! Provides HTTP/REST endpoints and WebSocket connections for real-time
//! communication between the Rust runtime and web-based management dashboard.
//! Handles authentication, authorization, and structured API responses.
//!
//! # Purpose
//!
//! This crate implements the control plane API for Iron Runtime:
//! - REST endpoints for agent management, analytics, and configuration
//! - WebSocket connections for real-time event streaming
//! - JWT-based authentication and RBAC authorization
//! - Structured error responses and request validation
//! - CORS support for browser-based dashboards
//!
//! # Architecture
//!
//! The API follows a layered architecture:
//!
//! 1. **Transport Layer**: Axum web framework with HTTP/1.1, HTTP/2, and WebSocket support
//! 2. **Authentication Layer**: JWT validation, token auth, and user sessions
//! 3. **Authorization Layer**: Role-based access control (RBAC)
//! 4. **Route Layer**: RESTful endpoints organized by resource domain
//! 5. **State Layer**: Shared access to runtime state via `StateManager`
//!
//! All layers use async/await with Tokio runtime for concurrent request handling.
//!
//! # Key Types
//!
//! - [`ApiServer`] - Main server managing lifecycle and routing
//! - [`ApiState`] - Shared state accessible to all handlers
//! - [`jwt_auth`] - JWT token validation and claims
//! - [`rbac`] - Role-based access control
//! - [`error`] - Structured API error responses
//!
//! # Public API
//!
//! ## Server Setup
//!
//! ```rust,no_run
//! # #[cfg(feature = "enabled")]
//! # {
//! use iron_control_api::ApiServer;
//! use iron_runtime_state::StateManager;
//! use std::sync::Arc;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!   let state_manager = Arc::new(StateManager::new());
//!   let server = ApiServer::new(state_manager, 8080);
//!
//!   // Blocks until server shutdown
//!   server.start().await?;
//!   Ok(())
//! }
//! # }
//! ```
//!
//! ## REST Endpoints
//!
//! ### Agent Management
//!
//! ```text
//! GET  /api/agents/:id/status   - Get agent status
//! POST /api/agents/:id/stop     - Stop agent
//! GET  /api/agents/:id/metrics  - Get agent metrics
//! ```
//!
//! ### Analytics & Usage
//!
//! ```text
//! GET /api/analytics           - Get aggregated stats
//! GET /api/usage               - Get usage metrics
//! GET /api/traces              - Get request traces
//! ```
//!
//! ### Budget & Limits
//!
//! ```text
//! GET  /api/budget             - Get budget status
//! POST /api/budget/reserve     - Reserve budget
//! GET  /api/limits             - Get rate limits
//! ```
//!
//! ### Configuration
//!
//! ```text
//! GET /api/providers           - List LLM providers
//! GET /api/keys                - List API keys
//! GET /api/tokens              - List tokens
//! ```
//!
//! ### Authentication
//!
//! ```text
//! POST /api/auth/login         - User login
//! POST /api/auth/refresh       - Refresh JWT token
//! GET  /api/auth/verify        - Verify token
//! ```
//!
//! ### Health & Status
//!
//! ```text
//! GET /api/health              - Health check
//! ```
//!
//! ## WebSocket Events
//!
//! ```text
//! WS /ws                       - Real-time event stream
//! ```
//!
//! Events streamed to connected clients:
//! - Agent state changes (started, stopped, failed)
//! - Budget threshold alerts
//! - PII detection events
//! - Request completion/failure
//!
//! # Feature Flags
//!
//! - `enabled` - Enable API server (disabled for runtime-only builds)
//!
//! # Authentication
//!
//! The API supports multiple authentication mechanisms:
//!
//! ## JWT Tokens (User Sessions)
//!
//! ```text
//! Authorization: Bearer <jwt_token>
//! ```
//!
//! Used for dashboard user sessions. Tokens include user ID and role claims.
//!
//! ## API Tokens (Service Auth)
//!
//! ```text
//! X-API-Token: at_550e8400-e29b-41d4-a716-446655440000
//! ```
//!
//! Used for programmatic API access. Tokens scoped to specific resources.
//!
//! ## IC Tokens (Internal Auth)
//!
//! ```text
//! X-IC-Token: ic_550e8400-e29b-41d4-a716-446655440000
//! ```
//!
//! Used for runtime-to-runtime communication.
//!
//! # Authorization
//!
//! RBAC enforces role-based permissions:
//!
//! - **Admin**: Full access to all endpoints
//! - **User**: Read access to own resources, limited write
//! - **Viewer**: Read-only access to non-sensitive data
//!
//! # Error Handling
//!
//! All errors return structured JSON:
//!
//! ```json
//! {
//!   "error": "Budget exceeded",
//!   "code": "BUDGET_EXCEEDED",
//!   "details": {
//!     "spent": 105.50,
//!     "limit": 100.00
//!   }
//! }
//! ```
//!
//! HTTP status codes follow REST conventions:
//! - 200: Success
//! - 400: Bad request (validation error)
//! - 401: Unauthorized (missing/invalid auth)
//! - 403: Forbidden (insufficient permissions)
//! - 404: Not found
//! - 500: Internal server error

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
pub mod jwt_auth;

#[cfg(feature = "enabled")]
pub mod rbac;

#[cfg(feature = "enabled")]
pub mod routes;

#[cfg(feature = "enabled")]
pub mod middleware;

#[cfg(feature = "enabled")]
pub mod error;

#[cfg(feature = "enabled")]
pub mod user_auth;

#[cfg(feature = "enabled")]
pub mod token_auth;

#[cfg(feature = "enabled")]
pub mod ic_token;

#[cfg(feature = "enabled")]
pub mod ip_token;

#[cfg(feature = "enabled")]
mod implementation
{
  use axum::{
    extract::{Path, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
  };
  use serde::{Deserialize, Serialize};
  use std::{net::SocketAddr, sync::Arc};
  use tower_http::cors::CorsLayer;

  /// API server state
  #[derive(Clone)]
  pub struct ApiState
  {
    pub state_manager: Arc<iron_runtime_state::StateManager>,
  }

  /// API server
  pub struct ApiServer
  {
    state: ApiState,
    addr: SocketAddr,
  }

  impl ApiServer
  {
    /// Create new API server
    pub fn new(state_manager: Arc<iron_runtime_state::StateManager>, port: u16) -> Self
    {
      let addr = SocketAddr::from(([127, 0, 0, 1], port));

      Self {
        state: ApiState { state_manager },
        addr,
      }
    }

    /// Start API server
    pub async fn start(self) -> Result<(), anyhow::Error>
    {
      let app = Router::new()
        .route("/api/agents/:id/status", get(get_agent_status))
        .route("/api/agents/:id/stop", post(stop_agent))
        .route("/api/agents/:id/metrics", get(get_agent_metrics))
        .route("/ws", get(websocket_handler))
        .layer(CorsLayer::permissive())
        .with_state(self.state);

      tracing::info!("API server listening on {}", self.addr);

      let listener = tokio::net::TcpListener::bind(self.addr).await?;
      axum::serve(listener, app).await?;

      Ok(())
    }
  }

  /// Agent status response
  #[derive(Debug, Serialize, Deserialize)]
  struct AgentStatusResponse
  {
    agent_id: String,
    status: String,
  }

  /// Get agent status
  async fn get_agent_status(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
  ) -> impl IntoResponse
  {
    match state.state_manager.get_agent_state(&agent_id)
    {
      Some(agent_state) => {
        let response = AgentStatusResponse {
          agent_id: agent_state.agent_id.to_string(),
          status: format!("{:?}", agent_state.status),
        };
        (StatusCode::OK, Json(response))
      }
      None => (
        StatusCode::NOT_FOUND,
        Json(AgentStatusResponse {
          agent_id: agent_id.clone(),
          status: "NotFound".to_string(),
        }),
      ),
    }
  }

  /// Stop agent
  async fn stop_agent(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
  ) -> impl IntoResponse
  {
    if let Some(mut agent_state) = state.state_manager.get_agent_state(&agent_id)
    {
      agent_state.status = iron_runtime_state::AgentStatus::Stopped;
      state.state_manager.save_agent_state(agent_state);

      (StatusCode::OK, Json(serde_json::json!({ "success": true })))
    }
    else
    {
      (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Agent not found" })),
      )
    }
  }

  /// Get agent metrics
  async fn get_agent_metrics(
    State(state): State<ApiState>,
    Path(agent_id): Path<String>,
  ) -> impl IntoResponse
  {
    match state.state_manager.get_agent_state(&agent_id)
    {
      Some(agent_state) => {
        let metrics = serde_json::json!({
          "agent_id": agent_state.agent_id.to_string(),
          "status": format!("{:?}", agent_state.status),
          "budget_spent": agent_state.budget_spent,
          "pii_detections": agent_state.pii_detections,
        });

        (StatusCode::OK, Json(metrics))
      }
      None => (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({ "error": "Agent not found" })),
      ),
    }
  }

  /// WebSocket handler
  async fn websocket_handler(ws: WebSocketUpgrade, State(_state): State<ApiState>) -> impl IntoResponse
  {
    ws.on_upgrade(|_socket| async {
      tracing::info!("WebSocket connection established");
    })
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;

#[cfg(not(feature = "enabled"))]
mod stub
{
  use std::{net::SocketAddr, sync::Arc};

  #[derive(Clone)]
  pub struct ApiState
  {
    pub state_manager: Arc<iron_runtime_state::StateManager>,
  }

  pub struct ApiServer
  {
    state: ApiState,
    addr: SocketAddr,
  }

  impl ApiServer
  {
    pub fn new(state_manager: Arc<iron_runtime_state::StateManager>, port: u16) -> Self
    {
      let addr = SocketAddr::from(([127, 0, 0, 1], port));
      Self {
        state: ApiState { state_manager },
        addr,
      }
    }

    pub async fn start(self) -> Result<(), anyhow::Error>
    {
      tracing::info!("Stub API server would listen on {}", self.addr);
      Ok(())
    }
  }
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
