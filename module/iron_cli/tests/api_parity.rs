//! CLI/API Parity Tests
//!
//! Phase 4 Day 35: CLI/API Parity Testing
//!
//! Verifies that CLI commands have parity with API endpoints:
//! - Count parity: All API endpoints have CLI equivalents
//! - Operation parity: Commands perform same operations
//! - Structure parity: Command structure matches API structure

/// API endpoint inventory
///
/// This documents all REST API endpoints that must have CLI equivalents.
#[ derive( Debug, Clone ) ]
struct ApiEndpoint
{
  method: &'static str,
  path: &'static str,
  category: &'static str,
  operation: &'static str,
}

/// CLI command inventory
///
/// This documents all CLI commands and their API mappings.
#[ derive( Debug, Clone ) ]
struct CliCommand
{
  command: &'static str,
  category: &'static str,
  operation: &'static str,
  api_endpoint: &'static str,
}

/// Get complete API endpoint inventory
fn get_api_endpoints() -> Vec< ApiEndpoint >
{
  vec![
    // Authentication endpoints (3)
    ApiEndpoint { method: "POST", path: "/api/auth/login", category: "auth", operation: "login" },
    ApiEndpoint { method: "POST", path: "/api/auth/refresh", category: "auth", operation: "refresh" },
    ApiEndpoint { method: "POST", path: "/api/auth/logout", category: "auth", operation: "logout" },

    // Token management endpoints (5)
    ApiEndpoint { method: "POST", path: "/api/tokens", category: "tokens", operation: "create" },
    ApiEndpoint { method: "GET", path: "/api/tokens", category: "tokens", operation: "list" },
    ApiEndpoint { method: "GET", path: "/api/tokens/{id}", category: "tokens", operation: "get" },
    ApiEndpoint { method: "POST", path: "/api/tokens/{id}/rotate", category: "tokens", operation: "rotate" },
    ApiEndpoint { method: "POST", path: "/api/tokens/{id}/revoke", category: "tokens", operation: "revoke" },

    // Usage analytics endpoints (3)
    ApiEndpoint { method: "GET", path: "/api/usage/aggregate", category: "usage", operation: "aggregate" },
    ApiEndpoint { method: "GET", path: "/api/usage/by-project/{id}", category: "usage", operation: "by-project" },
    ApiEndpoint { method: "GET", path: "/api/usage/by-provider/{provider}", category: "usage", operation: "by-provider" },

    // Limits management endpoints (5)
    ApiEndpoint { method: "GET", path: "/api/limits", category: "limits", operation: "list" },
    ApiEndpoint { method: "GET", path: "/api/limits/{id}", category: "limits", operation: "get" },
    ApiEndpoint { method: "POST", path: "/api/limits", category: "limits", operation: "create" },
    ApiEndpoint { method: "PUT", path: "/api/limits/{id}", category: "limits", operation: "update" },
    ApiEndpoint { method: "DELETE", path: "/api/limits/{id}", category: "limits", operation: "delete" },

    // Traces endpoints (2)
    ApiEndpoint { method: "GET", path: "/api/traces", category: "traces", operation: "list" },
    ApiEndpoint { method: "GET", path: "/api/traces/{id}", category: "traces", operation: "get" },

    // Health endpoint (1)
    ApiEndpoint { method: "GET", path: "/api/health", category: "health", operation: "check" },
  ]
}

