//! Request routing and dispatch.

/// Request router.
#[ derive( Debug ) ]
pub struct Router
{
  /// Placeholder
  _phantom : (),
}

impl Router
{
  /// Create new router.
  pub fn new() -> Self
  {
    Self { _phantom : () }
  }
}

impl Default for Router
{
  fn default() -> Self
  {
    Self::new()
  }
}
