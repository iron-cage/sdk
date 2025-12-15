//! Configuration precedence tests
//!
//! Verifies the 5-layer precedence system works correctly:
//! 1. Environment variables (highest)
//! 2. Project config
//! 3. User config
//! 4. Workspace defaults
//! 5. Crate defaults (lowest)

use iron_config::ConfigLoader;
use std::env;

#[ test ]
fn test_env_layer_priority()
{
  // Set environment variable
  env::set_var( "IRON_TEST_PRIORITY_DATABASE_URL", "sqlite://env-override.db" );

  // Create loader with defaults
  let defaults = r#"
[database]
url = "sqlite://default.db"
max_connections = 5
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_priority", defaults )
    .expect( "Failed to create loader" );

  // Environment variable should override default
  let url: String = loader.get( "database.url" )
    .expect( "Failed to get database.url" );

  assert_eq!( url, "sqlite://env-override.db", "Environment variable should override default" );

  // Non-overridden value should use default
  let max_conn: u32 = loader.get( "database.max_connections" )
    .expect( "Failed to get max_connections" );

  assert_eq!( max_conn, 5, "Non-overridden value should use default" );

  // Cleanup
  env::remove_var( "IRON_TEST_PRIORITY_DATABASE_URL" );
}

#[ test ]
fn test_env_layer_key_conversion()
{
  // Test key conversion: database.url → IRON_TEST_DATABASE_URL
  env::set_var( "IRON_TEST_KEYCONV_DATABASE_MAX_CONNECTIONS", "10" );

  let defaults = r#"
[database]
max_connections = 5
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_keyconv", defaults )
    .expect( "Failed to create loader" );

  let max_conn: u32 = loader.get( "database.max_connections" )
    .expect( "Failed to get max_connections" );

  assert_eq!( max_conn, 10, "Environment variable should override with correct key conversion" );

  // Cleanup
  env::remove_var( "IRON_TEST_KEYCONV_DATABASE_MAX_CONNECTIONS" );
}

#[ test ]
fn test_missing_key_error()
{
  let defaults = r#"
[database]
url = "sqlite://default.db"
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_missing", defaults )
    .expect( "Failed to create loader" );

  // Accessing missing key should return error
  let result = loader.get::< String >( "nonexistent.key" );

  assert!( result.is_err(), "Should error on missing key" );
  assert!( result.unwrap_err().to_string().contains( "Missing required configuration key" ), "Error should indicate missing key" );
}

#[ test ]
fn test_get_optional()
{
  let defaults = r#"
[database]
url = "sqlite://default.db"
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_optional", defaults )
    .expect( "Failed to create loader" );

  // Existing key
  let url: Option< String > = loader.get_opt( "database.url" )
    .expect( "Failed to get optional value" );
  assert_eq!( url, Some( "sqlite://default.db".to_string() ) );

  // Missing key
  let missing: Option< String > = loader.get_opt( "nonexistent.key" )
    .expect( "Failed to get optional value" );
  assert_eq!( missing, None, "Optional get should return None for missing key" );
}

#[ test ]
fn test_get_section()
{
  use serde::Deserialize;

  #[ derive( Debug, Deserialize, PartialEq ) ]
  struct DatabaseConfig
  {
    url: String,
    max_connections: u32,
  }

  let defaults = r#"
[database]
url = "sqlite://default.db"
max_connections = 5
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_section", defaults )
    .expect( "Failed to create loader" );

  let db_config: DatabaseConfig = loader.get_section( "database" )
    .expect( "Failed to get database section" );

  assert_eq!( db_config.url, "sqlite://default.db" );
  assert_eq!( db_config.max_connections, 5 );
}

#[ test ]
fn test_nested_configuration()
{
  let defaults = r#"
[database]
url = "sqlite://default.db"

[database.pool]
min_connections = 1
max_connections = 5
timeout = 30
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_nested", defaults )
    .expect( "Failed to create loader" );

  // Access nested values
  let min_conn: u32 = loader.get( "database.pool.min_connections" )
    .expect( "Failed to get nested value" );
  assert_eq!( min_conn, 1 );

  let timeout: u32 = loader.get( "database.pool.timeout" )
    .expect( "Failed to get nested value" );
  assert_eq!( timeout, 30 );
}

#[ test ]
fn test_env_override_nested()
{
  // Override nested value with environment variable
  env::set_var( "IRON_TEST_ENVNEST_DATABASE_POOL_MAX_CONNECTIONS", "10" );

  let defaults = r#"
[database.pool]
max_connections = 5
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_envnest", defaults )
    .expect( "Failed to create loader" );

  let max_conn: u32 = loader.get( "database.pool.max_connections" )
    .expect( "Failed to get nested value" );

  assert_eq!( max_conn, 10, "Environment variable should override nested value" );

  // Cleanup
  env::remove_var( "IRON_TEST_ENVNEST_DATABASE_POOL_MAX_CONNECTIONS" );
}

#[ test ]
fn test_get_with_source()
{
  let defaults = r#"
[database]
url = "sqlite://default.db"
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_source", defaults )
    .expect( "Failed to create loader" );

  let ( url, source ) = loader.get_with_source::< String >( "database.url" )
    .expect( "Failed to get value with source" );

  assert_eq!( url, "sqlite://default.db" );
  assert!( source.contains( "Crate Defaults" ), "Source should indicate crate defaults" );
}

#[ test ]
fn test_debug_summary()
{
  let defaults = r#"
[database]
url = "sqlite://default.db"
max_connections = 5
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_summary", defaults )
    .expect( "Failed to create loader" );

  let summary = loader.debug_summary();

  assert!( summary.contains( "iron_test_summary" ), "Summary should contain module name" );
  assert!( summary.contains( "database.url" ), "Summary should list keys" );
  assert!( summary.contains( "source:" ), "Summary should include source information" );
}

#[ test ]
fn test_multiple_env_overrides()
{
  // Set multiple environment variables
  env::set_var( "IRON_TEST_MULTI_DATABASE_URL", "sqlite://env.db" );
  env::set_var( "IRON_TEST_MULTI_DATABASE_MAX_CONNECTIONS", "10" );
  env::set_var( "IRON_TEST_MULTI_DEVELOPMENT_DEBUG", "true" );

  let defaults = r#"
[database]
url = "sqlite://default.db"
max_connections = 5

[development]
debug = false
"#;

  let loader = ConfigLoader::with_defaults( "iron_test_multi", defaults )
    .expect( "Failed to create loader" );

  // All environment variables should override
  let url: String = loader.get( "database.url" ).unwrap();
  assert_eq!( url, "sqlite://env.db" );

  let max_conn: u32 = loader.get( "database.max_connections" ).unwrap();
  assert_eq!( max_conn, 10 );

  let debug: bool = loader.get( "development.debug" ).unwrap();
  assert!( debug );

  // Cleanup
  env::remove_var( "IRON_TEST_MULTI_DATABASE_URL" );
  env::remove_var( "IRON_TEST_MULTI_DATABASE_MAX_CONNECTIONS" );
  env::remove_var( "IRON_TEST_MULTI_DEVELOPMENT_DEBUG" );
}
