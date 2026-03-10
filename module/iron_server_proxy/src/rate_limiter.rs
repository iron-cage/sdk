//! Per-IP auth rate limiter for the proxy endpoint.
//!
//! Sliding window algorithm: tracks timestamps of failed auth attempts per client IP.
//! After [`MAX_AUTH_FAILURES`] failures within [`AUTH_WINDOW`], subsequent requests
//! are rejected with 429 until the oldest failure expires.
//!
//! Uses an [`lru::LruCache`] bounded to [`MAX_ENTRIES`] so that eviction of the
//! least-recently-used entry is O(1) rather than the O(n log n) sort a plain
//! `HashMap` requires.

use core::num::NonZeroUsize;
use core::time::Duration;
use std::sync::PoisonError;
use std::{
  sync::{Arc, Mutex},
  time::Instant,
};

use lru::LruCache;

/// Maximum failed auth attempts before blocking.
pub const MAX_AUTH_FAILURES: usize = 20;

/// Sliding window duration.
const AUTH_WINDOW: Duration = Duration::from_secs(60);

/// Hard cap on tracked IPs to prevent memory exhaustion.
const MAX_ENTRIES: NonZeroUsize = const { NonZeroUsize::new(100_000).unwrap() };

/// Per-IP sliding window rate limiter for auth failures.
///
/// Thread-safe via `Arc<Mutex<_>>`. Cloning shares the same state.
#[derive(Clone, Debug)]
pub struct AuthRateLimiter {
  failures: Arc<Mutex<LruCache<String, Vec<Instant>>>>,
  window: Duration,
}

impl AuthRateLimiter {
  /// Create a new rate limiter with default window.
  #[must_use]
  pub fn new() -> Self {
    Self {
      failures: Arc::new(Mutex::new(LruCache::new(MAX_ENTRIES))),
      window: AUTH_WINDOW,
    }
  }

  /// Create a rate limiter with a custom window (useful for testing).
  #[must_use]
  pub fn with_window(window: Duration) -> Self {
    Self {
      failures: Arc::new(Mutex::new(LruCache::new(MAX_ENTRIES))),
      window,
    }
  }

  /// Lock mutex and sweep expired timestamps.
  fn lock_and_sweep(&self) -> std::sync::MutexGuard<'_, LruCache<String, Vec<Instant>>> {
    let mut cache = self.failures.lock().unwrap_or_else(PoisonError::into_inner);
    // Remove expired timestamps (skip if process uptime < window)
    if let Some(cutoff) = Instant::now().checked_sub(self.window) {
      let keys: Vec<String> = cache
        .iter()
        .map(|(k, _)| k.clone())
        .collect();
      for key in keys {
        if let Some(timestamps) = cache.get_mut(&key) {
          timestamps.retain(|t| *t > cutoff);
          if timestamps.is_empty() {
            cache.pop(&key);
          }
        }
      }
    }
    cache
  }

  /// Check if the IP is currently rate-limited.
  ///
  /// Returns `Ok(())` if allowed.
  ///
  /// # Errors
  ///
  /// Returns `Err(retry_after_secs)` if the IP has exceeded the failure threshold.
  pub fn check(&self, ip: &str) -> Result<(), u64> {
    let mut cache = self.lock_and_sweep();
    let Some(timestamps) = cache.get(ip) else {
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
  ///
  /// If the cache is at capacity, the least-recently-used entry is evicted in O(1).
  pub fn record_failure(&self, ip: &str) {
    let mut cache = self.lock_and_sweep();
    cache
      .get_or_insert_mut(ip.to_string(), Vec::new)
      .push(Instant::now());
  }
}

impl Default for AuthRateLimiter {
  fn default() -> Self {
    Self::new()
  }
}
