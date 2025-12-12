//! Error types for cost management

/// Errors that can occur in cost management
#[derive(Debug)]
pub enum CostError {
    /// Budget limit exceeded (including reserved funds)
    BudgetExceeded {
        spent_usd: f64,
        limit_usd: f64,
        reserved_usd: f64,
    },
    /// Insufficient budget available for reservation
    InsufficientBudget {
        available_usd: f64,
        requested_usd: f64,
    },
}

impl std::fmt::Display for CostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BudgetExceeded { spent_usd, limit_usd, reserved_usd } => {
                write!(
                    f,
                    "Budget exceeded: spent ${:.6}, reserved ${:.6}, limit ${:.6}",
                    spent_usd, reserved_usd, limit_usd
                )
            }
            Self::InsufficientBudget { available_usd, requested_usd } => {
                write!(
                    f,
                    "Insufficient budget: available ${:.6}, requested ${:.6}",
                    available_usd, requested_usd
                )
            }
        }
    }
}

impl std::error::Error for CostError {}
