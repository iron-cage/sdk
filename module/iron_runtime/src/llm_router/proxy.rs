//! Local HTTP proxy server for LLM requests

use axum::{
  body::Body,
  extract::{Request, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  routing::any,
  Router,
};
use iron_cost::budget::{CostController, Reservation};
use iron_cost::error::CostError;
use iron_cost::pricing::PricingManager;
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::llm_router::error::LlmRouterError;
use crate::llm_router::key_fetcher::KeyFetcher;
use crate::llm_router::translator::{translate_anthropic_to_openai, translate_openai_to_anthropic};

#[cfg(feature = "analytics")]
use iron_runtime_analytics::{EventStore, Provider};

/// Shared state for proxy handlers
#[derive(Clone)]
pub struct ProxyState {
  /// IC_TOKEN for validating incoming requests
  pub ic_token: String,
  /// Key fetcher for getting real API keys
  pub key_fetcher: Arc<KeyFetcher>,
  /// HTTP client for forwarding requests
  pub http_client: Client,
  /// Pricing manager for cost calculation
  pub pricing_manager: Arc<PricingManager>,
  /// Cost controller for budget enforcement and spending tracking (None = no budget)
  pub cost_controller: Option<Arc<CostController>>,
  /// Analytics event store
  #[cfg(feature = "analytics")]
  pub event_store: Arc<EventStore>,
  /// Agent ID for analytics attribution
  #[cfg(feature = "analytics")]
  pub agent_id: Option<Arc<str>>,
  /// Provider ID for analytics attribution
  #[cfg(feature = "analytics")]
  pub provider_id: Option<Arc<str>>,
}

/// Proxy server configuration
pub struct ProxyConfig {
  pub port: u16,
  pub ic_token: String,
  pub server_url: String,
  pub cache_ttl_seconds: u64,
  /// Cost controller for budget enforcement and spending tracking (None = no budget)
  pub cost_controller: Option<Arc<CostController>>,
  /// Direct provider API key (bypasses Iron Cage server when set)
  pub provider_key: Option<String>,
  /// Analytics event store
  #[cfg(feature = "analytics")]
  pub event_store: Arc<EventStore>,
  /// Agent ID for analytics attribution
  #[cfg(feature = "analytics")]
  pub agent_id: Option<Arc<str>>,
  /// Provider ID for analytics attribution
  #[cfg(feature = "analytics")]
  pub provider_id: Option<Arc<str>>,
}

/// Run the proxy server
pub async fn run_proxy(
  config: ProxyConfig,
  shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), LlmRouterError> {
  // Create key fetcher - static if provider_key given, otherwise fetch from server
  let key_fetcher = Arc::new(if let Some(ref pk) = config.provider_key {
    KeyFetcher::new_static(pk.clone(), None)
  } else {
    KeyFetcher::new(
      config.server_url,
      config.ic_token.clone(),
      config.cache_ttl_seconds,
    )
  });

  let http_client = Client::builder()
      .timeout(std::time::Duration::from_secs(300)) // 5 min timeout for LLM requests
      .build()
      .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?;

  let pricing_manager = Arc::new(
    PricingManager::new().map_err(LlmRouterError::ServerStart)?
  );

  let state = ProxyState {
    ic_token: config.ic_token,
    key_fetcher,
    http_client,
    pricing_manager,
    cost_controller: config.cost_controller,
    #[cfg(feature = "analytics")]
    event_store: config.event_store,
    #[cfg(feature = "analytics")]
    agent_id: config.agent_id,
    #[cfg(feature = "analytics")]
    provider_id: config.provider_id,
  };

  let app = Router::new()
      .route("/", any(handle_root))
      .route("/*path", any(handle_proxy))
      .with_state(state);

  let addr = std::net::SocketAddr::from(([127, 0, 0, 1], config.port));
  let listener = tokio::net::TcpListener::bind(addr)
      .await
      .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?;

  tracing::info!("LlmRouter proxy listening on http://{}", addr);

  axum::serve(listener, app)
      .with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
        tracing::info!("LlmRouter proxy shutting down");
      })
      .await
      .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?;

  Ok(())
}

/// Root handler (health check)
async fn handle_root() -> impl IntoResponse {
  "LlmRouter OK"
}

