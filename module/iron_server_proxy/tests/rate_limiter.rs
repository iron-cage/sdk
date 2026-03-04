//! Unit tests for [`AuthRateLimiter`].

use core::time::Duration;

use iron_server_proxy::rate_limiter::{AuthRateLimiter, MAX_AUTH_FAILURES};

#[test]
fn allows_under_threshold() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES - 1 {
    limiter.record_failure("192.168.1.1");
  }
  assert!(limiter.check("192.168.1.1").is_ok());
}

#[test]
fn blocks_at_threshold() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  let result = limiter.check("10.0.0.1");
  assert!(result.is_err());
  let retry_after = result.unwrap_err();
  assert!(retry_after > 0);
  assert!(retry_after <= 61);
}

#[test]
fn different_ips_independent() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  assert!(limiter.check("10.0.0.1").is_err());
  assert!(limiter.check("10.0.0.2").is_ok());
}

#[test]
fn unknown_ip_allowed() {
  let limiter = AuthRateLimiter::new();
  assert!(limiter.check("never_seen").is_ok());
}

#[test]
fn expires_after_window() {
  let limiter = AuthRateLimiter::with_window(Duration::from_millis(50));
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  assert!(limiter.check("10.0.0.1").is_err());

  std::thread::sleep(Duration::from_millis(60));

  assert!(limiter.check("10.0.0.1").is_ok());
}

#[test]
fn clone_shares_state() {
  let a = AuthRateLimiter::new();
  let b = a.clone();
  for _ in 0..MAX_AUTH_FAILURES {
    a.record_failure("10.0.0.1");
  }
  assert!(b.check("10.0.0.1").is_err());
}
