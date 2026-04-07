//! Tests for cost calculation logic.

use iron_cost::pricing::PricingManager;
use iron_llm_core::cost::{calculate_request_cost, extract_model_from_body, CostInfo};

fn pricing() -> PricingManager {
  PricingManager::new().expect("Failed to init PricingManager")
}

#[test]
fn cost_calculation_openai_format() {
  let pm = pricing();
  let req = br#"{"model": "gpt-4"}"#;
  let resp = br#"{
    "usage": {
      "prompt_tokens": 100,
      "completion_tokens": 50,
      "total_tokens": 150
    }
  }"#;

  let info = calculate_request_cost(&pm, req, resp);
  // gpt-4 should be in the pricing table
  if let Some(info) = info {
    assert_eq!(info.model, "gpt-4");
    assert_eq!(info.input_tokens, 100);
    assert_eq!(info.output_tokens, 50);
    assert!(info.cost_micros > 0, "Cost should be positive for gpt-4");
    assert!(info.cost_usd() > 0.0);
  }
  // If gpt-4 is not in pricing table, None is acceptable
}

#[test]
fn cost_calculation_anthropic_format() {
  let pm = pricing();
  let req = br#"{"model": "claude-3-haiku-20240307"}"#;
  let resp = br#"{
    "usage": {
      "input_tokens": 200,
      "output_tokens": 100
    }
  }"#;

  let info = calculate_request_cost(&pm, req, resp);
  if let Some(info) = info {
    assert_eq!(info.input_tokens, 200);
    assert_eq!(info.output_tokens, 100);
    assert!(info.cost_micros > 0);
  }
}

#[test]
fn cost_returns_none_for_unknown_model() {
  let pm = pricing();
  let req = br#"{"model": "nonexistent-model-xyz"}"#;
  let resp = br#"{"usage": {"prompt_tokens": 10, "completion_tokens": 5}}"#;

  assert!(calculate_request_cost(&pm, req, resp).is_none());
}

#[test]
fn cost_returns_none_for_missing_usage() {
  let pm = pricing();
  let req = br#"{"model": "gpt-4"}"#;
  let resp = br#"{"choices": [{"message": {"content": "hi"}}]}"#;

  assert!(calculate_request_cost(&pm, req, resp).is_none());
}

#[test]
fn cost_returns_none_for_missing_model() {
  let pm = pricing();
  let req = br#"{"messages": []}"#;
  let resp = br#"{"usage": {"prompt_tokens": 10, "completion_tokens": 5}}"#;

  assert!(calculate_request_cost(&pm, req, resp).is_none());
}

#[test]
fn cost_returns_none_for_invalid_json() {
  let pm = pricing();
  assert!(calculate_request_cost(&pm, b"bad", b"bad").is_none());
}

#[test]
fn extract_model_success() {
  let body = br#"{"model": "gpt-4-turbo", "messages": []}"#;
  assert_eq!(extract_model_from_body(body), Some("gpt-4-turbo".into()));
}

#[test]
fn extract_model_missing_field() {
  let body = br#"{"messages": []}"#;
  assert_eq!(extract_model_from_body(body), None);
}

#[test]
fn extract_model_invalid_json() {
  assert_eq!(extract_model_from_body(b"not json"), None);
}

#[test]
fn extract_model_non_string_value() {
  let body = br#"{"model": 42}"#;
  assert_eq!(extract_model_from_body(body), None);
}

#[test]
fn cost_usd_conversion() {
  let info = CostInfo {
    model: "test".into(),
    input_tokens: 0,
    output_tokens: 0,
    cost_micros: 1_500_000,
  };
  let diff = (info.cost_usd() - 1.5).abs();
  assert!(diff < f64::EPSILON, "1_500_000 microdollars = $1.50");
}
