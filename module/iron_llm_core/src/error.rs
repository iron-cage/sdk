//! Error types for the shared LLM routing core

use core::{
  error::Error,
  fmt::{Display, Formatter, Result as FmtResult},
};

use serde::Serialize;

/// Errors from the LLM core forwarding layer
#[derive(Debug)]
pub enum LlmCoreError {
  /// Failed to forward request to LLM provider
  Forward(String),

  /// Failed to translate request/response between `OpenAI` and Anthropic formats
  Translation(String),
}

impl Display for LlmCoreError {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    match self {
      Self::Forward(msg) => write!(f, "Forward failed: {msg}"),
      Self::Translation(msg) => write!(f, "Translation failed: {msg}"),
    }
  }
}

impl Error for LlmCoreError {}

/// `OpenAI`-compatible error response envelope.
///
/// Serializes to: `{"error": {"message": "...", "type": "...", "param": null, "code": "..."}}`
#[derive(Debug, Serialize)]
pub struct OpenAiErrorResponse {
  /// Error detail object.
  pub error: OpenAiErrorDetail,
}

/// Inner error detail following the `OpenAI` API error format.
#[derive(Debug, Serialize)]
pub struct OpenAiErrorDetail {
  /// Human-readable error description.
  pub message: String,
  /// Error category (e.g. `error`, `invalid_request_error`).
  #[serde(rename = "type")]
  pub error_type: String,
  /// Parameter that caused the error (always `None` for proxy errors).
  pub param: Option<String>,
  /// Machine-readable error code (e.g. `invalid_token`, `spending_cap_exceeded`).
  pub code: String,
}

/// Create a typed `OpenAI`-compatible error response.
///
/// Callers can serialize the result via `serde_json::to_string()` or pass it
/// directly to `axum::Json()`.
#[must_use]
pub fn create_openai_error(message: &str, error_type: &str, code: &str) -> OpenAiErrorResponse {
  OpenAiErrorResponse {
    error: OpenAiErrorDetail {
      message: message.to_string(),
      error_type: error_type.to_string(),
      param: None,
      code: code.to_string(),
    },
  }
}
