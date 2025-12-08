//! Message processing runtime engine.

/// Runtime configuration.
#[ derive( Debug, Clone ) ]
pub struct RuntimeConfig
{
  /// Maximum concurrent connections
  pub max_connections : usize,
}

impl Default for RuntimeConfig
{
  fn default() -> Self
  {
    Self { max_connections : 10 }
  }
}
