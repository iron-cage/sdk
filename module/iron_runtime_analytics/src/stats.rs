//! Statistics types for analytics aggregation.

use core::sync::atomic::{AtomicU64, Ordering};
use iron_cost::converter::micros_to_usd;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Atomic per-model statistics (lock-free).
#[derive(Debug, Default)]
pub struct AtomicModelStats {
  request_count: AtomicU64,
  input_tokens: AtomicU64,
  output_tokens: AtomicU64,
  cost_micros: AtomicU64,
}

impl AtomicModelStats {
  /// Create a new zeroed stats accumulator.
  #[must_use]
  pub fn new() -> Self {
    Self::default()
  }

  /// Atomically record one request with the given token and cost counts.
  pub fn add(&self, input: u64, output: u64, cost: u64) {
    self.request_count.fetch_add(1, Ordering::Relaxed);
    self.input_tokens.fetch_add(input, Ordering::Relaxed);
    self.output_tokens.fetch_add(output, Ordering::Relaxed);
    self.cost_micros.fetch_add(cost, Ordering::Relaxed);
  }

  /// Take a non-atomic snapshot of the current counters.
  pub fn snapshot(&self) -> ModelStats {
    self.into()
  }
}

/// Non-atomic snapshot of model statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
  /// Total number of requests recorded.
  pub request_count: u64,
  /// Total input tokens consumed.
  pub input_tokens: u64,
  /// Total output tokens generated.
  pub output_tokens: u64,
  /// Total cost in microdollars.
  pub cost_micros: u64,
}

impl From<&AtomicModelStats> for ModelStats {
  fn from(atomic: &AtomicModelStats) -> Self {
    Self {
      request_count: atomic.request_count.load(Ordering::Relaxed),
      input_tokens: atomic.input_tokens.load(Ordering::Relaxed),
      output_tokens: atomic.output_tokens.load(Ordering::Relaxed),
      cost_micros: atomic.cost_micros.load(Ordering::Relaxed),
    }
  }
}

impl ModelStats {
  /// Returns the total cost in USD.
  #[must_use]
  pub fn cost_usd(&self) -> f64 {
    micros_to_usd(self.cost_micros)
  }
}

/// Aggregated statistics snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputedStats {
  /// Total number of requests (successful + failed).
  pub total_requests: u64,
  /// Number of failed requests.
  pub failed_requests: u64,
  /// Total input tokens across all requests.
  pub total_input_tokens: u64,
  /// Total output tokens across all requests.
  pub total_output_tokens: u64,
  /// Total cost in microdollars across all requests.
  pub total_cost_micros: u64,
  /// Per-model statistics breakdown.
  pub by_model: HashMap<String, ModelStats>,
  /// Per-provider statistics breakdown.
  pub by_provider: HashMap<String, ModelStats>,
}

impl ComputedStats {
  /// Returns the total cost in USD.
  #[must_use]
  pub fn total_cost_usd(&self) -> f64 {
    micros_to_usd(self.total_cost_micros)
  }

  /// Returns the request success rate as a percentage (0–100).
  #[must_use]
  pub fn success_rate(&self) -> f64 {
    if self.total_requests == 0 {
      return 100.0;
    }
    let success = self.total_requests.saturating_sub(self.failed_requests);
    (success as f64 / self.total_requests as f64) * 100.0
  }

  /// Returns the average cost per request in USD.
  #[must_use]
  pub fn avg_cost_per_request_usd(&self) -> f64 {
    if self.total_requests == 0 {
      return 0.0;
    }
    self.total_cost_usd() / self.total_requests as f64
  }

  /// Returns the sum of input and output tokens.
  #[must_use]
  pub fn total_tokens(&self) -> u64 {
    self.total_input_tokens + self.total_output_tokens
  }

  /// Returns the average token count per request.
  #[must_use]
  pub fn avg_tokens_per_request(&self) -> f64 {
    if self.total_requests == 0 {
      return 0.0;
    }
    self.total_tokens() as f64 / self.total_requests as f64
  }
}
