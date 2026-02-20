//! Error types for cost management

/// Errors that can occur in cost management
#[derive(Debug)]
pub enum CostError {
  /// Budget limit exceeded (including reserved funds)
  BudgetExceeded {
    /// Total spent in microdollars
    spent_microdollars: i64,
    /// Budget limit in microdollars
    limit_microdollars: i64,
    /// Currently reserved in microdollars
    reserved_microdollars: i64,
  },
  /// Insufficient budget available for reservation
  InsufficientBudget {
    /// Available budget in microdollars
    available_microdollars: i64,
    /// Requested amount in microdollars
    requested_microdollars: i64,
  },
  /// JSON parsing error in pricing data
  JsonParseError(String),
}

impl core::fmt::Display for CostError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::BudgetExceeded {
        spent_microdollars,
        limit_microdollars,
        reserved_microdollars,
      } => {
        write!(
          f,
          "Budget exceeded: spent ${:.6}, reserved ${:.6}, limit ${:.6}",
          *spent_microdollars as f64 / 1_000_000.0,
          *reserved_microdollars as f64 / 1_000_000.0,
          *limit_microdollars as f64 / 1_000_000.0
        )
      }
      Self::InsufficientBudget {
        available_microdollars,
        requested_microdollars,
      } => {
        write!(
          f,
          "Insufficient budget: available ${:.6}, requested ${:.6}",
          *available_microdollars as f64 / 1_000_000.0,
          *requested_microdollars as f64 / 1_000_000.0
        )
      }
      Self::JsonParseError(msg) => {
        write!(f, "Failed to parse pricing JSON: {msg}")
      }
    }
  }
}

impl std::error::Error for CostError {}
