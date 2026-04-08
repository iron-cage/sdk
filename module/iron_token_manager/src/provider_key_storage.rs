//! AI Provider Key storage layer
//!
//! Manages encrypted storage of AI provider API keys (`OpenAI`, `Anthropic`, `Gemini`, `XAI`).
//!
//! Split into sub-modules by responsibility:
//! - [`key_crud`] — Create, read, update, delete operations
//! - [`key_projects`] — Key-to-project assignment management
//! - [`key_spending`] — Spending caps, reservations, and usage limits

mod key_crud;
mod key_projects;
mod key_spending;

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
  /// `Gemini` provider
  Gemini,
  /// `XAI` provider
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
  /// Optional description
  pub description: Option<String>,
  /// Whether key is enabled
  pub is_enabled: bool,
  /// Creation timestamp (ms since epoch)
  pub created_at: i64,
  /// Last use timestamp (ms since epoch)
  pub last_used_at: Option<i64>,
  /// Balance in cents
  pub balance_cents: Option<i64>,
  /// When balance was last updated (ms since epoch)
  pub balance_updated_at: Option<i64>,
  /// Owner user ID
  pub user_id: String,
  /// Spending cap in microdollars (None = unlimited)
  pub spending_cap_microdollars: Option<i64>,
  /// Current spending in microdollars
  pub spending_used_microdollars: i64,
}

/// Summary of spending state for a provider key
#[derive(Debug, Clone)]
pub struct SpendingSummary {
  /// Amount spent in microdollars
  pub used_microdollars: i64,
  /// Spending cap in microdollars (None = unlimited)
  pub cap_microdollars: Option<i64>,
}

/// Parameters for creating a new provider key within quota
#[derive(Debug)]
pub struct CreateKeyParams<'a> {
  /// Provider type
  pub provider: ProviderType,
  /// Encrypted API key (base64)
  pub encrypted_api_key: &'a str,
  /// Encryption nonce (base64)
  pub encryption_nonce: &'a str,
  /// Optional custom base URL
  pub base_url: Option<&'a str>,
  /// Optional description
  pub description: Option<&'a str>,
  /// Owner user ID
  pub user_id: &'a str,
  /// Maximum allowed keys per user per provider
  pub max_keys: i64,
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
}

// ─────────────────────────────────────────────────────────────────
// Internal helpers shared across sub-modules
// ─────────────────────────────────────────────────────────────────

/// Get current time in milliseconds since UNIX epoch
#[allow(clippy::cast_possible_truncation)]
fn current_time_ms() -> i64 {
  std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .expect("LOUD FAILURE: Time went backwards")
    .as_millis() as i64
}

fn row_to_metadata(row: &SqliteRow) -> Result<ProviderKeyMetadata> {
  let provider_str: String = row.get("provider");
  Ok(ProviderKeyMetadata {
    id: row.get("id"),
    provider: ProviderType::parse_str(&provider_str)
      .ok_or_else(|| TokenError::UnknownProvider(provider_str.clone()))?,
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
  })
}

fn row_to_record(row: &SqliteRow) -> Result<ProviderKeyRecord> {
  let provider_str: String = row.get("provider");
  Ok(ProviderKeyRecord {
    metadata: ProviderKeyMetadata {
      id: row.get("id"),
      provider: ProviderType::parse_str(&provider_str)
        .ok_or_else(|| TokenError::UnknownProvider(provider_str.clone()))?,
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
    },
    encrypted_api_key: row.get("encrypted_api_key"),
    encryption_nonce: row.get("encryption_nonce"),
  })
}
