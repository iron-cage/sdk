//! Control API configuration
//!
//! Loads configuration for Control API HTTP client.
//!
//! ## Configuration Sources (Priority Order)
//!
//! 1. Environment variables (highest priority)
//! 2. Config file (~/.iron/config.toml)
//! 3. Defaults (lowest priority)
//!
//! ## Environment Variables
//!
//! - IRON_CONTROL_API_URL: Base URL for Control API (default: http://localhost:8080)
//! - IRON_API_TOKEN: API authentication token
//!
//! ## Config File Format
//!
//! ```toml
//! [control_api]
//! base_url = "http://localhost:8080"
//! api_token = "your-api-token-here"
//! timeout_seconds = 30
//! ```

use std::time::Duration;

/// Control API configuration
#[derive(Debug, Clone)]
pub struct ControlApiConfig
{
  /// Base URL for Control API
  pub base_url: String,

  /// API authentication token (optional)
  pub api_token: Option<String>,

  /// HTTP request timeout
  pub timeout: Duration,
}

impl Default for ControlApiConfig
{
  fn default() -> Self
  {
    Self
    {
      base_url: "http://localhost:8080".to_string(),
      api_token: None,
      timeout: Duration::from_secs( 30 ),
    }
  }
}

impl ControlApiConfig
{
  /// Load configuration from environment and config file
  ///
  /// Priority: env vars > config file > defaults
  pub fn load() -> Self
  {
    let mut config = Self::default();

    // Load from environment variables
    if let Ok( url ) = std::env::var( "IRON_CONTROL_API_URL" )
    {
      config.base_url = url;
    }

    if let Ok( token ) = std::env::var( "IRON_API_TOKEN" )
    {
      config.api_token = Some( token );
    }

    // TODO: Load from config file (~/.iron/config.toml) in Phase 3

    config
  }

  /// Create configuration with explicit values
  pub fn new( base_url: String, api_token: Option<String> ) -> Self
  {
    Self
    {
      base_url,
      api_token,
      timeout: Duration::from_secs( 30 ),
    }
  }

  /// Set timeout
  pub fn with_timeout( mut self, timeout: Duration ) -> Self
  {
    self.timeout = timeout;
    self
  }
}
