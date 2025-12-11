//! Configuration management for token manager
//!
//! This module provides configuration loading from TOML files with environment
//! variable overrides. Supports development, test, and production configurations.
//!
//! # Configuration Files
//!
//! - `config.dev.toml` - Development configuration (default)
//! - `config.test.toml` - Test configuration
//! - `config.prod.toml` - Production configuration (gitignored)
//!
//! # Environment Variable Overrides
//!
//! All config values can be overridden via environment variables:
//! - `DATABASE_URL` - Override `database.url`
//! - `DATABASE_MAX_CONNECTIONS` - Override `database.max_connections`
//! - `DATABASE_AUTO_MIGRATE` - Override `database.auto_migrate`
//!
//! # Examples
//!
//! ```rust,ignore
//! use iron_token_manager::config::Config;
//!
//! // Load development config (default)
//! let config = Config::load()?;
//!
//! // Load specific environment
//! let config = Config::from_env("production")?;
//!
//! // Load from custom path
//! let config = Config::from_file("./my-config.toml")?;
//! ```

use serde::{ Deserialize, Serialize };
use std::path::Path;

/// Complete configuration for token manager
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct Config
{
  /// Database configuration
  pub database: DatabaseConfig,

  /// Optional development settings
  #[ serde( default ) ]
  pub development: Option< DevelopmentConfig >,

  /// Optional production settings
  #[ serde( default ) ]
  pub production: Option< ProductionConfig >,

  /// Optional test settings
  #[ serde( default ) ]
  pub test: Option< TestConfig >,
}

/// Database connection and behavior configuration
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct DatabaseConfig
{
  /// Database URL (`SQLite` format: `<sqlite:///path/to/db.db?mode=rwc>`)
  pub url: String,

  /// Maximum number of concurrent connections
  #[ serde( default = "default_max_connections" ) ]
  pub max_connections: u32,

  /// Automatically apply migrations on startup
  #[ serde( default = "default_auto_migrate" ) ]
  pub auto_migrate: bool,

  /// Enable foreign key constraints
  #[ serde( default = "default_foreign_keys" ) ]
  pub foreign_keys: bool,
}

/// Development-specific configuration
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct DevelopmentConfig
{
  /// Enable debug logging
  #[ serde( default ) ]
  pub debug: bool,

  /// Automatically seed test data on first run
  #[ serde( default ) ]
  pub auto_seed: bool,

  /// Wipe database and re-seed on every startup (DESTRUCTIVE!)
  ///
  /// When enabled, completely wipes all data from database and re-seeds
  /// with fresh test data on every initialization. Useful for manual testing
  /// to ensure clean state. NEVER enable in production!
  #[ serde( default ) ]
  pub wipe_and_seed: bool,
}

/// Production-specific configuration
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct ProductionConfig
{
  /// Enable debug logging (should be false in production)
  #[ serde( default ) ]
  pub debug: bool,

  /// Never auto-seed in production
  #[ serde( default ) ]
  pub auto_seed: bool,
}

/// Test-specific configuration
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ allow( clippy::struct_excessive_bools ) ]
pub struct TestConfig
{
  /// Use in-memory databases for tests
  #[ serde( default = "default_true" ) ]
  pub use_memory: bool,

  /// Enable debug logging in tests
  #[ serde( default ) ]
  pub debug: bool,

  /// Never auto-seed in tests
  #[ serde( default ) ]
  pub auto_seed: bool,

  /// Wipe database and re-seed on every startup (DESTRUCTIVE!)
  ///
  /// When enabled, completely wipes all data from database and re-seeds
  /// with fresh test data on every initialization. Useful for manual testing
  /// to ensure clean state.
  #[ serde( default ) ]
  pub wipe_and_seed: bool,
}

// Default value functions

fn default_max_connections() -> u32
{
  5
}

fn default_auto_migrate() -> bool
{
  true
}

fn default_foreign_keys() -> bool
{
  true
}

fn default_true() -> bool
{
  true
}

impl Config
{
  /// Load configuration from default environment
  ///
  /// Determines environment from `IRON_ENV` variable (defaults to "development").
  /// Looks for config file: `config.{env}.toml`
  ///
  /// # Errors
  ///
  /// Returns error if config file not found or invalid format.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// let config = Config::load()?;
  /// println!("Database URL: {}", config.database.url);
  /// ```
  pub fn load() -> crate::error::Result< Self >
  {
    let env = std::env::var( "IRON_ENV" ).unwrap_or_else( |_| "development".to_string() );
    Self::from_env( &env )
  }

  /// Load configuration from specific environment
  ///
  /// # Arguments
  ///
  /// * `env` - Environment name ("development", "test", "production")
  ///
  /// # Errors
  ///
  /// Returns error if config file not found or invalid format.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// let config = Config::from_env("production")?;
  /// ```
  pub fn from_env( env: &str ) -> crate::error::Result< Self >
  {
    let config_file = format!( "config.{env}.toml" );
    Self::from_file( &config_file )
  }

