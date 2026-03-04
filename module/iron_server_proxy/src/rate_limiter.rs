//! Per-IP auth rate limiter for the proxy endpoint.
//!
//! Sliding window algorithm: tracks timestamps of failed auth attempts per client IP.
//! After [`MAX_AUTH_FAILURES`] failures within [`AUTH_WINDOW`], subsequent requests
//! are rejected with 429 until the oldest failure expires.
//!
//! Includes LRU eviction at [`MAX_ENTRIES`] to prevent memory exhaustion from
//! spoofed IP headers.

use core::time::Duration;
use std::sync::PoisonError;
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
  time::Instant,
};

/// Maximum failed auth attempts before blocking.
pub const MAX_AUTH_FAILURES: usize = 20;

/// Sliding window duration.
const AUTH_WINDOW: Duration = Duration::from_secs(60);

/// Hard cap on tracked IPs to prevent memory exhaustion.
const MAX_ENTRIES: usize = 100_000;

/// Per-IP sliding window rate limiter for auth failures.
///
/// Thread-safe via `Arc<Mutex<_>>`. Cloning shares the same state.
#[derive(Clone, Debug)]
pub struct AuthRateLimiter {
  failures: Arc<Mutex<HashMap<String, Vec<Instant>>>>,
  window: Duration,
}

impl AuthRateLimiter {
  /// Create a new rate limiter with default window.
  #[must_use]
  pub fn new() -> Self {
    Self {
      failures: Arc::new(Mutex::new(HashMap::new())),
      window: AUTH_WINDOW,
    }
  }

  /// Create a rate limiter with a custom window (useful for testing).
  #[must_use]
  pub fn with_window(window: Duration) -> Self {
    Self {
      failures: Arc::new(Mutex::new(HashMap::new())),
      window,
    }
  }

  /// Lock mutex, sweep expired entries, and enforce LRU eviction.
  fn lock_and_sweep(&self) -> std::sync::MutexGuard<'_, HashMap<String, Vec<Instant>>> {
    let mut map = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
    // Remove expired timestamps (skip if process uptime < window)
    if let Some(cutoff) = Instant::now().checked_sub(self.window) {
      map.retain(|_, timestamps| {
        timestamps.retain(|t| *t > cutoff);
        !timestamps.is_empty()
      });
    }

    // LRU eviction if over hard cap
    if map.len() > MAX_ENTRIES {
      let excess = map.len() - MAX_ENTRIES;
      let mut keys_by_age: Vec<(String, Instant)> = map
        .iter()
        .filter_map(|(key, ts)| ts.first().map(|t| (key.clone(), *t)))
        .collect();
      keys_by_age.sort_unstable_by_key(|(_, t)| *t);
      for (key, _) in keys_by_age.into_iter().take(excess) {
        map.remove(&key);
      }
    }

    map
  }

  /// Check if the IP is currently rate-limited.
  ///
  /// Returns `Ok(())` if allowed.
  ///
  /// # Errors
  ///
  /// Returns `Err(retry_after_secs)` if the IP has exceeded the failure threshold.
  pub fn check(&self, ip: &str) -> Result<(), u64> {
    let map = self.lock_and_sweep();
    let Some(timestamps) = map.get(ip) else {
      return Ok(());
    };
    if timestamps.len() >= MAX_AUTH_FAILURES {
      let retry_after = timestamps.first().map_or(1, |oldest| {
        let expires_at = *oldest + self.window;
        expires_at
          .saturating_duration_since(Instant::now())
          .as_secs()
          + 1
      });
      Err(retry_after)
    } else {
      Ok(())
    }
  }

  /// Record a failed auth attempt for the given IP.
  pub fn record_failure(&self, ip: &str) {
    let mut map = self.lock_and_sweep();
    map.entry(ip.to_string()).or_default().push(Instant::now());
  }
}

impl Default for AuthRateLimiter {
  fn default() -> Self {
    Self::new()
  }
}
