//! Provider key spending and usage limit operations
//!
//! Spending caps, reservations, adjustments, and usage limit enforcement.

use crate::error::{Result, TokenError};

use super::{ProviderKeyStorage, SpendingSummary};

impl ProviderKeyStorage {
  /// Set spending cap for a provider key
  ///
  /// # Arguments
  ///
  /// * `key_id` - Provider key database ID
  /// * `cap_microdollars` - Spending cap in microdollars (None = unlimited)
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn set_spending_cap(&self, key_id: i64, cap_microdollars: Option<i64>) -> Result<()> {
    let result =
      sqlx::query("UPDATE ai_provider_keys SET spending_cap_microdollars = $1 WHERE id = $2")
        .bind(cap_microdollars)
        .bind(key_id)
        .execute(&self.pool)
        .await
        .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Atomically increment spending for a provider key
  ///
  /// Uses a conditional UPDATE to ensure the spending cap is not exceeded.
  /// If the cap was exceeded, no update occurs and an error is returned.
  ///
  /// # Arguments
  ///
  /// * `key_id` - Provider key database ID
  /// * `amount_microdollars` - Amount to add in microdollars
  ///
  /// # Errors
  ///
  /// Returns error if spending cap would be exceeded or database update fails
  pub async fn increment_spending(&self, key_id: i64, amount_microdollars: i64) -> Result<()> {
    if amount_microdollars <= 0 {
      return Err(TokenError::Generic);
    }
    let result = sqlx::query(
      "UPDATE ai_provider_keys \
       SET spending_used_microdollars = spending_used_microdollars + $1 \
       WHERE id = $2 \
       AND (spending_cap_microdollars IS NULL \
            OR spending_used_microdollars + $1 <= spending_cap_microdollars)",
    )
    .bind(amount_microdollars)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      // Distinguish: row missing vs. cap condition blocked the update
      let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ai_provider_keys WHERE id = $1)")
          .bind(key_id)
          .fetch_one(&self.pool)
          .await
          .map_err(TokenError::Database)?;
      if exists {
        return Err(TokenError::SpendingCapExceeded);
      }
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Atomically reserve estimated cost before forwarding to LLM provider.
  ///
  /// Increments `spending_used_microdollars` by `estimated_amount` if within cap.
  /// Returns `Ok(())` if reservation succeeded, `Err` if cap would be exceeded.
  /// After receiving the actual cost, call [`adjust_spending`] to correct the delta.
  ///
  /// # Errors
  ///
  /// Returns error if spending cap would be exceeded or database update fails
  pub async fn reserve_spending(&self, key_id: i64, estimated_amount: i64) -> Result<()> {
    if estimated_amount <= 0 {
      return Err(TokenError::Generic);
    }
    let result = sqlx::query(
      "UPDATE ai_provider_keys \
       SET spending_used_microdollars = spending_used_microdollars + $1 \
       WHERE id = $2 \
       AND (spending_cap_microdollars IS NULL \
            OR spending_used_microdollars + $1 <= spending_cap_microdollars)",
    )
    .bind(estimated_amount)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      // Distinguish: row missing vs. cap condition blocked the update
      let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ai_provider_keys WHERE id = $1)")
          .bind(key_id)
          .fetch_one(&self.pool)
          .await
          .map_err(TokenError::Database)?;
      if exists {
        return Err(TokenError::SpendingCapExceeded);
      }
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Adjust spending after actual cost is known.
  ///
  /// Corrects the difference between reserved and actual cost.
  /// If actual < reserved, releases the excess. If actual > reserved, adds the difference.
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn adjust_spending(&self, key_id: i64, reserved: i64, actual: i64) -> Result<()> {
    let delta = actual - reserved;
    if delta == 0 {
      return Ok(());
    }

    let result = if delta > 0 {
      // Actual exceeded estimate: enforce cap so we don't silently bust it
      sqlx::query(
        "UPDATE ai_provider_keys \
         SET spending_used_microdollars = spending_used_microdollars + $1 \
         WHERE id = $2 \
         AND (spending_cap_microdollars IS NULL \
              OR spending_used_microdollars + $1 <= spending_cap_microdollars)",
      )
      .bind(delta)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?
    } else {
      // Refund path: unconditional decrement, clamped to zero
      sqlx::query(
        "UPDATE ai_provider_keys \
         SET spending_used_microdollars = MAX(0, spending_used_microdollars + $1) \
         WHERE id = $2",
      )
      .bind(delta)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?
    };

    if result.rows_affected() == 0 {
      if delta > 0 {
        // Positive delta branch used a conditional UPDATE (cap check).
        // Distinguish: row missing vs. cap condition blocked the update.
        let exists: bool =
          sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM ai_provider_keys WHERE id = $1)")
            .bind(key_id)
            .fetch_one(&self.pool)
            .await
            .map_err(TokenError::Database)?;
        if exists {
          return Err(TokenError::SpendingCapExceeded);
        }
      }
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Get the `owner_id` for an agent by its database ID
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_owner_id(&self, agent_id: i64) -> Result<Option<String>> {
    sqlx::query_scalar("SELECT owner_id FROM agents WHERE id = ?")
      .bind(agent_id)
      .fetch_optional(&self.pool)
      .await
      .map_err(TokenError::Database)
  }

  /// Get the `provider_key_id` assigned to an agent
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_agent_provider_key_id(&self, agent_id: i64) -> Result<Option<Option<i64>>> {
    sqlx::query_scalar("SELECT provider_key_id FROM agents WHERE id = ?")
      .bind(agent_id)
      .fetch_optional(&self.pool)
      .await
      .map_err(TokenError::Database)
  }

  /// Get spending summary for a provider key
  ///
  /// # Errors
  ///
  /// Returns error if key not found or database query fails
  pub async fn get_spending_summary(&self, key_id: i64) -> Result<SpendingSummary> {
    let row: Option<(i64, Option<i64>)> = sqlx::query_as(
      "SELECT spending_used_microdollars, spending_cap_microdollars \
       FROM ai_provider_keys WHERE id = $1",
    )
    .bind(key_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some((used, cap)) => Ok(SpendingSummary {
        used_microdollars: used,
        cap_microdollars: cap,
      }),
      None => Err(TokenError::NotFound),
    }
  }

  /// Fetch a user's usage limits (monthly cap and current spend).
  ///
  /// Returns `None` if no `usage_limits` row exists for the user.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_usage_limits(
    &self,
    user_id: &str,
  ) -> Result<Option<(Option<i64>, Option<i64>)>> {
    sqlx::query_as(
      "SELECT max_cost_microdollars_per_month, current_cost_microdollars_this_month \
       FROM usage_limits WHERE user_id = ? LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)
  }

  /// Atomically increment the current monthly spend in `usage_limits` for a user.
  ///
  /// Only increments if the result would not exceed `max_cost_microdollars_per_month`.
  /// Returns `Ok(())` if the increment succeeded, `Err` if the cap would be exceeded
  /// or no `usage_limits` row exists for this user.
  ///
  /// # Errors
  ///
  /// Returns error if the monthly cap would be exceeded or database update fails
  pub async fn increment_usage_limits(&self, user_id: &str, amount: i64) -> Result<()> {
    let result = sqlx::query(
      "UPDATE usage_limits \
       SET current_cost_microdollars_this_month = current_cost_microdollars_this_month + ? \
       WHERE user_id = ? \
       AND (max_cost_microdollars_per_month IS NULL \
            OR current_cost_microdollars_this_month + ? <= max_cost_microdollars_per_month)",
    )
    .bind(amount)
    .bind(user_id)
    .bind(amount)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    if result.rows_affected() == 0 {
      // Distinguish: row missing vs. cap condition blocked the update
      let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM usage_limits WHERE user_id = ?)")
          .bind(user_id)
          .fetch_one(&self.pool)
          .await
          .map_err(TokenError::Database)?;
      if exists {
        return Err(TokenError::SpendingCapExceeded);
      }
      return Err(TokenError::NotFound);
    }
    Ok(())
  }
}
