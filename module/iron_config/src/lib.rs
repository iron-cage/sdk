//! Unified configuration management for Iron Runtime
//!
//! Provides a precedence-based configuration system that supports multiple
//! configuration sources with clear priority ordering.
//!
//! # Configuration Precedence
//!
//! Configuration is loaded from multiple layers with the following precedence
//! (highest to lowest):
//!
//! 1. **Environment Variables** - `{MODULE}_KEY_NAME` format
//! 2. **Project Config** - `{workspace}/config/{module}.{env}.toml`
//! 3. **User Config** - `~/.config/iron/{module}.toml`
//! 4. **Workspace Defaults** - `{workspace}/config/{module}.default.toml`
//! 5. **Crate Defaults** - Hardcoded defaults in crate
//!
//! Higher priority sources override lower priority sources.
//!
//! # Examples
//!
//! ```rust,ignore
//! use iron_config::ConfigLoader;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct DatabaseConfig {
//!   url: String,
//!   max_connections: u32,
//! }
//!
//! // Load configuration for module
//! let loader = ConfigLoader::new("iron_token_manager")?;
//!
//! // Get individual values
//! let url: String = loader.get("database.url")?;
//!
//! // Get configuration section as struct
//! let db_config: DatabaseConfig = loader.get_section("database")?;
//!
//! // Get value with source information
//! let (url, source) = loader.get_with_source::<String>("database.url")?;
//! println!("Database URL: {} (from {})", url, source);
//! ```
//!
//! # Environment Variable Overrides
//!
//! Any configuration value can be overridden via environment variables.
//! The variable name is constructed as: `{MODULE}_{KEY_PATH}`.
//!
//! Examples:
//! - `IRON_TOKEN_MANAGER_DATABASE_URL` → `database.url`
//! - `IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS` → `database.max_connections`
//!
//! # Configuration Files
//!
//! Configuration files use TOML format:
//!
//! ```toml
//! [database]
//! url = "sqlite:///./iron.db?mode=rwc"
//! max_connections = 5
//!
//! [development]
//! debug = true
//! ```

#![ warn( missing_docs ) ]

pub mod error;
pub mod layer;
pub mod loader;

// Re-exports
pub use error::{ ConfigError, Result };
pub use layer::{ ConfigLayer, ConfigValue, EnvLayer, LayersBuilder };
pub use loader::ConfigLoader;
