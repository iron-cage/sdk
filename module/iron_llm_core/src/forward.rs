//! Core LLM request forwarding logic
//!
//! Handles provider detection, request/response format translation,
//! and HTTP forwarding to LLM providers. This module contains no
//! authentication, budget enforcement, or analytics concerns — those
//! are the responsibility of the caller (`iron_runtime` or `iron_server_proxy`).

use reqwest::Client;
use secrecy::ExposeSecret;

use crate::{
  cost::{self, CostInfo},
  error::LlmCoreError,
  provider, translator,
};
use iron_cost::pricing::PricingManager;
use iron_secrets::ip_token::ProviderKey;

/// Request to be forwarded to an LLM provider
#[derive(Debug)]
pub struct ForwardRequest {
  /// HTTP method (GET, POST, etc.)
  pub method: reqwest::Method,
  /// Original request path (e.g., `/v1/chat/completions` or `/anthropic/v1/messages`)
  pub path: String,
  /// Query string including `?` prefix (e.g., `?stream=true`), empty if none
  pub query: String,
  /// Request body bytes
  pub body: Vec<u8>,
}

/// Response from an LLM provider after forwarding
#[derive(Debug)]
pub struct ForwardResponse {
  /// HTTP status code from the provider
  pub status: reqwest::StatusCode,
  /// Response body bytes (translated to `OpenAI` format if applicable)
  pub body: Vec<u8>,
  /// Cost information (present only for successful requests with known pricing)
  pub cost_info: Option<CostInfo>,
}

/// Forward a request to an LLM provider
///
/// This is the core forwarding logic extracted from the proxy handler.
/// It handles:
/// 1. Provider detection from path prefix or model name in body
/// 2. Request format translation (`OpenAI` -> Anthropic) when needed
/// 3. Building the correct provider URL and auth headers
/// 4. Sending the request and reading the response
/// 5. Response format translation (Anthropic -> `OpenAI`) when needed
/// 6. Cost calculation for successful requests
///
/// # Arguments
///
/// * `client` - HTTP client for making requests
/// * `pricing_manager` - For calculating request costs
/// * `provider_key` - Decrypted provider API key and base URL
/// * `request` - The forwarding request details
///
/// # Errors
///
/// Returns [`LlmCoreError::Translation`] if request/response translation fails,
/// or [`LlmCoreError::Forward`] if the HTTP request to the provider fails.
pub async fn forward_request(
  client: &Client,
  pricing_manager: &PricingManager,
  provider_key: &ProviderKey,
  request: ForwardRequest,
) -> Result<ForwardResponse, LlmCoreError> {
  // 1. Detect provider from path prefix or model name
  let (clean_path, path_provider) = provider::strip_provider_prefix(&request.path);
  let model_provider = provider::detect_provider_from_model(&request.body);
  let target_provider = path_provider.or(model_provider).unwrap_or("openai");

  // 2. Detect if translation is needed (OpenAI format + Anthropic model)
  let is_openai_format = clean_path.contains("/chat/completions");
  let needs_translation = is_openai_format && target_provider == "anthropic";

  // 3. Prepare request body (translate if needed)
  let (request_body, request_path) = if needs_translation {
    let translated = translator::translate_openai_to_anthropic(&request.body)
      .map_err(|e| LlmCoreError::Translation(format!("Request: {e}")))?;
    (translated, "/v1/messages".to_string())
  } else {
    (request.body.clone(), clean_path)
  };

  // 4. Build target URL
  let base_url = provider_key
    .base_url
    .as_deref()
    .unwrap_or(match target_provider {
      "anthropic" => "https://api.anthropic.com",
      _ => "https://api.openai.com",
    });

  let target_url = format!("{base_url}{request_path}{}", request.query);

  // 5. Build forwarded request with provider-specific auth headers
  let mut req_builder = client
    .request(request.method, &target_url)
    .header(reqwest::header::CONTENT_TYPE, "application/json");

  if target_provider == "anthropic" {
    req_builder = req_builder
      .header("x-api-key", provider_key.api_key.expose_secret().as_str())
      .header("anthropic-version", "2023-06-01");
  } else {
    req_builder = req_builder.header(
      reqwest::header::AUTHORIZATION,
      format!("Bearer {}", provider_key.api_key.expose_secret().as_str()),
    );
  }

  // 6. Send request to provider
  let provider_response = req_builder
    .body(request_body)
    .send()
    .await
    .map_err(|e| LlmCoreError::Forward(format!("Forward error: {e}")))?;

  // 7. Read response
  let status = provider_response.status();
  let resp_body = provider_response
    .bytes()
    .await
    .map_err(|e| LlmCoreError::Forward(format!("Response read error: {e}")))?;

  // 8. Translate response back to OpenAI format if needed
  let final_body = if needs_translation && status.is_success() {
    translator::translate_anthropic_to_openai(&resp_body)
      .map_err(|e| LlmCoreError::Translation(format!("Response: {e}")))?
  } else {
    resp_body.to_vec()
  };

  // 9. Calculate cost for successful requests
  let cost_info = if status.is_success() {
    cost::calculate_request_cost(pricing_manager, &request.body, &final_body)
  } else {
    None
  };

  tracing::debug!(
    target_provider,
    needs_translation,
    status = %status,
    "LLM request forwarded"
  );

  Ok(ForwardResponse {
    status,
    body: final_body,
    cost_info,
  })
}
