//! Iron Cage Runtime - Agent lifecycle management + LLM Router + PyO3 bridge
//!
//! Core runtime for AI agent execution with safety and cost controls.
//! Provides Python bindings via PyO3 for LangChain/CrewAI integration.
//!
//! Features:
//! - #1: Agent Lifecycle Management
//! - #2: Python-Rust Integration (PyO3)
//! - #3: LLM Router - Local proxy for OpenAI/Anthropic API requests
//! - #16-18: Demo agent (in python/examples/)

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

// LLM Router module
#[cfg(feature = "enabled")]
pub mod llm_router;

#[cfg(feature = "enabled")]
#[allow(clippy::useless_conversion)] // PyO3 macros generate useless conversions in PyResult return types
mod implementation
{
  use std::sync::Arc;

  /// Runtime configuration
  #[derive(Debug, Clone)]
  pub struct RuntimeConfig
  {
    pub budget: f64,
    pub verbose: bool,
  }

  /// Agent runtime handle
  pub struct AgentHandle
  {
    pub agent_id: String,
  }

  /// Main agent runtime
  pub struct AgentRuntime
  {
    #[allow(dead_code)] // Configuration stored for future use (budget enforcement, etc.)
    config: RuntimeConfig,
    state: Arc<iron_state::StateManager>,
  }

  impl AgentRuntime
  {
    /// Create new runtime with configuration
    pub fn new(config: RuntimeConfig) -> Self
    {
      Self {
        config,
        state: Arc::new(iron_state::StateManager::new()),
      }
    }

    /// Start an agent from Python script path
    pub async fn start_agent(&self, _script_path: &std::path::Path) -> Result<AgentHandle, anyhow::Error>
    {
      // TODO: Implement agent spawning and PyO3 bridge
      let agent_id = format!("agent-{}", uuid::Uuid::new_v4());

      iron_telemetry::log_agent_event(&agent_id, "agent_started");

      // Save initial state
      self.state.save_agent_state(iron_state::AgentState {
        agent_id: agent_id.clone(),
        status: iron_state::AgentStatus::Running,
        budget_spent: 0.0,
        pii_detections: 0,
      });

      Ok(AgentHandle { agent_id })
    }

    /// Stop a running agent
    pub async fn stop_agent(&self, agent_id: &str) -> Result<(), anyhow::Error>
    {
      iron_telemetry::log_agent_event(agent_id, "agent_stopped");

      if let Some(mut state) = self.state.get_agent_state(agent_id)
      {
        state.status = iron_state::AgentStatus::Stopped;
        self.state.save_agent_state(state);
      }

      Ok(())
    }

    /// Get agent metrics
    pub fn get_metrics(&self, agent_id: &str) -> Option<iron_state::AgentState>
    {
      self.state.get_agent_state(agent_id)
    }
  }

  // PyO3 bridge module
  #[cfg(feature = "pyo3")]
  pub mod pyo3_bridge
  {
    use pyo3::prelude::*;

    /// Python-exposed Runtime class
    #[pyclass]
    pub struct Runtime
    {
      inner: super::AgentRuntime,
    }

    #[pymethods]
    impl Runtime
    {
      /// Create new runtime
      #[new]
      #[pyo3(signature = (budget, verbose=None))]
      fn new(budget: f64, verbose: Option<bool>) -> Self
      {
        let config = super::RuntimeConfig {
          budget,
          verbose: verbose.unwrap_or(false),
        };

        Self {
          inner: super::AgentRuntime::new(config),
        }
      }

      /// Start an agent (synchronous wrapper for async)
      fn start_agent(&self, _script_path: String) -> PyResult<String>
      {
        // Async bridge with pyo3-asyncio not yet implemented
        // For now, return a placeholder
        Ok("agent-placeholder".to_string())
      }

      /// Stop an agent
      fn stop_agent(&self, _agent_id: String) -> PyResult<()>
      {
        // Async bridge not yet implemented
        Ok(())
      }

      /// Get agent metrics as JSON string
      fn get_metrics(&self, agent_id: String) -> PyResult<Option<String>>
      {
        match self.inner.get_metrics(&agent_id)
        {
          Some(state) => {
            let json = serde_json::json!({
              "agent_id": state.agent_id,
              "status": format!("{:?}", state.status),
              "budget_spent": state.budget_spent,
              "pii_detections": state.pii_detections,
            });
            Ok(Some(json.to_string()))
          }
          None => Ok(None),
        }
      }
    }

    /// Python module definition
    #[pymodule]
    fn iron_runtime(m: &Bound<'_, PyModule>) -> PyResult<()>
    {
      m.add_class::<Runtime>()?;
      m.add_class::<crate::llm_router::LlmRouter>()?;
      Ok(())
    }
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;

#[cfg(not(feature = "enabled"))]
mod stub
{
  use std::path::Path;

  /// Stub runtime config
  #[derive(Debug, Clone)]
  pub struct RuntimeConfig
  {
    pub budget: f64,
    pub verbose: bool,
  }

  /// Stub agent handle
  pub struct AgentHandle
  {
    pub agent_id: String,
  }

  /// Stub runtime
  pub struct AgentRuntime
  {
    config: RuntimeConfig,
  }

  impl AgentRuntime
  {
    pub fn new(config: RuntimeConfig) -> Self
    {
      Self { config }
    }

    pub async fn start_agent(&self, _script_path: &Path) -> Result<AgentHandle, anyhow::Error>
    {
      Ok(AgentHandle {
        agent_id: "stub-agent".to_string(),
      })
    }

    pub async fn stop_agent(&self, _agent_id: &str) -> Result<(), anyhow::Error>
    {
      Ok(())
    }

    pub fn get_metrics(&self, _agent_id: &str) -> Option<iron_state::AgentState>
    {
      None
    }
  }
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
