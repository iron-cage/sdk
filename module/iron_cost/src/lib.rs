//! Cost control module: budget tracking and enforcement
//!
//! Provides budget tracking and enforcement for multi-agent systems.
//!
//! ## Known Pitfalls
//!
//! ### DashMap Lock-Iteration Incompatibility
//!
//! DashMap is NOT a drop-in replacement for Mutex-wrapped HashMap. Never hold DashMap entry
//! locks while calling methods that iterate the same map, as this causes deadlock.
//!
//! **Why:** DashMap uses internal sharding with RwLocks. An exclusive write lock on one
//! shard prevents global iteration operations (like `iter()`, which powers `total_spent()`),
//! even for entries in other shards.
//!
//! **Correct pattern:**
//! ```rust,ignore
//! {
//!   let mut entry = map.entry(key).or_insert(default);
//!   *entry += value;
//! } // Explicit scope drops lock here
//! let total = map.iter().map(|e| *e.value()).sum(); // Safe now
//! ```
//!
//! **Incorrect pattern:**
//! ```rust,ignore
//! let mut entry = map.entry(key).or_insert(default);
//! *entry += value;
//! let total = map.iter().map(|e| *e.value()).sum(); // DEADLOCK!
//! ```
//!
//! See `tests/bug_deadlock_dashmap_fix.rs` for detailed bug analysis (issue deadlock-dashmap-001).

#![cfg_attr(not(feature = "enabled"), allow(unused))]

#[cfg(feature = "enabled")]
mod implementation
{
  use iron_types::{Result, Error};
  use dashmap::DashMap;
  use std::sync::Arc;

  /// Budget tracker with per-agent cost tracking
  pub struct BudgetTracker
  {
    budget_usd: f64,
    spent_usd: Arc< DashMap< String, f64 > >,
  }

  impl BudgetTracker
  {
    /// Create new tracker with specified budget
    pub fn new(budget_usd: f64) -> Self
    {
      Self
      {
        budget_usd,
        spent_usd: Arc::new(DashMap::new()),
      }
    }

    /// Record cost for an agent, returning error if budget exceeded
    ///
    /// Fix(deadlock-dashmap-001): Explicit scope drops entry lock before total_spent()
    /// Root cause: Holding DashMap entry lock while calling total_spent() caused deadlock
    /// because total_spent() iterates the same DashMap, and DashMap's sharded locking
    /// prevents iteration while any entry lock is held.
    /// Pitfall: DashMap is NOT a drop-in replacement for Mutex-wrapped HashMap. Never hold
    /// DashMap entry locks while calling methods that iterate the same map.
    ///
    /// Fix(budget-enforcement-001): Check budget BEFORE recording cost
    /// Root cause: Previous implementation recorded cost first, then checked budget, allowing
    /// budget to be exceeded by the final operation.
    /// Pitfall: Always validate constraints before modifying state, not after.
    pub fn record_cost(&self, agent_id: &str, cost: f64) -> Result< () >
    {
      // Check if adding this cost would exceed budget (check BEFORE recording)
      let current_total = self.total_spent();
      if current_total + cost > self.budget_usd
      {
        return Err(
          Error::BudgetExceeded(
            format!(
              "Budget ${} exceeded: current ${} + cost ${} = ${} > ${}",
              self.budget_usd,
              current_total,
              cost,
              current_total + cost,
              self.budget_usd
            )
          )
        );
      }

      // Only record cost if budget check passed
      {
        let mut entry = self.spent_usd.entry(agent_id.to_string()).or_insert(0.0);
        *entry += cost;
      } // Drop entry lock

      Ok(())
    }

    /// Get total amount spent across all agents
    pub fn total_spent(&self) -> f64
    {
      self.spent_usd.iter().map(|entry| *entry.value()).sum()
    }

    /// Get remaining budget
    pub fn remaining(&self) -> f64
    {
      self.budget_usd - self.total_spent()
    }
  }
}

#[cfg(feature = "enabled")]
pub use implementation::*;
