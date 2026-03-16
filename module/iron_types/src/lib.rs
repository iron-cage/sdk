//! Foundational types and type-safe identifiers for Iron Runtime.
//!
//! Provides shared data structures, configuration types, error types, and
//! validated entity identifiers used across all Iron Runtime modules.
//! Ensures type safety and consistency throughout the system.
//!
//! # Purpose
//!
//! This crate serves as the common type foundation for Iron Runtime:
//! - Configuration schemas for all modules (safety, cost, reliability)
//! - Unified error taxonomy with domain-specific variants
//! - Type-safe entity identifiers with compile-time validation
//! - Zero-cost abstractions for common patterns
//!
//! # Key Types
//!
//! ## Configuration
//!
//! - [`Config`] - Root configuration containing all module configs
//! - [`SafetyConfig`] - PII detection and audit logging settings
//! - [`CostConfig`] - Budget limits and alert thresholds
//! - [`ReliabilityConfig`] - Circuit breaker and failover settings
//!
//! ## Error Handling
//!
//! - [`Error`] - Unified error type with domain variants
//! - [`Result<T>`] - Standard result alias for Iron Runtime operations
//!
//! ## Entity Identifiers
//!
//! Type-safe IDs with validation and backward compatibility:
//! - [`AgentId`] - Runtime AI agents (`agent_<uuid>`)
//! - [`ProviderId`] - LLM providers (`ip_<uuid>`)
//! - [`ProjectId`] - User projects (`proj_<uuid>`)
//! - [`ApiTokenId`] - API tokens (`at_<uuid>`)
//! - [`BudgetRequestId`] - Budget requests (`breq_<uuid>`)
//! - [`LeaseId`] - Budget leases (`lease_<uuid>`)
//! - [`RequestId`] - Generic requests (`req_<uuid>`)
//! - [`IcTokenId`] - Iron Cage tokens (`ic_<uuid>`)
//!
//! See [`ids`] module for comprehensive ID documentation.
//!
//! # Public API
//!
//! ## Configuration Loading
//!
//! ```rust
//! # #[cfg(feature = "enabled")]
//! # fn main() -> Result< (), serde_json::Error > {
//! use iron_types::Config;
//! use serde_json;
//!
//! let json = r#"{
//!   "safety": {
//!     "pii_detection_enabled": true,
//!     "audit_log_path": "/var/log/iron/audit.log"
//!   },
//!   "cost": {
//!     "budget_usd": 100.0,
//!     "alert_threshold": 0.8
//!   },
//!   "reliability": {
//!     "circuit_breaker_enabled": true,
//!     "failure_threshold": 5
//!   }
//! }"#;
//!
//! let config: Config = serde_json::from_str(json)?;
//! assert_eq!(config.cost.budget_usd, 100.0);
//! # Ok( () )
//! # }
//! ```
//!
//! ## Error Handling
//!
//! ```rust
//! # #[cfg(feature = "enabled")]
//! # {
//! use iron_types::{Error, Result};
//!
//! fn check_budget(spent: f64, limit: f64) -> Result<()> {
//!   if spent > limit {
//!     return Err(Error::BudgetExceeded(
//!       format!("Spent ${} exceeds limit ${}", spent, limit)
//!     ));
//!   }
//!   Ok(())
//! }
//!
//! match check_budget(150.0, 100.0) {
//!   Err(Error::BudgetExceeded(msg)) => {
//!     eprintln!("Budget error: {}", msg);
//!   }
//!   _ => {}
//! }
//! # }
//! ```
//!
//! ## Type-Safe Identifiers
//!
//! ```rust
//! # #[cfg(feature = "enabled")]
//! # fn main() -> Result< (), iron_types::IdError > {
//! use iron_types::{AgentId, ProjectId};
//!
//! // Generate new IDs
//! let agent_id = AgentId::generate();
//! let project_id = ProjectId::generate();
//!
//! // Type safety prevents mixing
//! fn start_agent(id: &AgentId) { /* ... */ }
//! fn load_project(id: &ProjectId) { /* ... */ }
//!
//! start_agent(&agent_id);     // ✓ Compiles
//! load_project(&project_id);  // ✓ Compiles
//! // start_agent(&project_id); // ✗ Compile error
//!
//! // Parse from strings with validation
//! let parsed = AgentId::parse("agent_550e8400-e29b-41d4-a716-446655440000")?;
//! # Ok( () )
//! # }
//! ```
//!
//! # Feature Flags
//!
//! - `enabled` - Enable all types (disabled for minimal builds)
//!
//! # Design Principles
//!
//! ## Type Safety Over Convenience
//!
//! Entity IDs use distinct types (`AgentId`, `ProjectId`) rather than generic
//! `String` or `Uuid`. This prevents ID misuse at compile time:
//!
//! ```compile_fail
//! # use iron_types::{AgentId, ProjectId};
//! fn process(agent: &AgentId, project: &ProjectId) { }
//!
//! let agent = AgentId::generate();
//! let project = ProjectId::generate();
//! process(&project, &agent); // Compile error: type mismatch
//! ```
//!
//! ## Zero-Cost Abstractions
//!
//! ID types are transparent wrappers with no runtime overhead:
//! - `AgentId` is `#[repr(transparent)]` over `String`
//! - `as_str()` returns `&str` without allocation
//! - Serialization uses direct string format
//!
//! ## Migration-Friendly
//!
//! Configuration uses `#[serde(default)]` to support gradual rollout:
//! - New fields have default values
//! - Old configs remain valid during upgrades
//! - Explicit settings override defaults

