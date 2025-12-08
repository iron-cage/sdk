//! State management for Iron Cage runtime
//!
//! Provides unified state storage with multiple backends:
//! - In-memory (DashMap) for fast access
//! - SQLite for persistent audit logs
//! - Redis for distributed state (optional)
//!
//! Features #25: State Management

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
mod implementation
{
  use dashmap::DashMap;
  use serde::{Deserialize, Serialize};
  use std::sync::Arc;

  /// Agent state stored in memory
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct AgentState
  {
    pub agent_id: String,
    pub status: AgentStatus,
    pub budget_spent: f64,
    pub pii_detections: usize,
  }

  /// Agent execution status
  #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
  pub enum AgentStatus
  {
    Running,
    Stopped,
    Failed,
  }

  /// Audit log event
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct AuditEvent
  {
    pub agent_id: String,
    pub event_type: String,
    pub timestamp: i64,
    pub details: String,
  }

  /// State manager with multiple backends
  pub struct StateManager
  {
    memory: Arc<DashMap<String, AgentState>>,
    #[cfg(feature = "sqlite")]
    #[allow(dead_code)] // SQLite backend field, set via with_sqlite() but operations not yet implemented
    db: Option<sqlx::SqlitePool>,
  }

  impl StateManager
  {
    /// Create new state manager (in-memory only)
    pub fn new() -> Self
    {
      Self {
        memory: Arc::new(DashMap::new()),
        #[cfg(feature = "sqlite")]
        db: None,
      }
    }

    /// Get agent state from memory
    pub fn get_agent_state(&self, agent_id: &str) -> Option<AgentState>
    {
      self.memory.get(agent_id).map(|entry| entry.value().clone())
    }

    /// Save agent state to memory
    pub fn save_agent_state(&self, state: AgentState)
    {
      self.memory.insert(state.agent_id.clone(), state);
    }

    /// Save audit log event (memory only for now)
    pub fn save_audit_log(&self, event: AuditEvent)
    {
      // TODO: Implement SQLite persistence when feature enabled
      tracing::debug!(
        agent_id = %event.agent_id,
        event_type = %event.event_type,
        "Audit event logged"
      );
    }

    /// List all agent IDs
    pub fn list_agents(&self) -> Vec<String>
    {
      self.memory.iter().map(|entry| entry.key().clone()).collect()
    }
  }

  impl Default for StateManager
  {
    fn default() -> Self
    {
      Self::new()
    }
  }

  #[cfg(feature = "sqlite")]
  impl StateManager
  {
    /// Create state manager with SQLite backend
    pub async fn with_sqlite(db_path: &str) -> Result<Self, sqlx::Error>
    {
      let pool = sqlx::SqlitePool::connect(db_path).await?;

      Ok(Self {
        memory: Arc::new(DashMap::new()),
        db: Some(pool),
      })
    }
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;

#[cfg(not(feature = "enabled"))]
mod stub
{
  /// Stub agent state
  #[derive(Debug, Clone)]
  pub struct AgentState
  {
    pub agent_id: String,
    pub status: AgentStatus,
    pub budget_spent: f64,
    pub pii_detections: usize,
  }

  /// Stub status
  #[derive(Debug, Clone, Copy)]
  pub enum AgentStatus
  {
    Running,
    Stopped,
    Failed,
  }

  /// Stub audit event
  #[derive(Debug, Clone)]
  pub struct AuditEvent
  {
    pub agent_id: String,
    pub event_type: String,
    pub timestamp: i64,
    pub details: String,
  }

  /// Stub state manager
  pub struct StateManager;

  impl StateManager
  {
    pub fn new() -> Self
    {
      Self
    }

    pub fn get_agent_state(&self, _agent_id: &str) -> Option<AgentState>
    {
      None
    }

    pub fn save_agent_state(&self, _state: AgentState) {}

    pub fn save_audit_log(&self, _event: AuditEvent) {}

    pub fn list_agents(&self) -> Vec<String>
    {
      vec![]
    }
  }

  impl Default for StateManager
  {
    fn default() -> Self
    {
      Self::new()
    }
  }
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
