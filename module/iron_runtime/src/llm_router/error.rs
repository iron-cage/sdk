//! Error types for LLM Router

use core::{
  error::Error,
  fmt::{Display, Formatter, Result as FmtResult},
};

use iron_llm_core::LlmCoreError;

/// Errors that can occur in the LLM Router
#[derive(Debug)]
pub enum LlmRouterError {
  /// Failed to start the proxy server
  ServerStart(String),

  /// Failed to fetch API key from Iron Cage server
  KeyFetch(String),

  /// Failed to forward request to LLM provider
  Forward(String),

  /// Invalid or missing authentication token
  InvalidToken,
}

impl Display for LlmRouterError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      Self::ServerStart(msg) => write!(f, "Server start failed: {msg}"),
      Self::KeyFetch(msg) => write!(f, "Key fetch failed: {msg}"),
      Self::Forward(msg) => write!(f, "Forward failed: {msg}"),
      Self::InvalidToken => write!(f, "Invalid or missing authentication token"),
    }
  }
}

impl Error for LlmRouterError {}

impl From<LlmCoreError> for LlmRouterError {
  fn from(err: LlmCoreError) -> Self {
    Self::Forward(err.to_string())
  }
}
