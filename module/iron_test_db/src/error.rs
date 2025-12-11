//! Error types for test database operations

use thiserror::Error;

/// Result type for test database operations
pub type Result< T > = std::result::Result< T, TestDbError >;

/// Errors that can occur during test database operations
#[ derive( Debug, Error ) ]
pub enum TestDbError
{
  /// SQLx database error
  #[ error( "Database error: {0}" ) ]
  Database( #[ from ] sqlx::Error ),

  /// IO error (file operations, TempDir creation)
  #[ error( "IO error: {0}" ) ]
  Io( #[ from ] std::io::Error ),

  /// Configuration error
  #[ error( "Configuration error: {0}" ) ]
  Configuration( String ),

  /// Migration error
  #[ error( "Migration error: {0}" ) ]
  Migration( String ),

  /// Dependency cycle detected during table wiping
  #[ error( "Dependency cycle detected: {0}" ) ]
  DependencyCycle( String ),
}
