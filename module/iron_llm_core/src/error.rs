//! Error types for the shared LLM routing core

/// Errors from the LLM core forwarding layer
#[derive(Debug)]
pub enum LlmCoreError {
  /// Failed to forward request to LLM provider
  Forward(String),

  /// Failed to translate request/response between `OpenAI` and Anthropic formats
  Translation(String),
}

impl core::fmt::Display for LlmCoreError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Forward(msg) => write!(f, "Forward failed: {msg}"),
      Self::Translation(msg) => write!(f, "Translation failed: {msg}"),
    }
  }
}

impl core::error::Error for LlmCoreError {}

/// Create a JSON error body in `OpenAI`-compatible format
///
/// Returns a `serde_json::Value` that callers can serialize and wrap
/// in their framework-specific HTTP response type.
#[must_use]
pub fn create_openai_error_json(message: &str, error_type: &str, code: &str) -> serde_json::Value {
  serde_json::json!({
    "error": {
      "message": message,
      "type": error_type,
      "param": serde_json::Value::Null,
      "code": code
    }
  })
}
