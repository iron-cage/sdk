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

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_default_config()
  {
    let config = ControlApiConfig::default();
    assert_eq!( config.base_url, "http://localhost:8080" );
    assert_eq!( config.api_token, None );
    assert_eq!( config.timeout, Duration::from_secs( 30 ) );
  }

  #[test]
  fn test_new_config()
  {
    let config = ControlApiConfig::new(
      "https://api.example.com".to_string(),
      Some( "test-token".to_string() ),
    );

    assert_eq!( config.base_url, "https://api.example.com" );
    assert_eq!( config.api_token, Some( "test-token".to_string() ) );
  }

  #[test]
  fn test_with_timeout()
  {
    let config = ControlApiConfig::default()
      .with_timeout( Duration::from_secs( 60 ) );

    assert_eq!( config.timeout, Duration::from_secs( 60 ) );
  }
}
