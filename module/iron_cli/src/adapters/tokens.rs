//! Token management adapters
//!
//! Bridge unilang CLI to token handlers and services.
//!
//! ## Architecture
//!
//! ```text
//! VerifiedCommand → token_adapter() → token_handler() → TokenService
//!       ↓                  ↓                  ↓                ↓
//!   Extract params    Call handler      Validate        Async I/O (DB)
//! ```
//!
//! ## Adapter Responsibilities
//!
//! 1. Extract parameters from command
//! 2. Call handler for validation (pure, sync)
//! 3. Perform async I/O via TokenService
//! 4. Store results (if not dry-run)
//! 5. Format output

use super::AdapterError;
use super::services::TokenService;
use super::auth::HasParams;
use crate::handlers::token_handlers;
use crate::formatting::TreeFmtFormatter;
use std::collections::HashMap;

/// Extract parameters from command
fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// Check if dry-run mode enabled
fn is_dry_run(params: &HashMap<String, String>) -> bool
{
  params.get( "dry_run" )
    .and_then( |v| v.parse::<bool>().ok() )
    .unwrap_or( false )
}

/// Generate token adapter
///
/// Validates parameters, generates new token.
///
/// ## Flow
///
/// 1. Extract name/scope/ttl from command
/// 2. Call generate_token_handler() for validation
/// 3. Perform async token generation via TokenService
/// 4. Format output
pub async fn generate_token_adapter<T, S>(
  command: &T,
  token_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TokenService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = token_handlers::generate_token_handler( &params )?;

  let dry_run = is_dry_run( &params );

  if dry_run
  {
    // Dry-run: simulate output without creating token
    let mut output_data = HashMap::new();
    output_data.insert( "status".to_string(), "would generate (dry-run)".to_string() );
    output_data.insert( "name".to_string(), params.get( "name" ).unwrap_or( &"unknown".to_string() ).clone() );

    let output = formatter.format_single( &output_data );
    return Ok( output );
  }

  // Extract validated parameters
  let name = params.get( "name" ).ok_or_else( || {
    AdapterError::ExtractionError( "name missing after validation".to_string() )
  })?;

  let scope = params.get( "scope" ).ok_or_else( || {
    AdapterError::ExtractionError( "scope missing after validation".to_string() )
  })?;

  let ttl = params.get( "ttl" ).and_then( |s| s.parse::<i64>().ok() );

  // Perform async token generation
  let token = token_service.generate( name, scope, ttl ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "generated".to_string() );
  output_data.insert( "token_id".to_string(), token.id.clone() );
  output_data.insert( "name".to_string(), token.name.clone() );
  output_data.insert( "scope".to_string(), token.scope.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// List tokens adapter
///
/// Lists all tokens (optionally filtered).
///
/// ## Flow
///
/// 1. Extract filter parameter (optional)
/// 2. Call list_tokens_handler() for validation
/// 3. Perform async token listing via TokenService
/// 4. Format output
pub async fn list_tokens_adapter<T, S>(
  command: &T,
  token_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TokenService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = token_handlers::list_tokens_handler( &params )?;

  // Extract filter (optional)
  let filter = params.get( "filter" ).map( |s| s.as_str() );

  // Perform async token listing
  let tokens = token_service.list( filter ).await?;

  // Format output
  if tokens.is_empty()
  {
    return Ok( "No tokens found".to_string() );
  }

  // Format token list with names and IDs
  let mut output_data = HashMap::new();
  output_data.insert( "count".to_string(), tokens.len().to_string() );

  // Include token names/IDs in output
  let token_list: Vec<String> = tokens.iter()
    .map( |t| format!( "{} ({})", t.name, t.id ) )
    .collect();
  output_data.insert( "tokens".to_string(), token_list.join( ", " ) );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Get token adapter
///
/// Retrieves details for a specific token.
///
/// ## Flow
///
/// 1. Extract token_id from command
/// 2. Call get_token_handler() for validation
/// 3. Perform async token retrieval via TokenService
/// 4. Format output
pub async fn get_token_adapter<T, S>(
  command: &T,
  token_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TokenService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = token_handlers::get_token_handler( &params )?;

  // Extract validated token_id
  let token_id = params.get( "token_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "token_id missing after validation".to_string() )
  })?;

  // Perform async token retrieval
  let token = token_service.get( token_id ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "token_id".to_string(), token.id.clone() );
  output_data.insert( "name".to_string(), token.name.clone() );
  output_data.insert( "scope".to_string(), token.scope.clone() );
  output_data.insert( "created_at".to_string(), token.created_at.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Rotate token adapter
///
/// Rotates a token (generates new value, preserves scope).
///
/// ## Flow
///
/// 1. Extract token_id from command
/// 2. Call rotate_token_handler() for validation
/// 3. Perform async token rotation via TokenService
/// 4. Format output
pub async fn rotate_token_adapter<T, S>(
  command: &T,
  token_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TokenService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = token_handlers::rotate_token_handler( &params )?;

  let dry_run = is_dry_run( &params );

  // Extract validated token_id
  let token_id = params.get( "token_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "token_id missing after validation".to_string() )
  })?;

  if dry_run
  {
    // Dry-run: simulate output without rotating
    let mut output_data = HashMap::new();
    output_data.insert( "status".to_string(), "would rotate (dry-run)".to_string() );
    output_data.insert( "token_id".to_string(), token_id.clone() );

    let output = formatter.format_single( &output_data );
    return Ok( output );
  }

  // Extract optional new TTL
  let new_ttl = params.get( "ttl" ).and_then( |s| s.parse::<i64>().ok() );

  // Perform async token rotation
  let rotated_token = token_service.rotate( token_id, new_ttl ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "rotated".to_string() );
  output_data.insert( "token_id".to_string(), rotated_token.id.clone() );
  output_data.insert( "scope".to_string(), rotated_token.scope.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Revoke token adapter
///
/// Revokes a token (deletes it).
///
/// ## Flow
///
/// 1. Extract token_id from command
/// 2. Call revoke_token_handler() for validation
/// 3. Perform async token revocation via TokenService
/// 4. Format output
pub async fn revoke_token_adapter<T, S>(
  command: &T,
  token_service: S,
  formatter: &TreeFmtFormatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  S: TokenService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = token_handlers::revoke_token_handler( &params )?;

  let dry_run = is_dry_run( &params );

  // Extract validated token_id
  let token_id = params.get( "token_id" ).ok_or_else( || {
    AdapterError::ExtractionError( "token_id missing after validation".to_string() )
  })?;

  if dry_run
  {
    // Dry-run: simulate output without revoking
    let mut output_data = HashMap::new();
    output_data.insert( "status".to_string(), "would revoke (dry-run)".to_string() );
    output_data.insert( "token_id".to_string(), token_id.clone() );

    let output = formatter.format_single( &output_data );
    return Ok( output );
  }

  // Extract optional reason
  let reason = params.get( "reason" ).map( |s| s.as_str() );

  // Perform async token revocation
  token_service.revoke( token_id, reason ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "revoked".to_string() );
  output_data.insert( "token_id".to_string(), token_id.clone() );
  if let Some( r ) = reason
  {
    output_data.insert( "reason".to_string(), r.to_string() );
  }

  let output = formatter.format_single( &output_data );

  Ok( output )
}
