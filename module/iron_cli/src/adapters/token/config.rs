//! Token API configuration
//!
//! Loads configuration for Token Manager API HTTP client.
//!
//! ## Configuration Sources (Priority Order)
//!
//! 1. Environment variables (highest priority)
//! 2. Config file (~/.iron/config.toml)
//! 3. Defaults (lowest priority)
//!
//! ## Environment Variables
//!
//! - IRON_TOKEN_API_URL: Base URL for Token Manager API (default: http://localhost:8081)
//! - IRON_TOKEN_API_TIMEOUT: Request timeout in seconds (default: 30)
//!
//! ## Config File Format
//!
//! ```toml
//! [token_api]
//! base_url = "http://localhost:8081"
//! timeout_seconds = 30
//! ```
//!
//! ## Authentication
//!
//! Token API uses keyring-stored access tokens, not static API tokens.
//! See keyring module for auth token management.

use std::time::Duration;

/// Token API configuration
#[derive(Debug, Clone)]
pub struct TokenApiConfig
{
  /// Base URL for Token Manager API
  pub base_url: String,

  /// HTTP request timeout
  pub timeout: Duration,
}

impl Default for TokenApiConfig
{
  fn default() -> Self
  {
    Self
    {
      base_url: "http://localhost:8081".to_string(),
      timeout: Duration::from_secs( 30 ),
    }
  }
}

impl TokenApiConfig
{
  /// Load configuration from environment and config file
  ///
  /// Priority: env vars > config file > defaults
  pub fn load() -> Self
  {
    let mut config = Self::default();

    // Load from environment variables
    if let Ok( url ) = std::env::var( "IRON_TOKEN_API_URL" )
    {
      config.base_url = url;
    }

    if let Ok( timeout_str ) = std::env::var( "IRON_TOKEN_API_TIMEOUT" )
    {
      if let Ok( timeout_secs ) = timeout_str.parse::<u64>()
      {
        config.timeout = Duration::from_secs( timeout_secs );
      }
    }

    // TODO: Load from config file (~/.iron/config.toml) in future phase

    config
  }

  /// Create configuration with explicit values
  pub fn new( base_url: String ) -> Self
  {
    Self
    {
      base_url,
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
