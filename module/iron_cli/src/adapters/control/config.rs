//! Control API configuration
//!
//! Uses `iron_config` for unified configuration loading with 5-layer precedence.
//!
//! ## Configuration Sources (Priority Order)
//!
//! 1. Environment variables (`IRON_CONTROL_API_*` format, highest priority)
//! 2. Project config (`{workspace}/config/iron_control_api.{env}.toml`)
//! 3. User config (`~/.config/iron/iron_control_api.toml`)
//! 4. Workspace defaults (`{workspace}/config/iron_control_api.default.toml`)
//! 5. Crate defaults (lowest priority)
//!
//! ## Environment Variables
//!
//! - `IRON_CONTROL_API_URL`: Base URL for Control API (default: http://localhost:8080)
//! - `IRON_CONTROL_API_TOKEN`: API authentication token (optional)
//! - `IRON_CONTROL_API_TIMEOUT`: Request timeout in seconds (default: 30)
//!
//! ## Config File Format
//!
//! ```toml
//! url = "http://localhost:8080"
//! token = "your-api-token-here"
//! timeout = 30
//! ```

use iron_config::ConfigLoader;
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
  /// Load configuration using `iron_config` with 5-layer precedence
  ///
  /// Environment variables: `IRON_CONTROL_API_URL`, `IRON_CONTROL_API_TOKEN`, `IRON_CONTROL_API_TIMEOUT`
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  pub fn load() -> Self
  {
    let defaults = r#"
url = "http://localhost:8080"
timeout = 30
"#;

    let loader = ConfigLoader::with_defaults( "iron_control_api", defaults )
      .expect( "Failed to create control API config loader" );

    let base_url = loader.get::< String >( "url" )
      .unwrap_or_else( |_| "http://localhost:8080".to_string() );

    let api_token = loader.get_opt::< String >( "token" )
      .ok()
      .flatten();

    let timeout_secs = loader.get::< u64 >( "timeout" )
      .unwrap_or( 30 );

    Self
    {
      base_url,
      api_token,
      timeout: Duration::from_secs( timeout_secs ),
    }
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
