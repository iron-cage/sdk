use iron_cost::converter::micros_to_usd;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Default)]
pub struct AtomicModelStats {
    request_count: AtomicU64,
    input_tokens: AtomicU64,
    output_tokens: AtomicU64,
    cost_micros: AtomicU64,
}

impl AtomicModelStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&self, input: u64, output: u64, cost: u64) {
        self.request_count.fetch_add(1, Ordering::Relaxed);
        self.input_tokens.fetch_add(input, Ordering::Relaxed);
        self.output_tokens.fetch_add(output, Ordering::Relaxed);
        self.cost_micros.fetch_add(cost, Ordering::Relaxed);
    }

    pub fn snapshot(&self) -> ModelStats {
        self.into()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelStats {
    pub request_count: u64,
    pub input_tokens: u64,
    pub output_tokens: u64,
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
    pub fn cost_usd(&self) -> f64 {
        micros_to_usd(self.cost_micros)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputedStats {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_micros: u64,
    pub by_model: HashMap<String, ModelStats>,
    pub by_provider: HashMap<String, ModelStats>,
}

impl ComputedStats {
    pub fn total_cost_usd(&self) -> f64 {
        micros_to_usd(self.total_cost_micros)
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_requests == 0 {
            return 100.0;
        }
        let success = self.total_requests.saturating_sub(self.failed_requests);
        (success as f64 / self.total_requests as f64) * 100.0
    }

    pub fn avg_cost_per_request_usd(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_cost_usd() / self.total_requests as f64
    }

    pub fn total_tokens(&self) -> u64 {
        self.total_input_tokens + self.total_output_tokens
    }

    pub fn avg_tokens_per_request(&self) -> f64 {
        if self.total_requests == 0 {
            return 0.0;
        }
        self.total_tokens() as f64 / self.total_requests as f64
    }
}