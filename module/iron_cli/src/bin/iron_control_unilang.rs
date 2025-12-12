//! Unilang-based CLI entry point for iron (Control API)
//!
//! This CLI provides access to the Iron Cage Control API with 47 commands
//! for managing agents, providers, analytics, budgets, projects, and users.
//!
//! Architecture:
//! - Commands defined in YAML files (commands/control/*.yaml)
//! - Runtime YAML loading for flexibility and ease of development
//! - Pipeline handles: parsing → validation → execution
//! - Handlers are pure functions (no I/O, no async)
//! - Adapters bridge handlers to REST API (I/O layer)
//!
//! Commands (47 total):
//! - Agents (8): .agent.{list,create,get,update,delete,assign_providers,list_providers,remove_provider}
//! - Providers (8): .provider.{list,create,get,update,delete,assign_agents,list_agents,remove_agent}
//! - Analytics (8): .analytics.{usage,spending,metrics,usage_by_agent,usage_by_provider,spending_by_period,export_usage,export_spending}
//! - Budget Limits (2): .budget_limit.{get,set}
//! - API Tokens (4): .api_token.{list,create,get,revoke}
//! - Projects (2): .project.{list,get}
//! - Budget Requests (6): .budget_request.{list,create,get,approve,reject,cancel}
//! - Users (8): .user.{list,create,get,update,delete,set_role,reset_password,get_permissions}
//!
//! Implementation Status: Phase 3 - Command execution
//! - ✅ Binary entry point created
//! - ✅ YAML command definitions (47 commands)
//! - ✅ Handler implementations (47 handlers)
//! - ✅ HTTP adapter infrastructure
//! - 🔄 Pipeline integration (in progress)

use unilang::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;

fn main() -> Result< (), Box< dyn std::error::Error > >
{
  let args : Vec< String > = std::env::args().collect();

  if args.len() == 1
  {
    print_banner();
    return Ok( () );
  }

  // Load command registry from YAML files
  let registry = load_command_registry()?;

  // Create pipeline
  let pipeline = Pipeline::new( registry );

  // Execute command
  let command_line = args[ 1.. ].join( " " );
  let result = pipeline.process_command_simple( &command_line );

  // Handle result
  if result.success
  {
    // Print all outputs
    for output in &result.outputs
    {
      println!( "{}", output.content );
    }
    Ok( () )
  }
  else
  {
    if let Some( error ) = result.error
    {
      eprintln!( "Error: {}", error );
    }
    else
    {
      eprintln!( "Command failed" );
    }
    std::process::exit( 1 );
  }
}

/// Load command registry from YAML files in commands/control/
fn load_command_registry() -> Result< CommandRegistry, Box< dyn std::error::Error > >
{
  // Determine path to commands directory
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );
  let commands_dir = manifest_dir.join( "commands" ).join( "control" );

  if !commands_dir.exists()
  {
    return Err( format!( "Commands directory not found: {}", commands_dir.display() ).into() );
  }

  // Discover all YAML files
  let yaml_files = discover_yaml_files( &commands_dir )?;

  if yaml_files.is_empty()
  {
    return Err( "No YAML command files found in commands/control/".into() );
  }

  // Load command definitions from YAML and register with routines
  use unilang::loader::load_command_definitions_from_yaml_str;

  #[ allow( deprecated ) ]
  let mut registry = CommandRegistry::new();

  for yaml_path in &yaml_files
  {
    let yaml_content = std::fs::read_to_string( yaml_path )?;

    // Load command definitions from YAML
    let command_defs = load_command_definitions_from_yaml_str( &yaml_content )?;

    // Register each command with its routine
    for cmd_def in command_defs
    {
      let command_name = cmd_def.full_name();

      // Create routine for this command
      let routine = create_command_routine( &command_name );

      // Register command with routine
      #[ allow( deprecated ) ]
      registry.command_add_runtime( &cmd_def, routine )?;
    }
  }

  Ok( registry )
}

/// Discover all YAML files in directory
fn discover_yaml_files( dir: &PathBuf ) -> Result< Vec< PathBuf >, Box< dyn std::error::Error > >
{
  let mut files = Vec::new();

  for entry in std::fs::read_dir( dir )?
  {
    let entry = entry?;
    let path = entry.path();

    if path.is_file()
    {
      if let Some( ext ) = path.extension()
      {
        if ext == "yaml" || ext == "yml"
        {
          files.push( path );
        }
      }
    }
  }

  Ok( files )
}

/// Create command routine for a specific command
fn create_command_routine( command_name: &str ) -> CommandRoutine
{
  let name = command_name.to_string();

  Box::new( move | cmd, _ctx |
  {
    // Extract parameters from verified command
    let params = extract_parameters( &cmd );

    // Route to appropriate handler based on command name
    let result = route_to_handler( &name, &params );

    // Format result as output
    match result
    {
      Ok( content ) =>
      {
        Ok( OutputData
        {
          content,
          format : "text".to_string(),
          execution_time_ms : None,
        })
      }
      Err( e ) =>
      {
        use unilang::data::{ ErrorData, ErrorCode };
        Err( ErrorData::new( ErrorCode::InternalError, format!( "Handler error: {}", e ) ) )
      }
    }
  })
}

/// Extract parameters from verified command
fn extract_parameters( cmd: &VerifiedCommand ) -> HashMap< String, String >
{
  let mut params = HashMap::new();

  for ( key, value ) in &cmd.arguments
  {
    params.insert( key.clone(), value.to_string() );
  }

  params
}

