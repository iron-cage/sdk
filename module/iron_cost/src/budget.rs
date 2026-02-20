//! Budget control with atomic reservations
//!
//! Provides thread-safe budget tracking and enforcement for LLM API costs.
//! Uses a reservation system to prevent concurrent overspend in multi-threaded environments.
//!
//! **Core Concepts:**
//! - **Microdollar Precision:** All amounts stored as microdollars (1/1,000,000 USD) for financial accuracy
//! - **Atomic Operations:** Thread-safe budget updates using atomic primitives
//! - **Reservation Pattern:** Reserve maximum cost before request, commit actual cost after completion
//! - **Concurrent Safety:** Prevents race conditions when multiple threads access budget simultaneously
//!
//! **Typical Workflow:**
//! 1. `reserve()` - Atomically reserve maximum possible cost before LLM request
//! 2. Execute LLM request
//! 3. `commit()` - Add actual cost to spent, release reservation
//! 4. Or `cancel()` - Release reservation if request failed
//!
//! **Why Reservations:**
//! Without reservations, concurrent requests could all check budget simultaneously
//! and each proceed thinking budget is available, causing overspend. Reservations
//! atomically claim budget before the request starts, preventing this race condition.

use crate::error::CostError;
use core::sync::atomic::{AtomicU64, Ordering};

/// A budget reservation that must be committed or canceled.
///
/// Reservations prevent concurrent overspend by atomically reserving
/// the maximum possible cost before an LLM request starts.
#[derive(Debug, Clone, Copy)]
pub struct Reservation {
  /// Reserved amount in microdollars
  amount_micros: u64,
}

impl Reservation {
  /// Get the reserved amount in microdollars
  #[must_use]
  pub fn amount(&self) -> i64 {
    i64::try_from(self.amount_micros).unwrap_or(i64::MAX)
  }
}

/// Thread-safe budget controller with atomic reservation support.
#[derive(Debug)]
#[allow(clippy::struct_field_names)]
pub struct CostController {
  // All stored in micros (1/1,000,000 USD)
  // strict limit: 0 means strictly $0.00
  budget_limit_micros: AtomicU64,
  total_spent_micros: AtomicU64,
  /// Reserved amount for in-flight requests (prevents concurrent overspend)
  reserved_micros: AtomicU64,
}

impl CostController {
  /// Create a new controller with a strict starting budget (in microdollars).
  #[must_use]
  pub fn new(initial_budget_micros: i64) -> Self {
    Self {
      budget_limit_micros: AtomicU64::new(u64::try_from(initial_budget_micros.max(0)).unwrap_or(0)),
      total_spent_micros: AtomicU64::new(0),
      reserved_micros: AtomicU64::new(0),
    }
  }

  /// The Critical Check
  /// Returns Ok(()) if allowed, Err if budget exceeded.
  /// Considers both spent and reserved amounts.
  ///
  /// # Errors
  ///
  /// Returns [`CostError::BudgetExceeded`] if spent + reserved >= limit.
  pub fn check_budget(&self) -> Result<(), CostError> {
    // 1. Load all values atomically
    let limit = self.budget_limit_micros.load(Ordering::Acquire);
    let spent = self.total_spent_micros.load(Ordering::Acquire);
    let reserved = self.reserved_micros.load(Ordering::Acquire);

    // 2. Check if spent + reserved >= limit
    let used = spent.saturating_add(reserved);
    if used >= limit {
      return Err(CostError::BudgetExceeded {
        spent_microdollars: i64::try_from(spent).unwrap_or(i64::MAX),
        limit_microdollars: i64::try_from(limit).unwrap_or(i64::MAX),
        reserved_microdollars: i64::try_from(reserved).unwrap_or(i64::MAX),
      });
    }

    Ok(())
  }

  /// Add cost after a request finishes (in microdollars)
  pub fn add_spend(&self, cost_micros: i64) {
    self
      .total_spent_micros
      .fetch_add(u64::try_from(cost_micros.max(0)).unwrap_or(0), Ordering::Relaxed);
  }

  /// Get total spent in microdollars
  pub fn total_spent(&self) -> i64 {
    i64::try_from(self.total_spent_micros.load(Ordering::Relaxed)).unwrap_or(i64::MAX)
  }

  /// Get budget limit in microdollars
  pub fn budget_limit(&self) -> i64 {
    i64::try_from(self.budget_limit_micros.load(Ordering::Relaxed)).unwrap_or(i64::MAX)
  }

