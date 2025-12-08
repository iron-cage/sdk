//! Authentication and authorization.

/// Authentication manager.
#[ derive( Debug ) ]
pub struct AuthManager
{
  /// Placeholder
  _phantom : (),
}

impl AuthManager
{
  /// Create new auth manager.
  pub fn new() -> Self
  {
    Self { _phantom : () }
  }
}

impl Default for AuthManager
{
  fn default() -> Self
  {
    Self::new()
  }
}
