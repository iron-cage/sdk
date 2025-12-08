// qqq : implement rate limiting
//! Rate limiting service
//!
//! Token bucket algorithm for request rate limiting per user/project.

use governor::{ Quota, RateLimiter as GovernorRateLimiter };
use governor::clock::DefaultClock;
use core::num::NonZeroU32;
use core::time::Duration;
use std::sync::Arc;

/// Rate limiter key (`user_id` or `user_id:project_id`)
type LimiterKey = String;

/// Keyed rate limiter type (uses Governor's default keyed state store)
type KeyedLimiter = GovernorRateLimiter<
  LimiterKey,
  governor::state::keyed::DefaultKeyedStateStore< LimiterKey >,
  DefaultClock,
>;

/// Rate limiter
///
/// Uses token bucket algorithm for per-user/per-project rate limiting.
pub struct RateLimiter
{
  limiter: Option< Arc< KeyedLimiter > >,
  max_burst: u32,
}

impl core::fmt::Debug for RateLimiter
{
  fn fmt( &self, f: &mut core::fmt::Formatter< '_ > ) -> core::fmt::Result
  {
    f.debug_struct( "RateLimiter" )
      .field( "max_burst", &self.max_burst )
      .field( "enabled", &self.limiter.is_some() )
      .finish()
  }
}

impl Clone for RateLimiter
{
  fn clone( &self ) -> Self
  {
    Self {
      limiter: self.limiter.clone(),
      max_burst: self.max_burst,
    }
  }
}

impl RateLimiter
{
  /// Create new rate limiter
  ///
  /// # Arguments
  ///
  /// * `requests_per_period` - Number of requests allowed per period
  /// * `period` - Time period for rate limit
  ///
  /// # Returns
  ///
  /// Configured rate limiter
  ///
  /// # Panics
  ///
  /// Panics if period is invalid for quota configuration
  ///
  /// # Examples
  ///
  /// ```
  /// use iron_token_manager::rate_limiter::RateLimiter;
  /// use std::time::Duration;
  ///
  /// // 100 requests per second
  /// let limiter = RateLimiter::new( 100, Duration::from_secs( 1 ) );
  /// ```
  #[ must_use ]
  pub fn new( requests_per_period: u32, period: Duration ) -> Self
  {
    let limiter = if requests_per_period == 0 {
      // Zero quota = always reject
      None
    } else {
      let max_burst = NonZeroU32::new( requests_per_period ).expect( "Should be non-zero" );
      let quota = Quota::with_period( period )
        .expect( "Period must be valid" )
        .allow_burst( max_burst );
      Some( Arc::new( GovernorRateLimiter::keyed( quota ) ) )
    };

    Self {
      limiter,
      max_burst: requests_per_period,
    }
  }

  /// Create rate limiter key
  fn make_key( user_id: &str, project_id: Option< &str > ) -> LimiterKey
  {
    match project_id {
      Some( proj ) => format!( "{user_id}:{proj}" ),
      None => user_id.to_string(),
    }
  }

  /// Check if request is allowed under rate limit
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Returns
  ///
  /// True if request is allowed, false if rate limited
  #[ must_use ]
  pub fn check_rate_limit( &self, user_id: &str, project_id: Option< &str > ) -> bool
  {
    let Some( ref limiter ) = self.limiter else {
      // Zero quota - always reject
      return false;
    };

    let key = Self::make_key( user_id, project_id );
    limiter.check_key( &key ).is_ok()
  }

  /// Get remaining requests in current window
  ///
  /// # Arguments
  ///
  /// * `user_id` - User ID
  /// * `project_id` - Optional project ID
  ///
  /// # Returns
  ///
  /// Number of requests remaining before rate limit
  #[ must_use ]
  pub fn get_remaining_requests( &self, user_id: &str, project_id: Option< &str > ) -> u32
  {
    let Some( ref limiter ) = self.limiter else {
      // Zero quota - no remaining
      return 0;
    };

    let key = Self::make_key( user_id, project_id );

    // Governor doesn't expose direct remaining count
    // We estimate by checking without consuming
    let mut remaining = 0;
    for _ in 0..self.max_burst {
      if limiter.check_key( &key ).is_ok() {
        remaining += 1;
      } else {
        break;
      }
    }
    remaining
  }
}
