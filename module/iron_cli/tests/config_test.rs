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

// ============================================================================
// Configuration Loading Tests
// ============================================================================

#[ test ]
fn test_config_loads_defaults()
{
  let config = iron_cli::config::Config::new();

  assert_eq!( config.get( "api_url" ), Some( "https://api.ironcage.ai".to_string() ) );
  assert_eq!( config.get( "format" ), Some( "table".to_string() ) );
}

#[ test ]
fn test_config_loads_from_cli_args()
{
  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );

  let config = iron_cli::config::Config::with_cli_args( cli_args );

  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );
}

#[ test ]
fn test_config_loads_from_env()
{
  std::env::set_var( "IRON_API_URL", "https://custom.api.url" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "api_url" ), Some( "https://custom.api.url".to_string() ) );

  std::env::remove_var( "IRON_API_URL" );
}

#[ test ]
fn test_config_missing_key_returns_none()
{
  let config = iron_cli::config::Config::new();

  assert_eq!( config.get( "nonexistent_key" ), None );
}

#[ test ]
fn test_config_get_with_default()
{
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
  std::env::set_var( "IRON_FORMAT", "yaml" );

  let mut cli_args = HashMap::new();
  cli_args.insert( "format".to_string(), "json".to_string() );

  let config = iron_cli::config::Config::builder()
    .with_cli_args( cli_args )
    .with_env()
    .build();

  // CLI args should win over env vars
  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );

  std::env::remove_var( "IRON_FORMAT" );
}

#[ test ]
fn test_env_vars_override_defaults()
{
  std::env::set_var( "IRON_API_URL", "https://override.url" );

  let config = iron_cli::config::Config::builder()
    .with_env()
    .with_defaults()
    .build();

  // Env var should override default
  assert_eq!( config.get( "api_url" ), Some( "https://override.url".to_string() ) );

  std::env::remove_var( "IRON_API_URL" );
}

#[ test ]
fn test_defaults_used_when_no_overrides()
{
  let config = iron_cli::config::Config::builder()
    .with_defaults()
    .build();

  assert_eq!( config.get( "format" ), Some( "table".to_string() ) );
}

// ============================================================================
// Configuration Validation Tests
// ============================================================================

#[ test ]
fn test_config_validates_format_values()
{
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
  // Test that IRON_API_URL maps to "api_url" config key
  std::env::set_var( "IRON_API_URL", "https://test.com" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "api_url" ), Some( "https://test.com".to_string() ) );

  std::env::remove_var( "IRON_API_URL" );
}

#[ test ]
fn test_env_var_format_mapping()
{
  std::env::set_var( "IRON_FORMAT", "json" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "format" ), Some( "json".to_string() ) );

  std::env::remove_var( "IRON_FORMAT" );
}

#[ test ]
fn test_env_var_user_mapping()
{
  std::env::set_var( "IRON_USER", "testuser" );

  let config = iron_cli::config::Config::from_env();

  assert_eq!( config.get( "user" ), Some( "testuser".to_string() ) );

  std::env::remove_var( "IRON_USER" );
}
