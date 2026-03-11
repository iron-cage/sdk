//! Agent Budget Manager
//!
//! Protocol 005: Budget Control Protocol - Agent Budget Management
//!
//! Manages per-agent budget allocations. Each agent has exactly one budget (1:1 relationship)
//! that tracks total allocated, total spent, and remaining budget across all leases.
//!
//! Budget Invariant: `total_allocated` = `total_spent` + `budget_remaining`

use sqlx::{Row, SqlitePool};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::TokenError;

/// Explicit spending cap — no silent `None` semantics.
///
/// Database maps `NULL` → `Unlimited`, non-NULL → `Limited(value)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpendingCap {
  /// No cap enforced
  Unlimited,
  /// Capped at this many microdollars
  Limited(i64),
}

impl SpendingCap {
  /// Convert from the nullable database column
  #[must_use]
  pub fn from_db(value: Option<i64>) -> Self {
    match value {
      Some(v) => Self::Limited(v),
      None => Self::Unlimited,
    }
  }

  /// Convert to nullable value for database storage
  #[must_use]
  pub fn to_db(self) -> Option<i64> {
    match self {
      Self::Unlimited => None,
      Self::Limited(v) => Some(v),
    }
  }

  /// Returns `true` if the cap would be exceeded by adding `amount` to `used`
  #[must_use]
  pub fn would_exceed(&self, used: i64, amount: i64) -> bool {
    match self {
      Self::Unlimited => false,
      Self::Limited(cap) => used + amount > *cap,
    }
  }
}

/// Agent budget record
#[derive(Debug, Clone)]
pub struct AgentBudget {
  /// Agent database ID (1:1 relationship with agents table)
  pub agent_id: i64,
  /// Total microdollars budget allocated to this agent
  pub total_allocated: i64,
  /// Total microdollars spent across all leases
  pub total_spent: i64,
  /// Microdollars remaining (`total_allocated` - `total_spent`)
  pub budget_remaining: i64,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last update timestamp (milliseconds since epoch)
  pub updated_at: i64,
  /// Per-agent spending cap
  pub spending_cap: SpendingCap,
  /// Cumulative spending tracked against spending cap
  pub spending_used_microdollars: i64,
}

/// Result of a unified budget reservation attempt
#[derive(Debug, Clone)]
pub struct ReservationResult {
  /// Amount granted in microdollars
  pub granted: i64,
  /// Agent budget remaining after reservation
  pub agent_budget_remaining: i64,
  /// If reservation was blocked, the reason
  pub blocked_by: Option<BlockedBy>,
}

/// Reason a budget reservation was blocked
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockedBy {
  /// Agent (IC-key) spending cap exceeded
  AgentSpendingCap,
  /// Provider key (IP-key) spending cap exceeded
  ProviderKeyCap,
  /// Agent budget pool exhausted
  InsufficientBudget,
}

/// Summary of spending for an agent
#[derive(Debug, Clone)]
pub struct AgentSpendingSummary {
  /// Amount spent in microdollars
  pub used_microdollars: i64,
  /// Spending cap
  pub cap: SpendingCap,
}

/// Agent budget manager for budget CRUD operations
#[derive(Debug, Clone)]
pub struct AgentBudgetManager {
  pool: SqlitePool,
}

impl AgentBudgetManager {
  /// Create new agent budget manager from existing pool
  ///
  /// # Arguments
  ///
  /// * `pool` - Existing database connection pool
  #[must_use]
  pub fn from_pool(pool: SqlitePool) -> Self {
    Self { pool }
  }

  /// Create new agent budget
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `total_allocated` - Total microdollars budget allocated to this agent
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn create_budget(
    &self,
    agent_id: i64,
    total_allocated: i64,
  ) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    sqlx::query(
      "INSERT INTO agent_budgets
      (agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at)
      VALUES (?, ?, 0, ?, ?, ?)",
    )
    .bind(agent_id)
    .bind(total_allocated)
    .bind(total_allocated) // budget_remaining = total_allocated initially
    .bind(now)
    .bind(now)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Get agent budget status
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_budget_status(&self, agent_id: i64) -> Result<Option<AgentBudget>, sqlx::Error> {
    let row = sqlx::query(
      "SELECT agent_id, total_allocated, total_spent, budget_remaining, created_at, updated_at,
              spending_cap_microdollars, spending_used_microdollars
      FROM agent_budgets WHERE agent_id = ?",
    )
    .bind(agent_id)
    .fetch_optional(&self.pool)
    .await?;

    Ok(row.map(|r| AgentBudget {
      agent_id: r.get("agent_id"),
      total_allocated: r.get("total_allocated"),
      total_spent: r.get("total_spent"),
      budget_remaining: r.get("budget_remaining"),
      created_at: r.get("created_at"),
      updated_at: r.get("updated_at"),
      spending_cap: SpendingCap::from_db(r.get("spending_cap_microdollars")),
      spending_used_microdollars: r.get("spending_used_microdollars"),
    }))
  }

