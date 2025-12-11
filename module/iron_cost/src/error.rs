//! Error types for cost management

/// Errors that can occur in cost management
#[derive(Debug)]
pub enum CostError {
    /// Budget limit exceeded
    BudgetExceeded { spent_usd: f64, limit_usd: f64 },
}

impl std::fmt::Display for CostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BudgetExceeded { spent_usd, limit_usd } => {
                write!(f, "Budget exceeded: spent ${:.6}, limit ${:.6}", spent_usd, limit_usd)
            }
        }
    }
}

impl std::error::Error for CostError {}