#![cfg_attr(not(feature = "enabled"), allow(unused))]

#[cfg(feature = "enabled")]
mod types {
  use serde::{Deserialize, Serialize};
  use thiserror::Error;

  /// Main configuration for Iron Cage runtime
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Config {
    /// Safety module configuration (PII detection, audit logging).
    pub safety: SafetyConfig,
    /// Cost module configuration (budget limits and alert thresholds).
    pub cost: CostConfig,
    /// Reliability module configuration (circuit breaker settings).
    pub reliability: ReliabilityConfig,
  }

  /// Safety module configuration
  #[derive(Debug, Clone, Serialize, Deserialize, Default)]
  pub struct SafetyConfig {
    /// Whether PII detection is active for agent outputs.
    #[serde(default)]
    pub pii_detection_enabled: bool,
    /// Optional path for writing structured audit log entries.
    #[serde(default)]
    pub audit_log_path: Option<String>,
  }

  /// Cost module configuration
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CostConfig {
    /// Maximum allowed spend in USD before the budget is considered exceeded.
    pub budget_usd: f64,
    /// Fraction of the budget (0.0–1.0) at which a warning alert is triggered.
    pub alert_threshold: f64,
  }

  /// Reliability module configuration
  #[derive(Debug, Clone, Serialize, Deserialize, Default)]
  pub struct ReliabilityConfig {
    /// Whether the circuit breaker is active for downstream service calls.
    #[serde(default)]
    pub circuit_breaker_enabled: bool,
    /// Number of consecutive failures before the circuit opens.
    #[serde(default)]
    pub failure_threshold: u32,
  }

  /// Common error type
  #[derive(Debug, Error)]
  pub enum Error {
    #[error("Safety violation: {0}")]
    /// A safety policy violation was detected (e.g. PII in output).
    Safety(String),

    #[error("Budget exceeded: {0}")]
    /// The agent's USD budget was exceeded.
    BudgetExceeded(String),

    #[error("Circuit breaker open: {0}")]
    /// A downstream service is unavailable — circuit breaker is open.
    CircuitBreakerOpen(String),

    #[error("Configuration error: {0}")]
    /// A configuration value is invalid or missing.
    Config(String),
  }

  /// Convenience `Result` alias using the crate's [`Error`] type.
  pub type Result<T> = core::result::Result<T, Error>;
}

#[cfg(feature = "enabled")]
pub use types::*;

#[cfg(feature = "enabled")]
pub mod ids;

#[cfg(feature = "enabled")]
pub use ids::{
  AgentId, ApiTokenId, BudgetRequestId, IcTokenId, IdError, LeaseId, ProjectId, ProviderId,
  RequestId,
};
