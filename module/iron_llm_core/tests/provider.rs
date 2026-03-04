//! Tests for provider detection logic.

use iron_llm_core::provider::{detect_provider_from_model, strip_provider_prefix};

#[test]
fn strip_anthropic_prefix() {
  let (path, provider) = strip_provider_prefix("/anthropic/v1/messages");
  assert_eq!(path, "/v1/messages");
  assert_eq!(provider, Some("anthropic"));
}

#[test]
fn strip_openai_prefix() {
  let (path, provider) = strip_provider_prefix("/openai/v1/chat/completions");
  assert_eq!(path, "/v1/chat/completions");
  assert_eq!(provider, Some("openai"));
}

#[test]
fn no_prefix_passthrough() {
  let (path, provider) = strip_provider_prefix("/v1/chat/completions");
  assert_eq!(path, "/v1/chat/completions");
  assert_eq!(provider, None);
}

#[test]
fn bare_anthropic_without_trailing_slash() {
  let (path, provider) = strip_provider_prefix("/anthropic");
  assert_eq!(path, "/");
  assert_eq!(provider, Some("anthropic"));
}

#[test]
fn bare_openai_without_trailing_slash() {
  let (path, provider) = strip_provider_prefix("/openai");
  assert_eq!(path, "/");
  assert_eq!(provider, Some("openai"));
}

#[test]
fn unknown_prefix_treated_as_path() {
  let (path, provider) = strip_provider_prefix("/google/v1/generate");
  assert_eq!(path, "/google/v1/generate");
  assert_eq!(provider, None);
}

#[test]
fn detect_claude_model() {
  let body = br#"{"model": "claude-3-opus-20240229"}"#;
  assert_eq!(detect_provider_from_model(body), Some("anthropic"));
}

#[test]
fn detect_gpt_model() {
  let body = br#"{"model": "gpt-4"}"#;
  assert_eq!(detect_provider_from_model(body), Some("openai"));
}

#[test]
fn detect_o1_model() {
  let body = br#"{"model": "o1-preview"}"#;
  assert_eq!(detect_provider_from_model(body), Some("openai"));
}

#[test]
fn detect_o3_model() {
  let body = br#"{"model": "o3-mini"}"#;
  assert_eq!(detect_provider_from_model(body), Some("openai"));
}

#[test]
fn unknown_model_returns_none() {
  let body = br#"{"model": "llama-3-70b"}"#;
  assert_eq!(detect_provider_from_model(body), None);
}

#[test]
fn missing_model_field_returns_none() {
  let body = br#"{"messages": []}"#;
  assert_eq!(detect_provider_from_model(body), None);
}

#[test]
fn invalid_json_returns_none() {
  assert_eq!(detect_provider_from_model(b"not json"), None);
}

#[test]
fn empty_body_returns_none() {
  assert_eq!(detect_provider_from_model(b""), None);
}
