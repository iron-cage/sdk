//! Cost calculation for LLM requests

use serde_json::Value;

use iron_cost::pricing::PricingManager;

/// Cost information for a completed LLM request
#[derive(Debug, Clone)]
pub struct CostInfo {
  /// Model name used for the request
  pub model: String,
  /// Number of input/prompt tokens
  pub input_tokens: u64,
  /// Number of output/completion tokens
  pub output_tokens: u64,
  /// Cost in microdollars (1 USD = 1,000,000 microdollars)
  pub cost_micros: u64,
}

impl CostInfo {
  /// Returns cost in USD (for logging/display)
  #[must_use]
  pub fn cost_usd(&self) -> f64 {
    self.cost_micros as f64 / 1_000_000.0
  }
}

/// Calculate request cost from request and response bodies
///
/// Extracts model name from the request body and token usage from
/// the response body, then computes the cost using the pricing manager.
/// Handles both `OpenAI` (`prompt_tokens`/`completion_tokens`) and
/// Anthropic (`input_tokens`/`output_tokens`) usage formats.
#[must_use]
pub fn calculate_request_cost(
  pricing_manager: &PricingManager,
  request_body: &[u8],
  response_body: &[u8],
) -> Option<CostInfo> {
  let request_json = serde_json::from_slice::<Value>(request_body).ok()?;
  let model = request_json.get("model")?.as_str()?;

  let response_json = serde_json::from_slice::<Value>(response_body).ok()?;
  let usage = response_json.get("usage")?;

  // OpenAI: prompt_tokens/completion_tokens, Anthropic: input_tokens/output_tokens
  let input_tokens = usage
    .get("prompt_tokens")
    .or_else(|| usage.get("input_tokens"))
    .and_then(Value::as_u64)?;

  let output_tokens = usage
    .get("completion_tokens")
    .or_else(|| usage.get("output_tokens"))
    .and_then(Value::as_u64)?;

  let pricing = pricing_manager.get(model)?;
  let cost_micros = pricing.calculate_cost_micros(input_tokens, output_tokens);

  Some(CostInfo {
    model: model.to_string(),
    input_tokens,
    output_tokens,
    cost_micros,
  })
}

/// Extract model name from a JSON request body
///
/// Useful for error recording when the full cost calculation isn't needed.
#[must_use]
pub fn extract_model_from_body(body: &[u8]) -> Option<String> {
  serde_json::from_slice::<Value>(body)
    .ok()
    .and_then(|json| json.get("model")?.as_str().map(String::from))
}