/// Route command to appropriate handler
fn route_to_handler(
  command_name: &str,
  params: &HashMap< String, String >,
) -> Result< String, String >
{
  // Create tokio runtime for async adapter calls
  let runtime = tokio::runtime::Runtime::new()
    .map_err( |e| format!( "Failed to create async runtime: {}", e ) )?;

  match command_name
  {
    // Agent commands
    ".agent.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::list_agents_adapter( params ) )
    }
    ".agent.create" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::create_agent_adapter( params ) )
    }
    ".agent.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::get_agent_adapter( params ) )
    }
    ".agent.update" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::update_agent_adapter( params ) )
    }
    ".agent.delete" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::delete_agent_adapter( params ) )
    }
    ".agent.assign_providers" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::assign_providers_adapter( params ) )
    }
    ".agent.list_providers" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::list_agent_providers_adapter( params ) )
    }
    ".agent.remove_provider" =>
    {
      runtime.block_on( iron_cli::adapters::control::agent_adapters::remove_provider_adapter( params ) )
    }

    // Provider commands
    ".provider.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::list_providers_adapter( params ) )
    }
    ".provider.create" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::create_provider_adapter( params ) )
    }
    ".provider.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::get_provider_adapter( params ) )
    }
    ".provider.update" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::update_provider_adapter( params ) )
    }
    ".provider.delete" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::delete_provider_adapter( params ) )
    }
    ".provider.assign_agents" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::assign_agents_adapter( params ) )
    }
    ".provider.list_agents" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::list_provider_agents_adapter( params ) )
    }
    ".provider.remove_agent" =>
    {
      runtime.block_on( iron_cli::adapters::control::provider_adapters::remove_agent_adapter( params ) )
    }

    // Analytics commands
    ".analytics.usage" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::usage_adapter( params ) )
    }
    ".analytics.spending" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::spending_adapter( params ) )
    }
    ".analytics.metrics" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::metrics_adapter( params ) )
    }
    ".analytics.usage_by_agent" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::usage_by_agent_adapter( params ) )
    }
    ".analytics.usage_by_provider" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::usage_by_provider_adapter( params ) )
    }
    ".analytics.spending_by_period" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::spending_by_period_adapter( params ) )
    }
    ".analytics.export_usage" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::export_usage_adapter( params ) )
    }
    ".analytics.export_spending" =>
    {
      runtime.block_on( iron_cli::adapters::control::analytics_adapters::export_spending_adapter( params ) )
    }

    // Budget limit commands
    ".budget_limit.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_limit_adapters::get_budget_limit_adapter( params ) )
    }
    ".budget_limit.set" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_limit_adapters::set_budget_limit_adapter( params ) )
    }

    // API token commands
    ".api_token.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::api_token_adapters::list_api_tokens_adapter( params ) )
    }
    ".api_token.create" =>
    {
      runtime.block_on( iron_cli::adapters::control::api_token_adapters::create_api_token_adapter( params ) )
    }
    ".api_token.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::api_token_adapters::get_api_token_adapter( params ) )
    }
    ".api_token.revoke" =>
    {
      runtime.block_on( iron_cli::adapters::control::api_token_adapters::revoke_api_token_adapter( params ) )
    }

    // Project commands
    ".project.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::project_adapters::list_projects_adapter( params ) )
    }
    ".project.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::project_adapters::get_project_adapter( params ) )
    }

    // Budget request commands
    ".budget_request.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::list_budget_requests_adapter( params ) )
    }
    ".budget_request.create" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::create_budget_request_adapter( params ) )
    }
    ".budget_request.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::get_budget_request_adapter( params ) )
    }
    ".budget_request.approve" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::approve_budget_request_adapter( params ) )
    }
    ".budget_request.reject" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::reject_budget_request_adapter( params ) )
    }
    ".budget_request.cancel" =>
    {
      runtime.block_on( iron_cli::adapters::control::budget_request_adapters::cancel_budget_request_adapter( params ) )
    }

    // User commands
    ".user.list" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::list_users_adapter( params ) )
    }
    ".user.create" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::create_user_adapter( params ) )
    }
    ".user.get" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::get_user_adapter( params ) )
    }
    ".user.update" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::update_user_adapter( params ) )
    }
    ".user.delete" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::delete_user_adapter( params ) )
    }
    ".user.set_role" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::set_user_role_adapter( params ) )
    }
    ".user.reset_password" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::reset_user_password_adapter( params ) )
    }
    ".user.get_permissions" =>
    {
      runtime.block_on( iron_cli::adapters::control::user_adapters::get_user_permissions_adapter( params ) )
    }

    // Default: Command not implemented
    _ =>
    {
      Ok( format!( "Command '{}' recognized but handler not yet implemented", command_name ) )
    }
  }
}

fn print_banner()
{
  println!( "iron (Control API CLI) v0.1.0" );
  println!( "Using unilang CLI framework" );
  println!();
  println!( "Commands follow dot-prefix convention: iron .<resource>.<action> [param::value...]" );
  println!();
  println!( "Examples:" );
  println!( "  iron .agent.list                         # List all agents" );
  println!( "  iron .agent.create name::\"Agent1\" budget::100  # Create agent" );
  println!( "  iron .provider.list format::json         # List providers (JSON)" );
  println!( "  iron .analytics.usage v::3               # Usage stats (detailed)" );
  println!();
  println!( "Help:" );
  println!( "  iron .help                    # List all commands" );
  println!( "  iron .agent.list ?            # Quick help" );
  println!( "  iron .agent.list ??           # Detailed help" );
  println!();
  println!( "Status: Phase 3 - Command execution framework complete" );
  println!( "Pipeline integration successful ✓" );
}
