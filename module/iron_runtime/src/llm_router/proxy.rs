//! Local HTTP proxy server for LLM requests
//!
//! Handles IC Token validation, budget enforcement, and analytics locally.
//! Delegates core forwarding logic to `iron_llm_core`.

use std::fmt;

use axum::{
  body::Body,
  extract::{Request, State},
  http::{header, StatusCode},
  response::{IntoResponse, Response},
  routing::any,
  Router,
};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::llm_router::{error::LlmRouterError, key_fetcher::KeyFetcher};
use iron_cost::{
  budget::{CostController, Reservation},
  error::CostError,
  pricing::PricingManager,
};
use iron_llm_core::{CostInfo, ForwardBody, ForwardRequest, LlmCoreError};
use iron_secrets::ip_token::IpTokenKey;

#[cfg(feature = "analytics")]
use iron_runtime_analytics::{EventStore, Provider};

/// Shared state for proxy handlers
#[derive(Clone)]
pub struct ProxyState {
  /// `IC_TOKEN` for validating incoming requests
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
  /// Port to bind the local proxy server on
  pub port: u16,
  /// IC Token for authenticating outbound requests to the Iron Cage server
  pub ic_token: String,
  /// Iron Cage server URL for fetching provider keys
  pub server_url: String,
  /// How long to cache fetched provider keys (seconds)
  pub cache_ttl_seconds: u64,
  /// Cost controller for budget enforcement and spending tracking (None = no budget)
  pub cost_controller: Option<Arc<CostController>>,
  /// Direct provider API key (bypasses Iron Cage server when set)
  pub provider_key: Option<String>,
  /// IP Token encryption key for decrypting provider keys in transit (from `IP_TOKEN_KEY` env var)
  pub ip_token_key: Option<IpTokenKey>,
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

impl fmt::Debug for ProxyConfig {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ProxyConfig")
      .field("port", &self.port)
      .field("ic_token", &"[REDACTED]")
      .field("server_url", &self.server_url)
      .field("cache_ttl_seconds", &self.cache_ttl_seconds)
      .field("provider_key", &self.provider_key.as_ref().map(|_| "[REDACTED]"))
      .finish_non_exhaustive()
  }
}

/// Run the proxy server
///
/// # Errors
///
/// Returns [`LlmRouterError::ServerStart`] if `IP_TOKEN_KEY` is absent when no
/// static `provider_key` is configured, or if the TCP listener fails to bind.
pub async fn run_proxy(
  config: ProxyConfig,
  shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), LlmRouterError> {
  // Startup validation: IP_TOKEN_KEY is required when not using a static provider key
  if config.provider_key.is_none() && config.ip_token_key.is_none() {
    return Err(LlmRouterError::ServerStart(
      "IP_TOKEN_KEY must be configured when using Iron Cage server key fetching".into(),
    ));
  }

  // Create key fetcher - static if provider_key given, otherwise fetch from server
  let key_fetcher = Arc::new(if let Some(ref pk) = config.provider_key {
    KeyFetcher::new_static(pk.clone(), None)
  } else {
    KeyFetcher::new(
      config.server_url,
      config.ic_token.clone(),
      config.cache_ttl_seconds,
      config.ip_token_key,
    )
    .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?
  });

  let http_client = Client::builder()
    .timeout(core::time::Duration::from_secs(300)) // 5 min timeout for LLM requests
    .build()
    .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?;

  let pricing_manager =
    Arc::new(PricingManager::new().map_err(|e| LlmRouterError::ServerStart(e.to_string()))?);

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

  let addr = core::net::SocketAddr::from(([127, 0, 0, 1], config.port));
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
  let error_body = iron_llm_core::create_openai_error(message, error_type, code);
  let json =
    serde_json::to_string(&error_body).expect("OpenAiErrorResponse serialization never fails");

  Response::builder()
    .status(status)
    .header(header::CONTENT_TYPE, "application/json")
    .body(Body::from(json))
    .unwrap_or_else(|_| {
      // Fallback response if primary error response construction fails
      Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(Body::from("Internal error"))
        .expect("INVARIANT: Fallback response with static content and valid StatusCode never fails")
    })
}

