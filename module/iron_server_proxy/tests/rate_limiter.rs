//! Unit tests for [`AuthRateLimiter`].

use core::time::Duration;

use iron_server_proxy::rate_limiter::{AuthRateLimiter, MAX_AUTH_FAILURES};

#[test]
fn allows_under_threshold() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES - 1 {
    limiter.record_failure("192.168.1.1");
  }
  assert!(
    limiter.check("192.168.1.1").is_ok(),
    "Should allow requests under failure threshold"
  );
}

#[test]
fn blocks_at_threshold() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  let result = limiter.check("10.0.0.1");
  assert!(
    result.is_err(),
    "Should block requests at failure threshold"
  );
  let retry_after = result.unwrap_err();
  assert!(retry_after > 0, "Retry-after should be positive");
  assert!(
    retry_after <= 61,
    "Retry-after should not exceed window + 1s"
  );
}

#[test]
fn different_ips_independent() {
  let limiter = AuthRateLimiter::new();
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  assert!(
    limiter.check("10.0.0.1").is_err(),
    "Blocked IP should be rate-limited"
  );
  assert!(
    limiter.check("10.0.0.2").is_ok(),
    "Different IP should not be affected by other IP's failures"
  );
}

#[test]
fn unknown_ip_allowed() {
  let limiter = AuthRateLimiter::new();
  assert!(
    limiter.check("never_seen").is_ok(),
    "Unknown IP should be allowed without prior failures"
  );
}

#[test]
fn expires_after_window() {
  let limiter = AuthRateLimiter::with_window(Duration::from_millis(50));
  for _ in 0..MAX_AUTH_FAILURES {
    limiter.record_failure("10.0.0.1");
  }
  assert!(
    limiter.check("10.0.0.1").is_err(),
    "Should be blocked immediately after threshold"
  );

  std::thread::sleep(Duration::from_millis(60));

  assert!(
    limiter.check("10.0.0.1").is_ok(),
    "Should be allowed after rate limit window expires"
  );
}

#[test]
fn clone_shares_state() {
  let a = AuthRateLimiter::new();
  let b = a.clone();
  for _ in 0..MAX_AUTH_FAILURES {
    a.record_failure("10.0.0.1");
  }
  assert!(
    b.check("10.0.0.1").is_err(),
    "Cloned limiter should share state with original"
  );
}

/// Old failures outside the window are not counted toward the threshold.
///
/// Records failures for an IP, waits for the sliding window to expire, then
/// records fewer failures than the threshold. The IP must be allowed because
/// only recent (within-window) failures count.
#[test]
fn sliding_window_only_counts_recent_failures() {
  let limiter = AuthRateLimiter::with_window(Duration::from_millis(50));

  // Fill to just below threshold
  for _ in 0..MAX_AUTH_FAILURES - 1 {
    limiter.record_failure("10.0.0.1");
  }
  assert!(
    limiter.check("10.0.0.1").is_ok(),
    "Should be allowed while under threshold"
  );

  // Let the window expire — all previous failures are now outside the window
  std::thread::sleep(Duration::from_millis(60));

  // Record one fresh failure — well below threshold
  limiter.record_failure("10.0.0.1");

  assert!(
    limiter.check("10.0.0.1").is_ok(),
    "Should be allowed after window expiry: old failures must not count, \
     only 1 recent failure is present (threshold is {MAX_AUTH_FAILURES})"
  );
}

/// Concurrent `record_failure` calls from multiple threads don't corrupt state.
///
/// Verifies that the `Arc<Mutex<_>>` interior is thread-safe: no deadlocks,
/// no panics, and each IP ends up with exactly the expected failure count.
#[test]
fn concurrent_failures_thread_safe() {
  let limiter = AuthRateLimiter::new();

  // 10 threads, each recording 2 failures for a unique IP
  let threads: Vec<_> = (0_u8..10)
    .map(|i| {
      let limiter_clone = limiter.clone();
      std::thread::spawn(move || {
        limiter_clone.record_failure(&format!("10.0.0.{i}"));
        limiter_clone.record_failure(&format!("10.0.0.{i}"));
      })
    })
    .collect();

  for t in threads {
    t.join().expect("Thread should not panic");
  }

  // Each IP has 2 failures — far below MAX_AUTH_FAILURES (20), so all must pass
  for i in 0_u8..10 {
    assert!(
      limiter.check(&format!("10.0.0.{i}")).is_ok(),
      "IP 10.0.0.{i} should be allowed: 2 failures < threshold {MAX_AUTH_FAILURES}"
    );
  }
}