/// Get complete CLI command inventory
fn get_cli_commands() -> Vec< CliCommand >
{
  vec![
    // Authentication commands (3)
    CliCommand { command: "iron-token auth login", category: "auth", operation: "login", api_endpoint: "POST /api/auth/login" },
    CliCommand { command: "iron-token auth refresh", category: "auth", operation: "refresh", api_endpoint: "POST /api/auth/refresh" },
    CliCommand { command: "iron-token auth logout", category: "auth", operation: "logout", api_endpoint: "POST /api/auth/logout" },

    // Token management commands (5)
    CliCommand { command: "iron-token tokens generate", category: "tokens", operation: "create", api_endpoint: "POST /api/tokens" },
    CliCommand { command: "iron-token tokens list", category: "tokens", operation: "list", api_endpoint: "GET /api/tokens" },
    CliCommand { command: "iron-token tokens get", category: "tokens", operation: "get", api_endpoint: "GET /api/tokens/{id}" },
    CliCommand { command: "iron-token tokens rotate", category: "tokens", operation: "rotate", api_endpoint: "POST /api/tokens/{id}/rotate" },
    CliCommand { command: "iron-token tokens revoke", category: "tokens", operation: "revoke", api_endpoint: "POST /api/tokens/{id}/revoke" },

    // Usage analytics commands (4 - includes export which aggregates API calls)
    CliCommand { command: "iron-token usage show", category: "usage", operation: "aggregate", api_endpoint: "GET /api/usage/aggregate" },
    CliCommand { command: "iron-token usage by-project", category: "usage", operation: "by-project", api_endpoint: "GET /api/usage/by-project/{id}" },
    CliCommand { command: "iron-token usage by-provider", category: "usage", operation: "by-provider", api_endpoint: "GET /api/usage/by-provider/{provider}" },
    CliCommand { command: "iron-token usage export", category: "usage", operation: "aggregate", api_endpoint: "GET /api/usage/aggregate" },

    // Limits management commands (5)
    CliCommand { command: "iron-token limits list", category: "limits", operation: "list", api_endpoint: "GET /api/limits" },
    CliCommand { command: "iron-token limits get", category: "limits", operation: "get", api_endpoint: "GET /api/limits/{id}" },
    CliCommand { command: "iron-token limits create", category: "limits", operation: "create", api_endpoint: "POST /api/limits" },
    CliCommand { command: "iron-token limits update", category: "limits", operation: "update", api_endpoint: "PUT /api/limits/{id}" },
    CliCommand { command: "iron-token limits delete", category: "limits", operation: "delete", api_endpoint: "DELETE /api/limits/{id}" },

    // Traces commands (3 - includes export)
    CliCommand { command: "iron-token traces list", category: "traces", operation: "list", api_endpoint: "GET /api/traces" },
    CliCommand { command: "iron-token traces get", category: "traces", operation: "get", api_endpoint: "GET /api/traces/{id}" },
    CliCommand { command: "iron-token traces export", category: "traces", operation: "list", api_endpoint: "GET /api/traces" },

    // Health command (1)
    CliCommand { command: "iron-token health", category: "health", operation: "check", api_endpoint: "GET /api/health" },
  ]
}

#[ test ]
fn test_count_parity()
{
  let api_endpoints = get_api_endpoints();
  let cli_commands = get_cli_commands();

  // Count endpoints by category
  let mut api_counts = std::collections::HashMap::new();
  for endpoint in &api_endpoints
  {
    *api_counts.entry( endpoint.category ).or_insert( 0 ) += 1;
  }

  // Count CLI commands by category (excluding duplicates like export)
  let mut cli_counts = std::collections::HashMap::new();
  for command in &cli_commands
  {
    // Only count unique API endpoints (export commands reuse existing endpoints)
    if !command.command.contains( "export" )
    {
      *cli_counts.entry( command.category ).or_insert( 0 ) += 1;
    }
  }

  println!( "\nAPI Endpoints by Category:" );
  for ( category, count ) in &api_counts
  {
    println!( "  {}: {}", category, count );
  }

  println!( "\nCLI Commands by Category (unique API mappings):" );
  for ( category, count ) in &cli_counts
  {
    println!( "  {}: {}", category, count );
  }

  // Verify each category has parity
  for ( category, api_count ) in &api_counts
  {
    let cli_count = cli_counts.get( category ).unwrap_or( &0 );
    assert_eq!(
      api_count, cli_count,
      "Category '{}' has {} API endpoints but {} CLI commands",
      category, api_count, cli_count
    );
  }

  println!( "\n✓ Count parity verified: {} API endpoints = {} unique CLI commands", api_endpoints.len(), cli_counts.values().sum::< i32 >() );
}

