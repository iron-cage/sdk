//! Core LLM request forwarding logic
//!
//! Handles provider detection, request/response format translation,
//! and HTTP forwarding to LLM providers. This module contains no
//! authentication, budget enforcement, or analytics concerns — those
//! are the responsibility of the caller (`iron_runtime` or `iron_server_proxy`).

use reqwest::{header, Client, StatusCode};
use secrecy::ExposeSecret;

use crate::{
  cost::{self, CostInfo},
  error::LlmCoreError,
  provider, translator,
};
use iron_cost::pricing::PricingManager;
use iron_secrets::ip_token::ProviderKey;

/// Anthropic API version header value.
const ANTHROPIC_API_VERSION: &str = "2023-06-01";

/// Fallback Gemini model when none is specified in the request.
///
/// Note: "gemini-pro" is an older model. Update to a current model when the
/// pricing database is expanded to cover newer Gemini models.
const GEMINI_FALLBACK_MODEL: &str = "gemini-pro";

/// Fallback estimated cost when the model is not found in the pricing database.
///
/// $1.00 (1,000,000 microdollars) is a conservative overestimate used to
/// ensure spending caps are not bypassed for unknown models.
#[allow(dead_code)]
const DEFAULT_COST_FALLBACK_MICRODOLLARS: i64 = 1_000_000;

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

/// Body of a forwarded provider response.
///
/// Non-streaming requests are fully buffered (and translated when needed).
/// Streaming requests hand the raw provider `Response` to the caller so it can be
/// piped directly to the client via `.bytes_stream()`.
pub enum ForwardBody {
  /// Fully buffered, optionally translated response body.
  Buffered(Vec<u8>),
  /// Raw provider response for streaming — consume via `.bytes_stream()`.
  ///
  /// Note: when `needs_translation` was true the body is in Anthropic SSE format;
  /// OpenAI-compatible SSE translation is not yet implemented.
  Streaming(reqwest::Response),
}

impl core::fmt::Debug for ForwardBody {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Buffered(b) => write!(f, "Buffered({} bytes)", b.len()),
      Self::Streaming(_) => write!(f, "Streaming(...)"),
    }
  }
}

/// Response from an LLM provider after forwarding
#[derive(Debug)]
pub struct ForwardResponse {
  /// HTTP status code from the provider
  pub status: StatusCode,
  /// Response headers from the provider (includes `Content-Type` for stream detection).
  pub headers: reqwest::header::HeaderMap,
  /// Response body — buffered for normal requests, streaming for `stream: true` requests.
  pub body: ForwardBody,
  /// Cost information (present only for successful buffered requests with known pricing).
  pub cost_info: Option<CostInfo>,
}

