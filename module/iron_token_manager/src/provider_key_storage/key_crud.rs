//! Provider key CRUD operations
//!
//! Create, read, update, delete operations for provider keys.

use crate::error::{Result, TokenError};

use super::{
  current_time_ms, row_to_metadata, row_to_record, CreateKeyParams, ProviderKeyMetadata,
  ProviderKeyRecord, ProviderKeyStorage, ProviderType,
};

impl ProviderKeyStorage {
  /// Create a new provider key
  ///
  /// # Arguments
  ///
  /// * `provider` - Provider type (openai, anthropic)
  /// * `encrypted_api_key` - Encrypted API key (base64)
  /// * `encryption_nonce` - Encryption nonce (base64)
  /// * `base_url` - Optional custom base URL
  /// * `description` - Optional description
  /// * `user_id` - Owner user ID
  ///
  /// # Returns
  ///
  /// Database ID of created key
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn create_key(
    &self,
    provider: ProviderType,
    encrypted_api_key: &str,
    encryption_nonce: &str,
    base_url: Option<&str>,
    description: Option<&str>,
    user_id: &str,
  ) -> Result<i64> {
    let now_ms = current_time_ms();
    let provider_str = provider.as_str();
    let result = sqlx::query(
      "INSERT INTO ai_provider_keys \
       ( provider, encrypted_api_key, encryption_nonce, base_url, description, user_id, created_at ) \
       VALUES ( $1, $2, $3, $4, $5, $6, $7 )",
    )
    .bind(provider_str)
    .bind(encrypted_api_key)
    .bind(encryption_nonce)
    .bind(base_url)
    .bind(description)
    .bind(user_id)
    .bind(now_ms)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(result.last_insert_rowid())
  }

  /// Get a provider key by ID (full record including encrypted data)
  ///
  /// # Arguments
  ///
  /// * `key_id` - Database ID
  ///
  /// # Returns
  ///
  /// Full key record with encrypted data
  ///
  /// # Errors
  ///
  /// Returns error if key not found or database query fails
  pub async fn get_key(&self, key_id: i64) -> Result<ProviderKeyRecord> {
    let row = sqlx::query(
      "SELECT id, provider, encrypted_api_key, encryption_nonce, base_url, \
       description, is_enabled, created_at, last_used_at, balance_cents, \
       balance_updated_at, user_id, spending_cap_microdollars, spending_used_microdollars \
       FROM ai_provider_keys WHERE id = ?",
    )
    .bind(key_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some(row) => Ok(row_to_record(&row)?),
      None => Err(TokenError::NotFound),
    }
  }

  /// Get a provider key by ID (metadata only, no encrypted data)
  ///
  /// # Errors
  ///
  /// Returns error if key not found or database query fails
  pub async fn get_key_metadata(&self, key_id: i64) -> Result<ProviderKeyMetadata> {
    let row = sqlx::query(
      "SELECT id, provider, base_url, description, is_enabled, created_at, \
       last_used_at, balance_cents, balance_updated_at, user_id, \
       spending_cap_microdollars, spending_used_microdollars \
       FROM ai_provider_keys WHERE id = ?",
    )
    .bind(key_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some(row) => Ok(row_to_metadata(&row)?),
      None => Err(TokenError::NotFound),
    }
  }

