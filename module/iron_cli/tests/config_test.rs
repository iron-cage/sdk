//! Configuration system tests
//!
//! ## Test Coverage
//!
//! Tests configuration hierarchy and precedence:
//! 1. CLI arguments (highest priority)
//! 2. Environment variables (IRON_*)
//! 3. Local temp config (.iron.local.tmp.toml)
//! 4. Local project config (.iron.local.toml)
//! 5. Global user config (~/.config/iron-token/config.toml)
//! 6. Built-in defaults (lowest priority)
//!
//! ## Testing Strategy
//!
//! - Test each configuration source individually
//! - Test precedence order (higher priority overrides lower)
//! - Test missing configuration (falls back to defaults)
//! - Test invalid configuration (returns error)

use std::collections::HashMap;
use std::ffi::OsString;
use std::sync::{ Mutex, MutexGuard };

static ENV_LOCK: Mutex<()> = Mutex::new( () );

struct EnvSnapshot
{
  api_url: Option< OsString >,
  format: Option< OsString >,
  user: Option< OsString >,
  token: Option< OsString >,
}

impl Default for EnvSnapshot
{
  fn default() -> Self
  {
    Self {
      api_url: None,
      format: None,
      user: None,
      token: None,
    }
  }
}

impl EnvSnapshot
{
  fn capture() -> Self
  {
    Self {
      api_url: std::env::var_os( "IRON_CLI_API_URL" ),
      format: std::env::var_os( "IRON_CLI_FORMAT" ),
      user: std::env::var_os( "IRON_CLI_USER" ),
      token: std::env::var_os( "IRON_CLI_TOKEN" ),
    }
  }

  fn clear()
  {
    std::env::remove_var( "IRON_CLI_API_URL" );
    std::env::remove_var( "IRON_CLI_FORMAT" );
    std::env::remove_var( "IRON_CLI_USER" );
    std::env::remove_var( "IRON_CLI_TOKEN" );
  }

  fn restore(self)
  {
    EnvSnapshot::restore_one( "IRON_CLI_API_URL", self.api_url );
    EnvSnapshot::restore_one( "IRON_CLI_FORMAT", self.format );
    EnvSnapshot::restore_one( "IRON_CLI_USER", self.user );
    EnvSnapshot::restore_one( "IRON_CLI_TOKEN", self.token );
  }

  fn restore_one(key: &'static str, value: Option< OsString >)
  {
    match value
    {
      Some( v ) => std::env::set_var( key, v ),
      None => std::env::remove_var( key ),
    }
  }
}

struct EnvTestGuard
{
  _lock: MutexGuard<'static, ()>,
  snapshot: EnvSnapshot,
}

impl EnvTestGuard
{
  fn new() -> Self
  {
    let lock = ENV_LOCK.lock().unwrap();
    let snapshot = EnvSnapshot::capture();
    EnvSnapshot::clear();

    Self {
      _lock: lock,
      snapshot,
    }
  }
}

impl Drop for EnvTestGuard
{
  fn drop(&mut self)
  {
    std::mem::take( &mut self.snapshot ).restore();
  }
}

// ============================================================================
// Configuration Loading Tests
// ============================================================================

#[ test ]
fn test_config_loads_defaults()
{
  let _env = EnvTestGuard::new();

  let config = iron_cli::config::Config::new();

  assert_eq!( config.get( "api_url" ), Some( "https://api.ironcage.ai".to_string() ) );
  assert_eq!( config.get( "format" ), Some( "table".to_string() ) );
}

#[ test ]
fn test_config_loads_from_cli_args()
{
  let _env = EnvTestGuard::new();

  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );

  let config = iron_cli::config::Config::with_cli_args( cli_args );

  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );
}

#[ test ]
fn test_config_loads_from_env()
{
  let _env = EnvTestGuard::new();

  std::env::set_var( "IRON_CLI_API_URL", "https://custom.api.url" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "api_url" ), Some( "https://custom.api.url".to_string() ) );
}

#[ test ]
fn test_config_missing_key_returns_none()
{
  let _env = EnvTestGuard::new();

  let config = iron_cli::config::Config::new();

  assert_eq!( config.get( "nonexistent_key" ), None );
}

#[ test ]
fn test_config_get_with_default()
{
  let _env = EnvTestGuard::new();

  let config = iron_cli::config::Config::new();

  assert_eq!(
    config.get_or( "nonexistent", "fallback" ),
    "fallback".to_string()
  );
}

// ============================================================================
// Configuration Precedence Tests
// ============================================================================

#[ test ]
fn test_cli_args_override_env_vars()
{
  let _env = EnvTestGuard::new();

  std::env::set_var( "IRON_CLI_FORMAT", "yaml" );

  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );

  let config = iron_cli::config::Config::builder()
    .with_iron_config()
    .with_cli_args( cli_args )
    .build();

  // CLI args should win over env vars
  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );
}

#[ test ]
fn test_env_vars_override_defaults()
{
  let _env = EnvTestGuard::new();

  std::env::set_var( "IRON_CLI_API_URL", "https://override.url" );

  let config = iron_cli::config::Config::builder()
    .with_iron_config()
    .build();

  // Env var should override default
  assert_eq!( config.get( "api_url" ), Some( "https://override.url".to_string() ) );
}

#[ test ]
fn test_defaults_used_when_no_overrides()
{
  let _env = EnvTestGuard::new();

  let config = iron_cli::config::Config::new();

  assert_eq!( config.get( "format" ), Some( "table".to_string() ) );
}

// ============================================================================
// Configuration Validation Tests
// ============================================================================

#[ test ]
fn test_config_validates_format_values()
{
  let _env = EnvTestGuard::new();

  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "invalid_format".to_string() );

  let result = iron_cli::config::Config::builder()
    .with_cli_args( cli_args )
    .validate()
    .build_result();

  assert!( result.is_err(), "Should reject invalid format value" );
}

#[ test ]
fn test_config_accepts_valid_format_values()
{
  let _env = EnvTestGuard::new();

  let formats = vec![ "table", "expanded", "json", "yaml" ];

  for format in formats
  {
    let mut cli_args = HashMap::new();
    cli_args.insert( "format".to_string(), format.to_string() );

    let result = iron_cli::config::Config::builder()
      .with_cli_args( cli_args )
      .validate()
      .build_result();

    assert!( result.is_ok(), "Should accept format: {}", format );
  }
}

// ============================================================================
// Environment Variable Mapping Tests
// ============================================================================

#[ test ]
fn test_env_var_name_mapping()
{
  let _env = EnvTestGuard::new();

  // Test that IRON_CLI_API_URL maps to "api_url" config key
  std::env::set_var( "IRON_CLI_API_URL", "https://test.com" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "api_url" ), Some( "https://test.com".to_string() ) );
}

#[ test ]
fn test_env_var_format_mapping()
{
  let _env = EnvTestGuard::new();

  std::env::set_var( "IRON_CLI_FORMAT", "json" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );
}

#[ test ]
fn test_env_var_user_mapping()
{
  let _env = EnvTestGuard::new();

  std::env::set_var( "IRON_CLI_USER", "testuser" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "user" ), Some( "testuser".to_string() ) );
}
