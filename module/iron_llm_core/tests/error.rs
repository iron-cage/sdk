//! Tests for error types and OpenAI-compatible error responses.

use core::error::Error;

use iron_llm_core::error::{create_openai_error, LlmCoreError};

#[test]
fn openai_error_serialization() {
  let error = create_openai_error("Token expired", "authentication_error", "invalid_token");

  let json = serde_json::to_value(&error).unwrap();
  assert_eq!(json["error"]["message"], "Token expired");
  assert_eq!(json["error"]["type"], "authentication_error");
  assert_eq!(json["error"]["code"], "invalid_token");
  assert!(json["error"]["param"].is_null());
}

#[test]
fn llm_core_error_display_forward() {
  let err = LlmCoreError::Forward("connection refused".into());
  assert_eq!(err.to_string(), "Forward failed: connection refused");
}

#[test]
fn llm_core_error_display_translation() {
  let err = LlmCoreError::Translation("missing field".into());
  assert_eq!(err.to_string(), "Translation failed: missing field");
}

#[test]
fn llm_core_error_implements_error_trait() {
  let err: Box<dyn Error> = Box::new(LlmCoreError::Forward("test".into()));
  assert!(err.to_string().contains("test"));
}