  /// List all keys for a user (metadata only)
  ///
  /// # Arguments
  ///
  /// * `user_id` - Owner user ID
  ///
  /// # Returns
  ///
  /// List of key metadata (no encrypted data)
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn list_keys(&self, user_id: &str) -> Result<Vec<ProviderKeyMetadata>> {
    let rows = sqlx::query(
      // qqq: [Low] no covering index for sort — (user_id, created_at) composite index would eliminate sort step; negligible at current 20-key quota
      "SELECT id, provider, base_url, description, is_enabled, created_at, \
       last_used_at, balance_cents, balance_updated_at, user_id, \
       spending_cap_microdollars, spending_used_microdollars \
       FROM ai_provider_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    rows.iter().map(row_to_metadata).collect()
  }

  /// Update balance
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_balance(&self, key_id: i64, balance_cents: i64) -> Result<()> {
    let now_ms = current_time_ms();
    sqlx::query(
      "UPDATE ai_provider_keys SET balance_cents = $1, balance_updated_at = $2 WHERE id = $3",
    )
    .bind(balance_cents)
    .bind(now_ms)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Update last used timestamp
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_last_used(&self, key_id: i64) -> Result<()> {
    let now_ms = current_time_ms();
    sqlx::query("UPDATE ai_provider_keys SET last_used_at = $1 WHERE id = $2")
      .bind(now_ms)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Delete a key
  ///
  /// # Errors
  ///
  /// Returns error if key not found or database delete fails
  pub async fn delete_key(&self, key_id: i64) -> Result<()> {
    let result = sqlx::query("DELETE FROM ai_provider_keys WHERE id = $1")
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;

    if result.rows_affected() == 0 {
      return Err(TokenError::NotFound);
    }
    Ok(())
  }

  /// Atomically create a provider key within the per-user per-provider quota.
  ///
  /// Runs the COUNT check and the INSERT inside a single `BEGIN IMMEDIATE`
  /// transaction so two concurrent requests cannot both pass the guard when
  /// the count is at the limit.
  ///
  /// # Arguments
  ///
  /// * `params` - Grouped creation parameters (see [`CreateKeyParams`])
  ///
  /// # Returns
  ///
  /// Database ID of the created key, or [`TokenError::KeyQuotaExceeded`] if
  /// the count was already at or above `max_keys`.
  ///
  /// # Errors
  ///
  /// Returns [`TokenError::KeyQuotaExceeded`] if quota is already reached,
  /// or a database error if the transaction fails.
  pub async fn create_key_within_quota(&self, params: &CreateKeyParams<'_>) -> Result<i64> {
    let now_ms = current_time_ms();
    let provider_str = params.provider.as_str();

    // Acquire a raw connection so we can issue BEGIN IMMEDIATE ourselves.
    // pool.begin() emits BEGIN (deferred); we need BEGIN IMMEDIATE to block
    // concurrent readers from sneaking in between our COUNT and INSERT.
    let mut conn = self.pool.acquire().await.map_err(TokenError::Database)?;

    sqlx::query("BEGIN IMMEDIATE")
      .execute(&mut *conn)
      .await
      .map_err(TokenError::Database)?;

    let result = self
      .create_key_within_quota_inner(&mut conn, params, provider_str, now_ms)
      .await;

    match result {
      Ok(key_id) => {
        sqlx::query("COMMIT")
          .execute(&mut *conn)
          .await
          .map_err(TokenError::Database)?;
        Ok(key_id)
      }
      Err(e) => {
        // Best-effort rollback; ignore secondary error
        let _ = sqlx::query("ROLLBACK").execute(&mut *conn).await;
        Err(e)
      }
    }
  }

  async fn create_key_within_quota_inner(
    &self,
    conn: &mut sqlx::pool::PoolConnection<sqlx::Sqlite>,
    params: &CreateKeyParams<'_>,
    provider_str: &str,
    now_ms: i64,
  ) -> Result<i64> {
    let count: i64 = sqlx::query_scalar(
      "SELECT COUNT(*) FROM ai_provider_keys WHERE user_id = $1 AND provider = $2",
    )
    .bind(params.user_id)
    .bind(provider_str)
    .fetch_one(&mut **conn)
    .await
    .map_err(TokenError::Database)?;

    if count >= params.max_keys {
      return Err(TokenError::KeyQuotaExceeded);
    }

    let result = sqlx::query(
      "INSERT INTO ai_provider_keys \
       ( provider, encrypted_api_key, encryption_nonce, base_url, description, user_id, created_at ) \
       VALUES ( $1, $2, $3, $4, $5, $6, $7 )",
    )
    .bind(provider_str)
    .bind(params.encrypted_api_key)
    .bind(params.encryption_nonce)
    .bind(params.base_url)
    .bind(params.description)
    .bind(params.user_id)
    .bind(now_ms)
    .execute(&mut **conn)
    .await
    .map_err(TokenError::Database)?;

    Ok(result.last_insert_rowid())
  }

  /// Atomically update one or more mutable fields of a provider key.
  ///
  /// Each parameter is `Option` — `None` means "leave unchanged". All provided
  /// changes are applied in a single UPDATE statement, so either all columns are
  /// written together or none are (on query failure).
  ///
  /// Ownership (`user_id`) and encrypted key material are intentionally excluded
  /// from this method. Use [`create_key`] / [`delete_key`] for those changes.
  ///
  /// # Arguments
  ///
  /// * `key_id` — Database ID of the key to update
  /// * `description` — `None` = skip; `Some(v)` = set to `v` (pass `Some(None)` to clear)
  /// * `base_url`    — `None` = skip; `Some(v)` = set to `v` (pass `Some(None)` to clear)
  /// * `is_enabled`  — `None` = skip; `Some(b)` = enable/disable
  /// * `spending_cap_microdollars` — `None` = skip; `Some(v)` = set cap (`Some(None)` = remove)
  ///
  /// # Errors
  ///
  /// Returns error if any update fails or the transaction cannot be committed
  pub async fn update_key_fields(
    &self,
    key_id: i64,
    description: Option<Option<&str>>,
    base_url: Option<Option<&str>>,
    is_enabled: Option<bool>,
    spending_cap_microdollars: Option<Option<i64>>,
  ) -> Result<()> {
    // Single UPDATE with CASE WHEN guards — atomic without a transaction.
    // Each field is updated only when its Option is Some; otherwise the
    // ELSE branch keeps the existing column value unchanged.
    sqlx::query(
      "UPDATE ai_provider_keys SET \
         description              = CASE WHEN $1 THEN $2 ELSE description              END, \
         base_url                 = CASE WHEN $3 THEN $4 ELSE base_url                 END, \
         is_enabled               = CASE WHEN $5 THEN $6 ELSE is_enabled               END, \
         spending_cap_microdollars= CASE WHEN $7 THEN $8 ELSE spending_cap_microdollars END \
       WHERE id = $9",
    )
    .bind(description.is_some())
    .bind(description.flatten())
    .bind(base_url.is_some())
    .bind(base_url.flatten())
    .bind(is_enabled.is_some())
    .bind(is_enabled)
    .bind(spending_cap_microdollars.is_some())
    .bind(spending_cap_microdollars.flatten())
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)
    .and_then(|r| {
      if r.rows_affected() == 0 {
        Err(TokenError::NotFound)
      } else {
        Ok(())
      }
    })?;

    Ok(())
  }

  /// Get all keys of provider (unscoped — admin/internal use only)
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_keys_by_provider(&self, provider: ProviderType) -> Result<Vec<i64>> {
    let rows: Vec<(i64,)> = sqlx::query_as("SELECT id FROM ai_provider_keys WHERE provider = $1")
      .bind(provider.as_str())
      .fetch_all(&self.pool)
      .await
      .map_err(TokenError::Database)?;

    Ok(rows.into_iter().map(|r| r.0).collect())
  }

  /// Get IDs of all keys belonging to a specific owner for a given provider (owner-scoped)
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  // qqq: [Low] idx_ai_provider_keys_user_id is now redundant — composite index (026) subsumes it; drop old index in a follow-up migration to reduce write amplification
  pub async fn get_keys_by_owner_and_provider(
    &self,
    user_id: &str,
    provider: ProviderType,
  ) -> Result<Vec<i64>> {
    let rows: Vec<(i64,)> = sqlx::query_as(
      "SELECT id FROM ai_provider_keys WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .fetch_all(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(rows.into_iter().map(|r| r.0).collect())
  }

  /// Count keys belonging to a specific owner for a given provider
  ///
  /// Used for quota enforcement before key creation.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn count_keys_by_owner_and_provider(
    &self,
    user_id: &str,
    provider: ProviderType,
  ) -> Result<i64> {
    let row: (i64,) = sqlx::query_as(
      "SELECT COUNT(*) FROM ai_provider_keys WHERE user_id = $1 AND provider = $2",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .fetch_one(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(row.0)
  }
}
