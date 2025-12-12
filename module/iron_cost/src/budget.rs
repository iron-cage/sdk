use std::sync::atomic::{AtomicU64, Ordering};
use crate::error::CostError;
use crate::converter::{usd_to_micros, micros_to_usd};

/// A budget reservation that must be committed or cancelled.
///
/// Reservations prevent concurrent overspend by atomically reserving
/// the maximum possible cost before an LLM request starts.
#[derive(Debug)]
pub struct Reservation {
    /// Reserved amount in microdollars
    amount_micros: u64,
}

impl Reservation {
    /// Get the reserved amount in USD
    pub fn amount_usd(&self) -> f64 {
        micros_to_usd(self.amount_micros)
    }

    /// Get the reserved amount in microdollars
    pub fn amount_micros(&self) -> u64 {
        self.amount_micros
    }
}

#[derive(Debug)]
pub struct CostController {
    // All stored in micros (1/1,000,000 USD)
    // strict limit: 0 means strictly $0.00
    budget_limit_micros: AtomicU64,
    total_spent_micros: AtomicU64,
    /// Reserved amount for in-flight requests (prevents concurrent overspend)
    reserved_micros: AtomicU64,
}

impl CostController {
    /// Create a new controller with a strict starting budget.
    pub fn new(initial_budget_usd: f64) -> Self {
        Self {
            budget_limit_micros: AtomicU64::new(usd_to_micros(initial_budget_usd)),
            total_spent_micros: AtomicU64::new(0),
            reserved_micros: AtomicU64::new(0),
        }
    }

    /// The Critical Check
    /// Returns Ok(()) if allowed, Err if budget exceeded.
    /// Considers both spent and reserved amounts.
    pub fn check_budget(&self) -> Result<(), CostError> {
        // 1. Load all values atomically
        let limit = self.budget_limit_micros.load(Ordering::Acquire);
        let spent = self.total_spent_micros.load(Ordering::Acquire);
        let reserved = self.reserved_micros.load(Ordering::Acquire);

        // 2. Check if spent + reserved >= limit
        let used = spent.saturating_add(reserved);
        if used >= limit {
            return Err(CostError::BudgetExceeded {
                spent_usd: micros_to_usd(spent),
                limit_usd: micros_to_usd(limit),
                reserved_usd: micros_to_usd(reserved),
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
    /// Returns (spent_usd, limit_usd)
    pub fn get_status(&self) -> (f64, f64) {
        let limit = self.budget_limit_micros.load(Ordering::Relaxed);
        let spent = self.total_spent_micros.load(Ordering::Relaxed);
        (micros_to_usd(spent), micros_to_usd(limit))
    }

    /// Get current status including reserved (for API/Python)
    /// Returns (spent_usd, reserved_usd, limit_usd)
    pub fn get_full_status(&self) -> (f64, f64, f64) {
        let limit = self.budget_limit_micros.load(Ordering::Acquire);
        let spent = self.total_spent_micros.load(Ordering::Acquire);
        let reserved = self.reserved_micros.load(Ordering::Acquire);
        (micros_to_usd(spent), micros_to_usd(reserved), micros_to_usd(limit))
    }

    /// Get available budget (limit - spent - reserved) in USD
    pub fn available(&self) -> f64 {
        let limit = self.budget_limit_micros.load(Ordering::Acquire);
        let spent = self.total_spent_micros.load(Ordering::Acquire);
        let reserved = self.reserved_micros.load(Ordering::Acquire);
        let available = limit.saturating_sub(spent).saturating_sub(reserved);
        micros_to_usd(available)
    }

    /// Reserve budget atomically before an LLM call.
    ///
    /// This prevents concurrent overspend by reserving the maximum possible cost
    /// before the request starts. The reservation must be committed (with actual cost)
    /// or cancelled after the request completes.
    ///
    /// # Arguments
    ///
    /// * `max_cost_micros` - Maximum possible cost in microdollars (based on max_tokens)
    ///
    /// # Returns
    ///
    /// * `Ok(Reservation)` - Budget reserved, proceed with request
    /// * `Err(CostError::InsufficientBudget)` - Not enough budget available
    pub fn reserve(&self, max_cost_micros: u64) -> Result<Reservation, CostError> {
        loop {
            let limit = self.budget_limit_micros.load(Ordering::Acquire);
            let spent = self.total_spent_micros.load(Ordering::Acquire);
            let reserved = self.reserved_micros.load(Ordering::Acquire);

            // Calculate available budget
            let available = limit.saturating_sub(spent).saturating_sub(reserved);

            if max_cost_micros > available {
                return Err(CostError::InsufficientBudget {
                    available_usd: micros_to_usd(available),
                    requested_usd: micros_to_usd(max_cost_micros),
                });
            }

            // Atomic CAS to reserve the amount
            match self.reserved_micros.compare_exchange_weak(
                reserved,
                reserved.saturating_add(max_cost_micros),
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(Reservation {
                        amount_micros: max_cost_micros,
                    });
                }
                Err(_) => {
                    // CAS failed, another thread modified reserved_micros
                    // Spin and retry with updated values
                    std::hint::spin_loop();
                }
            }
        }
    }

    /// Reserve budget in USD (convenience wrapper)
    pub fn reserve_usd(&self, max_cost_usd: f64) -> Result<Reservation, CostError> {
        self.reserve(usd_to_micros(max_cost_usd))
    }

    /// Commit a reservation with actual cost.
    ///
    /// Releases the reserved amount and adds the actual cost to spent.
    /// Call this after an LLM request completes successfully.
    ///
    /// # Arguments
    ///
    /// * `reservation` - The reservation to commit (consumes it)
    /// * `actual_cost_micros` - Actual cost incurred (usually less than reserved)
    pub fn commit(&self, reservation: Reservation, actual_cost_micros: u64) {
        // Release reservation
        self.reserved_micros
            .fetch_sub(reservation.amount_micros, Ordering::AcqRel);
        // Add actual cost to spent
        self.total_spent_micros
            .fetch_add(actual_cost_micros, Ordering::AcqRel);
    }

    /// Commit a reservation with actual cost in USD (convenience wrapper)
    pub fn commit_usd(&self, reservation: Reservation, actual_cost_usd: f64) {
        self.commit(reservation, usd_to_micros(actual_cost_usd));
    }

    /// Cancel a reservation without adding any cost.
    ///
    /// Releases the reserved amount. Call this if an LLM request fails
    /// or is cancelled before completing.
    ///
    /// # Arguments
    ///
    /// * `reservation` - The reservation to cancel (consumes it)
    pub fn cancel(&self, reservation: Reservation) {
        self.reserved_micros
            .fetch_sub(reservation.amount_micros, Ordering::AcqRel);
    }

    /// Get total reserved amount in USD
    pub fn total_reserved(&self) -> f64 {
        micros_to_usd(self.reserved_micros.load(Ordering::Acquire))
    }
}