/// Create an OpenAI-compatible error response
fn create_openai_error_response(
  status: StatusCode,
  message: &str,
  error_type: &str,
  code: &str,
) -> Response<Body> {
  let error_json = serde_json::json!({
    "error": {
      "message": message,
      "type": error_type,
      "param": serde_json::Value::Null,
      "code": code
    }
  });

  Response::builder()
    .status(status)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(error_json.to_string()))
    .unwrap_or_else(|_| {
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal error"))
        .unwrap()
    })
}

/// Check if budget limit is exceeded using CostController
fn check_budget(state: &ProxyState) -> Result<(), Box<Response<Body>>> {
  // Skip check if no budget is set
  let Some(ref controller) = state.cost_controller else {
    return Ok(());
  };

  // Use CostController's check_budget method
  match controller.check_budget() {
    Ok(()) => Ok(()),
    Err(CostError::BudgetExceeded { spent_microdollars, limit_microdollars, reserved_microdollars }) => {
      let spent_usd = spent_microdollars as f64 / 1_000_000.0;
      let limit_usd = limit_microdollars as f64 / 1_000_000.0;
      let reserved_usd = reserved_microdollars as f64 / 1_000_000.0;
      Err(Box::new(create_openai_error_response(
        StatusCode::PAYMENT_REQUIRED, // 402 - distinct from 429 rate limit
        &format!(
          "Iron Cage budget limit exceeded. Spent: ${:.2}, Reserved: ${:.2}, Limit: ${:.2}. \
           Increase budget with router.set_budget() or check your pricing calculations.",
          spent_usd, reserved_usd, limit_usd
        ),
        "iron_cage_budget_exceeded", // Unique type - never from OpenAI
        "budget_exceeded",
      )))
    }
    Err(CostError::InsufficientBudget { available_microdollars, requested_microdollars }) => {
      let available_usd = available_microdollars as f64 / 1_000_000.0;
      let requested_usd = requested_microdollars as f64 / 1_000_000.0;
      Err(Box::new(create_openai_error_response(
        StatusCode::PAYMENT_REQUIRED,
        &format!(
          "Iron Cage insufficient budget. Available: ${:.2}, Requested: ${:.2}. \
           Wait for in-flight requests to complete or increase budget.",
          available_usd, requested_usd
        ),
        "iron_cage_insufficient_budget",
        "insufficient_budget",
      )))
    }
  }
}

/// Strip provider prefix from path if present, returns (clean_path, requested_provider)
pub fn strip_provider_prefix(path: &str) -> (String, Option<&'static str>) {
  if path.starts_with("/anthropic/") || path.starts_with("/anthropic") {
    let clean = path.strip_prefix("/anthropic").unwrap_or(path);
    let clean = if clean.is_empty() {
      "/".to_string()
    } else {
      clean.to_string()
    };
    (clean, Some("anthropic"))
  } else if path.starts_with("/openai/") || path.starts_with("/openai") {
    let clean = path.strip_prefix("/openai").unwrap_or(path);
    let clean = if clean.is_empty() {
      "/".to_string()
    } else {
      clean.to_string()
    };
    (clean, Some("openai"))
  } else {
    (path.to_string(), None)
  }
}

/// Detect requested provider from model name in body
pub fn detect_provider_from_model(body: &[u8]) -> Option<&'static str> {
  if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body) {
    if let Some(model) = json.get("model").and_then(|m| m.as_str()) {
      if model.starts_with("claude") {
        return Some("anthropic");
      }
      if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3") {
        return Some("openai");
      }
    }
  }
  None
}

