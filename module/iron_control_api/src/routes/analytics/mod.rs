//! Analytics REST API endpoints (Protocol 012)
//!
//! Provides endpoints for:
//! - Event ingestion from LlmRouter
//! - Spending analytics (total, by-agent, by-provider, avg-per-request)
//! - Budget status monitoring
//! - Usage statistics (requests, tokens, models)
//!
//! # Cost Units
//!
//! All costs are stored and transferred in **microdollars** (1 USD = 1,000,000 microdollars)
//! for precision. Responses convert to USD for display.
//!
//! # Module Organization
//!
//! - `shared`: Common types, state, query parameters, response structures
//! - `ingestion`: Event ingestion endpoint (POST /api/v1/analytics/events)
//! - `spending`: Spending analytics endpoints (4 handlers)
//! - `budget`: Budget status endpoint (1 handler)
//! - `usage`: Usage statistics endpoints (3 handlers)

mod shared;
mod ingestion;
mod spending;
mod budget;
mod usage;

// Re-export all public types and handlers
pub use shared::{
  AnalyticsState,
  Period,
  AnalyticsQuery,
  PaginationQuery,
  BudgetStatusQuery,
  AnalyticsEventRequest,
  EventResponse,
  Filters,
  Pagination,
  SpendingTotalResponse,
  AgentSpending,
  SpendingByAgentResponse,
  SpendingSummary,
  ProviderSpending,
  SpendingByProviderResponse,
  ProviderSpendingSummary,
  AvgCostResponse,
  BudgetStatus,
  BudgetSummary,
  BudgetStatusResponse,
  RequestUsageResponse,
  AgentTokenUsage,
  TokenUsageSummary,
  TokenUsageResponse,
  ModelUsage,
  ModelUsageSummary,
  ModelUsageResponse,
};

pub use ingestion::post_event;
pub use spending::{
  get_spending_total,
  get_spending_by_agent,
  get_spending_by_provider,
  get_spending_avg,
};
pub use budget::get_budget_status;
pub use usage::{
  get_usage_requests,
  get_usage_tokens,
  get_usage_models,
};
