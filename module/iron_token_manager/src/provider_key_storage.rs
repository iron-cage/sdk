//! AI Provider Key storage layer
//!
//! Manages encrypted storage of AI provider API keys (`OpenAI`, `Anthropic`, `Gemini`, `XAI`).

use core::fmt::{Display, Formatter, Result as FmtResult};

use sqlx::{sqlite::SqliteRow, Row, SqlitePool};

use crate::error::{Result, TokenError};

/// Provider type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderType {
  /// `OpenAI` provider
  OpenAI,
  /// `Anthropic` provider
  Anthropic,
  /// Google Gemini provider
  Gemini,
  /// xAI provider
  XAI,
}

impl ProviderType {
  /// Convert to database string representation
  #[must_use]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::OpenAI => "openai",
      Self::Anthropic => "anthropic",
      Self::Gemini => "gemini",
      Self::XAI => "xai",
    }
  }

  /// Parse from database string
  ///
  /// Note: Named `parse_str` instead of `from_str` to avoid confusion with `FromStr` trait
  #[must_use]
  pub fn parse_str(s: &str) -> Option<Self> {
    match s {
      "openai" => Some(Self::OpenAI),
      "anthropic" => Some(Self::Anthropic),
      "gemini" => Some(Self::Gemini),
      "xai" => Some(Self::XAI),
      _ => None,
    }
  }
}

impl Display for ProviderType {
  fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
    write!(f, "{}", self.as_str())
  }
}

/// Provider key metadata (excludes encrypted key)
#[derive(Debug, Clone)]
pub struct ProviderKeyMetadata {
  /// Database ID
  pub id: i64,
  /// Provider type
  pub provider: ProviderType,
  /// Optional custom base URL
  pub base_url: Option<String>,
  /// Human-friendly description
  pub description: Option<String>,
  /// Whether key is enabled
  pub is_enabled: bool,
  /// Creation timestamp (milliseconds since epoch)
  pub created_at: i64,
  /// Last used timestamp (milliseconds since epoch)
  pub last_used_at: Option<i64>,
  /// Balance in cents
  pub balance_cents: Option<i64>,
  /// When balance was last updated
  pub balance_updated_at: Option<i64>,
  /// User ID who owns this key
  pub user_id: String,
  /// Spending cap in microdollars (None = unlimited)
  pub spending_cap_microdollars: Option<i64>,
  /// Cumulative spending in microdollars
  pub spending_used_microdollars: i64,
  /// Optional human-friendly alias
  pub alias: Option<String>,
}

/// Summary of spending for a provider key
#[derive(Debug, Clone)]
pub struct SpendingSummary {
  /// Amount spent in microdollars
  pub used_microdollars: i64,
  /// Spending cap in microdollars (None = unlimited)
  pub cap_microdollars: Option<i64>,
}

/// Full provider key record (includes encrypted data)
#[derive(Debug, Clone)]
pub struct ProviderKeyRecord {
  /// Metadata
  pub metadata: ProviderKeyMetadata,
  /// Encrypted API key (base64)
  pub encrypted_api_key: String,
  /// Encryption nonce (base64)
  pub encryption_nonce: String,
}

/// Provider key storage layer
#[derive(Debug, Clone)]
pub struct ProviderKeyStorage {
  pool: SqlitePool,
}

impl ProviderKeyStorage {
  /// Create new provider key storage from existing pool
  #[must_use]
  pub fn new(pool: SqlitePool) -> Self {
    Self { pool }
  }

  /// Get the underlying pool for sharing with other storage types
  #[must_use]
  pub fn pool(&self) -> &SqlitePool {
    &self.pool
  }

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
  #[allow(clippy::too_many_arguments)]
  pub async fn create_key(
    &self,
    provider: ProviderType,
    encrypted_api_key: &str,
    encryption_nonce: &str,
    base_url: Option<&str>,
    description: Option<&str>,
    user_id: &str,
    alias: Option<&str>,
  ) -> Result<i64> {
    let now_ms = current_time_ms();

    let result = sqlx::query(
      "INSERT INTO ai_provider_keys \
       ( provider, encrypted_api_key, encryption_nonce, base_url, description, user_id, created_at, alias ) \
       VALUES ( $1, $2, $3, $4, $5, $6, $7, $8 )"
    )
    .bind( provider.as_str() )
    .bind( encrypted_api_key )
    .bind( encryption_nonce )
    .bind( base_url )
    .bind( description )
    .bind( user_id )
    .bind( now_ms )
    .bind( alias )
    .execute( &self.pool )
    .await
    .map_err( TokenError::Database )?;

    Ok(result.last_insert_rowid())
  }

