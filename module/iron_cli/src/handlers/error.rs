//! Handler error types
//!
//! Errors returned by pure handler functions. These should NOT contain
//! async operations, I/O, or service-specific error types.

use core::fmt;

/// CLI error types returned by handler functions
#[derive(Debug, Clone, PartialEq)]
pub enum CliError {
  /// Required parameter missing from input
  MissingParameter(&'static str),

  /// Parameter present but invalid
  InvalidParameter {
    /// Name of the parameter that failed validation
    param: &'static str,
    /// Human-readable reason for validation failure
    reason: &'static str,
  },

  /// Business logic validation failed
  ValidationError(String),

  /// Data formatting failed
  FormattingError(String),
}

impl fmt::Display for CliError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    match self {
      Self::MissingParameter(param) => {
        write!(f, "Missing required parameter: {param}")
      }
      Self::InvalidParameter { param, reason } => {
        write!(f, "Invalid parameter '{param}': {reason}")
      }
      Self::ValidationError(msg) => {
        write!(f, "Validation error: {msg}")
      }
      Self::FormattingError(msg) => {
        write!(f, "Formatting error: {msg}")
      }
    }
  }
}

impl core::error::Error for CliError {}
