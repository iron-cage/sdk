//! Authentication adapters
//!
//! Bridge unilang CLI to auth handlers and services.
//!
//! ## Architecture
//!
//! ```text
//! VerifiedCommand → login_adapter() → login_handler() → AuthService::login()
//!       ↓                  ↓                  ↓                    ↓
//!   Extract params    Call handler      Validate params    Async I/O (DB/API)
//! ```
//!
//! ## Adapter Responsibilities
//!
//! 1. Extract parameters from command
//! 2. Call handler for validation (pure, sync)
//! 3. Perform async I/O via services
//! 4. Store results (if not dry-run)
//! 5. Format output

use super::{ AdapterError, ServiceError };
use super::services::{ AuthService, StorageService };
use crate::handlers::auth_handlers;
use crate::formatting::Formatter;
use std::collections::HashMap;

/// Extract parameters from mock VerifiedCommand
///
/// TODO: Replace with real unilang VerifiedCommand when available
fn extract_params<T>(command: &T) -> HashMap<String, String>
where
  T: HasParams,
{
  command.get_params()
}

/// Temporary trait for parameter extraction (until unilang types available)
pub trait HasParams
{
  fn get_params(&self) -> HashMap<String, String>;
}

/// Check if dry-run mode enabled
fn is_dry_run(params: &HashMap<String, String>) -> bool
{
  params.get( "dry_run" )
    .and_then( |v| v.parse::<bool>().ok() )
    .unwrap_or( false )
}

/// Login adapter
///
/// Validates credentials, performs authentication, stores tokens.
///
/// ## Flow
///
/// 1. Extract username/password from command
/// 2. Call login_handler() for validation
/// 3. Perform async login via AuthService (stores tokens internally)
/// 4. Format output
pub async fn login_adapter<T, A>(
  command: &T,
  auth_service: A,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  A: AuthService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = auth_handlers::login_handler( &params )?;

  // Extract validated parameters
  let username = params.get( "username" ).ok_or_else( || {
    AdapterError::ExtractionError( "username missing after validation".to_string() )
  })?;

  let password = params.get( "password" ).ok_or_else( || {
    AdapterError::ExtractionError( "password missing after validation".to_string() )
  })?;

  // Perform async authentication
  let tokens = auth_service.login( username, password ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "success".to_string() );
  output_data.insert( "user".to_string(), username.clone() );
  output_data.insert( "access_token".to_string(), tokens.access_token.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Refresh adapter
///
/// Refreshes access token using stored refresh token.
///
/// ## Flow
///
/// 1. Load refresh token from storage
/// 2. Call refresh_handler() for validation
/// 3. Perform async refresh via AuthService
/// 4. Store new tokens (if not dry-run)
/// 5. Format output
pub async fn refresh_adapter<T, A, S>(
  command: &T,
  auth_service: A,
  storage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  A: AuthService,
  S: StorageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = auth_handlers::refresh_handler( &params )?;

  // Load tokens from storage
  let stored_tokens = storage_service.load_tokens().await?
    .ok_or( ServiceError::NotFound )?;

  let dry_run = is_dry_run( &params );

  if dry_run
  {
    // Dry-run: simulate output without performing refresh
    let mut output_data = HashMap::new();
    output_data.insert( "status".to_string(), "refreshed (dry-run)".to_string() );
    output_data.insert( "access_token".to_string(), stored_tokens.access_token.clone() );

    let output = formatter.format_single( &output_data );
    return Ok( output );
  }

  // Perform async refresh (not dry-run)
  let new_tokens = auth_service.refresh( &stored_tokens.refresh_token ).await?;

  // Store new tokens
  storage_service.save_tokens( &new_tokens ).await?;

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), "refreshed".to_string() );
  output_data.insert( "access_token".to_string(), new_tokens.access_token.clone() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}

/// Logout adapter
///
/// Clears authentication tokens from storage.
///
/// ## Flow
///
/// 1. Call logout_handler() for validation
/// 2. Load tokens (if any)
/// 3. Perform async logout via AuthService (if logged in)
/// 4. Clear storage (if not dry-run)
/// 5. Format output
pub async fn logout_adapter<T, A, S>(
  command: &T,
  auth_service: A,
  storage_service: S,
  formatter: &Formatter,
) -> Result<String, AdapterError>
where
  T: HasParams,
  A: AuthService,
  S: StorageService,
{
  // Extract parameters
  let params = extract_params( command );

  // Call handler for validation (pure, sync)
  let _ = auth_handlers::logout_handler( &params )?;

  let dry_run = is_dry_run( &params );

  // Load tokens (might not exist if already logged out)
  let tokens_opt = storage_service.load_tokens().await?;

  let message = if let Some( tokens ) = tokens_opt
  {
    if dry_run
    {
      // Dry-run: simulate logout without clearing tokens
      "Logout successful (dry-run - no changes made)"
    }
    else
    {
      // Perform async logout
      auth_service.logout( &tokens.access_token ).await?;

      // Clear storage
      storage_service.clear_tokens().await?;

      "Logout successful"
    }
  }
  else
  {
    "Already logged out (not logged in)"
  };

  // Format output
  let mut output_data = HashMap::new();
  output_data.insert( "status".to_string(), message.to_string() );

  let output = formatter.format_single( &output_data );

  Ok( output )
}
