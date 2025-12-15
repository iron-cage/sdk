//! Token API configuration
//!
//! Uses `iron_config` for unified configuration loading with 5-layer precedence.
//!
//! ## Configuration Sources (Priority Order)
//!
//! 1. Environment variables (`IRON_TOKEN_API_*` format, highest priority)
//! 2. Project config (`{workspace}/config/iron_token_api.{env}.toml`)
//! 3. User config (`~/.config/iron/iron_token_api.toml`)
//! 4. Workspace defaults (`{workspace}/config/iron_token_api.default.toml`)
//! 5. Crate defaults (lowest priority)
//!
//! ## Environment Variables
//!
//! - `IRON_TOKEN_API_URL`: Base URL for Token Manager API (default: http://localhost:8081)
//! - `IRON_TOKEN_API_TIMEOUT`: Request timeout in seconds (default: 30)
//!
//! ## Config File Format
//!
//! ```toml
//! url = "http://localhost:8081"
//! timeout = 30
//! ```
//!
//! ## Authentication
//!
//! Token API uses keyring-stored access tokens, not static API tokens.
//! See keyring module for auth token management.

use iron_config::ConfigLoader;
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
  /// Load configuration using `iron_config` with 5-layer precedence
  ///
  /// Environment variables: `IRON_TOKEN_API_URL`, `IRON_TOKEN_API_TIMEOUT`
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  pub fn load() -> Self
  {
    let defaults = r#"
url = "http://localhost:8081"
timeout = 30
"#;

    let loader = ConfigLoader::with_defaults( "iron_token_api", defaults )
      .expect( "Failed to create token API config loader" );

    let base_url = loader.get::< String >( "url" )
      .unwrap_or_else( |_| "http://localhost:8081".to_string() );

    let timeout_secs = loader.get::< u64 >( "timeout" )
      .unwrap_or( 30 );

    Self
    {
      base_url,
      timeout: Duration::from_secs( timeout_secs ),
    }
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
