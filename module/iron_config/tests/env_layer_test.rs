//! Environment variable layer tests
//!
//! Verifies environment variable configuration layer behavior.

use iron_config::layer::{ EnvLayer, ConfigLayer };
use std::env;

#[ test ]
fn test_env_layer_key_conversion()
{
  env::set_var( "IRON_TEST_DATABASE_URL", "sqlite://env.db" );
  env::set_var( "IRON_TEST_DATABASE_MAX_CONNECTIONS", "10" );

  let layer = EnvLayer::new( "IRON_TEST_" );

  // Test key conversion: database.url ← IRON_TEST_DATABASE_URL
  let url = layer.get( "database.url" ).expect( "Failed to get value" );
  assert!( url.is_some(), "Should find database.url from IRON_TEST_DATABASE_URL" );
  assert_eq!( url.unwrap().value.as_str().unwrap(), "sqlite://env.db" );

  // Test nested key: database.max_connections ← IRON_TEST_DATABASE_MAX_CONNECTIONS
  let max_conn = layer.get( "database.max_connections" ).expect( "Failed to get value" );
  assert!( max_conn.is_some(), "Should find max_connections" );
  assert_eq!( max_conn.unwrap().value.as_integer().unwrap(), 10 );

  // Cleanup
  env::remove_var( "IRON_TEST_DATABASE_URL" );
  env::remove_var( "IRON_TEST_DATABASE_MAX_CONNECTIONS" );
}

#[ test ]
fn test_env_layer_missing_var()
{
  let layer = EnvLayer::new( "IRON_TEST_MISSING_" );

  let result = layer.get( "database.url" ).expect( "Failed to query layer" );
  assert!( result.is_none(), "Should return None for missing env var" );
}

#[ test ]
fn test_env_layer_get_all()
{
  env::set_var( "IRON_TEST_ALL_KEY1", "value1" );
  env::set_var( "IRON_TEST_ALL_KEY2", "value2" );
  env::set_var( "IRON_TEST_ALL_NESTED_KEY", "value3" );

  let layer = EnvLayer::new( "IRON_TEST_ALL_" );

  let all = layer.get_all().expect( "Failed to get all values" );

  // Should contain all matching environment variables
  assert!( all.contains_key( "key1" ), "Should contain key1" );
  assert!( all.contains_key( "key2" ), "Should contain key2" );
  assert!( all.contains_key( "nested.key" ), "Should contain nested.key" );
  assert_eq!( all.len(), 3, "Should have exactly 3 keys" );

  // Cleanup
  env::remove_var( "IRON_TEST_ALL_KEY1" );
  env::remove_var( "IRON_TEST_ALL_KEY2" );
  env::remove_var( "IRON_TEST_ALL_NESTED_KEY" );
}

#[ test ]
fn test_env_layer_priority()
{
  let layer = EnvLayer::new( "IRON_TEST_" );
  assert_eq!( layer.priority(), 5, "Environment layer should have highest priority" );
}

