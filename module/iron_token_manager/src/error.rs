//! Error types
//!
//! Error types for token management operations.

/// Token management error type
///
/// Fix(issue-001): Added Database variant to preserve underlying sqlx errors
/// for proper FK constraint handling
///
/// Root cause: Original `TokenError` was a unit struct that discarded all error
/// details from `SQLite`, making it impossible to distinguish FK constraint
/// violations from other database errors
///
/// Pitfall: Never discard error details when converting between error types.
/// Always preserve the underlying cause using enum variants or error wrapping
/// so handlers can make informed decisions about error responses
#[derive(Debug)]
pub enum TokenError {
  /// Generic token management error
  Generic,
  /// The requested resource was not found (row does not exist)
  NotFound,
  /// Database error preserving sqlx details for FK constraint detection
  Database(sqlx::Error),
  /// Operation would exceed the spending cap
  //qqq: [Info] callers in handshake.rs currently hit a wildcard Err arm — consider explicit match arm returning 402 Payment Required
  SpendingCapExceeded,
  /// Key creation would exceed the per-user per-provider quota
  KeyQuotaExceeded,
  /// Validation error with specific field and reason
  Validation {
    /// Field that failed validation
    field: String,
    /// Reason for validation failure
    reason: String,
  },
}

impl core::fmt::Display for TokenError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::Generic => write!(f, "Token management error"),
      Self::NotFound => write!(f, "Resource not found"),
      Self::Database(e) => write!(f, "Database error: {e}"),
      Self::SpendingCapExceeded => write!(f, "Spending cap exceeded"),
      Self::KeyQuotaExceeded => write!(f, "Key quota exceeded"),
      Self::Validation { field, reason } => write!(f, "Validation error: {field} - {reason}"),
    }
  }
}

impl core::error::Error for TokenError {}

/// Result type for token management operations
pub type Result<T> = core::result::Result<T, TokenError>;
