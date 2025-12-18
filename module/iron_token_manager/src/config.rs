//! Configuration management for token manager
//!
//! This module provides unified configuration loading using `iron_config` with
//! environment variable overrides. Supports development, test, and production configurations.
//!
//! # Configuration Layers (Precedence: highest to lowest)
//!
//! 1. Environment Variables - `IRON_TOKEN_MANAGER_{KEY}` format (highest priority)
//! 2. Project Config - `{workspace}/config/iron_token_manager.{env}.toml`
//! 3. User Config - `~/.config/iron/iron_token_manager.toml`
//! 4. Workspace Defaults - `{workspace}/config/iron_token_manager.default.toml`
//! 5. Crate Defaults - Hardcoded defaults (lowest priority)
//!
//! # Environment Variable Overrides
//!
//! All config values can be overridden via environment variables:
//! - `IRON_TOKEN_MANAGER_DATABASE_URL` - Override `database.url`
//! - `IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS` - Override `database.max_connections`
//! - `IRON_TOKEN_MANAGER_DATABASE_AUTO_MIGRATE` - Override `database.auto_migrate`
//! - `IRON_TOKEN_MANAGER_DATABASE_FOREIGN_KEYS` - Override `database.foreign_keys`
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
//! ```

use serde::{ Deserialize, Serialize };
use iron_config_loader::ConfigLoader;

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
  /// Uses `iron_config`'s 5-layer precedence system.
  ///
  /// # Errors
  ///
  /// Returns error if configuration cannot be loaded.
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
  /// Uses `iron_config`'s `ConfigLoader` with 5-layer precedence:
  /// 1. Environment variables (`IRON_TOKEN_MANAGER_*`)
  /// 2. Project config (`{workspace}/config/iron_token_manager.{env}.toml`)
  /// 3. User config (`~/.config/iron/iron_token_manager.toml`)
  /// 4. Workspace defaults (`{workspace}/config/iron_token_manager.default.toml`)
  /// 5. Crate defaults (hardcoded)
  ///
  /// # Arguments
  ///
  /// * `env` - Environment name ("development", "test", "production")
  ///
  /// # Errors
  ///
  /// Returns error if configuration cannot be loaded.
  ///
  /// # Examples
  ///
  /// ```rust,ignore
  /// let config = Config::from_env("production")?;
  /// ```
  pub fn from_env( _env: &str ) -> crate::error::Result< Self >
  {
    // Get default configuration as TOML
    let defaults = Self::get_defaults_toml();

    // Use iron_config's ConfigLoader with defaults
    // Note: iron_config automatically detects IRON_ENV for environment-specific configs
    let loader = ConfigLoader::with_defaults( "iron_token_manager", &defaults )
      .map_err( |_| crate::error::TokenError::Generic )?;

    // Load configuration from the loader
    Self::from_loader( &loader )
  }

  /// Helper to load configuration from `ConfigLoader`
  fn from_loader( loader: &ConfigLoader ) -> crate::error::Result< Self >
  {
    // Load database section
    let database: DatabaseConfig = loader.get_section( "database" )
      .map_err( |_| crate::error::TokenError::Generic )?;

    // Try to load optional environment-specific sections
    // These may not exist depending on the config, so we use Ok() as fallback
    let development = loader.get_section::< DevelopmentConfig >( "development" ).ok();
    let production = loader.get_section::< ProductionConfig >( "production" ).ok();
    let test = loader.get_section::< TestConfig >( "test" ).ok();

    Ok( Self
    {
      database,
      development,
      production,
      test,
    } )
  }

  /// Get default configuration as TOML string
  fn get_defaults_toml() -> String
  {
    r#"
[database]
url = "sqlite:///./iron.db?mode=rwc"
max_connections = 5
auto_migrate = true
foreign_keys = true

[development]
debug = true
auto_seed = false
wipe_and_seed = false
"#.to_string()
  }

  /// Create a default development configuration
  ///
  /// Uses the canonical `./iron.db` database path relative to current directory.
  ///
  /// Useful for testing and examples.
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  #[ must_use ]
  pub fn default_dev() -> Self
  {
    let defaults = Self::get_defaults_toml();

    let loader = ConfigLoader::with_defaults( "iron_token_manager", &defaults )
      .expect( "Failed to create default dev config" );

    Self::from_loader( &loader )
      .expect( "Failed to load default dev config" )
  }

  /// Create a default test configuration
  ///
  /// Useful for testing.
  ///
  /// # Panics
  ///
  /// Panics if `ConfigLoader` creation fails (should never happen with valid defaults).
  #[ must_use ]
  pub fn default_test() -> Self
  {
    let defaults = r#"
[database]
url = "sqlite:///:memory:?mode=rwc"
max_connections = 5
auto_migrate = true
foreign_keys = true

[test]
use_memory = true
debug = false
auto_seed = false
wipe_and_seed = false
"#;

    let loader = ConfigLoader::with_defaults( "iron_token_manager", defaults )
      .expect( "Failed to create default test config" );

    Self::from_loader( &loader )
      .expect( "Failed to load default test config" )
  }
}
