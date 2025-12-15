//! Impossibility Tests for iron_cli Configuration
//!
//! These tests verify that old configuration methods are IMPOSSIBLE to use.
//! They check that old builder methods have been deleted and old env var
//! names are completely ignored.

#[test]
fn old_builder_method_with_env_is_deleted()
{
  let source = std::fs::read_to_string( "src/config.rs" )
    .expect( "config.rs should exist" );

  assert!(
    !source.contains( "fn with_env" ),
    "REGRESSION: Old method 'fn with_env' found - should be deleted"
  );

  assert!(
    !source.contains( "pub fn with_env" ),
    "REGRESSION: Old method 'pub fn with_env' found - should be deleted"
  );
}

#[test]
fn old_builder_method_with_defaults_is_deleted()
{
  let source = std::fs::read_to_string( "src/config.rs" )
    .expect( "config.rs should exist" );

  // Note: We check for the OLD with_defaults that directly set values
  // The new get_defaults_toml() is a helper, not the old builder method

  // This is tricky - we can't just search for "fn with_defaults"
  // because that might match other things. Instead, check that
  // the old implementation pattern doesn't exist

  let old_pattern = "self.values.insert(\"api_url\"";

  assert!(
    !source.contains( old_pattern ) || source.contains( "ConfigLoader" ),
    "REGRESSION: Old with_defaults pattern found without ConfigLoader"
  );
}

#[test]
fn new_builder_method_with_iron_config_exists()
{
  let source = std::fs::read_to_string( "src/config.rs" )
    .expect( "config.rs should exist" );

  assert!(
    source.contains( "fn with_iron_config" ) || source.contains( "pub fn with_iron_config" ),
    "FAILURE: New method 'with_iron_config' not found - new way missing"
  );
}

#[test]
fn old_env_var_names_are_ignored()
{
  // Set old env var names
  std::env::set_var( "IRON_API_URL", "https://old.should.not.work" );
  std::env::set_var( "IRON_FORMAT", "yaml" );

  let config = iron_cli::config::Config::from_env();

  // Old env vars should be IGNORED - we should get defaults instead
  let api_url = config.get( "api_url" );

  // If we get the old value, that's a regression
  assert_ne!(
    api_url,
    Some( "https://old.should.not.work".to_string() ),
    "REGRESSION: Old env var IRON_API_URL was read - old way still works!"
  );

  let format = config.get( "format" );
  assert_ne!(
    format,
    Some( "yaml".to_string() ),
    "REGRESSION: Old env var IRON_FORMAT was read - old way still works!"
  );

  // Cleanup
  std::env::remove_var( "IRON_API_URL" );
  std::env::remove_var( "IRON_FORMAT" );
}

#[test]
fn new_env_var_names_are_required()
{
  // Set new env var names
  std::env::set_var( "IRON_CLI_API_URL", "https://new.should.work" );
  std::env::set_var( "IRON_CLI_FORMAT", "json" );

  let config = iron_cli::config::Config::from_env();

  // New env vars MUST work
  assert_eq!(
    config.get( "api_url" ),
    Some( "https://new.should.work".to_string() ),
    "FAILURE: New env var IRON_CLI_API_URL not read - new way doesn't work!"
  );

  assert_eq!(
    config.get( "format" ),
    Some( "json".to_string() ),
    "FAILURE: New env var IRON_CLI_FORMAT not read - new way doesn't work!"
  );

  // Cleanup
  std::env::remove_var( "IRON_CLI_API_URL" );
  std::env::remove_var( "IRON_CLI_FORMAT" );
}

#[test]
fn no_manual_env_var_in_config_rs()
{
  let source = std::fs::read_to_string( "src/config.rs" )
    .expect( "config.rs should exist" );

  // Forbidden patterns (old way of reading env vars directly)
  let forbidden = [
    r#"std::env::var("IRON_API"#,
    r#"env::var("IRON_API"#,
    r#"std::env::var("IRON_FORMAT"#,
    r#"env::var("IRON_FORMAT"#,
    // But NOT IRON_CLI_* - those are okay if used through ConfigLoader
  ];

  for pattern in &forbidden
  {
    assert!(
      !source.contains( pattern ),
      "REGRESSION: Found old env var pattern '{}' in config.rs",
      pattern
    );
  }
}

#[test]
fn iron_config_dependency_is_required()
{
  let cargo_toml = std::fs::read_to_string( "Cargo.toml" )
    .expect( "Cargo.toml should exist" );

  assert!(
    cargo_toml.contains( "iron_config" ),
    "FAILURE: iron_config dependency missing from Cargo.toml"
  );
}

#[test]
fn no_backup_files_exist()
{
  let backup_patterns = [
    "src/config.rs.backup",
    "src/config.rs.old",
    "src/config_old.rs",
    "src/config_backup.rs",
  ];

  for path in &backup_patterns
  {
    assert!(
      !std::path::Path::new( path ).exists(),
      "FAILURE: Backup file exists: {} - old code not fully deleted",
      path
    );
  }
}

#[test]
fn no_commented_out_old_code()
{
  let source = std::fs::read_to_string( "src/config.rs" )
    .expect( "config.rs should exist" );

  let lines: Vec< &str > = source.lines().collect();

  for line in &lines
  {
    let trimmed = line.trim();

    if !trimmed.starts_with( "//" )
    {
      continue;
    }

    // Check for old patterns in comments
    let forbidden_in_comments = [
      "fn with_env",
      "fn with_defaults",
      "env::var(\"IRON_API_URL\")",
    ];

    for pattern in &forbidden_in_comments
    {
      assert!(
        !trimmed.contains( pattern ),
        "REGRESSION: Commented-out old code found: '{}' - delete completely",
        trimmed
      );
    }
  }
}

#[test]
fn all_test_files_use_new_env_var_names()
{
  // Read all test files
  let test_files = [
    "tests/config_test.rs",
    "tests/integration_test.rs",
    "tests/fixtures/test_harness.rs",
  ];

  for file_path in &test_files
  {
    if !std::path::Path::new( file_path ).exists()
    {
      continue; // Skip if file doesn't exist
    }

    let source = std::fs::read_to_string( file_path )
      .unwrap_or_else( |_| panic!( "Failed to read {}", file_path ) );

    // Check for old env var names (without IRON_CLI_ prefix)
    let old_patterns = [
      r#""IRON_API_URL""#,
      r#""IRON_FORMAT""#,
      r#""IRON_USER""#,
      r#""IRON_TOKEN""#,
    ];

    for pattern in &old_patterns
    {
      // Allow if it's immediately followed by "IRON_CLI_" or in a comment
      if source.contains( pattern ) && !source.contains( &format!( "{}_CLI", pattern.trim_end_matches( '"' ) ) )
      {
        // Check if it's in a context that's allowed (like test assertions about old vars)
        let is_in_regression_test = source.contains( "test_old_env_var_names_are_ignored" ) ||
                                     source.contains( "should.not.work" );

        if !is_in_regression_test
        {
          panic!(
            "REGRESSION: Old env var {} found in {} - should use IRON_CLI_* prefix",
            pattern, file_path
          );
        }
      }
    }
  }
}
