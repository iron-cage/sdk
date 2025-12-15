//! Error types
//!
//! Error types for token management operations.

// qqq : implement proper error types using error_tools

/// Token management error type
///
/// Fix(issue-001): Added Database variant to preserve underlying sqlx errors
/// for proper FK constraint handling
///
/// Root cause: Original `TokenError` was a unit struct that discarded all error
/// details from `SQLite`, making it impossible to distinguish FK constraint
/// violations from other database errors
///
/// Pitfall: Never discard error details when converting between error types.
/// Always preserve the underlying cause using enum variants or error wrapping
/// so handlers can make informed decisions about error responses
#[ derive( Debug ) ]
pub enum TokenError
{
  /// Generic token management error
  Generic,
  /// Database error preserving sqlx details for FK constraint detection
  Database( sqlx::Error ),
}

impl core::fmt::Display for TokenError
{
  fn fmt( &self, f: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    match self
    {
      Self::Generic => write!( f, "Token management error" ),
      Self::Database( e ) => write!( f, "Database error: {e}" ),
    }
  }
}

impl core::error::Error for TokenError {}

/// Result type for token management operations
pub type Result< T > = core::result::Result< T, TokenError >;
