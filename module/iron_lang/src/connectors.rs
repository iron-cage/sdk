//! Connector trait and common utilities.

/// Connector trait that all data source connectors must implement.
pub trait Connector : Send + Sync
{
  /// Get connector name.
  fn name( &self ) -> &str;
}