  /// Load configuration from specific file
  ///
  /// # Arguments
  ///
  /// * `path` - Path to config file
  ///
  /// # Errors
  ///
  /// Returns error if file not found or invalid TOML format.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// let config = Config::from_file("./my-config.toml")?;
  /// ```
  pub fn from_file( path: &str ) -> crate::error::Result< Self >
  {
    let config_path = Path::new( path );

    if !config_path.exists()
    {
      return Err( crate::error::TokenError );
    }

    let contents = std::fs::read_to_string( config_path )
      .map_err( |_| crate::error::TokenError )?;

    let mut config: Config = toml::from_str( &contents )
      .map_err( |_| crate::error::TokenError )?;

    // Apply environment variable overrides
    config.apply_env_overrides();

    Ok( config )
  }

  /// Apply environment variable overrides to configuration
  ///
  /// Checks for environment variables and overrides config values if present:
  /// - `DATABASE_URL` → `database.url`
  /// - `DATABASE_MAX_CONNECTIONS` → `database.max_connections`
  /// - `DATABASE_AUTO_MIGRATE` → `database.auto_migrate`
  /// - `DATABASE_FOREIGN_KEYS` → `database.foreign_keys`
  fn apply_env_overrides( &mut self )
  {
    if let Ok( url ) = std::env::var( "DATABASE_URL" )
    {
      self.database.url = url;
    }

    if let Ok( max_conn ) = std::env::var( "DATABASE_MAX_CONNECTIONS" )
    {
      if let Ok( value ) = max_conn.parse::< u32 >()
      {
        self.database.max_connections = value;
      }
    }

    if let Ok( auto_migrate ) = std::env::var( "DATABASE_AUTO_MIGRATE" )
    {
      if let Ok( value ) = auto_migrate.parse::< bool >()
      {
        self.database.auto_migrate = value;
      }
    }

    if let Ok( foreign_keys ) = std::env::var( "DATABASE_FOREIGN_KEYS" )
    {
      if let Ok( value ) = foreign_keys.parse::< bool >()
      {
        self.database.foreign_keys = value;
      }
    }
  }

  /// Create a default development configuration
  ///
  /// Useful for testing and examples.
  #[ must_use ]
  pub fn default_dev() -> Self
  {
    Self
    {
      database: DatabaseConfig
      {
        url: "sqlite:///./dev_tokens.db?mode=rwc".to_string(),
        max_connections: 5,
        auto_migrate: true,
        foreign_keys: true,
      },
      development: Some( DevelopmentConfig
      {
        debug: true,
        auto_seed: false,
        wipe_and_seed: false,
      } ),
      production: None,
      test: None,
    }
  }

  /// Create a default test configuration
  ///
  /// Useful for testing.
  #[ must_use ]
  pub fn default_test() -> Self
  {
    Self
    {
      database: DatabaseConfig
      {
        url: "sqlite:///:memory:?mode=rwc".to_string(),
        max_connections: 5,
        auto_migrate: true,
        foreign_keys: true,
      },
      development: None,
      production: None,
      test: Some( TestConfig
      {
        use_memory: true,
        debug: false,
        auto_seed: false,
        wipe_and_seed: false,
      } ),
    }
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;

  #[ test ]
  fn test_default_dev_config()
  {
    let config = Config::default_dev();
    assert_eq!( config.database.url, "sqlite:///./dev_tokens.db?mode=rwc" );
    assert_eq!( config.database.max_connections, 5 );
    assert!( config.database.auto_migrate );
    assert!( config.database.foreign_keys );
    assert!( config.development.is_some() );
  }

  #[ test ]
  fn test_default_test_config()
  {
    let config = Config::default_test();
    assert_eq!( config.database.url, "sqlite:///:memory:?mode=rwc" );
    assert_eq!( config.database.max_connections, 5 );
    assert!( config.database.auto_migrate );
    assert!( config.test.is_some() );
  }

  #[ test ]
  fn test_load_dev_config_file()
  {
    let config = Config::from_file( "config.dev.toml" );
    assert!( config.is_ok(), "Should load dev config file" );

    let config = config.unwrap();
    assert!( config.database.url.contains( "dev_tokens.db" ) );
    assert_eq!( config.database.max_connections, 5 );
  }

  #[ test ]
  fn test_env_override()
  {
    std::env::set_var( "DATABASE_URL", "sqlite:///override.db?mode=rwc" );
    std::env::set_var( "DATABASE_MAX_CONNECTIONS", "10" );

    let mut config = Config::default_dev();
    config.apply_env_overrides();

    assert_eq!( config.database.url, "sqlite:///override.db?mode=rwc" );
    assert_eq!( config.database.max_connections, 10 );

    // Cleanup
    std::env::remove_var( "DATABASE_URL" );
    std::env::remove_var( "DATABASE_MAX_CONNECTIONS" );
  }

  #[ test ]
  fn test_missing_config_file()
  {
    let config = Config::from_file( "nonexistent.toml" );
    assert!( config.is_err(), "Should error on missing config file" );
  }
}
