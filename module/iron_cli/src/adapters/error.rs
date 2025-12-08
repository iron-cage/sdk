//! Adapter layer error types
//!
//! Adapters bridge the CLI layer to business logic handlers and async services.
//! This module defines the error types for adapter operations.

use std::fmt;
use crate::handlers::CliError;

/// Adapter layer errors
#[ derive( Debug ) ]
pub enum AdapterError
{
  /// Error from handler validation (business logic)
  HandlerError( CliError ),

  /// Error from service operations (I/O, database, API)
  ServiceError( ServiceError ),

  /// Error extracting parameters from VerifiedCommand
  ExtractionError( String ),

  /// Error formatting output
  FormattingError( String ),
}

/// Service layer errors (async I/O operations)
#[ derive( Debug, Clone ) ]
pub enum ServiceError
{
  /// Resource not found
  NotFound,

  /// Authentication failed
  Unauthorized,

  /// Permission denied
  Forbidden,

  /// Resource already exists
  Conflict,

  /// Network/HTTP error
  NetworkError( String ),

  /// Database error
  DatabaseError( String ),

  /// Storage error (file, keyring, etc.)
  StorageError( String ),

  /// Validation error (should be caught by handlers, but here for completeness)
  ValidationError( String ),
}

impl fmt::Display for AdapterError
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    match self
    {
      Self::HandlerError( e ) => write!( f, "Handler error: {}", e ),
      Self::ServiceError( e ) => write!( f, "Service error: {}", e ),
      Self::ExtractionError( msg ) => write!( f, "Parameter extraction error: {}", msg ),
      Self::FormattingError( msg ) => write!( f, "Formatting error: {}", msg ),
    }
  }
}

impl fmt::Display for ServiceError
{
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result
  {
    match self
    {
      Self::NotFound => write!( f, "Resource not found" ),
      Self::Unauthorized => write!( f, "Authentication failed" ),
      Self::Forbidden => write!( f, "Permission denied" ),
      Self::Conflict => write!( f, "Resource already exists" ),
      Self::NetworkError( msg ) => write!( f, "Network error: {}", msg ),
      Self::DatabaseError( msg ) => write!( f, "Database error: {}", msg ),
      Self::StorageError( msg ) => write!( f, "Storage error: {}", msg ),
      Self::ValidationError( msg ) => write!( f, "Validation error: {}", msg ),
    }
  }
}

impl std::error::Error for AdapterError {}
impl std::error::Error for ServiceError {}

impl From<CliError> for AdapterError
{
  fn from(e: CliError) -> Self
  {
    Self::HandlerError( e )
  }
}

impl From<ServiceError> for AdapterError
{
  fn from(e: ServiceError) -> Self
  {
    Self::ServiceError( e )
  }
}
