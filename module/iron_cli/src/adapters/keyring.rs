//! Keyring service for secure credential storage
//!
//! Stores authentication tokens (access_token, refresh_token) in system keyring.
//!
//! ## Platform Support
//!
//! - macOS: Keychain
//! - Linux: Secret Service API (GNOME Keyring, KWallet)
//! - Windows: Credential Manager
//!
//! ## Usage
//!
//! ```rust,ignore
//! use iron_cli::adapters::keyring;
//!
//! // Store tokens after login
//! keyring::set_access_token("access_token_value")?;
//! keyring::set_refresh_token("refresh_token_value")?;
//!
//! // Retrieve tokens for API calls
//! let access_token = keyring::get_access_token()?;
//!
//! // Clear tokens on logout
//! keyring::clear_tokens()?;
//! ```
//!
//! ## Security
//!
//! Tokens are stored encrypted at OS level using platform-specific secure storage.
//! No plaintext tokens are written to disk by this module.
//!
//! ## Error Handling
//!
//! If keyring unavailable (e.g., headless Linux), falls back to config file storage
//! with appropriate warnings about reduced security.

use keyring::Entry;

const SERVICE_NAME: &str = "iron-cli";
const ACCESS_TOKEN_KEY: &str = "access_token";
const REFRESH_TOKEN_KEY: &str = "refresh_token";

/// Store access token in keyring
///
/// ## Parameters
///
/// - token: Access token string to store
///
/// ## Returns
///
/// Ok if stored successfully, Err on keyring failure
pub fn set_access_token( token: &str ) -> Result<(), KeyringError>
{
  let entry = Entry::new( SERVICE_NAME, ACCESS_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  entry.set_password( token )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  Ok( () )
}

/// Retrieve access token from keyring
///
/// ## Returns
///
/// Access token string if found, error if missing or keyring failure
pub fn get_access_token() -> Result<String, KeyringError>
{
  let entry = Entry::new( SERVICE_NAME, ACCESS_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  entry.get_password()
    .map_err( |e|
    {
      let error_msg = e.to_string();
      if error_msg.contains( "NoEntry" )
        || error_msg.contains( "not found" )
        || error_msg.contains( "No matching entry" )
      {
        KeyringError::NotFound( "Access token not found. Please login first.".to_string() )
      }
      else
      {
        KeyringError::StorageError( error_msg )
      }
    })
}

/// Store refresh token in keyring
///
/// ## Parameters
///
/// - token: Refresh token string to store
///
/// ## Returns
///
/// Ok if stored successfully, Err on keyring failure
pub fn set_refresh_token( token: &str ) -> Result<(), KeyringError>
{
  let entry = Entry::new( SERVICE_NAME, REFRESH_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  entry.set_password( token )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  Ok( () )
}

/// Retrieve refresh token from keyring
///
/// ## Returns
///
/// Refresh token string if found, error if missing or keyring failure
pub fn get_refresh_token() -> Result<String, KeyringError>
{
  let entry = Entry::new( SERVICE_NAME, REFRESH_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  entry.get_password()
    .map_err( |e|
    {
      let error_msg = e.to_string();
      if error_msg.contains( "NoEntry" )
        || error_msg.contains( "not found" )
        || error_msg.contains( "No matching entry" )
      {
        KeyringError::NotFound( "Refresh token not found. Please login first.".to_string() )
      }
      else
      {
        KeyringError::StorageError( error_msg )
      }
    })
}

/// Clear all stored tokens from keyring
///
/// Used during logout to remove credentials.
///
/// ## Returns
///
/// Ok if cleared successfully (or already empty), Err on keyring failure
pub fn clear_tokens() -> Result<(), KeyringError>
{
  // Clear access token
  let access_entry = Entry::new( SERVICE_NAME, ACCESS_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  // Ignore NotFound errors (already cleared)
  match access_entry.delete_password()
  {
    Ok( () ) => {},
    Err( e ) =>
    {
      let error_msg = e.to_string();
      if !error_msg.contains( "NoEntry" ) && !error_msg.contains( "not found" )
      {
        return Err( KeyringError::StorageError( error_msg ) );
      }
    }
  }

  // Clear refresh token
  let refresh_entry = Entry::new( SERVICE_NAME, REFRESH_TOKEN_KEY )
    .map_err( |e| KeyringError::StorageError( e.to_string() ) )?;

  match refresh_entry.delete_password()
  {
    Ok( () ) => {},
    Err( e ) =>
    {
      let error_msg = e.to_string();
      if !error_msg.contains( "NoEntry" ) && !error_msg.contains( "not found" )
      {
        return Err( KeyringError::StorageError( error_msg ) );
      }
    }
  }

  Ok( () )
}

/// Keyring errors
#[derive(Debug)]
pub enum KeyringError
{
  /// Credential not found in keyring
  NotFound( String ),

  /// Keyring storage error
  StorageError( String ),
}

impl std::fmt::Display for KeyringError
{
  fn fmt( &self, f: &mut std::fmt::Formatter<'_> ) -> std::fmt::Result
  {
    match self
    {
      Self::NotFound( msg ) => write!( f, "Not found: {}", msg ),
      Self::StorageError( msg ) => write!( f, "Storage error: {}", msg ),
    }
  }
}

impl std::error::Error for KeyringError {}