  /// Update the budget limit (in microdollars)
  pub fn set_budget(&self, budget_micros: i64) {
    self
      .budget_limit_micros
      .store(u64::try_from(budget_micros.max(0)).unwrap_or(0), Ordering::Relaxed);
  }

  /// Get current status
  /// Returns (`spent_microdollars`, `limit_microdollars`)
  pub fn get_status(&self) -> (i64, i64) {
    let limit = self.budget_limit_micros.load(Ordering::Relaxed);
    let spent = self.total_spent_micros.load(Ordering::Relaxed);
    (
      i64::try_from(spent).unwrap_or(i64::MAX),
      i64::try_from(limit).unwrap_or(i64::MAX),
    )
  }

  /// Get current status including reserved
  /// Returns (`spent_microdollars`, `reserved_microdollars`, `limit_microdollars`)
  pub fn get_full_status(&self) -> (i64, i64, i64) {
    let limit = self.budget_limit_micros.load(Ordering::Acquire);
    let spent = self.total_spent_micros.load(Ordering::Acquire);
    let reserved = self.reserved_micros.load(Ordering::Acquire);
    (
      i64::try_from(spent).unwrap_or(i64::MAX),
      i64::try_from(reserved).unwrap_or(i64::MAX),
      i64::try_from(limit).unwrap_or(i64::MAX),
    )
  }

  /// Get available budget (limit - spent - reserved) in microdollars
  pub fn available(&self) -> i64 {
    let limit = self.budget_limit_micros.load(Ordering::Acquire);
    let spent = self.total_spent_micros.load(Ordering::Acquire);
    let reserved = self.reserved_micros.load(Ordering::Acquire);
    let available = limit.saturating_sub(spent).saturating_sub(reserved);
    i64::try_from(available).unwrap_or(i64::MAX)
  }

  /// Reserve budget atomically before an LLM call.
  ///
  /// This prevents concurrent overspend by reserving the maximum possible cost
  /// before the request starts. The reservation must be committed (with actual cost)
  /// or cancelled after the request completes.
  ///
  /// # Arguments
  ///
  /// * `max_cost_micros` - Maximum possible cost in microdollars (based on `max_tokens`)
  ///
  /// # Returns
  ///
  /// * `Ok(Reservation)` - Budget reserved, proceed with request
  /// * `Err(CostError::InsufficientBudget)` - Not enough budget available
  ///
  /// # Errors
  ///
  /// Returns [`CostError::InsufficientBudget`] if not enough budget is available.
  pub fn reserve(&self, max_cost_micros: u64) -> Result<Reservation, CostError> {
    loop {
      let limit = self.budget_limit_micros.load(Ordering::Acquire);
      let spent = self.total_spent_micros.load(Ordering::Acquire);
      let reserved = self.reserved_micros.load(Ordering::Acquire);

      // Calculate available budget
      let available = limit.saturating_sub(spent).saturating_sub(reserved);

      if max_cost_micros > available {
        return Err(CostError::InsufficientBudget {
          available_microdollars: i64::try_from(available).unwrap_or(i64::MAX),
          requested_microdollars: i64::try_from(max_cost_micros).unwrap_or(i64::MAX),
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
          core::hint::spin_loop();
        }
      }
    }
  }

  /// Commit a reservation with actual cost.
  ///
  /// Adds actual cost to spent FIRST, then releases the reservation.
  /// This ordering is intentional: momentarily "double counts" the usage,
  /// which is safe (errs on the side of blocking) rather than unsafe
  /// (allowing overspend during the brief window between operations).
  ///
  /// # Arguments
  ///
  /// * `reservation` - The reservation to commit (consumes it)
  /// * `actual_cost_micros` - Actual cost incurred (usually less than reserved)
  pub fn commit(&self, reservation: Reservation, actual_cost_micros: u64) {
    // Add actual cost to spent FIRST (safe: errs on blocking side)
    self
      .total_spent_micros
      .fetch_add(actual_cost_micros, Ordering::AcqRel);
    // Then release reservation
    self
      .reserved_micros
      .fetch_sub(reservation.amount_micros, Ordering::AcqRel);
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
    self
      .reserved_micros
      .fetch_sub(reservation.amount_micros, Ordering::AcqRel);
  }

  /// Get total reserved amount in microdollars
  pub fn total_reserved(&self) -> i64 {
    i64::try_from(self.reserved_micros.load(Ordering::Acquire)).unwrap_or(i64::MAX)
  }
}
