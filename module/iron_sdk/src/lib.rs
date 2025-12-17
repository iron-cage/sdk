//! Iron SDK - Python bindings for Iron Cage AI agent protection
//!
//! This crate provides PyO3 bindings for the `iron_runtime` crate,
//! exposing LlmRouter and Runtime to Python.
//!
//! # Architecture
//!
//! This crate is a thin PyO3 wrapper around `iron_runtime`:
//! - `iron_runtime` - Pure Rust implementation (no PyO3)
//! - `iron_sdk` - PyO3 bindings (this crate)
//!
//! See spec.md for architecture details.
//!
//! # Python Usage
//!
//! ```python
//! from iron_cage import LlmRouter, Runtime
//!
//! # Use LlmRouter for LLM API proxying
//! with LlmRouter(api_key="ic_xxx", server_url="https://...") as router:
//!     client = OpenAI(base_url=router.base_url, api_key=router.api_key)
//!     response = client.chat.completions.create(...)
//! ```

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
use pyo3::prelude::*;

#[cfg(feature = "enabled")]
use iron_runtime::llm_router::LlmRouter as RustLlmRouter;

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
///     server_url="https://api.iron-cage.io",
/// )
/// client = OpenAI(base_url=router.base_url, api_key=router.api_key)
/// response = client.chat.completions.create(...)
/// router.stop()
/// ```
#[cfg(feature = "enabled")]
#[pyclass]
pub struct LlmRouter {
  inner: Option<RustLlmRouter>,
}

#[cfg(feature = "enabled")]
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
    if provider_key.is_none() && (api_key.is_none() || server_url.is_none()) {
      return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
        "Either 'provider_key' or both 'api_key' and 'server_url' must be provided. \
         Use provider_key + api_key + server_url for direct mode with analytics sync.",
      ));
    }

    let api_key = api_key.unwrap_or_else(|| "direct".to_string());
    let server_url = server_url.unwrap_or_default();

    let inner = RustLlmRouter::create_full(api_key, server_url, cache_ttl_seconds, budget, provider_key)
      .map_err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>)?;

    Ok(Self { inner: Some(inner) })
  }

  /// Get the base URL for the OpenAI client
  ///
  /// Returns URL like "http://127.0.0.1:52431/v1"
  #[getter]
  fn base_url(&self) -> PyResult<String> {
    self
      .inner
      .as_ref()
      .map(|r| r.get_base_url())
      .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Router is stopped"))
  }

  /// Get the API key to use with the OpenAI client
  #[getter]
  fn api_key(&self) -> PyResult<String> {
    self
      .inner
      .as_ref()
      .map(|r| r.get_api_key().to_string())
      .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Router is stopped"))
  }

  /// Get the port the proxy is listening on
  #[getter]
  fn port(&self) -> PyResult<u16> {
    self
      .inner
      .as_ref()
      .map(|r| r.get_port())
      .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Router is stopped"))
  }

  /// Get the auto-detected provider ("openai" or "anthropic")
  #[getter]
  fn provider(&self) -> PyResult<String> {
    self
      .inner
      .as_ref()
      .map(|r| r.get_provider().to_string())
      .ok_or_else(|| PyErr::new::<pyo3::exceptions::PyRuntimeError, _>("Router is stopped"))
  }

  /// Check if the proxy server is running
  #[getter]
  fn is_running(&self) -> bool {
    self.inner.as_ref().map(|r| r.is_running()).unwrap_or(false)
  }

  /// Get total spent in USD (0.0 if no budget set)
  fn total_spent(&self) -> f64 {
    self.inner.as_ref().map(|r| r.total_spent()).unwrap_or(0.0)
  }

  /// Set budget limit in USD
  fn set_budget(&self, amount_usd: f64) {
    if let Some(ref r) = self.inner {
      r.set_budget(amount_usd);
    }
  }

  /// Get current budget limit in USD (None if no budget set)
  #[getter]
  fn budget(&self) -> Option<f64> {
    self.inner.as_ref().and_then(|r| r.get_budget())
  }

  /// Get budget status as (spent, limit) tuple in USD
  #[getter]
  fn budget_status(&self) -> Option<(f64, f64)> {
    self.inner.as_ref().and_then(|r| r.get_budget_status())
  }

  /// Stop the proxy server
  fn stop(&mut self) {
    if let Some(mut r) = self.inner.take() {
      r.shutdown();
    }
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
    self.stop();
  }
}

#[cfg(feature = "enabled")]
impl Drop for LlmRouter {
  fn drop(&mut self) {
    // Inner LlmRouter handles cleanup in its own Drop
    self.inner.take();
  }
}

/// Runtime - Agent lifecycle management
///
/// Manages agent processes and coordinates with Iron Cage subsystems.
///
/// # Example
///
/// ```python
/// from iron_cage import Runtime
///
/// runtime = Runtime(budget=100.0, verbose=True)
/// # ... use runtime
/// ```
#[cfg(feature = "enabled")]
#[pyclass]
pub struct Runtime {
  inner: iron_runtime::AgentRuntime,
}

#[cfg(feature = "enabled")]
#[pymethods]
impl Runtime {
  /// Create new runtime
  #[new]
  #[pyo3(signature = (budget, verbose=None))]
  fn new(budget: f64, verbose: Option<bool>) -> Self {
    let config = iron_runtime::RuntimeConfig {
      budget,
      verbose: verbose.unwrap_or(false),
    };

    Self {
      inner: iron_runtime::AgentRuntime::new(config),
    }
  }

  /// Get the budget
  #[getter]
  fn budget(&self) -> f64 {
    self.inner.config().budget
  }

  /// Get verbose setting
  #[getter]
  fn verbose(&self) -> bool {
    self.inner.config().verbose
  }

  /// Start an agent (placeholder - async not yet implemented)
  fn start_agent(&self, _script_path: String) -> PyResult<String> {
    // TODO: Implement async bridge with pyo3-asyncio
    Ok("agent_placeholder".to_string())
  }

  /// Stop an agent
  fn stop_agent(&self, _agent_id: String) -> PyResult<()> {
    // TODO: Implement async bridge
    Ok(())
  }

  /// Get agent metrics as JSON string
  fn get_metrics(&self, agent_id: String) -> PyResult<Option<String>> {
    match self.inner.get_metrics(&agent_id) {
      Some(state) => {
        let json = serde_json::json!({
          "agent_id": state.agent_id.as_str(),
          "status": format!("{:?}", state.status),
          "budget_spent": state.budget_spent,
          "pii_detections": state.pii_detections,
        });
        Ok(Some(json.to_string()))
      }
      None => Ok(None),
    }
  }
}

/// Python module definition
#[cfg(feature = "enabled")]
#[pymodule]
fn iron_cage(m: &Bound<'_, PyModule>) -> PyResult<()> {
  m.add_class::<LlmRouter>()?;
  m.add_class::<Runtime>()?;
  Ok(())
}
