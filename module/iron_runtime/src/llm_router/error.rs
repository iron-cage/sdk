//! Error types for LLM Router

/// Errors that can occur in the LLM Router
#[derive(Debug)]
pub enum LlmRouterError
{
  /// Failed to start the proxy server
  ServerStart(String),

  /// Failed to fetch API key from Iron Cage server
  KeyFetch(String),

  /// Failed to forward request to LLM provider
  Forward(String),

  /// Invalid or missing authentication token
  InvalidToken,
}

impl std::fmt::Display for LlmRouterError
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      Self::ServerStart(msg) => write!(f, "Server start failed: {}", msg),
      Self::KeyFetch(msg) => write!(f, "Key fetch failed: {}", msg),
      Self::Forward(msg) => write!(f, "Forward failed: {}", msg),
      Self::InvalidToken => write!(f, "Invalid or missing authentication token"),
    }
  }
}

impl std::error::Error for LlmRouterError {}
