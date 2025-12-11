use std::sync::atomic::{AtomicU64, Ordering};
use crate::error::CostError;
use crate::converter::{usd_to_micros, micros_to_usd};

#[derive(Debug)]
pub struct CostController {
    // Both stored in micros (1/1,000,000 USD)
    // strict limit: 0 means strictly $0.00
    budget_limit_micros: AtomicU64,
    total_spent_micros: AtomicU64,
}

impl CostController {
    /// Create a new controller with a strict starting budget.
    pub fn new(initial_budget_usd: f64) -> Self {
        Self {
            budget_limit_micros: AtomicU64::new(usd_to_micros(initial_budget_usd)),
            total_spent_micros: AtomicU64::new(0),
        }
    }

    /// The Critical Check
    /// Returns Ok(()) if allowed, Err if budget exceeded.
    pub fn check_budget(&self) -> Result<(), CostError> {
        // 1. Load both values atomically (instantly)
        let limit = self.budget_limit_micros.load(Ordering::Relaxed);
        let spent = self.total_spent_micros.load(Ordering::Relaxed);

        // 2. Strict comparison
        if spent >= limit {
            return Err(CostError::BudgetExceeded {
                spent_usd: micros_to_usd(spent),
                limit_usd: micros_to_usd(limit),
            });
        }

        Ok(())
    }

    /// Add cost after a request finishes (USD)
    pub fn add_spend(&self, cost_usd: f64) {
        let cost = usd_to_micros(cost_usd);
        self.total_spent_micros.fetch_add(cost, Ordering::Relaxed);
    }

    /// Add cost after a request finishes (microdollars)
    /// Use when cost is already calculated in micros to avoid conversion.
    pub fn add_spend_micros(&self, cost_micros: u64) {
        self.total_spent_micros.fetch_add(cost_micros, Ordering::Relaxed);
    }

    /// Get total spent in USD
    pub fn total_spent(&self) -> f64 {
        micros_to_usd(self.total_spent_micros.load(Ordering::Relaxed))
    }

    /// Get budget limit in USD
    pub fn budget_limit(&self) -> f64 {
        micros_to_usd(self.budget_limit_micros.load(Ordering::Relaxed))
    }

    /// Update the strict limit (e.g. from Python)
    pub fn set_budget(&self, budget_usd: f64) {
        let limit = usd_to_micros(budget_usd);
        self.budget_limit_micros.store(limit, Ordering::Relaxed);
    }

    /// Get current status (for API/Python)
    pub fn get_status(&self) -> (f64, f64) {
        let limit = self.budget_limit_micros.load(Ordering::Relaxed);
        let spent = self.total_spent_micros.load(Ordering::Relaxed);
        (micros_to_usd(spent), micros_to_usd(limit))
    }
}
