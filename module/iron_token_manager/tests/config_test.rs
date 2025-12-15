#![allow(missing_docs)]

use iron_token_manager::*;

#[ test ]
fn test_default_dev_config()
{
  let config = Config::default_dev();
  // Database URL should be workspace-relative (absolute path) or local fallback
  assert!(
    config.database.url.contains( "iron.db" ) && config.database.url.contains( "?mode=rwc" ),
    "Default dev config should use iron.db with mode=rwc, got: {}",
    config.database.url
  );
  assert_eq!( config.database.max_connections, 5, "Default dev config should limit connections to 5 for SQLite" );
  assert!( config.database.auto_migrate, "Default dev config should have auto_migrate enabled for convenience" );
  assert!( config.database.foreign_keys, "Default dev config should have foreign_keys enabled for data integrity" );
  assert!( config.development.is_some(), "Default dev config should include development-specific settings" );
}

#[ test ]
fn test_default_test_config()
{
  let config = Config::default_test();
  assert_eq!( config.database.url, "sqlite:///:memory:?mode=rwc", "Default test config should use in-memory SQLite database for isolation" );
  assert_eq!( config.database.max_connections, 5, "Default test config should limit connections to 5 for SQLite" );
  assert!( config.database.auto_migrate, "Default test config should have auto_migrate enabled for test setup" );
  assert!( config.test.is_some(), "Default test config should include test-specific settings" );
}

#[ test ]
fn test_load_config()
{
  // Test loading config using default environment
  let config = Config::load();
  assert!( config.is_ok(), "Should load config" );

  let config = config.unwrap();
  assert!( config.database.url.contains( "iron.db" ), "Config should specify iron.db database path" );
  assert_eq!( config.database.max_connections, 5, "Config should specify 5 max connections" );
}

#[ test ]
fn test_env_override()
{
  // Use iron_config standard environment variable naming: IRON_TOKEN_MANAGER_*
  std::env::set_var( "IRON_TOKEN_MANAGER_DATABASE_URL", "sqlite:///override.db?mode=rwc" );
  std::env::set_var( "IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS", "10" );

  // Load config - iron_config automatically applies environment variable overrides
  let config = Config::load().expect( "Should load config with env overrides" );

  assert_eq!( config.database.url, "sqlite:///override.db?mode=rwc", "IRON_TOKEN_MANAGER_DATABASE_URL environment variable should override database URL" );
  assert_eq!( config.database.max_connections, 10, "IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS environment variable should override max_connections" );

  // Cleanup
  std::env::remove_var( "IRON_TOKEN_MANAGER_DATABASE_URL" );
  std::env::remove_var( "IRON_TOKEN_MANAGER_DATABASE_MAX_CONNECTIONS" );
}

#[ test ]
fn test_from_env()
{
  // Test loading with explicit environment name
  let config = Config::from_env( "development" );
  assert!( config.is_ok(), "Should load config from environment" );
}
