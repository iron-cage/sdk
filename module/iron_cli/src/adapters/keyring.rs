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

#[cfg(test)]
mod tests
{
  use super::*;

  #[test]
  fn test_token_roundtrip()
  {
    // Clean state
    let _ = clear_tokens();

    // Store tokens
    let access = "test_access_token_12345";
    let refresh = "test_refresh_token_67890";

    set_access_token( access ).expect( "Failed to store access token" );
    set_refresh_token( refresh ).expect( "Failed to store refresh token" );

    // Retrieve tokens
    let retrieved_access = get_access_token().expect( "Failed to retrieve access token" );
    let retrieved_refresh = get_refresh_token().expect( "Failed to retrieve refresh token" );

    assert_eq!( retrieved_access, access );
    assert_eq!( retrieved_refresh, refresh );

    // Clear tokens
    clear_tokens().expect( "Failed to clear tokens" );

    // Verify cleared
    assert!( get_access_token().is_err() );
    assert!( get_refresh_token().is_err() );
  }

  #[test]
  fn test_get_missing_token()
  {
    // Ensure clean state
    let _ = clear_tokens();

    // Should error on missing token
    let result = get_access_token();
    assert!( result.is_err() );

    match result
    {
      Err( KeyringError::NotFound( msg ) ) =>
      {
        assert!( msg.contains( "not found" ) );
      }
      Err( KeyringError::StorageError( msg ) ) =>
      {
        panic!( "Expected NotFound error, got StorageError: {}", msg );
      }
      Ok( _ ) => panic!( "Expected error, got Ok" ),
    }
  }
}
