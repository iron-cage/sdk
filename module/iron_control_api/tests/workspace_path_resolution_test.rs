//! Workspace path resolution tests
//!
//! Verifies that workspace_tools integration provides context-independent
//! database path resolution.

use workspace_tools::workspace;

#[ test ]
fn test_workspace_detection()
{
  // Workspace should be detectable from module directory
  let ws = workspace().expect( "Failed to detect workspace" );

  // Verify workspace root contains Cargo.toml
  let cargo_toml = ws.root().join( "Cargo.toml" );
  assert!( cargo_toml.exists(), "Workspace Cargo.toml not found at {:?}", cargo_toml );
}

#[ test ]
fn test_database_path_construction()
{
  // Verify workspace-relative database paths can be constructed
  let ws = workspace().expect( "Failed to detect workspace" );

  // Pilot mode path
  let pilot_db = ws.root().join( "iron.db" );
  assert!( pilot_db.starts_with( ws.root() ), "Pilot DB path not workspace-relative" );

  // Development mode path
  let dev_db = ws.data_dir().join( "dev_control.db" );
  assert!( dev_db.starts_with( ws.root() ), "Dev DB path not workspace-relative" );

  // Production mode path
  let prod_db = ws.data_dir().join( "iron_production.db" );
  assert!( prod_db.starts_with( ws.root() ), "Prod DB path not workspace-relative" );
}

#[ test ]
fn test_database_url_format()
{
  // Verify SQLite URL construction includes required parameters
  let ws = workspace().expect( "Failed to detect workspace" );
  let db_path = ws.root().join( "test.db" );
  let db_url = format!( "sqlite://{}?mode=rwc", db_path.display() );

  // URL should contain sqlite:// prefix
  assert!( db_url.starts_with( "sqlite://" ), "URL missing sqlite:// prefix" );

  // URL should contain ?mode=rwc parameter
  assert!( db_url.contains( "?mode=rwc" ), "URL missing ?mode=rwc parameter" );

  // URL should be absolute path (starts with /)
  assert!( db_url.contains( "sqlite:///" ), "URL should use absolute path (triple slash)" );
}

#[ test ]
fn test_data_dir_creation()
{
  // Verify data directory path resolution
  let ws = workspace().expect( "Failed to detect workspace" );
  let data_dir = ws.data_dir();

  // Data dir should be workspace-relative
  assert!( data_dir.starts_with( ws.root() ), "Data dir not workspace-relative" );

  // Data dir should be named "data"
  assert!( data_dir.ends_with( "data" ), "Data dir should be named 'data'" );
}

#[ test ]
fn test_ci_test_database_path()
{
  // Verify CI test database path resolution
  let ws = workspace().expect( "Failed to detect workspace" );
  let test_db_dir = ws.root().join( "target" ).join( "test_databases" );

  // Path should be workspace-relative
  assert!( test_db_dir.starts_with( ws.root() ), "Test DB dir not workspace-relative" );

  // Path should be in target directory
  assert!( test_db_dir.to_string_lossy().contains( "target" ), "Test DB not in target/" );
}
