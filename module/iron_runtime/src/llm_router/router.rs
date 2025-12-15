//! LlmRouter - PyO3 class for Python integration

use pyo3::prelude::*;
use std::net::TcpListener;
use std::sync::{Arc, Once};
use tokio::sync::oneshot;

use iron_cost::budget::CostController;
use iron_telemetry::{init_logging, LogLevel};

/// Initialize logging once
static LOGGING_INIT: Once = Once::new();

fn ensure_logging() {
  LOGGING_INIT.call_once(|| {
    let _ = init_logging(LogLevel::Info);
  });
}
use crate::llm_router::key_fetcher::KeyFetcher;
use crate::llm_router::proxy::{run_proxy, ProxyConfig};

#[cfg(feature = "analytics")]
use iron_runtime_analytics::EventStore;

#[cfg(feature = "analytics")]
use iron_runtime_analytics::{SyncClient, SyncConfig, SyncHandle};

/// LLM Router - Local proxy server for OpenAI/Anthropic API requests
///
/// Creates a local HTTP server that intercepts LLM API requests,
/// fetches real API keys from Iron Cage server, and forwards
/// requests to the actual provider.
///
/// # Example
///
/// ```python
/// from iron_cage import LlmRouter
/// from openai import OpenAI
///
/// router = LlmRouter(
///     api_key="ic_xxx",
///     server_url="[https://api.iron-cage.io](https://api.iron-cage.io)",
/// )
/// client = OpenAI(base_url=router.base_url, api_key=router.api_key)
/// response = client.chat.completions.create(...)
/// router.stop()
/// ```
#[pyclass]
pub struct LlmRouter {
  /// Port the proxy is listening on
  pub port: u16,
  /// API key (IC_TOKEN)
  pub api_key: String,
  /// Iron Cage server URL
  #[allow(dead_code)]
  pub server_url: String,
  /// Auto-detected provider from API key format ("openai" or "anthropic")
  pub provider: String,
  /// Tokio runtime
  #[allow(dead_code)]
  runtime: tokio::runtime::Runtime,
  /// Shutdown channel
  shutdown_tx: Option<oneshot::Sender<()>>,
  /// Cost controller for budget enforcement and spending tracking (None = no budget)
  cost_controller: Option<Arc<CostController>>,
  /// Analytics event store
  #[cfg(feature = "analytics")]
  event_store: Arc<EventStore>,
  /// Agent ID for analytics attribution
  #[cfg(feature = "analytics")]
  #[allow(dead_code)]
  agent_id: Option<Arc<str>>,
  /// Provider ID for analytics attribution
  #[cfg(feature = "analytics")]
  #[allow(dead_code)]
  provider_id: Option<Arc<str>>,
  /// Analytics sync handle - auto-flushes on drop
  #[cfg(feature = "analytics")]
  #[allow(dead_code)] // Used for Drop behavior
  sync_handle: Option<SyncHandle>,
  /// Lease ID from server handshake (for budget return on shutdown)
  lease_id: Option<String>,
}

