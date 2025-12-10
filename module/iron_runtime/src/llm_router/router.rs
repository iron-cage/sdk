//! LlmRouter - PyO3 class for Python integration

use pyo3::prelude::*;
use std::net::TcpListener;
use std::sync::Arc;
use tokio::sync::oneshot;

use crate::llm_router::key_fetcher::KeyFetcher;
use crate::llm_router::proxy::{run_proxy, ProxyConfig};

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
#[pyclass]
pub struct LlmRouter
{
  /// Port the proxy is listening on
  pub port: u16,
  /// API key (IC_TOKEN)
  pub api_key: String,
  /// Iron Cage server URL
  #[allow( dead_code )]
  pub server_url: String,
  /// Auto-detected provider from API key format ("openai" or "anthropic")
  pub provider: String,
  /// Tokio runtime
  #[allow( dead_code )]
  runtime: tokio::runtime::Runtime,
  /// Shutdown channel
  shutdown_tx: Option< oneshot::Sender< () > >,
}

#[pymethods]
impl LlmRouter
{
  /// Create a new LlmRouter instance
  ///
  /// # Arguments
  ///
  /// * `api_key` - Iron Cage API token (IC_TOKEN)
  /// * `server_url` - Iron Cage server URL (required)
  /// * `cache_ttl_seconds` - How long to cache API keys (default: 300)
  ///
  /// # Returns
  ///
  /// LlmRouter instance with running proxy server
  ///
  /// # Raises
  ///
  /// RuntimeError if server fails to start
  #[new]
  #[pyo3(signature = (api_key, server_url, cache_ttl_seconds=300))]
  fn new( api_key: String, server_url: String, cache_ttl_seconds: u64 ) -> PyResult< Self >
  {
    Self::create_inner( api_key, server_url, cache_ttl_seconds )
      .map_err( PyErr::new::< pyo3::exceptions::PyRuntimeError, _ > )
  }

  /// Get the base URL for the OpenAI client
  ///
  /// Returns URL like "http://127.0.0.1:52431/v1"
  #[getter]
  fn base_url( &self ) -> String
  {
    self.get_base_url()
  }

  /// Get the API key to use with the OpenAI client
  ///
  /// Returns the IC_TOKEN which the proxy validates
  #[getter]
  fn api_key( &self ) -> String
  {
    self.api_key.clone()
  }

  /// Get the port the proxy is listening on
  #[getter]
  fn port( &self ) -> u16
  {
    self.port
  }

  /// Get the auto-detected provider ("openai" or "anthropic")
  ///
  /// Detected from the API key format returned by Iron Cage server:
  /// - sk-ant-* → "anthropic"
  /// - sk-* → "openai"
  #[getter]
  fn provider( &self ) -> String
  {
    self.provider.clone()
  }

  /// Check if the proxy server is running
  #[getter]
  fn is_running( &self ) -> bool
  {
    self.running()
  }

  /// Stop the proxy server
  fn stop( &mut self )
  {
    self.shutdown();
  }

  // Context manager support
  fn __enter__( slf: PyRef< '_, Self > ) -> PyRef< '_, Self >
  {
    slf
  }

  #[pyo3(signature = (_exc_type=None, _exc_val=None, _exc_tb=None))]
  fn __exit__(
    &mut self,
    _exc_type: Option< PyObject >,
    _exc_val: Option< PyObject >,
    _exc_tb: Option< PyObject >,
  )
  {
    self.shutdown();
  }
}

impl Drop for LlmRouter
{
  fn drop( &mut self )
  {
    self.stop_inner();
  }
}

// Native Rust methods (for testing and non-Python usage)
impl LlmRouter
{
  /// Create a new LlmRouter instance (Rust API)
  ///
  /// Returns Result instead of PyResult for Rust usage.
  pub fn create(
    api_key: String,
    server_url: String,
    cache_ttl_seconds: u64,
  ) -> Result< Self, String >
  {
    Self::create_inner( api_key, server_url, cache_ttl_seconds )
  }

  /// Get the base URL for the OpenAI client (Rust API)
  pub fn get_base_url( &self ) -> String
  {
    format!( "http://127.0.0.1:{}/v1", self.port )
  }

  /// Check if running (Rust API)
  pub fn running( &self ) -> bool
  {
    self.shutdown_tx.is_some()
  }

  /// Stop the router (Rust API)
  pub fn shutdown( &mut self )
  {
    self.stop_inner();
  }

  /// Internal creation logic shared by Python and Rust APIs
  fn create_inner(
    api_key: String,
    server_url: String,
    cache_ttl_seconds: u64,
  ) -> Result< Self, String >
  {
    // Find free port
    let port = find_free_port().map_err( | e | format!( "Failed to find free port: {}", e ) )?;

    // Create tokio runtime
    let runtime = tokio::runtime::Builder::new_multi_thread()
      .worker_threads( 2 )
      .enable_all()
      .build()
      .map_err( | e | format!( "Failed to create runtime: {}", e ) )?;

    // Fetch provider key at startup to detect provider early
    let key_fetcher = Arc::new( KeyFetcher::new(
      server_url.clone(),
      api_key.clone(),
      cache_ttl_seconds,
    ) );

    let provider = runtime.block_on( async {
      key_fetcher
        .get_key()
        .await
        .map( | k | k.provider )
        .unwrap_or_else( | _ | "unknown".to_string() )
    } );

    // Create shutdown channel
    let ( shutdown_tx, shutdown_rx ) = oneshot::channel();

    // Create config
    let config = ProxyConfig {
      port,
      ic_token: api_key.clone(),
      server_url: server_url.clone(),
      cache_ttl_seconds,
    };

    // Spawn proxy server
    runtime.spawn( async move {
      if let Err( e ) = run_proxy( config, shutdown_rx ).await
      {
        tracing::error!( "Proxy server error: {}", e );
      }
    } );

    // Wait for server to start
    std::thread::sleep( std::time::Duration::from_millis( 50 ) );

    Ok( Self {
      port,
      api_key,
      server_url,
      provider,
      runtime,
      shutdown_tx: Some( shutdown_tx ),
    } )
  }

  fn stop_inner( &mut self )
  {
    if let Some( tx ) = self.shutdown_tx.take()
    {
      let _ = tx.send( () );
    }
  }
}

/// Find an available port on localhost
fn find_free_port() -> std::io::Result< u16 >
{
  let listener = TcpListener::bind( "127.0.0.1:0" )?;
  Ok( listener.local_addr()?.port() )
}