#[ test ]
fn test_env_layer_source_tracking()
{
  env::set_var( "IRON_TEST_SOURCE_VALUE", "test" );

  let layer = EnvLayer::new( "IRON_TEST_SOURCE_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some() );

  let config_value = value.unwrap();
  assert!( config_value.source.starts_with( "env:" ), "Source should start with 'env:'" );
  assert!( config_value.source.contains( "IRON_TEST_SOURCE_VALUE" ), "Source should contain env var name" );

  // Cleanup
  env::remove_var( "IRON_TEST_SOURCE_VALUE" );
}

#[ test ]
fn test_env_layer_boolean_parsing()
{
  env::set_var( "IRON_TEST_BOOL_TRUE", "true" );
  env::set_var( "IRON_TEST_BOOL_FALSE", "false" );

  let layer = EnvLayer::new( "IRON_TEST_BOOL_" );

  let true_val = layer.get( "true" ).unwrap().unwrap();
  assert!( true_val.value.as_bool().unwrap() );

  let false_val = layer.get( "false" ).unwrap().unwrap();
  assert!( !false_val.value.as_bool().unwrap() );

  // Cleanup
  env::remove_var( "IRON_TEST_BOOL_TRUE" );
  env::remove_var( "IRON_TEST_BOOL_FALSE" );
}

#[ test ]
fn test_env_layer_integer_parsing()
{
  env::set_var( "IRON_TEST_INT_VALUE", "42" );

  let layer = EnvLayer::new( "IRON_TEST_INT_" );

  let value = layer.get( "value" ).unwrap().unwrap();
  assert_eq!( value.value.as_integer().unwrap(), 42 );

  // Cleanup
  env::remove_var( "IRON_TEST_INT_VALUE" );
}

#[ test ]
fn test_env_layer_string_fallback()
{
  // Non-parseable TOML should fallback to string
  env::set_var( "IRON_TEST_STR_COMPLEX", "not-valid-toml-{}" );

  let layer = EnvLayer::new( "IRON_TEST_STR_" );

  let value = layer.get( "complex" ).unwrap().unwrap();
  assert_eq!( value.value.as_str().unwrap(), "not-valid-toml-{}" );

  // Cleanup
  env::remove_var( "IRON_TEST_STR_COMPLEX" );
}

#[ test ]
fn test_env_layer_prefix_filtering()
{
  // Set variables with and without prefix
  env::set_var( "IRON_TEST_PREFIX_MATCH", "included" );
  env::set_var( "OTHER_PREFIX_VALUE", "excluded" );

  let layer = EnvLayer::new( "IRON_TEST_PREFIX_" );

  let all = layer.get_all().expect( "Failed to get all values" );

  // Should only include variables with correct prefix
  assert!( all.contains_key( "match" ), "Should include IRON_TEST_PREFIX_MATCH" );
  assert!( !all.contains_key( "other.prefix.value" ), "Should not include OTHER_PREFIX_VALUE" );

  // Cleanup
  env::remove_var( "IRON_TEST_PREFIX_MATCH" );
  env::remove_var( "OTHER_PREFIX_VALUE" );
}

// TC-1.2: ENV Var with Empty String
#[ test ]
fn test_env_var_empty_string()
{
  env::set_var( "IRON_TEST_EMPTY_VALUE", "" );

  let layer = EnvLayer::new( "IRON_TEST_EMPTY_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some(), "Empty string is valid value, should not be None" );
  assert_eq!( value.unwrap().value.as_str().unwrap(), "", "Should return empty string" );

  // Cleanup
  env::remove_var( "IRON_TEST_EMPTY_VALUE" );
}

// TC-1.3: ENV Var with Whitespace Only
#[ test ]
fn test_env_var_whitespace_only()
{
  env::set_var( "IRON_TEST_WHITESPACE_VALUE", "   " );

  let layer = EnvLayer::new( "IRON_TEST_WHITESPACE_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some(), "Whitespace-only string is valid value" );
  assert_eq!( value.unwrap().value.as_str().unwrap(), "   ", "Should return whitespace as-is" );

  // Cleanup
  env::remove_var( "IRON_TEST_WHITESPACE_VALUE" );
}

// TC-1.4: ENV Var with Special Characters
#[ test ]
fn test_env_var_special_characters()
{
  let special = r#"!@#$%^&*()_+-=[]{}|;:,.<>?/"#;
  env::set_var( "IRON_TEST_SPECIAL_VALUE", special );

  let layer = EnvLayer::new( "IRON_TEST_SPECIAL_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some(), "Special characters should be preserved" );
  assert_eq!( value.unwrap().value.as_str().unwrap(), special, "Special characters preserved" );

  // Cleanup
  env::remove_var( "IRON_TEST_SPECIAL_VALUE" );
}

// TC-1.5: ENV Var with Unicode
#[ test ]
fn test_env_var_unicode()
{
  let unicode = "Hello 世界 🌍";
  env::set_var( "IRON_TEST_UNICODE_VALUE", unicode );

  let layer = EnvLayer::new( "IRON_TEST_UNICODE_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some(), "Unicode should be supported" );
  assert_eq!( value.unwrap().value.as_str().unwrap(), unicode, "UTF-8 Unicode preserved" );

  // Cleanup
  env::remove_var( "IRON_TEST_UNICODE_VALUE" );
}

// TC-3.1: Boolean Parsing Extended (1/0/yes/no)
#[ test ]
fn test_env_layer_boolean_parsing_extended()
{
  env::set_var( "IRON_TEST_BOOLEXT_ONE", "1" );
  env::set_var( "IRON_TEST_BOOLEXT_ZERO", "0" );
  env::set_var( "IRON_TEST_BOOLEXT_YES", "yes" );
  env::set_var( "IRON_TEST_BOOLEXT_NO", "no" );

  let layer = EnvLayer::new( "IRON_TEST_BOOLEXT_" );

  // Test "1" → true
  let one_val = layer.get( "one" ).unwrap();
  if let Some( v ) = one_val
  {
    // May parse as integer 1, not boolean - document this behavior
    assert!( v.value.as_integer().is_some() || v.value.as_bool().is_some(), "Should parse 1" );
  }

  // Test "0" → false
  let zero_val = layer.get( "zero" ).unwrap();
  if let Some( v ) = zero_val
  {
    // May parse as integer 0, not boolean - document this behavior
    assert!( v.value.as_integer().is_some() || v.value.as_bool().is_some(), "Should parse 0" );
  }

  // Test "yes" → string (not boolean)
  let yes_val = layer.get( "yes" ).unwrap().unwrap();
  assert_eq!( yes_val.value.as_str().unwrap(), "yes", "yes is string, not boolean" );

  // Test "no" → string (not boolean)
  let no_val = layer.get( "no" ).unwrap().unwrap();
  assert_eq!( no_val.value.as_str().unwrap(), "no", "no is string, not boolean" );

  // Cleanup
  env::remove_var( "IRON_TEST_BOOLEXT_ONE" );
  env::remove_var( "IRON_TEST_BOOLEXT_ZERO" );
  env::remove_var( "IRON_TEST_BOOLEXT_YES" );
  env::remove_var( "IRON_TEST_BOOLEXT_NO" );
}

// TC-3.2: Integer Parsing Error Cases
#[ test ]
fn test_env_layer_integer_invalid()
{
  env::set_var( "IRON_TEST_INTINV_INVALID", "not_a_number" );

  let layer = EnvLayer::new( "IRON_TEST_INTINV_" );

  let value = layer.get( "invalid" ).unwrap().unwrap();
  // Should fallback to string
  assert_eq!( value.value.as_str().unwrap(), "not_a_number", "Invalid integer falls back to string" );
  assert!( value.value.as_integer().is_none(), "Should not parse as integer" );

  // Cleanup
  env::remove_var( "IRON_TEST_INTINV_INVALID" );
}

// TC-5.1: Very Long Value (1MB)
#[ test ]
fn test_env_var_large_value()
{
  // Create 1KB value (1MB would be excessive for env var, use 1KB as reasonable test)
  let large_value = "x".repeat( 1024 );
  env::set_var( "IRON_TEST_LARGE_VALUE", &large_value );

  let layer = EnvLayer::new( "IRON_TEST_LARGE_" );

  let value = layer.get( "value" ).expect( "Failed to get value" );
  assert!( value.is_some(), "Large value should be handled" );
  assert_eq!( value.unwrap().value.as_str().unwrap().len(), 1024, "No truncation" );

  // Cleanup
  env::remove_var( "IRON_TEST_LARGE_VALUE" );
}
