//! Centralized logging and tracing abstraction for Iron Cage runtime
//!
//! Provides unified logging infrastructure across all Iron Cage crates.
//! Features:
//! - Structured logging with tracing
//! - Multiple output formats (JSON, text)
//! - Log level configuration via environment variables
//! - Agent context injection in all logs

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
mod implementation
{
  use tracing::level_filters::LevelFilter;

  /// Log level configuration
  #[derive(Debug, Clone, Copy)]
  pub enum LogLevel
  {
    Debug,
    Info,
    Warn,
    Error,
  }

  impl From<LogLevel> for LevelFilter
  {
    fn from(level: LogLevel) -> Self
    {
      match level
      {
        LogLevel::Debug => LevelFilter::DEBUG,
        LogLevel::Info => LevelFilter::INFO,
        LogLevel::Warn => LevelFilter::WARN,
        LogLevel::Error => LevelFilter::ERROR,
      }
    }
  }

  /// Initialize logging infrastructure
  ///
  /// Sets up tracing subscriber with specified log level.
  /// Call this once at application startup.
  pub fn init_logging(level: LogLevel) -> Result<(), Box<dyn std::error::Error>>
  {
    use tracing_subscriber::FmtSubscriber;

    let subscriber = FmtSubscriber::builder()
      .with_max_level(level)
      .with_target(false)
      .with_thread_ids(true)
      .with_line_number(true)
      .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
  }

  /// Log an agent lifecycle event
  pub fn log_agent_event(agent_id: &str, event: &str)
  {
    tracing::info!(
      agent_id = %agent_id,
      event = %event,
      "Agent event"
    );
  }

  /// Log a PII detection event
  pub fn log_pii_detection(agent_id: &str, pii_type: &str, location: usize)
  {
    tracing::warn!(
      agent_id = %agent_id,
      pii_type = %pii_type,
      location = location,
      "PII detected"
    );
  }

  /// Log a budget warning
  pub fn log_budget_warning(agent_id: &str, spent: f64, limit: f64)
  {
    tracing::warn!(
      agent_id = %agent_id,
      spent = spent,
      limit = limit,
      percentage = (spent / limit) * 100.0,
      "Budget threshold reached"
    );
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;

#[cfg(not(feature = "enabled"))]
mod stub
{
  /// Stub log level for disabled feature
  #[derive(Debug, Clone, Copy)]
  pub enum LogLevel
  {
    Debug,
    Info,
    Warn,
    Error,
  }

  /// Stub init function
  pub fn init_logging(_level: LogLevel) -> Result<(), Box<dyn std::error::Error>>
  {
    Ok(())
  }

  /// Stub log function
  pub fn log_agent_event(_agent_id: &str, _event: &str) {}

  /// Stub log function
  pub fn log_pii_detection(_agent_id: &str, _pii_type: &str, _location: usize) {}

  /// Stub log function
  pub fn log_budget_warning(_agent_id: &str, _spent: f64, _limit: f64) {}
}

#[cfg(not(feature = "enabled"))]
pub use stub::*;