/// Forward a request to an LLM provider
///
/// This is the core forwarding logic extracted from the proxy handler.
/// It handles:
/// 1. Provider detection from path prefix or model name in body
/// 2. Request format translation (`OpenAI` -> Anthropic) when needed
/// 3. Building the correct provider URL and auth headers
/// 4. Sending the request; for streaming (`stream: true`) the raw provider response is
///    returned immediately as [`ForwardBody::Streaming`] — buffering, translation, and
///    cost calculation are skipped
/// 5. Response format translation (Anthropic -> `OpenAI`) when needed (buffered path only)
/// 6. Cost calculation for successful requests (buffered path only)
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

  // 2. Detect if translation is needed (OpenAI format + non-OpenAI provider)
  let is_openai_format = clean_path.contains("/chat/completions");
  let needs_anthropic_translation = is_openai_format && target_provider == "anthropic";
  let needs_gemini_translation = is_openai_format && target_provider == "gemini";

  // 3. Prepare request body (translate if needed)
  let (request_body, request_path) = if needs_anthropic_translation {
    let translated = translator::translate_openai_to_anthropic(&request.body)
      .map_err(|e| LlmCoreError::Translation(format!("Request: {e}")))?;
    (translated, "/v1/messages".to_string())
  } else if needs_gemini_translation {
    let translated = translator::translate_openai_to_gemini(&request.body)
      .map_err(|e| LlmCoreError::Translation(format!("Request: {e}")))?;
    // Gemini endpoint: /v1beta/models/{model}:generateContent
    let model = translator::extract_model(&request.body).unwrap_or_else(|| GEMINI_FALLBACK_MODEL.into());
    (
      translated,
      format!("/v1beta/models/{model}:generateContent"),
    )
  } else {
    (request.body.clone(), clean_path)
  };

  // 4. Build target URL
  let base_url = if let Some(custom_url) = provider_key.base_url.as_deref() {
    validate_provider_base_url(custom_url)?;
    custom_url
  } else {
    match target_provider {
      "anthropic" => "https://api.anthropic.com",
      "gemini" => "https://generativelanguage.googleapis.com",
      "xai" => "https://api.x.ai",
      _ => "https://api.openai.com",
    }
  };

  let target_url = format!("{base_url}{request_path}{}", request.query);

  // 5. Build forwarded request with provider-specific auth headers
  let mut req_builder = client
    .request(request.method, &target_url)
    .header(header::CONTENT_TYPE, "application/json");

  match target_provider {
    "anthropic" => {
      req_builder = req_builder
        .header("x-api-key", provider_key.api_key.expose_secret().as_str())
        .header("anthropic-version", ANTHROPIC_API_VERSION);
    }
    "gemini" => {
      req_builder = req_builder.header(
        "x-goog-api-key",
        provider_key.api_key.expose_secret().as_str(),
      );
    }
    _ => {
      // OpenAI, xAI, and other Bearer-token providers
      req_builder = req_builder.header(
        header::AUTHORIZATION,
        format!("Bearer {}", provider_key.api_key.expose_secret().as_str()),
      );
    }
  }

  // 6. Detect streaming from original request body
  let is_streaming = serde_json::from_slice::<serde_json::Value>(&request.body)
    .ok()
    .and_then(|v| v["stream"].as_bool())
    .unwrap_or(false);

  // 7. Send request to provider
  let provider_response = req_builder
    .body(request_body)
    .send()
    .await
    .map_err(|e| LlmCoreError::Forward(format!("Forward error: {e}")))?;

  let status = provider_response.status();
  let headers = provider_response.headers().clone();

  // 8. For streaming requests, hand the raw response to the caller immediately.
  // Response translation and cost calculation are skipped — the stream is piped through.
  if is_streaming {
    tracing::debug!(
      target_provider,
      needs_anthropic_translation,
      needs_gemini_translation,
      status = %status,
      "LLM streaming request forwarded"
    );
    return Ok(ForwardResponse {
      status,
      headers,
      body: ForwardBody::Streaming(provider_response),
      cost_info: None,
    });
  }

  // 9. Buffer non-streaming response
  let resp_body = provider_response
    .bytes()
    .await
    .map_err(|e| LlmCoreError::Forward(format!("Response read error: {e}")))?;

  // 10. Translate response back to OpenAI format if needed.
  // For 401/403 responses, replace body with a generic error to prevent
  // provider error messages from leaking partial API key material.
  let final_body = if [StatusCode::UNAUTHORIZED, StatusCode::FORBIDDEN].contains(&status) {
    tracing::warn!(
      target_provider,
      status = %status,
      "Provider auth error (original body suppressed to prevent key leakage)"
    );
    br#"{"error":{"message":"Provider authentication failed","type":"auth_error"}}"#.to_vec()
  } else if needs_anthropic_translation && status.is_success() {
    translator::translate_anthropic_to_openai(&resp_body)
      .map_err(|e| LlmCoreError::Translation(format!("Response: {e}")))?
  } else if needs_gemini_translation && status.is_success() {
    translator::translate_gemini_to_openai(&resp_body)
      .map_err(|e| LlmCoreError::Translation(format!("Response: {e}")))?
  } else {
    resp_body.to_vec()
  };

  // 11. Calculate cost for successful requests
  let cost_info = if status.is_success() {
    cost::calculate_request_cost(pricing_manager, &request.body, &final_body)
  } else {
    None
  };

  tracing::debug!(
    target_provider,
    needs_anthropic_translation,
    needs_gemini_translation,
    status = %status,
    "LLM request forwarded"
  );

  Ok(ForwardResponse {
    status,
    headers,
    body: ForwardBody::Buffered(final_body),
    cost_info,
  })
}

/// Validate that a user-supplied base URL is from a known LLM provider.
///
/// Prevents SSRF: a manager could set `base_url` to an internal service.
/// The default URLs are safe (hardcoded constants); only user-overrides are validated.
fn validate_provider_base_url(url: &str) -> Result<(), LlmCoreError> {
  // In test builds with the `allow-insecure-base-urls` feature, permit loopback
  // URLs so that wiremock mock servers work in integration tests.
  // This bypass is NEVER compiled into production binaries.
  #[cfg(feature = "allow-insecure-base-urls")]
  if url.starts_with("http://127.0.0.1")
    || url.starts_with("http://[::1]")
    || url.starts_with("http://localhost")
  {
    return Ok(());
  }

  const ALLOWED_PREFIXES: &[&str] = &[
    "https://api.openai.com",
    "https://api.anthropic.com",
    "https://generativelanguage.googleapis.com",
    "https://api.x.ai",
  ];
  if ALLOWED_PREFIXES.iter().any(|prefix| url.starts_with(prefix)) {
    return Ok(());
  }
  Err(LlmCoreError::Forward(format!(
    "provider key base_url '{url}' is not an allowed LLM endpoint; \
     only official provider URLs are permitted"
  )))
}
