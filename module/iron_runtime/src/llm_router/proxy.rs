//! Local HTTP proxy server for LLM requests

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

use crate::llm_router::error::LlmRouterError;
use crate::llm_router::key_fetcher::KeyFetcher;

/// Shared state for proxy handlers
#[derive(Clone)]
pub struct ProxyState
{
  /// IC_TOKEN for validating incoming requests
  pub ic_token: String,
  /// Key fetcher for getting real API keys
  pub key_fetcher: Arc<KeyFetcher>,
  /// HTTP client for forwarding requests
  pub http_client: Client,
}

/// Proxy server configuration
pub struct ProxyConfig
{
  pub port: u16,
  pub ic_token: String,
  pub server_url: String,
  pub cache_ttl_seconds: u64,
}

/// Run the proxy server
pub async fn run_proxy(
  config: ProxyConfig,
  shutdown_rx: oneshot::Receiver<()>,
) -> Result<(), LlmRouterError>
{
  let key_fetcher = Arc::new(KeyFetcher::new(
    config.server_url,
    config.ic_token.clone(),
    config.cache_ttl_seconds,
  ));

  let http_client = Client::builder()
    .timeout(std::time::Duration::from_secs(300)) // 5 min timeout for LLM requests
    .build()
    .map_err(|e| LlmRouterError::ServerStart(e.to_string()))?;

  let state = ProxyState {
    ic_token: config.ic_token,
    key_fetcher,
    http_client,
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
async fn handle_root() -> impl IntoResponse
{
  "LlmRouter OK"
}

/// Strip provider prefix from path if present, returns (clean_path, requested_provider)
pub fn strip_provider_prefix( path: &str ) -> ( String, Option< &'static str > )
{
  if path.starts_with("/anthropic/") || path.starts_with("/anthropic")
  {
    let clean = path.strip_prefix("/anthropic").unwrap_or(path);
    let clean = if clean.is_empty() { "/".to_string() } else { clean.to_string() };
    (clean, Some("anthropic"))
  }
  else if path.starts_with("/openai/") || path.starts_with("/openai")
  {
    let clean = path.strip_prefix("/openai").unwrap_or(path);
    let clean = if clean.is_empty() { "/".to_string() } else { clean.to_string() };
    (clean, Some("openai"))
  }
  else
  {
    (path.to_string(), None)
  }
}

/// Detect requested provider from model name in body
pub fn detect_provider_from_model( body: &[ u8 ] ) -> Option< &'static str >
{
  if let Ok(json) = serde_json::from_slice::<serde_json::Value>(body)
  {
    if let Some(model) = json.get("model").and_then(|m| m.as_str())
    {
      if model.starts_with("claude")
      {
        return Some("anthropic");
      }
      if model.starts_with("gpt") || model.starts_with("o1") || model.starts_with("o3")
      {
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
) -> Result<Response<Body>, (StatusCode, String)>
{
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

  if !is_valid
  {
    return Err((StatusCode::UNAUTHORIZED, "Invalid API key".to_string()));
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

  // 3. Get real API key from Iron Cage server (cached, auto-detected provider)
  let provider_key = state
    .key_fetcher
    .get_key()
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

  // 4. Use auto-detected provider from key format
  let configured_provider = provider_key.provider.as_str();
  let (clean_path, path_provider) = strip_provider_prefix(&orig_path);
  let model_provider = detect_provider_from_model(&body_bytes);

  // 5. Check for provider mismatch - give clear error to user
  let requested_provider = path_provider.or(model_provider);
  if let Some(requested) = requested_provider
  {
    if requested != configured_provider
    {
      return Err((
        StatusCode::BAD_REQUEST,
        format!(
          "Provider mismatch: you requested '{}' but your Iron Cage token has '{}' key configured. \
           Please configure a '{}' provider key in your Iron Cage dashboard.",
          requested, configured_provider, requested
        ),
      ));
    }
  }

  let provider = configured_provider;

  // 5. Build target URL
  let base_url = provider_key.base_url.as_deref().unwrap_or_else(|| {
    match provider
    {
      "anthropic" => "https://api.anthropic.com",
      _ => "https://api.openai.com",
    }
  });

  let target_url = format!("{}{}{}", base_url, clean_path, query);

  // 6. Build forwarded request with real API key
  let mut req_builder = state
    .http_client
    .request(method, &target_url)
    .header(header::CONTENT_TYPE, "application/json");

  // Set provider-specific auth headers
  if provider == "anthropic"
  {
    req_builder = req_builder
      .header("x-api-key", &provider_key.api_key)
      .header("anthropic-version", "2023-06-01");
  }
  else
  {
    req_builder =
      req_builder.header(header::AUTHORIZATION, format!("Bearer {}", provider_key.api_key));
  }

  // 7. Send request to provider
  let provider_response = req_builder
    .body(body_bytes.to_vec())
    .send()
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Forward error: {}", e)))?;

  // 8. Build response (pass through as-is)
  let status = provider_response.status();
  let resp_headers = provider_response.headers().clone();
  let resp_body = provider_response
    .bytes()
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, format!("Response read error: {}", e)))?;

  let mut response = Response::builder().status(status);

  // Copy content-type header
  if let Some(ct) = resp_headers.get(header::CONTENT_TYPE)
  {
    response = response.header(header::CONTENT_TYPE, ct);
  }

  response
    .body(Body::from(resp_body))
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