/// Check if budget limit is exceeded using `CostController`
fn check_budget(state: &ProxyState) -> Result<(), Box<Response<Body>>> {
  // Skip check if no budget is set
  let Some(ref controller) = state.cost_controller else {
    return Ok(());
  };

  // Use CostController's check_budget method
  match controller.check_budget() {
    Ok(()) => Ok(()),
    Err(CostError::BudgetExceeded {
      spent_microdollars,
      limit_microdollars,
      reserved_microdollars,
    }) => {
      let spent_usd = spent_microdollars as f64 / 1_000_000.0;
      let limit_usd = limit_microdollars as f64 / 1_000_000.0;
      let reserved_usd = reserved_microdollars as f64 / 1_000_000.0;
      Err(Box::new(create_openai_error_response(
        StatusCode::PAYMENT_REQUIRED, // 402 - distinct from 429 rate limit
        &format!(
          "Iron Cage budget limit exceeded. Spent: ${spent_usd:.2}, Reserved: ${reserved_usd:.2}, Limit: ${limit_usd:.2}. \
           Increase budget with router.set_budget() or check your pricing calculations."
        ),
        "iron_cage_budget_exceeded", // Unique type - never from OpenAI
        "budget_exceeded",
      )))
    }
    Err(CostError::InsufficientBudget {
      available_microdollars,
      requested_microdollars,
    }) => {
      let available_usd = available_microdollars as f64 / 1_000_000.0;
      let requested_usd = requested_microdollars as f64 / 1_000_000.0;
      Err(Box::new(create_openai_error_response(
        StatusCode::PAYMENT_REQUIRED,
        &format!(
          "Iron Cage insufficient budget. Available: ${available_usd:.2}, Requested: ${requested_usd:.2}. \
           Wait for in-flight requests to complete or increase budget."
        ),
        "iron_cage_insufficient_budget",
        "insufficient_budget",
      )))
    }
    Err(e) => {
      // Unexpected error (e.g., JsonParseError should never occur here)
      tracing::error!("Unexpected cost error: {}", e);
      Err(Box::new(create_openai_error_response(
        StatusCode::INTERNAL_SERVER_ERROR,
        &format!("Internal error: {e}"),
        "internal_error",
        "internal_error",
      )))
    }
  }
}

