//! Integration tests for `RateLimiter`
//!
//! Tests token bucket rate limiting with real Governor implementation.
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_allow_requests_within_rate` | Requests within rate limit | 10 req/sec, make 10 requests | All 10 allowed | ✅ |
//! | `test_reject_requests_exceeding_rate` | Requests exceed rate limit | 5 req/sec, make 6 requests | First 5 allowed, 6th denied | ✅ |
//! | `test_rate_limit_recovery_over_time` | Rate limit refills over time | 2 req/100ms, exhaust, wait 150ms | Quota refilled, request allowed | ✅ |
//! | `test_per_user_isolation` | Users have independent quotas | 3 req/sec, `user_004` exhausts, check `user_005` | `user_005` has quota | ✅ |
//! | `test_project_level_rate_limiting` | Projects have independent quotas | Same user, `project_alpha` exhausts, check `project_beta` | `project_beta` has quota | ✅ |
//! | `test_get_remaining_quota` | Quota tracking | 10 req/sec, use 3 requests | 7 remaining | ✅ |
//! | `test_burst_handling` | Burst capacity | 10 req/sec, make 10 immediately | All 10 allowed, 11th denied | ✅ |
//! | `test_zero_quota_always_rejects` | Zero quota = disabled | 0 req/sec | Always denies | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ Requests within rate limit (10/10 allowed)
//! - ✅ Burst handling (immediate burst of 10 requests)
//! - ✅ Quota tracking (remaining requests)
//!
//! **Boundary Conditions:**
//! - ✅ Exactly at limit (5/5 allowed, 6th denied)
//! - ✅ Zero quota (0 req/sec → always deny)
//! - ✅ Quota refill (wait period → quota restored)
//!
//! **Error Conditions:**
//! - ✅ Exceeding rate limit (6th request > 5/sec → deny)
//! - ✅ Exhausted quota (burst of 10 → 11th denied)
//!
//! **Edge Cases:**
//! - ✅ Per-user isolation (`user_004` exhausted ≠ `user_005` exhausted)
//! - ✅ Project-level isolation (same user, different projects = independent quotas)
//! - ✅ Time-based recovery (quota refills after waiting)
//! - ✅ Zero quota edge case (always rejects)
//!
//! **State Transitions:**
//! - ✅ Full quota → Partially used → Exhausted → Wait → Refilled → Allow
//! - ✅ User quota independent from other users
//! - ✅ Project quota independent from other projects
//!
//! **Concurrent Access:** Not tested (Governor handles thread-safety internally)
//! **Resource Limits:** Not applicable (in-memory token bucket, bounded by configuration)
//! **Precondition Violations:** Not applicable (`RateLimiter` validates configuration, handles zero quota gracefully)

use iron_token_manager::rate_limiter::RateLimiter;
use core::time::Duration;

#[ tokio::test ]
async fn test_allow_requests_within_rate()
{
  // 10 requests per second
  let limiter = RateLimiter::new( 10, Duration::from_secs( 1 ) );

  // First 10 requests should succeed
  for _ in 0..10 {
    let allowed = limiter.check_rate_limit( "user_001", None );
    assert!( allowed, "Should allow requests within rate limit" );
  }
}

#[ tokio::test ]
async fn test_reject_requests_exceeding_rate()
{
  // 5 requests per second
  let limiter = RateLimiter::new( 5, Duration::from_secs( 1 ) );

  // First 5 should succeed
  for _ in 0..5 {
    assert!( limiter.check_rate_limit( "user_002", None ) );
  }

  // 6th should fail
  let allowed = limiter.check_rate_limit( "user_002", None );
  assert!( !allowed, "Should reject request exceeding rate limit" );
}

#[ tokio::test ]
async fn test_rate_limit_recovery_over_time()
{
  // 2 requests per 100ms
  let limiter = RateLimiter::new( 2, Duration::from_millis( 100 ) );

  // Use both requests
  assert!( limiter.check_rate_limit( "user_003", None ) );
  assert!( limiter.check_rate_limit( "user_003", None ) );

  // Should be exhausted
  assert!( !limiter.check_rate_limit( "user_003", None ) );

  // Wait for replenishment
  tokio::time::sleep( Duration::from_millis( 150 ) ).await;

  // Should allow again
  let allowed = limiter.check_rate_limit( "user_003", None );
  assert!( allowed, "Rate limit should recover over time" );
}

#[ tokio::test ]
async fn test_per_user_isolation()
{
  // 3 requests per second
  let limiter = RateLimiter::new( 3, Duration::from_secs( 1 ) );

  // User 1 exhausts their quota
  for _ in 0..3 {
    assert!( limiter.check_rate_limit( "user_004", None ) );
  }
  assert!( !limiter.check_rate_limit( "user_004", None ) );

  // User 2 should have independent quota
  let allowed = limiter.check_rate_limit( "user_005", None );
  assert!( allowed, "Different users should have independent rate limits" );
}

#[ tokio::test ]
async fn test_project_level_rate_limiting()
{
  // 5 requests per second
  let limiter = RateLimiter::new( 5, Duration::from_secs( 1 ) );

  // Same user, different projects should have independent limits
  for _ in 0..5 {
    assert!( limiter.check_rate_limit( "user_006", Some( "project_alpha" ) ) );
  }
  assert!( !limiter.check_rate_limit( "user_006", Some( "project_alpha" ) ) );

  // Different project should still have quota
  let allowed = limiter.check_rate_limit( "user_006", Some( "project_beta" ) );
  assert!( allowed, "Different projects should have independent rate limits" );
}

#[ tokio::test ]
async fn test_get_remaining_quota()
{
  // 10 requests per second
  let limiter = RateLimiter::new( 10, Duration::from_secs( 1 ) );

  // Use 3 requests
  for _ in 0..3 {
    let _ = limiter.check_rate_limit( "user_007", None );
  }

  let remaining = limiter.get_remaining_requests( "user_007", None );
  assert_eq!( remaining, 7, "Should have 7 requests remaining" );
}

#[ tokio::test ]
async fn test_burst_handling()
{
  // 10 requests per second with burst capacity
  let limiter = RateLimiter::new( 10, Duration::from_secs( 1 ) );

  // Should allow burst of 10 requests immediately
  for i in 0..10 {
    let allowed = limiter.check_rate_limit( "user_008", None );
    assert!( allowed, "Should allow burst request {i}" );
  }

  // 11th should fail (burst exhausted)
  assert!( !limiter.check_rate_limit( "user_008", None ) );
}

#[ tokio::test ]
async fn test_zero_quota_always_rejects()
{
  // 0 requests per second (disabled)
  let limiter = RateLimiter::new( 0, Duration::from_secs( 1 ) );

  let allowed = limiter.check_rate_limit( "user_009", None );
  assert!( !allowed, "Zero quota should always reject" );
}
