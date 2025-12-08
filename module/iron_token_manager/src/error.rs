//! Error types
//!
//! Error types for token management operations.

// qqq : implement proper error types using error_tools

/// Token management error type
///
/// Uses `error_tools` facade for anyhow+thiserror functionality.
#[ derive( Debug ) ]
pub struct TokenError;

impl core::fmt::Display for TokenError
{
  fn fmt( &self, f: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    write!( f, "Token management error" )
  }
}

impl core::error::Error for TokenError {}

/// Result type for token management operations
pub type Result< T > = core::result::Result< T, TokenError >;