/// Main proxy handler - forwards requests to LLM provider
async fn handle_proxy(
  State(state): State<ProxyState>,
  request: Request,
) -> Result<Response<Body>, (StatusCode, String)> {
  // 1. Validate IC_TOKEN from Authorization header OR x-api-key header
  // OpenAI uses: Authorization: Bearer {token}
  // Anthropic uses: x-api-key: {token}
  let auth_header = request
      .headers()
      .get(header::AUTHORIZATION)
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");

  let x_api_key = request
      .headers()
      .get("x-api-key")
      .and_then(|v| v.to_str().ok())
      .unwrap_or("");

  let expected_bearer = format!("Bearer {}", state.ic_token);
  let is_valid = auth_header == expected_bearer || x_api_key == state.ic_token;

  if !is_valid {
    return Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
  }

  // 1.5 Check budget before processing request (quick check)
  if let Err(error_response) = check_budget(&state) {
    return Ok(*error_response);
  }

  // 2. Read request body
  let method = request.method().clone();
  let orig_path = request.uri().path().to_string();
  let query = request
      .uri()
      .query()
      .map(|q| format!("?{}", q))
      .unwrap_or_default();

  let body_bytes = axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024) // 10MB limit
      .await
      .map_err(|e| (StatusCode::BAD_REQUEST, format!("Body read error: {}", e)))?;

  // 2.5 Reserve budget for this request (prevents concurrent overspend)
  let reservation: Option<Reservation> = if let Some(ref controller) = state.cost_controller {
    if let Some(max_cost) = state.pricing_manager.estimate_max_cost(&body_bytes) {
      match controller.reserve(max_cost) {
        Ok(res) => Some(res),
        Err(CostError::InsufficientBudget { available_microdollars, requested_microdollars }) => {
          let available_usd = available_microdollars as f64 / 1_000_000.0;
          let requested_usd = requested_microdollars as f64 / 1_000_000.0;
          return Ok(create_openai_error_response(
            StatusCode::PAYMENT_REQUIRED,
            &format!(
              "Iron Cage insufficient budget for request. Available: ${:.4}, Estimated max cost: ${:.4}. \
               Reduce max_tokens or wait for in-flight requests to complete.",
              available_usd, requested_usd
            ),
            "iron_cage_insufficient_budget",
            "insufficient_budget",
          ));
        }
        Err(e) => {
          tracing::warn!("Budget reservation failed: {}", e);
          None // Proceed without reservation (fallback)
        }
      }
    } else {
      None // Unknown model/pricing, skip reservation
    }
  } else {
    None // No budget controller
  };

  // 3. Get real API key from Iron Cage server (cached, auto-detected provider)
  let provider_key = state
      .key_fetcher
      .get_key()
      .await
      .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  // 4. Detect provider from model name in request
  let (clean_path, path_provider) = strip_provider_prefix(&orig_path);
  let model_provider = detect_provider_from_model(&body_bytes);
  let target_provider = path_provider.or(model_provider).unwrap_or("openai");

  // 5. Detect if translation is needed
  // OpenAI format (path=/v1/chat/completions) + Claude model → translate
  let is_openai_format = clean_path.contains("/chat/completions");
  let needs_translation = is_openai_format && target_provider == "anthropic";

  // 6. Prepare request body (translate if needed)
  let (request_body, request_path) = if needs_translation {
    let translated = translate_openai_to_anthropic(&body_bytes)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Translation error: {}", e)))?;
    (translated, "/v1/messages".to_string())
  } else {
    (body_bytes.to_vec(), clean_path)
  };

  // 7. Build target URL
  let base_url = provider_key
      .base_url
      .as_deref()
      .unwrap_or(match target_provider {
        "anthropic" => "https://api.anthropic.com",
        _ => "https://api.openai.com",
      });

  let target_url = format!("{}{}{}", base_url, request_path, query);

  // 8. Build forwarded request with real API key
  let mut req_builder = state
      .http_client
      .request(method, &target_url)
      .header(header::CONTENT_TYPE, "application/json");

  // Set provider-specific auth headers
  if target_provider == "anthropic" {
    req_builder = req_builder
        .header("x-api-key", &provider_key.api_key)
        .header("anthropic-version", "2023-06-01");
  } else {
    req_builder = req_builder.header(
      header::AUTHORIZATION,
      format!("Bearer {}", provider_key.api_key),
    );
  }

  // 9. Send request to provider
  let provider_response = req_builder
      .body(request_body)
      .send()
      .await
      .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Forward error: {}", e)))?;

  // 10. Read and translate response if needed
  let status = provider_response.status();
  let resp_headers = provider_response.headers().clone();
  let resp_body = provider_response
      .bytes()
      .await
      .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Response read error: {}", e)))?;

  // Translate response back to OpenAI format if we translated the request
  let final_body = if needs_translation && status.is_success() {
    translate_anthropic_to_openai(&resp_body).map_err(|e| {
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Response translation error: {}", e),
      )
    })?
  } else {
    resp_body.to_vec()
  };

  // 11. Calculate and log request cost, commit/cancel reservation
  if status.is_success() {
    if let Some(cost_info) = calculate_request_cost(&state.pricing_manager, &body_bytes, &final_body) {
      // Commit reservation with actual cost (or add directly if no reservation)
      if let Some(ref controller) = state.cost_controller {
        if let Some(res) = reservation {
          controller.commit(res, cost_info.cost_micros);
        } else {
          // No reservation was made, add directly (fallback for unknown models)
          controller.add_spend(cost_info.cost_micros as i64);
        }
      }

      // Record analytics event
      #[cfg(feature = "analytics")]
      {
        let provider = Provider::from(target_provider);
        state.event_store.record_llm_completed_with_provider(
          &state.pricing_manager,
          &cost_info.model,
          provider,
          cost_info.input_tokens,
          cost_info.output_tokens,
          state.agent_id.as_deref(),
          state.provider_id.as_deref(),
        );
      }

      tracing::info!(
        model = %cost_info.model,
        input_tokens = cost_info.input_tokens,
        output_tokens = cost_info.output_tokens,
        cost_usd = %format!("{:.6}", cost_info.cost_usd()),
        "LLM request completed"
      );
    } else if let Some(res) = reservation {
      // Cost couldn't be calculated, cancel reservation
      if let Some(ref controller) = state.cost_controller {
        controller.cancel(res);
      }
    }
  } else {
    // Request failed - cancel reservation (no cost incurred)
    if let Some(res) = reservation {
      if let Some(ref controller) = state.cost_controller {
        controller.cancel(res);
      }
    }

    // Record failed request for non-2xx responses
    #[cfg(feature = "analytics")]
    if let Some(model) = extract_model_from_body(&body_bytes) {
      let error_msg = serde_json::from_slice::<serde_json::Value>(&resp_body)
        .ok()
        .and_then(|v| v.get("error")?.get("message")?.as_str().map(String::from));

      state.event_store.record_llm_failed(
        &model,
        state.agent_id.as_deref(),
        state.provider_id.as_deref(),
        Some(status.as_str()),
        error_msg.as_deref(),
      );
    }
  }

  let mut response = Response::builder().status(status);

  // Copy content-type header
  if let Some(ct) = resp_headers.get(header::CONTENT_TYPE) {
    response = response.header(header::CONTENT_TYPE, ct);
  }

  response
      .body(Body::from(final_body))
      .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Cost calculation result
