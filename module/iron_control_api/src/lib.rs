//! REST API + WebSocket server for Iron Cage dashboard
//!
//! Provides real-time communication between Rust runtime and web dashboard.
//!
//! Features:
//! - #26: API & Communication (REST + WebSocket)

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
pub mod jwt_auth;

#[cfg(feature = "enabled")]
pub mod rbac;

#[cfg(feature = "enabled")]
pub mod routes;

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
      println!("Stub API server would listen on {}", self.addr);
      Ok(())
    }
  }
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