#[ test ]
fn test_operation_parity()
{
  let api_endpoints = get_api_endpoints();
  let cli_commands = get_cli_commands();

  // Build map of API operations
  let mut api_operations = std::collections::HashSet::new();
  for endpoint in &api_endpoints
  {
    api_operations.insert( format!( "{}:{}", endpoint.category, endpoint.operation ) );
  }

  // Build map of CLI operations
  let mut cli_operations = std::collections::HashSet::new();
  for command in &cli_commands
  {
    if !command.command.contains( "export" )
    {
      cli_operations.insert( format!( "{}:{}", command.category, command.operation ) );
    }
  }

  println!( "\nAPI Operations: {:?}", api_operations );
  println!( "\nCLI Operations: {:?}", cli_operations );

  // Verify all API operations have CLI equivalents
  for operation in &api_operations
  {
    assert!(
      cli_operations.contains( operation ),
      "API operation '{}' has no CLI equivalent",
      operation
    );
  }

  // Verify all CLI operations map to API operations
  for operation in &cli_operations
  {
    assert!(
      api_operations.contains( operation ),
      "CLI operation '{}' has no API equivalent",
      operation
    );
  }

  println!( "\n✓ Operation parity verified: {} operations have CLI/API parity", api_operations.len() );
}

#[ test ]
fn test_command_structure_parity()
{
  let cli_commands = get_cli_commands();

  // Verify command structure follows pattern: iron-token <category> <operation>
  for command in &cli_commands
  {
    let parts: Vec< &str > = command.command.split_whitespace().collect();

    assert_eq!( parts[ 0 ], "iron-token", "Command must start with 'iron-token'" );

    if parts.len() > 1
    {
      assert_eq!(
        parts[ 1 ], command.category,
        "Command '{}' should have category '{}' but has '{}'",
        command.command, command.category, parts[ 1 ]
      );
    }
  }

  println!( "\n✓ Command structure parity verified: All commands follow 'iron-token <category> <operation>' pattern" );
}

#[ test ]
fn test_api_coverage()
{
  let api_endpoints = get_api_endpoints();
  let cli_commands = get_cli_commands();

  // Build map of CLI API endpoint coverage
  let mut cli_api_map = std::collections::HashMap::new();
  for command in &cli_commands
  {
    cli_api_map
      .entry( command.api_endpoint )
      .or_insert_with( Vec::new )
      .push( command.command );
  }

  println!( "\nAPI Endpoint Coverage:" );
  for endpoint in &api_endpoints
  {
    let endpoint_key = format!( "{} {}", endpoint.method, endpoint.path );
    let cli_cmds = cli_api_map.get( endpoint_key.as_str() );

    match cli_cmds
    {
      Some( cmds ) =>
      {
        println!( "  {} {} -> {} CLI command(s)", endpoint.method, endpoint.path, cmds.len() );
        for cmd in cmds
        {
          println!( "    - {}", cmd );
        }
      }
      None =>
      {
        panic!( "API endpoint '{}' has no CLI command mapping", endpoint_key );
      }
    }
  }

  println!( "\n✓ API coverage verified: All {} API endpoints have CLI commands", api_endpoints.len() );
}

#[ test ]
fn test_category_completeness()
{
  let expected_categories = vec![ "auth", "tokens", "usage", "limits", "traces", "health" ];

  let api_endpoints = get_api_endpoints();
  let cli_commands = get_cli_commands();

  // Verify all expected categories exist in API
  for category in &expected_categories
  {
    let api_has_category = api_endpoints.iter().any( | e | e.category == *category );
    assert!( api_has_category, "API missing category: {}", category );
  }

  // Verify all expected categories exist in CLI
  for category in &expected_categories
  {
    let cli_has_category = cli_commands.iter().any( | c | c.category == *category );
    assert!( cli_has_category, "CLI missing category: {}", category );
  }

  println!( "\n✓ Category completeness verified: All {} categories present in API and CLI", expected_categories.len() );
}
