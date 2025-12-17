//! Error types for configuration management

use std::path::PathBuf;

/// Result type for configuration operations
pub type Result< T > = std::result::Result< T, ConfigError >;

/// Configuration error types
#[ derive( Debug, thiserror::Error ) ]
pub enum ConfigError
{
  /// Failed to detect workspace
  #[ error( "Failed to detect workspace: {0}" ) ]
  WorkspaceNotFound( String ),

  /// Config file not found
  #[ error( "Config file not found: {0}" ) ]
  FileNotFound( PathBuf ),

  /// Invalid TOML format
  #[ error( "Invalid TOML format in {path}: {error}" ) ]
  InvalidToml
  {
    /// Path to the file
    path: PathBuf,
    /// Parse error
    error: String,
  },

  /// IO error
  #[ error( "IO error: {0}" ) ]
  Io( #[ from ] std::io::Error ),

  /// Missing required key
  #[ error( "Missing required configuration key: {0}" ) ]
  MissingKey( String ),

  /// Invalid value type
  #[ error( "Invalid value type for key '{key}': expected {expected}, got {actual}" ) ]
  InvalidType
  {
    /// Configuration key
    key: String,
    /// Expected type
    expected: String,
    /// Actual type
    actual: String,
  },

  /// Environment variable parse error
  #[ error( "Failed to parse environment variable {var}: {error}" ) ]
  EnvParseError
  {
    /// Variable name
    var: String,
    /// Parse error
    error: String,
  },
}
