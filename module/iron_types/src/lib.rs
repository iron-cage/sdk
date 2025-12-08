//! Shared types and traits for Iron Cage
//!
//! This crate provides the foundational types used across all iron_cage modules.
//! All types are feature-gated behind the `enabled` feature.

#![cfg_attr(not(feature = "enabled"), allow(unused))]

#[cfg(feature = "enabled")]
mod types
{
  use serde::{Deserialize, Serialize};
  use thiserror::Error;

  /// Main configuration for Iron Cage runtime
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct Config
  {
    pub safety: SafetyConfig,
    pub cost: CostConfig,
    pub reliability: ReliabilityConfig,
  }

  /// Safety module configuration
  #[derive(Debug, Clone, Serialize, Deserialize, Default)]
  pub struct SafetyConfig
  {
    #[serde(default)]
    pub pii_detection_enabled: bool,
    #[serde(default)]
    pub audit_log_path: Option< String >,
  }

  /// Cost module configuration
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct CostConfig
  {
    pub budget_usd: f64,
    pub alert_threshold: f64,
  }

  /// Reliability module configuration
  #[derive(Debug, Clone, Serialize, Deserialize, Default)]
  pub struct ReliabilityConfig
  {
    #[serde(default)]
    pub circuit_breaker_enabled: bool,
    #[serde(default)]
    pub failure_threshold: u32,
  }

  /// Common error type
  #[derive(Debug, Error)]
  pub enum Error
  {
    #[error("Safety violation: {0}")]
    Safety(String),

    #[error("Budget exceeded: {0}")]
    BudgetExceeded(String),

    #[error("Circuit breaker open: {0}")]
    CircuitBreakerOpen(String),

    #[error("Configuration error: {0}")]
    Config(String),
  }

  pub type Result< T > = std::result::Result< T, Error >;
}

#[cfg(feature = "enabled")]
pub use types::*;