#[pymethods]
impl LlmRouter {
  /// Create a new LlmRouter instance
  ///
  /// # Arguments
  ///
  /// * `api_key` - Iron Cage API token (required unless provider_key is set)
  /// * `server_url` - Iron Cage server URL (required unless provider_key is set)
  /// * `cache_ttl_seconds` - How long to cache API keys (default: 300)
  /// * `budget` - Optional budget limit in USD
  /// * `provider_key` - Direct provider API key (bypasses Iron Cage server)
  ///
  /// # Usage
  ///
  /// Mode 1 - Iron Cage server:
  /// ```python
  /// router = LlmRouter(api_key="ic_xxx", server_url="https://...")
  /// ```
  ///
  /// Mode 2 - Direct provider key:
  /// ```python
  /// router = LlmRouter(provider_key="sk-xxx", budget=10.0)
  /// ```
  ///
  /// # Returns
  ///
  /// LlmRouter instance with running proxy server
  ///
  /// # Raises
  ///
  /// RuntimeError if server fails to start or if neither mode is configured
  #[new]
  #[pyo3(signature = (api_key=None, server_url=None, cache_ttl_seconds=300, budget=None, provider_key=None))]
  fn new(
    api_key: Option<String>,
    server_url: Option<String>,
    cache_ttl_seconds: u64,
    budget: Option<f64>,
    provider_key: Option<String>,
  ) -> PyResult<Self> {
    // Validate: either provider_key OR (api_key + server_url) must be provided
    // Can also use provider_key + api_key + server_url for direct mode with analytics sync
    if provider_key.is_none() && (api_key.is_none() || server_url.is_none()) {
      return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
        "Either 'provider_key' or both 'api_key' and 'server_url' must be provided. \
         Use provider_key + api_key + server_url for direct mode with analytics sync."
      ));
    }

    let api_key = api_key.unwrap_or_else(|| "direct".to_string());
    let server_url = server_url.unwrap_or_default();

    Self::create_inner(api_key, server_url, cache_ttl_seconds, budget, provider_key)
        .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)
  }

  /// Get the base URL for the OpenAI client
  ///
  /// Returns URL like "http://127.0.0.1:52431/v1"
  #[getter]
  fn base_url(&self) -> String {
    self.get_base_url()
  }

  /// Get the API key to use with the OpenAI client
  ///
  /// Returns the IC_TOKEN which the proxy validates
  #[getter]
  fn api_key(&self) -> String {
    self.api_key.clone()
  }

  /// Get the port the proxy is listening on
  #[getter]
  fn port(&self) -> u16 {
    self.port
  }

  /// Get the auto-detected provider ("openai" or "anthropic")
  ///
  /// Detected from the API key format returned by Iron Cage server:
  /// - sk-ant-* → "anthropic"
  /// - sk-* → "openai"
  #[getter]
  fn provider(&self) -> String {
    self.provider.clone()
  }

  /// Check if the proxy server is running
  #[getter]
  fn is_running(&self) -> bool {
    self.running()
  }

  /// Get total spent in USD (0.0 if no budget set)
  fn total_spent(&self) -> f64 {
    self.cost_controller.as_ref().map(|c| c.total_spent() as f64 / 1_000_000.0).unwrap_or(0.0)
  }

  /// Set budget limit in USD
  ///
  /// # Arguments
  /// * `amount_usd` - New budget limit in USD (e.g., 10.0 for $10)
  fn set_budget(&self, amount_usd: f64) {
    if let Some(ref controller) = self.cost_controller {
      let budget_micros = (amount_usd * 1_000_000.0) as i64;
      controller.set_budget(budget_micros);
    }
  }

  /// Get current budget limit in USD (None if no budget set)
  #[getter]
  fn budget(&self) -> Option<f64> {
    self.cost_controller.as_ref().map(|c| c.budget_limit() as f64 / 1_000_000.0)
  }

  /// Get budget status as (spent, limit) tuple in USD
  /// Returns None if no budget is set
  #[getter]
  fn budget_status(&self) -> Option<(f64, f64)> {
    self.cost_controller.as_ref().map(|c| {
      let (spent_micros, limit_micros) = c.get_status();
      (spent_micros as f64 / 1_000_000.0, limit_micros as f64 / 1_000_000.0)
    })
  }

  /// Stop the proxy server
  fn stop(&mut self) {
    self.shutdown();
  }

  // Context manager support
  fn __enter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
    slf
  }

  #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
  fn __exit__(
    &mut self,
    _exc_type: Option<PyObject>,
    _exc_val: Option<PyObject>,
    _exc_tb: Option<PyObject>,
  ) {
    self.shutdown();
  }
}

impl Drop for LlmRouter {
  fn drop(&mut self) {
    self.stop_inner();
  }
}

// Native Rust methods (for testing and non-Python usage)
impl LlmRouter {
  /// Create a new LlmRouter instance (Rust API)
  ///
  /// Returns Result instead of PyResult for Rust usage.
  pub fn create(
    api_key: String,
    server_url: String,
    cache_ttl_seconds: u64,
  ) -> Result<Self, String> {
    Self::create_inner(api_key, server_url, cache_ttl_seconds, None, None)
  }

  /// Create a new LlmRouter instance with budget (Rust API)
  pub fn create_with_budget(
    api_key: String,
    server_url: String,
    cache_ttl_seconds: u64,
    budget: f64,
  ) -> Result<Self, String> {
    Self::create_inner(api_key, server_url, cache_ttl_seconds, Some(budget), None)
  }

