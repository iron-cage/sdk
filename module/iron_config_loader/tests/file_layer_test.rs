//! File-based configuration layer tests
//!
//! Tests loading configuration from TOML files with proper precedence.

use iron_config_loader::layer::FileLayer;
use iron_config_loader::ConfigLayer;
use std::fs;
use tempfile::TempDir;

#[ test ]
fn test_file_layer_from_file()
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let config_path = temp_dir.path().join( "test.toml" );

  let config_content = r#"
[database]
url = "sqlite://file.db"
max_connections = 10
"#;

  fs::write( &config_path, config_content ).expect( "Failed to write config file" );

  let layer = FileLayer::from_file( "Test Config", 4, config_path )
    .expect( "Failed to create file layer" );

  // Verify values
  let url = layer.get( "database.url" ).expect( "Failed to get value" );
  assert!( url.is_some(), "Should find database.url" );
  assert_eq!( url.unwrap().value.as_str().unwrap(), "sqlite://file.db" );

  let max_conn = layer.get( "database.max_connections" ).expect( "Failed to get value" );
  assert!( max_conn.is_some(), "Should find max_connections" );
  assert_eq!( max_conn.unwrap().value.as_integer().unwrap(), 10 );
}

#[ test ]
fn test_file_layer_missing_file()
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let config_path = temp_dir.path().join( "nonexistent.toml" );

  // Should succeed but return None for all keys
  let layer = FileLayer::from_file( "Missing Config", 4, config_path )
    .expect( "Failed to create file layer" );

  let result = layer.get( "database.url" ).expect( "Failed to query layer" );
  assert!( result.is_none(), "Should return None for missing file" );
}

#[ test ]
fn test_file_layer_invalid_toml()
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let config_path = temp_dir.path().join( "invalid.toml" );

  let invalid_content = r#"
[database
url = "invalid"
"#;

  fs::write( &config_path, invalid_content ).expect( "Failed to write file" );

  // Should error on invalid TOML
  let result = FileLayer::from_file( "Invalid Config", 4, config_path );
  assert!( result.is_err(), "Should error on invalid TOML" );
}

#[ test ]
fn test_file_layer_from_str()
{
  let config_str = r#"
[database]
url = "sqlite://string.db"
max_connections = 7
"#;

  let layer = FileLayer::from_str( "String Config", 2, config_str )
    .expect( "Failed to create layer from string" );

  let url = layer.get( "database.url" ).expect( "Failed to get value" );
  assert!( url.is_some() );
  assert_eq!( url.unwrap().value.as_str().unwrap(), "sqlite://string.db" );
}

#[ test ]
fn test_file_layer_nested_access()
{
  let config_str = r#"
[database]
url = "sqlite://test.db"

[database.pool]
min_connections = 1
max_connections = 5

[database.pool.timeout]
connection = 30
query = 10
"#;

  let layer = FileLayer::from_str( "Nested Config", 2, config_str )
    .expect( "Failed to create layer" );

  // Access nested values at different depths
  let url = layer.get( "database.url" ).unwrap().unwrap();
  assert_eq!( url.value.as_str().unwrap(), "sqlite://test.db" );

  let min_conn = layer.get( "database.pool.min_connections" ).unwrap().unwrap();
  assert_eq!( min_conn.value.as_integer().unwrap(), 1 );

  let conn_timeout = layer.get( "database.pool.timeout.connection" ).unwrap().unwrap();
  assert_eq!( conn_timeout.value.as_integer().unwrap(), 30 );
}

#[ test ]
fn test_file_layer_get_all()
{
  let config_str = r#"
[database]
url = "sqlite://test.db"
max_connections = 5

[development]
debug = true
"#;

  let layer = FileLayer::from_str( "Test Config", 2, config_str )
    .expect( "Failed to create layer" );

  let all = layer.get_all().expect( "Failed to get all values" );

  // Should contain all flattened keys
  assert!( all.contains_key( "database.url" ), "Should contain database.url" );
  assert!( all.contains_key( "database.max_connections" ), "Should contain max_connections" );
  assert!( all.contains_key( "development.debug" ), "Should contain debug" );
  assert_eq!( all.len(), 3, "Should have exactly 3 keys" );
}

#[ test ]
fn test_file_layer_priority()
{
  let layer1 = FileLayer::from_str( "High Priority", 5, "[test]\nvalue = 1" )
    .expect( "Failed to create layer" );

  let layer2 = FileLayer::from_str( "Low Priority", 1, "[test]\nvalue = 2" )
    .expect( "Failed to create layer" );

  assert_eq!( layer1.priority(), 5 );
  assert_eq!( layer2.priority(), 1 );
}

#[ test ]
fn test_file_layer_source_tracking()
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let config_path = temp_dir.path().join( "source.toml" );

  fs::write( &config_path, "[test]\nvalue = 42" ).expect( "Failed to write file" );

  let layer = FileLayer::from_file( "Source Test", 4, config_path.clone() )
    .expect( "Failed to create layer" );

  let value = layer.get( "test.value" ).unwrap().unwrap();

  // Source should include layer name and file path
  assert!( value.source.contains( "Source Test" ), "Source should contain layer name" );
  assert!( value.source.contains( "source.toml" ), "Source should contain filename" );
}

// TC-2.2: Empty Config File
#[ test ]
fn test_file_layer_empty_file()
{
  let temp_dir = TempDir::new().expect( "Failed to create temp dir" );
  let config_path = temp_dir.path().join( "empty.toml" );

  fs::write( &config_path, "" ).expect( "Failed to write empty file" );

  // Should succeed - empty file is valid TOML
  let layer = FileLayer::from_file( "Empty Config", 4, config_path )
    .expect( "Failed to create file layer from empty file" );

  // Should return None for all keys
  let result = layer.get( "database.url" ).expect( "Failed to query layer" );
  assert!( result.is_none(), "Should return None for keys in empty file" );

  let all = layer.get_all().expect( "Failed to get all values" );
  assert!( all.is_empty(), "Empty file should have no keys" );
}

// TC-2.4: Config File with Unknown Fields (Forward Compatibility)
#[ test ]
fn test_file_layer_unknown_fields()
{
  let config_str = r#"
[database]
url = "sqlite://test.db"
max_connections = 5

# Unknown fields should be preserved
unknown_field = "value"
experimental_feature = true

[unknown_section]
key = "val"
"#;

  let layer = FileLayer::from_str( "Forward Compat Config", 2, config_str )
    .expect( "Failed to create layer with unknown fields" );

  // Known fields should parse correctly
  let url = layer.get( "database.url" ).unwrap().unwrap();
  assert_eq!( url.value.as_str().unwrap(), "sqlite://test.db" );

  // Unknown fields should be accessible
  let unknown = layer.get( "database.unknown_field" ).unwrap();
  assert!( unknown.is_some(), "Unknown fields should be preserved" );
  assert_eq!( unknown.unwrap().value.as_str().unwrap(), "value" );

  let unknown_section = layer.get( "unknown_section.key" ).unwrap();
  assert!( unknown_section.is_some(), "Unknown sections should be preserved" );
}