  /// Get a provider key by ID (includes encrypted data)
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
       balance_updated_at, user_id, spending_cap_microdollars, spending_used_microdollars, alias \
       FROM ai_provider_keys WHERE id = ?",
    )
    .bind(key_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some(row) => Ok(row_to_record(&row)),
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
       last_used_at, balance_cents, encrypted_api_key, balance_updated_at, user_id, \
       spending_cap_microdollars, spending_used_microdollars, alias \
       FROM ai_provider_keys WHERE id = ?",
    )
    .bind(key_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    match row {
      Some(row) => Ok(row_to_metadata(&row)),
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
      "SELECT id, provider, base_url, description, is_enabled, created_at, \
       last_used_at, balance_cents, balance_updated_at, user_id, \
       spending_cap_microdollars, spending_used_microdollars, alias \
       FROM ai_provider_keys WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(rows.iter().map(row_to_metadata).collect())
  }

  /// Set key enabled/disabled status
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn set_enabled(&self, key_id: i64, enabled: bool) -> Result<()> {
    sqlx::query("UPDATE ai_provider_keys SET is_enabled = $1 WHERE id = $2")
      .bind(enabled)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Update description
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_description(&self, key_id: i64, description: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE ai_provider_keys SET description = $1 WHERE id = $2")
      .bind(description)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Update base URL
  ///
  /// # Errors
  ///
  /// Returns error if database update fails
  pub async fn update_base_url(&self, key_id: i64, base_url: Option<&str>) -> Result<()> {
    sqlx::query("UPDATE ai_provider_keys SET base_url = $1 WHERE id = $2")
      .bind(base_url)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;
    Ok(())
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

  /// Assign a key to a project
  ///
  /// # Errors
  ///
  /// Returns error if database insert fails
  pub async fn assign_to_project(&self, key_id: i64, project_id: &str) -> Result<()> {
    let now_ms = current_time_ms();
    sqlx::query(
      "INSERT OR REPLACE INTO project_provider_key_assignments \
       ( project_id, provider_key_id, assigned_at ) VALUES ( $1, $2, $3 )",
    )
    .bind(project_id)
    .bind(key_id)
    .bind(now_ms)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Remove key assignment from a project
  ///
  /// # Errors
  ///
  /// Returns error if database delete fails
  pub async fn unassign_from_project(&self, key_id: i64, project_id: &str) -> Result<()> {
    sqlx::query(
      "DELETE FROM project_provider_key_assignments \
       WHERE project_id = $1 AND provider_key_id = $2",
    )
    .bind(project_id)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
  }

  /// Get key assigned to a project
  ///
  /// Returns the most recently assigned key for the project, or `None` if no key is assigned.
  /// Uses `ORDER BY assigned_at DESC LIMIT 1` to give deterministic results when multiple
  /// assignments exist.
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_project_key(&self, project_id: &str) -> Result<Option<i64>> {
    let row: Option<(i64,)> = sqlx::query_as(
      "SELECT provider_key_id FROM project_provider_key_assignments \
       WHERE project_id = $1 ORDER BY assigned_at DESC LIMIT 1",
    )
    .bind(project_id)
    .fetch_optional(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(row.map(|r| r.0))
  }

  /// Get all project assignments for a key
  ///
  /// # Errors
  ///
  /// Returns error if database query fails
  pub async fn get_key_projects(&self, key_id: i64) -> Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
      "SELECT project_id FROM project_provider_key_assignments WHERE provider_key_id = $1",
    )
    .bind(key_id)
    .fetch_all(&self.pool)
    .await
    .map_err(TokenError::Database)?;

    Ok(rows.into_iter().map(|r| r.0).collect())
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
    alias: Option<Option<&str>>,
  ) -> Result<()> {
    // Single UPDATE with CASE WHEN guards — atomic without a transaction.
    // Each field is updated only when its Option is Some; otherwise the
    // ELSE branch keeps the existing column value unchanged.
    sqlx::query(
      "UPDATE ai_provider_keys SET \
         description              = CASE WHEN $1  THEN $2  ELSE description              END, \
         base_url                 = CASE WHEN $3  THEN $4  ELSE base_url                 END, \
         is_enabled               = CASE WHEN $5  THEN $6  ELSE is_enabled               END, \
         spending_cap_microdollars= CASE WHEN $7  THEN $8  ELSE spending_cap_microdollars END, \
         alias                    = CASE WHEN $9  THEN $10 ELSE alias                    END \
       WHERE id = $11",
    )
    .bind(description.is_some())
    .bind(description.flatten())
    .bind(base_url.is_some())
    .bind(base_url.flatten())
    .bind(is_enabled.is_some())
    .bind(is_enabled)
    .bind(spending_cap_microdollars.is_some())
    .bind(spending_cap_microdollars.flatten())
    .bind(alias.is_some())
    .bind(alias.flatten())
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
    sqlx::query("UPDATE ai_provider_keys SET spending_cap_microdollars = $1 WHERE id = $2")
      .bind(cap_microdollars)
      .bind(key_id)
      .execute(&self.pool)
      .await
      .map_err(TokenError::Database)?;
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
      return Err(TokenError::Generic);
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
      return Err(TokenError::Generic);
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
    sqlx::query(
      "UPDATE ai_provider_keys \
       SET spending_used_microdollars = spending_used_microdollars + $1 \
       WHERE id = $2",
    )
    .bind(delta)
    .bind(key_id)
    .execute(&self.pool)
    .await
    .map_err(TokenError::Database)?;
    Ok(())
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
}

/// Get current time in milliseconds since UNIX epoch
#[allow(clippy::cast_possible_truncation)]
fn current_time_ms() -> i64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .expect("LOUD FAILURE: Time went backwards")
    .as_millis() as i64
}

fn row_to_metadata(row: &SqliteRow) -> ProviderKeyMetadata {
  let provider_str: String = row.get("provider");
  ProviderKeyMetadata {
    id: row.get("id"),
    provider: ProviderType::parse_str(&provider_str).unwrap_or(ProviderType::OpenAI),
    base_url: row.get("base_url"),
    description: row.get("description"),
    is_enabled: row.get("is_enabled"),
    created_at: row.get("created_at"),
    last_used_at: row.get("last_used_at"),
    balance_cents: row.get("balance_cents"),
    balance_updated_at: row.get("balance_updated_at"),
    user_id: row.get("user_id"),
    spending_cap_microdollars: row.get("spending_cap_microdollars"),
    spending_used_microdollars: row.get("spending_used_microdollars"),
    alias: row.get("alias"),
  }
}

fn row_to_record(row: &SqliteRow) -> ProviderKeyRecord {
  let provider_str: String = row.get("provider");
  ProviderKeyRecord {
    metadata: ProviderKeyMetadata {
      id: row.get("id"),
      provider: ProviderType::parse_str(&provider_str).unwrap_or(ProviderType::OpenAI),
      base_url: row.get("base_url"),
      description: row.get("description"),
      is_enabled: row.get("is_enabled"),
      created_at: row.get("created_at"),
      last_used_at: row.get("last_used_at"),
      balance_cents: row.get("balance_cents"),
      balance_updated_at: row.get("balance_updated_at"),
      user_id: row.get("user_id"),
      spending_cap_microdollars: row.get("spending_cap_microdollars"),
      spending_used_microdollars: row.get("spending_used_microdollars"),
      alias: row.get("alias"),
    },
    encrypted_api_key: row.get("encrypted_api_key"),
    encryption_nonce: row.get("encryption_nonce"),
  }
}
