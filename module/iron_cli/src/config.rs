//! Configuration system with hierarchical precedence
//!
//! ## Configuration Hierarchy (highest to lowest priority)
//!
//! 1. CLI arguments (keyword::value parameters)
//! 2. Environment variables (IRON_*)
//! 3. Local temp config (.iron.local.tmp.toml)
//! 4. Local project config (.iron.local.toml)
//! 5. Global user config (~/.config/iron-token/config.toml)
//! 6. Built-in defaults
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
  pub fn new() -> Self
  {
    Self::builder()
      .with_defaults()
      .build()
  }

  /// Create configuration from CLI arguments
  pub fn with_cli_args(cli_args: HashMap<String, String>) -> Self
  {
    Self::builder()
      .with_cli_args( cli_args )
      .with_defaults()
      .build()
  }

  /// Create configuration from environment variables
  pub fn from_env() -> Self
  {
    Self::builder()
      .with_env()
      .with_defaults()
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

  /// Add CLI arguments (highest priority)
  pub fn with_cli_args(mut self, cli_args: HashMap<String, String>) -> Self
  {
    // CLI args override everything
    self.values.extend( cli_args );
    self
  }

  /// Add environment variables (IRON_* prefix)
  pub fn with_env(mut self) -> Self
  {
    // Map environment variables to config keys
    let env_mappings = vec![
      ( "IRON_API_URL", "api_url" ),
      ( "IRON_FORMAT", "format" ),
      ( "IRON_USER", "user" ),
      ( "IRON_TOKEN", "token" ),
    ];

    for (env_var, config_key) in env_mappings
    {
      if let Ok( value ) = std::env::var( env_var )
      {
        // Only set if not already set by higher priority source
        self.values.entry( config_key.to_string() )
          .or_insert( value );
      }
    }

    self
  }

  /// Add built-in defaults (lowest priority)
  pub fn with_defaults(mut self) -> Self
  {
    let defaults = vec![
      ( "api_url", "https://api.iron.dev" ),
      ( "format", "table" ),
    ];

    for (key, value) in defaults
    {
      // Only set if not already set by higher priority source
      self.values.entry( key.to_string() )
        .or_insert( value.to_string() );
    }

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
      self.validate_values().expect( "Configuration validation failed" );
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

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_config_builder_precedence()
  {
    let mut cli_args = HashMap::new();
    cli_args.insert( "format".to_string(), "json".to_string() );

    std::env::set_var( "IRON_FORMAT", "yaml" );

    let config = Config::builder()
      .with_defaults()
      .with_env()
      .with_cli_args( cli_args )
      .build();

    // CLI should override env
    assert_eq!( config.get( "format" ), Some( "json".to_string() ) );

    std::env::remove_var( "IRON_FORMAT" );
  }

  #[ test ]
  fn test_config_validation_rejects_invalid_format()
  {
    let mut cli_args = HashMap::new();
    cli_args.insert( "format".to_string(), "invalid".to_string() );

    let result = Config::builder()
      .with_cli_args( cli_args )
      .validate()
      .build_result();

    assert!( result.is_err() );
  }

  #[ test ]
  fn test_config_get_or_returns_default()
  {
    let config = Config::new();

    assert_eq!( config.get_or( "missing_key", "default_value" ), "default_value" );
  }
}