struct CostInfo {
  model: String,
  input_tokens: u64,
  output_tokens: u64,
  /// Cost in microdollars (1 USD = 1,000,000 microdollars)
  cost_micros: u64,
}

impl CostInfo {
  /// Returns cost in USD (for logging/display)
  fn cost_usd(&self) -> f64 {
    self.cost_micros as f64 / 1_000_000.0
  }
}

/// Calculate request cost from request and response bodies
fn calculate_request_cost(
  pricing_manager: &PricingManager,
  request_body: &[u8],
  response_body: &[u8],
) -> Option<CostInfo> {
  // Extract model from request
  let request_json: serde_json::Value = serde_json::from_slice(request_body).ok()?;
  let model = request_json.get("model")?.as_str()?;

  // Extract usage from response (OpenAI format)
  let response_json: serde_json::Value = serde_json::from_slice(response_body).ok()?;
  let usage = response_json.get("usage")?;

  // OpenAI: prompt_tokens/completion_tokens, Anthropic: input_tokens/output_tokens
  let input_tokens = usage
      .get("prompt_tokens")
      .or_else(|| usage.get("input_tokens"))
      .and_then(|v| v.as_u64())?;

  let output_tokens = usage
      .get("completion_tokens")
      .or_else(|| usage.get("output_tokens"))
      .and_then(|v| v.as_u64())?;

  // Get pricing and calculate cost in microdollars (integer arithmetic)
  let pricing = pricing_manager.get(model)?;
  let cost_micros = pricing.calculate_cost_micros(input_tokens, output_tokens);

  Some(CostInfo {
    model: model.to_string(),
    input_tokens,
    output_tokens,
    cost_micros,
  })
}

/// Extract model name from request body (for error recording)
#[cfg(feature = "analytics")]
fn extract_model_from_body(body: &[u8]) -> Option<String> {
  serde_json::from_slice::<serde_json::Value>(body)
    .ok()
    .and_then(|json| json.get("model")?.as_str().map(String::from))
}
