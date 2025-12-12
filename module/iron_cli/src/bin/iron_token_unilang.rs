//! Unilang-based CLI entry point for iron-token
//!
//! This CLI provides access to Iron Cage Token Management API with 22 commands
//! for authentication, token management, usage tracking, limits, traces, and health.
//!
//! Architecture:
//! - Commands defined in YAML files (commands/*.yaml)
//! - Runtime YAML loading for flexibility and ease of development
//! - Pipeline handles: parsing → validation → execution
//! - Handlers are pure functions (no I/O, no async)
//! - Adapters bridge handlers to services (I/O layer)
//!
//! Commands (22 total):
//! - Auth (3): .auth.{login,refresh,logout}
//! - Tokens (5): .tokens.{generate,list,get,rotate,revoke}
//! - Usage (4): .usage.{show,by_project,by_provider,export}
//! - Limits (5): .limits.{list,get,create,update,delete}
//! - Traces (3): .traces.{list,get,export}
//! - Health (2): .health.{check,version}
//!
//! Implementation Status: Phase 3 - Runtime YAML loading with command execution
//! - ✅ Binary entry point created
//! - ✅ YAML command definitions (22 commands)
//! - ✅ Handler implementations (22 handlers)
//! - ✅ Adapter infrastructure
//! - ✅ Pipeline integration

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

/// Load command registry from YAML files in commands/
fn load_command_registry() -> Result< CommandRegistry, Box< dyn std::error::Error > >
{
  // Determine path to commands directory
  let manifest_dir = PathBuf::from( env!( "CARGO_MANIFEST_DIR" ) );
  let commands_dir = manifest_dir.join( "commands" );

  if !commands_dir.exists()
  {
    return Err( format!( "Commands directory not found: {}", commands_dir.display() ).into() );
  }

  // Discover all YAML files (excluding control/ subdirectory)
  let yaml_files = discover_yaml_files( &commands_dir )?;

  if yaml_files.is_empty()
  {
    return Err( "No YAML command files found in commands/".into() );
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

/// Discover all YAML files in directory (excluding subdirectories)
fn discover_yaml_files( dir: &PathBuf ) -> Result< Vec< PathBuf >, Box< dyn std::error::Error > >
{
  let mut files = Vec::new();

  for entry in std::fs::read_dir( dir )?
  {
    let entry = entry?;
    let path = entry.path();

    // Only process files, not subdirectories
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
    .map_err( |e| format!( "Failed to create runtime: {}", e ) )?;

  match command_name
  {
    // Auth commands
    ".auth.login" =>
    {
      runtime.block_on( iron_cli::adapters::auth_adapters::login_adapter( params ) )
    }
    ".auth.refresh" =>
    {
      runtime.block_on( iron_cli::adapters::auth_adapters::refresh_adapter( params ) )
    }
    ".auth.logout" =>
    {
      runtime.block_on( iron_cli::adapters::auth_adapters::logout_adapter( params ) )
    }

    // Token commands
    ".tokens.generate" =>
    {
      runtime.block_on( iron_cli::adapters::token_adapters::generate_token_adapter( params ) )
    }
    ".tokens.list" =>
    {
      runtime.block_on( iron_cli::adapters::token_adapters::list_tokens_adapter( params ) )
    }
    ".tokens.get" =>
    {
      runtime.block_on( iron_cli::adapters::token_adapters::get_token_adapter( params ) )
    }
    ".tokens.rotate" =>
    {
      runtime.block_on( iron_cli::adapters::token_adapters::rotate_token_adapter( params ) )
    }
    ".tokens.revoke" =>
    {
      runtime.block_on( iron_cli::adapters::token_adapters::revoke_token_adapter( params ) )
    }

    // Usage commands
    ".usage.show" =>
    {
      runtime.block_on( iron_cli::adapters::usage_adapters::show_usage_adapter( params ) )
    }
    ".usage.by_project" =>
    {
      runtime.block_on( iron_cli::adapters::usage_adapters::get_usage_by_project_adapter( params ) )
    }
    ".usage.by_provider" =>
    {
      runtime.block_on( iron_cli::adapters::usage_adapters::get_usage_by_provider_adapter( params ) )
    }
    ".usage.export" =>
    {
      runtime.block_on( iron_cli::adapters::usage_adapters::export_usage_adapter( params ) )
    }

    // Limits commands
    ".limits.list" =>
    {
      runtime.block_on( iron_cli::adapters::limits_adapters::show_limits_adapter( params ) )
    }
    ".limits.get" =>
    {
      runtime.block_on( iron_cli::adapters::limits_adapters::get_limit_adapter( params ) )
    }
    ".limits.create" =>
    {
      runtime.block_on( iron_cli::adapters::limits_adapters::create_limit_adapter( params ) )
    }
    ".limits.update" =>
    {
      runtime.block_on( iron_cli::adapters::limits_adapters::update_limit_adapter( params ) )
    }
    ".limits.delete" =>
    {
      runtime.block_on( iron_cli::adapters::limits_adapters::delete_limit_adapter( params ) )
    }

    // Traces commands
    ".traces.list" =>
    {
      runtime.block_on( iron_cli::adapters::traces_adapters::list_traces_adapter( params ) )
    }
    ".traces.get" =>
    {
      runtime.block_on( iron_cli::adapters::traces_adapters::get_trace_adapter( params ) )
    }
    ".traces.export" =>
    {
      runtime.block_on( iron_cli::adapters::traces_adapters::export_traces_adapter( params ) )
    }

    // Health commands (using actual YAML command names)
    ".health" =>
    {
      runtime.block_on( iron_cli::adapters::health_adapters::health_check_adapter( params ) )
    }
    ".version" =>
    {
      runtime.block_on( iron_cli::adapters::health_adapters::version_adapter( params ) )
    }

    // Default: Command not implemented
    _ =>
    {
      Ok( format!( "Command '{}' not recognized", command_name ) )
    }
  }
}

fn print_banner()
{
  println!( "iron-token (Token Management CLI) v0.1.0" );
  println!( "Using unilang CLI framework" );
  println!();
  println!( "Commands follow dot-prefix convention: iron-token .<resource>.<action> [param::value...]" );
  println!();
  println!( "Examples:" );
  println!( "  iron-token .auth.login username::admin password::secret" );
  println!( "  iron-token .tokens.generate name::\"MyToken\" scope::\"read:write\"" );
  println!( "  iron-token .tokens.list format::json" );
  println!( "  iron-token .usage.show" );
  println!();
  println!( "Help:" );
  println!( "  iron-token .help                    # List all commands" );
  println!( "  iron-token .tokens.list ?           # Command help" );
  println!();
  println!( "Status: Phase 5 - Full adapter integration complete ✓" );
  println!( "All 22 commands integrated with HTTP adapters" );
  println!( "Keyring authentication enabled" );
}
