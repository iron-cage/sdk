//! Configuration system with hierarchical precedence
//!
//! Uses `iron_config_loader` for unified configuration loading with 5-layer precedence.
//!
//! ## Configuration Hierarchy (highest to lowest priority)
//!
//! 1. CLI arguments (keyword::value parameters, applied after iron_config_loader loading)
//! 2. Environment variables (`IRON_CLI_*` format, e.g., `IRON_CLI_API_URL`)
//! 3. Project config (`{workspace}/config/iron_cli.{env}.toml`)
//! 4. User config (`~/.config/iron/iron_cli.toml`)
//! 5. Workspace defaults (`{workspace}/config/iron_cli.default.toml`)
//! 6. Crate defaults (hardcoded in this file)
//!
//! ## Environment Variables
//!
//! - `IRON_CLI_API_URL`: API base URL
//! - `IRON_CLI_FORMAT`: Output format (table/expanded/json/yaml)
//! - `IRON_CLI_USER`: Default user
//! - `IRON_CLI_TOKEN`: Authentication token
//!
//! ## Usage
//!
//! ```rust
//! use iron_cli::config::Config;
//!
//! // Simple usage with defaults
//! let config = Config::new();
//! let api_url = config.get("api_url").unwrap_or_default();
//\! ```

use iron_config_loader::ConfigLoader;
use std::collections::HashMap;

/// Configuration error types
#[ derive( Debug ) ]
pub enum ConfigError
{
  /// Invalid configuration value
  InvalidValue( String ),

  /// Missing required configuration
  MissingRequired( String ),

  /// IO error reading configuration file
  IoError( String ),
}

impl std::fmt::Display for ConfigError
{
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result
  {
    match self
    {
      ConfigError::InvalidValue( msg ) => write!( f, "Invalid configuration value: {}", msg ),
      ConfigError::MissingRequired( msg ) => write!( f, "Missing required configuration: {}", msg ),
      ConfigError::IoError( msg ) => write!( f, "Configuration IO error: {}", msg ),
    }
  }
}

impl std::error::Error for ConfigError {}

/// Configuration container with hierarchical precedence
#[ derive( Debug, Clone ) ]
pub struct Config
{
  values: HashMap<String, String>,
}

impl Config
{
  /// Create new configuration with defaults only
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  pub fn new() -> Self
  {
    Self::builder()
      .with_iron_config()
      .build()
  }

  /// Create configuration from CLI arguments
  pub fn with_cli_args(cli_args: HashMap<String, String>) -> Self
  {
    Self::builder()
      .with_iron_config()
      .with_cli_args( cli_args )
      .build()
  }

  /// Create configuration from environment variables
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  pub fn from_env() -> Self
  {
    Self::builder()
      .with_iron_config()
      .build()
  }

  /// Create configuration builder
  pub fn builder() -> ConfigBuilder
  {
    ConfigBuilder::new()
  }

  /// Get configuration value by key
  pub fn get(&self, key: &str) -> Option<String>
  {
    self.values.get( key ).cloned()
  }

  /// Get configuration value with fallback default
  pub fn get_or(&self, key: &str, default: &str) -> String
  {
    self.values.get( key )
      .cloned()
      .unwrap_or_else( || default.to_string() )
  }

  /// Get all configuration values
  pub fn all(&self) -> &HashMap<String, String>
  {
    &self.values
  }

  /// Get default configuration as TOML string
  fn get_defaults_toml() -> String
  {
    r#"
api_url = "https://api.ironcage.ai"
format = "table"
"#.to_string()
  }
}

impl Default for Config
{
  fn default() -> Self
  {
    Self::new()
  }
}

/// Builder for creating configuration with multiple sources
pub struct ConfigBuilder
{
  values: HashMap<String, String>,
  validate: bool,
}

impl ConfigBuilder
{
  /// Create new configuration builder
  pub fn new() -> Self
  {
    Self {
      values: HashMap::new(),
      validate: false,
    }
  }

  /// Load configuration from `iron_config_loader` with 5-layer precedence
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  pub fn with_iron_config(mut self) -> Self
  {
    let defaults = Config::get_defaults_toml();

    // Create ConfigLoader with defaults
    let loader = ConfigLoader::with_defaults( "iron_cli", &defaults )
      .expect( "Failed to create ConfigLoader" );

    // Load all config keys (flat structure)
    // Environment variables follow IRON_CLI_* pattern (e.g., IRON_CLI_API_URL)
    let config_keys = [ "api_url", "format", "user", "token" ];

    for key in &config_keys
    {
      if let Ok( value ) = loader.get::< String >( key )
      {
        self.values.insert( key.to_string(), value );
      }
    }

    self
  }

  /// Add CLI arguments (highest priority, overrides iron_config_loader)
  pub fn with_cli_args(mut self, cli_args: HashMap<String, String>) -> Self
  {
    // CLI args override everything, including iron_config
    self.values.extend( cli_args );
    self
  }

  /// Enable validation of configuration values
  pub fn validate(mut self) -> Self
  {
    self.validate = true;
    self
  }

  /// Build configuration (panics on validation error)
  pub fn build(self) -> Config
  {
    if self.validate
    {
      self.validate_values().expect( "LOUD FAILURE: Configuration validation failed" );
    }

    Config {
      values: self.values,
    }
  }

  /// Build configuration returning Result
  pub fn build_result(self) -> Result<Config, ConfigError>
  {
    if self.validate
    {
      self.validate_values()?;
    }

    Ok( Config {
      values: self.values,
    })
  }

  /// Validate configuration values
  fn validate_values(&self) -> Result<(), ConfigError>
  {
    // Validate format value if present
    if let Some( format ) = self.values.get( "format" )
    {
      let valid_formats = [ "table", "expanded", "json", "yaml" ];

      if !valid_formats.contains( &format.as_str() )
      {
        return Err( ConfigError::InvalidValue(
          format!( "Invalid format '{}'. Must be one of: {}", format, valid_formats.join( ", " ) )
        ));
      }
    }

    Ok( () )
  }
}

impl Default for ConfigBuilder
{
  fn default() -> Self
  {
    Self::new()
  }
}

