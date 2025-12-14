//! In-memory rate limiter for login endpoint
//!
//! **Authority:** pilot_implementation_gaps.md § GAP-006
//!
//! Implements per-IP rate limiting to prevent brute-force attacks.
//!
//! # Configuration
//!
//! - **Limit:** 5 attempts per 5 minutes per IP address
//! - **Storage:** In-memory HashMap (pilot phase)
//! - **Cleanup:** Automatic expiration of old entries
//! - **Response:** 429 Too Many Requests with Retry-After header
//!
//! # Future Enhancements
//!
//! Post-pilot: Replace with Redis for distributed deployment

use std::
{
  collections::HashMap,
  net::IpAddr,
  sync::{ Arc, Mutex },
  time::{ Duration, Instant },
};

/// Rate limiter configuration
const MAX_ATTEMPTS: usize = 5;
const WINDOW_DURATION: Duration = Duration::from_secs( 300 ); // 5 minutes

/// Login attempt record
#[ derive( Debug, Clone ) ]
struct AttemptRecord
{
  timestamp: Instant,
}

/// In-memory rate limiter for login attempts
///
/// Tracks login attempts per IP address using a sliding window approach.
/// Thread-safe using Arc<Mutex<>> for concurrent access.
#[ derive( Clone ) ]
pub struct LoginRateLimiter
{
  attempts: Arc< Mutex< HashMap< IpAddr, Vec< AttemptRecord > > > >,
}

impl LoginRateLimiter
{
  /// Create new rate limiter
  pub fn new() -> Self
  {
    Self
    {
      attempts: Arc::new( Mutex::new( HashMap::new() ) ),
    }
  }

  /// Check if IP address is allowed to attempt login
  ///
  /// Returns:
  /// - Ok(()) if allowed (< 5 attempts in last 5 minutes)
  /// - Err(retry_after_seconds) if rate limited
  ///
  /// # Arguments
  ///
  /// * `ip` - IP address to check
  pub fn check_and_record( &self, ip: IpAddr ) -> Result< (), u64 >
  {
    let mut attempts = self.attempts.lock().unwrap();
    let now = Instant::now();

    // Get or create attempt history for this IP
    let ip_attempts = attempts.entry( ip ).or_default();

    // Remove expired attempts (older than 5 minutes)
    ip_attempts.retain( |attempt| now.duration_since( attempt.timestamp ) < WINDOW_DURATION );

    // Check if rate limit exceeded
    if ip_attempts.len() >= MAX_ATTEMPTS
    {
      // Calculate when the oldest attempt will expire
      if let Some( oldest ) = ip_attempts.first()
      {
        let elapsed = now.duration_since( oldest.timestamp );
        let retry_after = WINDOW_DURATION.saturating_sub( elapsed ).as_secs();
        return Err( retry_after.max( 1 ) ); // At least 1 second
      }
    }

    // Record this attempt
    ip_attempts.push( AttemptRecord { timestamp: now } );

    Ok(())
  }

  /// Clear all rate limit data (for testing)
  #[ cfg( test ) ]
  pub fn clear( &self )
  {
    let mut attempts = self.attempts.lock().unwrap();
    attempts.clear();
  }
}

impl Default for LoginRateLimiter
{
  fn default() -> Self
  {
    Self::new()
  }
}

#[ cfg( test ) ]
mod tests
{
  use super::*;
  use std::{ net::Ipv4Addr, time::Duration };

  #[ test ]
  fn test_rate_limiter_allows_initial_attempts()
  {
    let limiter = LoginRateLimiter::new();
    let ip = IpAddr::V4( Ipv4Addr::new( 192, 168, 1, 1 ) );

    // First 5 attempts should succeed
    for i in 0..5
    {
      assert!(
        limiter.check_and_record( ip ).is_ok(),
        "Attempt {} should be allowed", i + 1
      );
    }
  }

  #[ test ]
  fn test_rate_limiter_blocks_after_max_attempts()
  {
    let limiter = LoginRateLimiter::new();
    let ip = IpAddr::V4( Ipv4Addr::new( 192, 168, 1, 2 ) );

    // Use up all 5 attempts
    for _ in 0..5
    {
      limiter.check_and_record( ip ).unwrap();
    }

    // 6th attempt should be blocked
    assert!(
      limiter.check_and_record( ip ).is_err(),
      "6th attempt should be blocked"
    );
  }

  #[ test ]
  fn test_rate_limiter_per_ip_isolation()
  {
    let limiter = LoginRateLimiter::new();
    let ip1 = IpAddr::V4( Ipv4Addr::new( 192, 168, 1, 3 ) );
    let ip2 = IpAddr::V4( Ipv4Addr::new( 192, 168, 1, 4 ) );

    // Use up all attempts for IP1
    for _ in 0..5
    {
      limiter.check_and_record( ip1 ).unwrap();
    }

    // IP1 should be blocked
    assert!( limiter.check_and_record( ip1 ).is_err(), "IP1 should be blocked" );

    // IP2 should still be allowed
    assert!( limiter.check_and_record( ip2 ).is_ok(), "IP2 should be allowed" );
  }

  #[ test ]
  fn test_rate_limiter_expiration()
  {
    let limiter = LoginRateLimiter::new();
    let ip = IpAddr::V4( Ipv4Addr::new( 192, 168, 1, 5 ) );

    // Manually insert old attempts
    {
      let mut attempts = limiter.attempts.lock().unwrap();
      let old_time = Instant::now() - Duration::from_secs( 301 ); // 5 minutes + 1 second ago
      attempts.insert(
        ip,
        vec![
          AttemptRecord { timestamp: old_time },
          AttemptRecord { timestamp: old_time },
          AttemptRecord { timestamp: old_time },
          AttemptRecord { timestamp: old_time },
          AttemptRecord { timestamp: old_time },
        ],
      );
    }

    // Old attempts should be expired, so new attempt should succeed
    assert!(
      limiter.check_and_record( ip ).is_ok(),
      "Expired attempts should not count"
    );
  }
}