  /// Create a new LlmRouter instance with direct provider key (Rust API)
  /// Bypasses Iron Cage server - useful for testing or direct provider access
  pub fn create_with_provider_key(
    provider_key: String,
    budget: Option<f64>,
  ) -> Result<Self, String> {
    Self::create_inner(
      "direct".to_string(),
      String::new(),
      0,
      budget,
      Some(provider_key),
    )
  }

  /// Get the base URL for the OpenAI client (Rust API)
  pub fn get_base_url(&self) -> String {
    format!("http://127.0.0.1:{}/v1", self.port)
  }

  /// Check if running (Rust API)
  pub fn running(&self) -> bool {
    self.shutdown_tx.is_some()
  }

  /// Stop the router (Rust API)
  pub fn shutdown(&mut self) {
    self.stop_inner();
  }

  /// Internal creation logic shared by Python and Rust APIs
  fn create_inner(
    api_key: String,
    server_url: String,
    cache_ttl_seconds: u64,
    budget: Option<f64>,
    provider_key: Option<String>,
  ) -> Result<Self, String> {
    // Initialize logging
    ensure_logging();

    // Find free port
    let port = find_free_port().map_err(|e| format!("Failed to find free port: {}", e))?;

    // Create tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .map_err(|e| format!("Failed to create runtime: {}", e))?;

    // Create key fetcher - static if provider_key given, otherwise fetch from server
    let key_fetcher = Arc::new(if let Some(ref pk) = provider_key {
      KeyFetcher::new_static(pk.clone(), None)
    } else {
      KeyFetcher::new(
        server_url.clone(),
        api_key.clone(),
        cache_ttl_seconds,
      )
    });

    let provider = runtime.block_on(async {
      key_fetcher
          .get_key()
          .await
          .map(|k| k.provider)
          .unwrap_or_else(|_| "unknown".to_string())
    });

    // Create shutdown channel
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Determine budget: use explicit budget, or fetch from server handshake, or default to 0
    // Also capture lease_id from handshake for budget return on shutdown
    // Note: Python API accepts dollars (f64), server returns microdollars (i64)
    let (effective_budget_micros, lease_id): (i64, Option<String>) = if let Some(b) = budget {
      // Convert dollars from Python API to microdollars
      ((b * 1_000_000.0) as i64, None)
    } else if !server_url.is_empty() {
      // Fetch budget from server handshake (Protocol 005) - already in microdollars
      match runtime.block_on(async {
        fetch_budget_from_handshake(&server_url, &api_key).await
      }) {
        Some(result) => (result.budget, Some(result.lease_id)),
        None => (0, None),
      }
    } else {
      (0, None)
    };

    // Create cost controller with effective budget (always created, defaults to 0)
    let cost_controller = Some(Arc::new(CostController::new(effective_budget_micros)));

    // Create analytics event store (feature-gated)
    #[cfg(feature = "analytics")]
    let event_store = Arc::new(EventStore::new());
    #[cfg(feature = "analytics")]
    let agent_id_arc: Option<Arc<str>> = None; // Can be extended to accept from Python
    #[cfg(feature = "analytics")]
    let provider_id_arc: Option<Arc<str>> = None;

    // Record router started event
    #[cfg(feature = "analytics")]
    event_store.record_router_started(port);

    // Create config
    let config = ProxyConfig {
      port,
      ic_token: api_key.clone(),
      server_url: server_url.clone(),
      cache_ttl_seconds,
      cost_controller: cost_controller.clone(),
      provider_key: provider_key.clone(),
      #[cfg(feature = "analytics")]
      event_store: event_store.clone(),
      #[cfg(feature = "analytics")]
      agent_id: agent_id_arc.clone(),
      #[cfg(feature = "analytics")]
      provider_id: provider_id_arc.clone(),
    };

    // Spawn proxy server
    runtime.spawn(async move {
      if let Err(e) = run_proxy(config, shutdown_rx).await {
        tracing::error!("Proxy server error: {}", e);
      }
    });

    // Start analytics sync (if server_url is provided)
    #[cfg(feature = "analytics")]
    let sync_handle = if !server_url.is_empty() {
      let sync_config = SyncConfig::new(&server_url, &api_key);
      let sync_client = SyncClient::new(event_store.clone(), sync_config);
      Some(sync_client.start(runtime.handle()))
    } else {
      None
    };

    // Wait for server to start
    std::thread::sleep(std::time::Duration::from_millis(50));

    Ok(Self {
      port,
      api_key,
      server_url,
      provider,
      runtime,
      shutdown_tx: Some(shutdown_tx),
      cost_controller,
      #[cfg(feature = "analytics")]
      event_store,
      #[cfg(feature = "analytics")]
      agent_id: agent_id_arc,
      #[cfg(feature = "analytics")]
      provider_id: provider_id_arc,
      #[cfg(feature = "analytics")]
      sync_handle,
      lease_id,
    })
  }

