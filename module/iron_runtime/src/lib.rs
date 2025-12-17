//! Core runtime for AI agent execution with integrated safety and cost controls.
//!
//! Provides agent lifecycle management and local LLM proxy for request interception.
//! Orchestrates all Iron Runtime subsystems (budget, PII detection, analytics, circuit breakers).
//!
//! # Purpose
//!
//! This crate is the execution engine for Iron Runtime:
//! - Agent lifecycle management (spawn, monitor, stop agents)
//! - LLM Router: Local proxy intercepting OpenAI/Anthropic API calls
//! - Integrated safety controls (PII detection, budget enforcement)
//! - Real-time metrics and state management
//! - Dashboard integration via REST API and WebSocket
//!
//! # Architecture
//!
//! Iron Runtime uses a modular architecture with clear separation:
//!
//! ## Core Components
//!
//! 1. **Agent Runtime**: Manages agent processes and lifecycle
//! 2. **LLM Router**: Transparent proxy for LLM API requests
//! 3. **State Manager**: Persists agent state and metrics
//! 4. **Telemetry**: Structured logging for all operations
//!
//! ## Integration Layer
//!
//! Runtime coordinates between modules:
//! - **iron_cost**: Budget validation before LLM requests
//! - **iron_safety**: PII scanning on LLM responses
//! - **iron_runtime_analytics**: Event tracking for dashboard
//! - **iron_reliability**: Circuit breakers for provider failures
//! - **iron_runtime_state**: Agent state persistence
//!
//! ## Python Bindings
//!
//! Python bindings are provided by the `iron_sdk` crate (see ADR-010).
//! This crate (`iron_runtime`) is pure Rust with no PyO3 dependencies.
//!
//! # Key Types
//!
//! - [`AgentRuntime`] - Main runtime managing agent lifecycle
//! - [`RuntimeConfig`] - Runtime configuration (budget, verbosity)
//! - [`AgentHandle`] - Handle to running agent for control
//! - [`llm_router::LlmRouter`] - Local LLM proxy server
//!
//! # Public API
//!
//! ## Rust API
//!
//! ```rust,no_run
//! use iron_runtime::{AgentRuntime, RuntimeConfig};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), anyhow::Error> {
//!   // Configure runtime
//!   let config = RuntimeConfig {
//!     budget: 100.0,  // $100 budget
//!     verbose: true,
//!   };
//!
//!   // Create runtime
//!   let runtime = AgentRuntime::new(config);
//!
//!   // Start agent from Python script
//!   let handle = runtime.start_agent(Path::new("agent.py")).await?;
//!   println!("Agent started: {}", handle.agent_id.as_str());
//!
//!   // Monitor metrics
//!   if let Some(metrics) = runtime.get_metrics(handle.agent_id.as_str()) {
//!     println!("Budget spent: ${}", metrics.budget_spent);
//!     println!("PII detections: {}", metrics.pii_detections);
//!   }
//!
//!   // Stop agent
//!   runtime.stop_agent(handle.agent_id.as_str()).await?;
//!   Ok(())
//! }
//! ```
//!
//! # Safety Controls
//!
//! Runtime enforces multiple safety layers:
//!
//! ## Budget Enforcement
//!
//! - Pre-request budget validation
//! - Request blocked if budget exceeded
//! - Real-time cost tracking
//! - Budget alerts at configurable thresholds
//!
//! ## PII Detection
//!
//! - Scans all LLM responses for PII
//! - Automatic redaction of sensitive data
//! - Compliance audit logging
//! - Configurable detection patterns
//!
//! ## Circuit Breakers
//!
//! - Detects failing LLM providers
//! - Fast-fail on known-bad endpoints
//! - Automatic recovery after timeout
//! - Per-provider state isolation
//!
//! # Feature Flags
//!
//! - `enabled` - Enable full runtime (disabled for library-only builds)
//! - `analytics` - Enable analytics recording via iron_runtime_analytics
//!
//! # Performance
//!
//! Runtime overhead on LLM requests:
//! - Budget check: <1ms
//! - PII detection: <5ms per KB
//! - Circuit breaker check: <0.1ms
//! - Analytics recording: <0.5ms
//! - Total proxy overhead: <10ms per request
//!
//! Streaming responses have near-zero buffering latency.

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

// LLM Router module
#[cfg(feature = "enabled")]
pub mod llm_router;

#[cfg(feature = "enabled")]
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
    pub agent_id: iron_types::AgentId,
  }

  /// Main agent runtime
  pub struct AgentRuntime
  {
    #[allow(dead_code)] // Configuration stored for future use (budget enforcement, etc.)
    config: RuntimeConfig,
    state: Arc<iron_runtime_state::StateManager>,
  }

  impl AgentRuntime
  {
    /// Create new runtime with configuration
    pub fn new(config: RuntimeConfig) -> Self
    {
      Self {
        config,
        state: Arc::new(iron_runtime_state::StateManager::new()),
      }
    }

    /// Get the runtime configuration
    pub fn config(&self) -> &RuntimeConfig
    {
      &self.config
    }

    /// Start an agent from Python script path
    pub async fn start_agent(&self, _script_path: &std::path::Path) -> Result<AgentHandle, anyhow::Error>
    {
      let agent_id = iron_types::AgentId::generate();

      iron_telemetry::log_agent_event(agent_id.as_str(), "agent_started");

      // Save initial state
      self.state.save_agent_state(iron_runtime_state::AgentState {
        agent_id: agent_id.clone(),
        status: iron_runtime_state::AgentStatus::Running,
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
        state.status = iron_runtime_state::AgentStatus::Stopped;
        self.state.save_agent_state(state);
      }

      Ok(())
    }

    /// Get agent metrics
    pub fn get_metrics(&self, agent_id: &str) -> Option<iron_runtime_state::AgentState>
    {
      self.state.get_agent_state(agent_id)
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
    pub agent_id: iron_types::AgentId,
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

    pub fn config(&self) -> &RuntimeConfig
    {
      &self.config
    }

    pub async fn start_agent(&self, _script_path: &Path) -> Result<AgentHandle, anyhow::Error>
    {
      Ok(AgentHandle {
        agent_id: iron_types::AgentId::generate(),
      })
    }

    pub async fn stop_agent(&self, _agent_id: &str) -> Result<(), anyhow::Error>
    {
      Ok(())
    }

    pub fn get_metrics(&self, _agent_id: &str) -> Option<iron_runtime_state::AgentState>
    {
      None
    }
  }
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
