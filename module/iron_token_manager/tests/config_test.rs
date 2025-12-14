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
fn test_load_dev_config_file()
{
  let config = Config::from_file( "config.dev.toml" );
  assert!( config.is_ok(), "Should load dev config file" );

  let config = config.unwrap();
  assert!( config.database.url.contains( "iron.db" ), "Dev config file should specify iron.db database path" );
  assert_eq!( config.database.max_connections, 5, "Dev config file should specify 5 max connections" );
}

#[ test ]
fn test_env_override()
{
  std::env::set_var( "DATABASE_URL", "sqlite:///override.db?mode=rwc" );
  std::env::set_var( "DATABASE_MAX_CONNECTIONS", "10" );

  let mut config = Config::default_dev();
  config.apply_env_overrides();

  assert_eq!( config.database.url, "sqlite:///override.db?mode=rwc", "DATABASE_URL environment variable should override config file database URL" );
  assert_eq!( config.database.max_connections, 10, "DATABASE_MAX_CONNECTIONS environment variable should override config file max_connections" );

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