  fn stop_inner(&mut self) {
    if let Some(tx) = self.shutdown_tx.take() {
      // Return unused budget to server before shutting down (Protocol 005)
      if let Some(lease_id) = self.lease_id.take() {
        if !self.server_url.is_empty() {
          // Get spent amount from cost_controller (already in microdollars)
          let spent_microdollars = self.cost_controller
            .as_ref()
            .map(|cc| cc.total_spent())
            .unwrap_or(0);

          let url = format!("{}/api/v1/budget/return", self.server_url);
          let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build();

          match client {
            Ok(client) => {
              match client
                .post(&url)
                .header("Content-Type", "application/json")
                .json(&serde_json::json!({"lease_id": lease_id, "spent_microdollars": spent_microdollars}))
                .send()
              {
                Ok(resp) if resp.status().is_success() => {
                  if let Ok(body) = resp.json::<serde_json::Value>() {
                    if let Some(returned) = body.get("returned").and_then(|v| v.as_i64()) {
                      tracing::info!("Budget returned to server: ${:.6} (spent: ${:.6})",
                        returned as f64 / 1_000_000.0, spent_microdollars as f64 / 1_000_000.0);
                    }
                  }
                }
                Ok(resp) => {
                  tracing::warn!("Budget return failed with status: {}", resp.status());
                }
                Err(e) => {
                  tracing::warn!("Budget return request failed: {}", e);
                }
              }
            }
            Err(e) => {
              tracing::warn!("Failed to create HTTP client for budget return: {}", e);
            }
          }
        }
      }

      #[cfg(feature = "analytics")]
      self.event_store.record_router_stopped();

      // Stop analytics sync (triggers flush) before stopping proxy
      #[cfg(feature = "analytics")]
      if let Some(handle) = self.sync_handle.take() {
        handle.stop(); // This triggers flush
        // Give sync task time to complete flush
        std::thread::sleep(std::time::Duration::from_millis(500));
      }

      let _ = tx.send(());
    }
  }
}

/// Find an available port on localhost
fn find_free_port() -> std::io::Result<u16> {
  let listener = TcpListener::bind("127.0.0.1:0")?;
  Ok(listener.local_addr()?.port())
}

/// Result from server handshake containing budget and lease info
struct HandshakeResult {
  budget: i64,  // microdollars
  lease_id: String,
}

/// Fetch budget from server handshake (Protocol 005)
async fn fetch_budget_from_handshake(server_url: &str, ic_token: &str) -> Option<HandshakeResult> {
  let client = reqwest::Client::new();
  let url = format!("{}/api/v1/budget/handshake", server_url);

  let response = match client
    .post(&url)
    .header("Authorization", format!("Bearer {}", ic_token))
    .json(&serde_json::json!({
      "ic_token": ic_token,
      "provider": "openai"
    }))
    .send()
    .await
  {
    Ok(r) => r,
    Err(e) => {
      tracing::warn!("Failed to connect to server for handshake: {}", e);
      return None;
    }
  };

  if !response.status().is_success() {
    tracing::warn!("Handshake failed with status: {}", response.status());
    return None;
  }

  #[derive(serde::Deserialize)]
  struct HandshakeResponse {
    budget_granted: i64,  // microdollars
    lease_id: String,
  }

  match response.json::<HandshakeResponse>().await {
    Ok(data) => {
      tracing::info!("Budget from server handshake: ${:.6} ({}μ$), lease_id: {}",
        data.budget_granted as f64 / 1_000_000.0, data.budget_granted, data.lease_id);
      Some(HandshakeResult {
        budget: data.budget_granted,
        lease_id: data.lease_id,
      })
    }
    Err(e) => {
      tracing::warn!("Failed to parse handshake response: {}", e);
      None
    }
  }
}