/// Main proxy handler - forwards requests to LLM provider
///
/// Local concerns handled here: IC Token validation, budget reservation/commit,
/// analytics recording. Core forwarding delegated to `iron_llm_core::forward_request`.
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
    .map(|q| format!("?{q}"))
    .unwrap_or_default();

  let body_bytes = axum::body::to_bytes(request.into_body(), 10 * 1024 * 1024) // 10MB limit
    .await
    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Body read error: {e}")))?;

  // 2.5 Reserve budget for this request (prevents concurrent overspend)
  let reservation: Option<Reservation> = if let Some(ref controller) = state.cost_controller {
    if let Some(max_cost) = state.pricing_manager.estimate_max_cost(&body_bytes) {
      match controller.reserve(max_cost) {
        Ok(res) => Some(res),
        Err(CostError::InsufficientBudget {
          available_microdollars,
          requested_microdollars,
        }) => {
          let available_usd = available_microdollars as f64 / 1_000_000.0;
          let requested_usd = requested_microdollars as f64 / 1_000_000.0;
          return Ok(create_openai_error_response(
            StatusCode::PAYMENT_REQUIRED,
            &format!(
              "Iron Cage insufficient budget for request. Available: ${available_usd:.4}, Estimated max cost: ${requested_usd:.4}. \
               Reduce max_tokens or wait for in-flight requests to complete."
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

  // 4. Forward request to LLM provider via iron_llm_core
  let forward_req = ForwardRequest {
    method,
    path: orig_path.clone(),
    query,
    body: body_bytes.to_vec(),
  };

  let forward_resp = iron_llm_core::forward_request(
    &state.http_client,
    &state.pricing_manager,
    &provider_key,
    forward_req,
  )
  .await
  .map_err(|e| match &e {
    LlmCoreError::Translation(_) => (StatusCode::BAD_REQUEST, e.to_string()),
    LlmCoreError::Forward(_) => (StatusCode::BAD_GATEWAY, e.to_string()),
  })?;

  let status =
    StatusCode::from_u16(forward_resp.status.as_u16()).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

  // 5. Settle budget reservation
  let cost_info = if status.is_success() {
    forward_resp.cost_info
  } else {
    None
  };

  if let Some(ref controller) = state.cost_controller {
    settle_reservation(controller, reservation, cost_info.as_ref());
  }

  // 6. Log cost, record analytics, and build response
  if let Some(ref ci) = cost_info {
    tracing::info!(
      model = %ci.model,
      input_tokens = ci.input_tokens,
      output_tokens = ci.output_tokens,
      cost_usd = %format!("{:.6}", ci.cost_usd()),
      "LLM request completed"
    );

    #[cfg(feature = "analytics")]
    record_success_analytics(&state, &orig_path, &body_bytes, ci);
  }

  let content_type = forward_resp
    .headers
    .get(header::CONTENT_TYPE)
    .and_then(|v| v.to_str().ok())
    .unwrap_or("application/json")
    .to_owned();

  let (response_body, is_streaming) = match forward_resp.body {
    ForwardBody::Buffered(bytes) => {
      if !status.is_success() {
        #[cfg(feature = "analytics")]
        record_failure_analytics(&state, status, &body_bytes, &bytes);
      }
      (Body::from(bytes), false)
    }
    ForwardBody::Streaming(provider_resp) => (
      Body::from_stream(futures_util::stream::unfold(provider_resp, |mut resp| async move {
        match resp.chunk().await {
          Ok(Some(chunk)) => Some((Ok::<_, String>(chunk), resp)),
          _ => None,
        }
      })),
      true,
    ),
  };

  let _ = is_streaming; // available for future analytics differentiation

  Response::builder()
    .status(status)
    .header(header::CONTENT_TYPE, content_type)
    .body(response_body)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Settle budget reservation based on forwarding result
fn settle_reservation(
  controller: &CostController,
  reservation: Option<Reservation>,
  cost_info: Option<&CostInfo>,
) {
  match (cost_info, reservation) {
    (Some(ci), Some(res)) => controller.commit(res, ci.cost_micros),
    (Some(ci), None) => {
      // No reservation was made, add directly (fallback for unknown models)
      controller.add_spend(i64::try_from(ci.cost_micros).unwrap_or(i64::MAX));
    }
    (None, Some(res)) => controller.cancel(res),
    (None, None) => {}
  }
}

/// Record analytics event for a successful LLM request
#[cfg(feature = "analytics")]
fn record_success_analytics(
  state: &ProxyState,
  orig_path: &str,
  request_body: &[u8],
  cost_info: &CostInfo,
) {
  let (_, path_provider) = iron_llm_core::strip_provider_prefix(orig_path);
  let model_provider = iron_llm_core::detect_provider_from_model(request_body);
  let target_provider = path_provider.or(model_provider).unwrap_or("openai");

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

/// Record analytics event for a failed LLM request
#[cfg(feature = "analytics")]
fn record_failure_analytics(
  state: &ProxyState,
  status: StatusCode,
  request_body: &[u8],
  response_body: &[u8],
) {
  if let Some(model) = iron_llm_core::extract_model_from_body(request_body) {
    let error_msg = serde_json::from_slice::<serde_json::Value>(response_body)
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