  /// Record spending against agent budget
  ///
  /// Updates `total_spent` and `budget_remaining`.
  /// Maintains invariant: `total_allocated` = `total_spent` + `budget_remaining`
  ///
  /// Fix(issue-budget-003): Use explicit transaction for atomic concurrent updates
  ///
  /// Root cause: Direct UPDATE statements from concurrent requests can cause lost updates
  /// in `SQLite` when using connection pooling. Same issue as `lease_manager::record_usage()`.
  /// Without explicit transaction control, concurrent updates to `total_spent` and
  /// `budget_remaining` can cause inconsistent state or lost spending records.
  ///
  /// Pitfall: Same as `lease_manager::record_usage()` - never rely on implicit atomicity
  /// for read-modify-write SQL operations. Always wrap in explicit transactions.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `cost_microdollars` - Cost to add to `total_spent` (in microdollars)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn record_spending(
    &self,
    agent_id: i64,
    cost_microdollars: i64,
  ) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    // Use explicit transaction with IMMEDIATE locking for atomic updates
    let mut tx = self.pool.begin().await?;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent + ?,
          budget_remaining = budget_remaining - ?,
          updated_at = ?
      WHERE agent_id = ?",
    )
    .bind(cost_microdollars)
    .bind(cost_microdollars)
    .bind(now)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
  }

  /// Atomically check and reserve budget for a request
  ///
  /// Fix(issue-budget-006): Prevent TOCTOU race in budget handshake
  ///
  /// Root cause: Handshake function checked `budget_remaining` with `get_budget_status()`,
  /// then separately called `record_spending()` in non-atomic operations. This created
  /// a race window where 2 concurrent requests could both pass the check before either
  /// recorded spending, allowing budget to go negative (violating budget invariant).
  ///
  /// Pitfall: Never split check-and-use into separate database operations for
  /// concurrent resource allocation. Use conditional UPDATE with WHERE clause that
  /// prevents negative budget, then verify `rows_affected` to detect race conditions.
  /// `SQLite`'s row-level write lock ensures the UPDATE is atomic even across concurrent
  /// requests - only one UPDATE can succeed when budget is insufficient for both.
  /// Under high concurrency (10+ simultaneous requests), `SQLite` may return deadlock
  /// errors - always implement retry logic with exponential backoff for database
  /// busy/locked/deadlocked errors.
  ///
  /// This method atomically:
  /// 1. Reads current `budget_remaining` within transaction
  /// 2. Calculates granted = min(requested, `budget_remaining`)
  /// 3. Updates budget only if granted > 0 AND wont go negative
  /// 4. Returns granted amount or 0
  ///
  /// Supports **partial grants**: If agent has $5 (`5_000_000` microdollars) and requests $10 (`10_000_000`), grants $5.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `requested_amount` - Microdollars amount requested
  ///
  /// # Returns
  ///
  /// * `Ok(granted_amount)` - Amount granted in microdollars (0 if no budget available)
  ///
  /// # Errors
  ///
  /// Returns error if database operation fails (not for insufficient budget)
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn check_and_reserve_budget(
    &self,
    agent_id: i64,
    requested_amount: i64,
  ) -> Result<i64, sqlx::Error> {
    // Retry logic for SQLite database busy/locked/deadlocked errors under high concurrency
    const MAX_RETRIES: u32 = 50;

    for attempt in 0..MAX_RETRIES {
      // Exponential backoff on retries
      if attempt > 0 {
        let backoff_ms = 2_u64.pow(attempt.min(8)); // Cap at 256ms
        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
      }

      match self
        .try_reserve_budget_once(agent_id, requested_amount)
        .await
      {
        Ok(granted) => return Ok(granted),
        Err(e) => {
          // Check if error is database busy/locked/deadlocked - retry if so
          let err_msg = e.to_string().to_lowercase();
          let is_retryable = err_msg.contains("database is locked")
            || err_msg.contains("database is busy")
            || err_msg.contains("deadlock");

          if is_retryable && attempt < MAX_RETRIES - 1 {
            // Retry on busy/deadlock error
          } else {
            // Not a retryable error, or max retries reached
            return Err(e);
          }
        }
      }
    }

    // Should never reach here
    Ok(0)
  }

  /// Single attempt to reserve budget (internal helper)
  async fn try_reserve_budget_once(
    &self,
    agent_id: i64,
    requested_amount: i64,
  ) -> Result<i64, sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    // Use explicit transaction for atomic check-and-reserve
    let mut tx = self.pool.begin().await?;

    // Read total_spent BEFORE update to calculate granted amount later
    let row = sqlx::query("SELECT total_spent FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_optional(&mut *tx)
      .await?;

    let spent_before = if let Some(r) = row {
      r.get::<i64, _>("total_spent")
    } else {
      // Agent doesnt exist
      tx.rollback().await?;
      return Ok(0);
    };

    // Single atomic UPDATE that calculates partial grant inline using CASE expression
    // This eliminates TOCTOU race by doing check+grant in one SQL statement
    //
    // CASE WHEN budget_remaining < requested THEN budget_remaining ELSE requested END
    // = min(budget_remaining, requested)
    //
    // WHERE budget_remaining > 0 ensures we only update if budget available
    let result = sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent +
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          budget_remaining = budget_remaining -
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          updated_at = ?
      WHERE agent_id = ? AND budget_remaining > 0",
    )
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(now)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // Calculate granted amount from change in total_spent
    let granted_amount = if result.rows_affected() == 1 {
      // Read total_spent AFTER update
      let row = sqlx::query("SELECT total_spent FROM agent_budgets WHERE agent_id = ?")
        .bind(agent_id)
        .fetch_one(&mut *tx)
        .await?;

      let spent_after: i64 = row.get("total_spent");

      // Granted = difference in total_spent
      spent_after - spent_before
    } else {
      // UPDATE failed - no budget available
      0
    };

    tx.commit().await?;
    Ok(granted_amount)
  }

  /// Add budget to agent allocation
  ///
  /// Increases `total_allocated` and `budget_remaining`.
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `additional_budget` - Microdollars to add to allocation
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn add_budget(&self, agent_id: i64, additional_budget: i64) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_allocated = total_allocated + ?,
          budget_remaining = budget_remaining + ?,
          updated_at = ?
      WHERE agent_id = ?",
    )
    .bind(additional_budget)
    .bind(additional_budget)
    .bind(now)
    .bind(agent_id)
    .execute(&self.pool)
    .await?;

    Ok(())
  }

  /// Check if agent has sufficient budget
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `required_amount` - Microdollars amount needed
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn has_sufficient_budget(
    &self,
    agent_id: i64,
    required_amount: i64,
  ) -> Result<bool, sqlx::Error> {
    let budget = self.get_budget_status(agent_id).await?;

    match budget {
      Some(b) => Ok(b.budget_remaining >= required_amount),
      None => Ok(false),
    }
  }

  /// Restore reserved budget that was returned unused
  ///
  /// Called by `/api/budget/return` endpoint when a lease is closed with unused budget.
  /// This reverses the reservation made by `check_and_reserve_budget()`.
  ///
  /// Updates: `total_spent` -= `returned_amount`, `budget_remaining` += `returned_amount`
  /// Maintains invariant: `total_allocated` = `total_spent` + `budget_remaining`
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `returned_amount` - Microdollars to restore (amount returned from lease)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn restore_reserved_budget(
    &self,
    agent_id: i64,
    returned_amount: i64,
  ) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    // Use explicit transaction for atomic updates
    let mut tx = self.pool.begin().await?;

    sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent - ?,
          budget_remaining = budget_remaining + ?,
          updated_at = ?
      WHERE agent_id = ?",
    )
    .bind(returned_amount)
    .bind(returned_amount)
    .bind(now)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(())
  }

  /// Set spending cap for an agent
  ///
  /// # Arguments
  ///
  /// * `agent_id` - Agent database ID
  /// * `cap` - Spending cap in microdollars (None = unlimited)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn set_spending_cap(
    &self,
    agent_id: i64,
    cap: SpendingCap,
  ) -> core::result::Result<(), TokenError> {
    let result =
      sqlx::query("UPDATE agent_budgets SET spending_cap_microdollars = ? WHERE agent_id = ?")
        .bind(cap.to_db())
        .bind(agent_id)
        .execute(&self.pool)
        .await
        .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Get spending summary for an agent (cap and used)
  ///
  /// # Errors
  ///
  /// Returns error if agent not found or database query fails
  pub async fn get_spending_summary(
    &self,
    agent_id: i64,
  ) -> core::result::Result<AgentSpendingSummary, TokenError> {
    let row: Option<(Option<i64>, i64)> = sqlx::query_as(
      "SELECT spending_cap_microdollars, spending_used_microdollars \
       FROM agent_budgets WHERE agent_id = ?",
    )
    .bind(agent_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some((cap, used)) => Ok(AgentSpendingSummary {
        used_microdollars: used,
        cap: SpendingCap::from_db(cap),
      }),
      None => Err(TokenError::NotFound),
    }
  }

  /// Reset spending counter for an agent
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn reset_spending(
    &self,
    agent_id: i64,
  ) -> core::result::Result<(), TokenError> {
    let result = sqlx::query(
      "UPDATE agent_budgets SET spending_used_microdollars = 0 WHERE agent_id = ?",
    )
    .bind(agent_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Atomically reserve budget checking both IC-key (agent) and IP-key (provider) caps.
  ///
  /// Single transaction:
  /// 1. Check agent spending cap
  /// 2. Check provider key spending cap
  /// 3. Reserve from agent budget (partial grants supported)
  /// 4. Increment agent `spending_used`
  /// 5. Increment provider key `spending_used`
  ///
  /// # Returns
  ///
  /// `ReservationResult` with granted amount and optional block reason.
  ///
  /// # Errors
  ///
  /// Returns error on database failure (not for cap/budget exhaustion).
  pub async fn reserve_budget_with_limits(
    &self,
    agent_id: i64,
    provider_key_id: Option<i64>,
    requested_amount: i64,
  ) -> Result<ReservationResult, sqlx::Error> {
    // Retry logic for SQLite database busy/locked/deadlocked errors
    const MAX_RETRIES: u32 = 50;

    for attempt in 0..MAX_RETRIES {
      if attempt > 0 {
        let backoff_ms = 2_u64.pow(attempt.min(8));
        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
      }

      match self
        .try_reserve_with_limits_once(agent_id, provider_key_id, requested_amount)
        .await
      {
        Ok(result) => return Ok(result),
        Err(e) => {
          let err_msg = e.to_string().to_lowercase();
          let is_retryable = err_msg.contains("database is locked")
            || err_msg.contains("database is busy")
            || err_msg.contains("deadlock");

          if is_retryable && attempt < MAX_RETRIES - 1 {
            // Retry
          } else {
            return Err(e);
          }
        }
      }
    }

    Ok(ReservationResult {
      granted: 0,
      agent_budget_remaining: 0,
      blocked_by: Some(BlockedBy::InsufficientBudget),
    })
  }

  /// Single attempt to reserve budget with limits (internal helper)
  async fn try_reserve_with_limits_once(
    &self,
    agent_id: i64,
    provider_key_id: Option<i64>,
    requested_amount: i64,
  ) -> Result<ReservationResult, sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    let mut tx = self.pool.begin().await?;

    // 1. Check agent spending cap
    let agent_row = sqlx::query(
      "SELECT spending_cap_microdollars, spending_used_microdollars, total_spent, budget_remaining \
       FROM agent_budgets WHERE agent_id = ?",
    )
    .bind(agent_id)
    .fetch_optional(&mut *tx)
    .await?;

    let Some(agent_row) = agent_row else {
      tx.rollback().await?;
      return Ok(ReservationResult {
        granted: 0,
        agent_budget_remaining: 0,
        blocked_by: Some(BlockedBy::InsufficientBudget),
      });
    };

    let agent_cap = SpendingCap::from_db(agent_row.get("spending_cap_microdollars"));
    let agent_used: i64 = agent_row.get("spending_used_microdollars");
    let budget_remaining: i64 = agent_row.get("budget_remaining");

    // Check IC-key (agent) spending cap
    if agent_cap.would_exceed(agent_used, requested_amount) {
      tx.rollback().await?;
      return Ok(ReservationResult {
        granted: 0,
        agent_budget_remaining: budget_remaining,
        blocked_by: Some(BlockedBy::AgentSpendingCap),
      });
    }

    // 2. Check IP-key (provider key) spending cap
    if let Some(pkey_id) = provider_key_id {
      let pkey_row = sqlx::query(
        "SELECT spending_cap_microdollars, spending_used_microdollars \
         FROM ai_provider_keys WHERE id = ?",
      )
      .bind(pkey_id)
      .fetch_optional(&mut *tx)
      .await?;

      if let Some(pkey_row) = pkey_row {
        let pkey_cap = SpendingCap::from_db(pkey_row.get("spending_cap_microdollars"));
        let pkey_used: i64 = pkey_row.get("spending_used_microdollars");

        if pkey_cap.would_exceed(pkey_used, requested_amount) {
          tx.rollback().await?;
          return Ok(ReservationResult {
            granted: 0,
            agent_budget_remaining: budget_remaining,
            blocked_by: Some(BlockedBy::ProviderKeyCap),
          });
        }
      }
    }

    // 3. Reserve from agent budget (partial grants supported)
    let spent_before: i64 = agent_row.get("total_spent");

    let result = sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent +
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          budget_remaining = budget_remaining -
        CASE WHEN budget_remaining < ? THEN budget_remaining ELSE ? END,
          updated_at = ?
      WHERE agent_id = ? AND budget_remaining > 0",
    )
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(requested_amount)
    .bind(now)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    let granted = if result.rows_affected() == 1 {
      let row = sqlx::query("SELECT total_spent, budget_remaining FROM agent_budgets WHERE agent_id = ?")
        .bind(agent_id)
        .fetch_one(&mut *tx)
        .await?;

      let spent_after: i64 = row.get("total_spent");
      spent_after - spent_before
    } else {
      tx.rollback().await?;
      return Ok(ReservationResult {
        granted: 0,
        agent_budget_remaining: budget_remaining,
        blocked_by: Some(BlockedBy::InsufficientBudget),
      });
    };

    // 4. Increment agent spending_used
    sqlx::query(
      "UPDATE agent_budgets SET spending_used_microdollars = spending_used_microdollars + ? WHERE agent_id = ?",
    )
    .bind(granted)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // 5. Increment provider key spending_used
    if let Some(pkey_id) = provider_key_id {
      sqlx::query(
        "UPDATE ai_provider_keys SET spending_used_microdollars = spending_used_microdollars + ? WHERE id = ?",
      )
      .bind(granted)
      .bind(pkey_id)
      .execute(&mut *tx)
      .await?;
    }

    // Get final budget_remaining
    let final_row = sqlx::query("SELECT budget_remaining FROM agent_budgets WHERE agent_id = ?")
      .bind(agent_id)
      .fetch_one(&mut *tx)
      .await?;
    let final_remaining: i64 = final_row.get("budget_remaining");

    tx.commit().await?;

    Ok(ReservationResult {
      granted,
      agent_budget_remaining: final_remaining,
      blocked_by: None,
    })
  }

  /// Atomically restore budget to both agent and provider key.
  ///
  /// Single transaction reverses reservation:
  /// 1. Restore agent budget (`total_spent`, `budget_remaining`, `spending_used`)
  /// 2. Restore provider key `spending_used` (if `provider_key_id` present)
  ///
  /// # Errors
  ///
  /// Returns error on database failure.
  ///
  /// # Panics
  ///
  /// Panics if system time is before UNIX epoch (should never happen on modern systems)
  pub async fn restore_budget_with_limits(
    &self,
    agent_id: i64,
    provider_key_id: Option<i64>,
    returned_amount: i64,
  ) -> Result<(), sqlx::Error> {
    #[allow(clippy::cast_possible_truncation)]
    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("LOUD FAILURE: Time went backwards")
      .as_millis() as i64;

    let mut tx = self.pool.begin().await?;

    // Restore agent budget + decrement spending_used
    sqlx::query(
      "UPDATE agent_budgets
      SET total_spent = total_spent - ?,
          budget_remaining = budget_remaining + ?,
          spending_used_microdollars = spending_used_microdollars - ?,
          updated_at = ?
      WHERE agent_id = ?",
    )
    .bind(returned_amount)
    .bind(returned_amount)
    .bind(returned_amount)
    .bind(now)
    .bind(agent_id)
    .execute(&mut *tx)
    .await?;

    // Restore provider key spending_used
    if let Some(pkey_id) = provider_key_id {
      sqlx::query(
        "UPDATE ai_provider_keys SET spending_used_microdollars = spending_used_microdollars - ? WHERE id = ?",
      )
      .bind(returned_amount)
      .bind(pkey_id)
      .execute(&mut *tx)
      .await?;
    }

    tx.commit().await?;

    Ok(())
  }
}
