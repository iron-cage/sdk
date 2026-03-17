//! Analytics REST API endpoints (Protocol 012)
//!
//! Provides endpoints for:
//! - Event ingestion from `LlmRouter`
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

mod budget;
mod ingestion;
mod shared;
mod spending;
mod usage;

// Re-export all public types and handlers
pub use shared::{
  AgentSpending, AgentTokenUsage, AnalyticsEventRequest, AnalyticsEventWithAgent, AnalyticsQuery,
  AnalyticsState, AvgCostResponse, BudgetStatus, BudgetStatusQuery, BudgetStatusResponse,
  BudgetSummary, EventResponse, EventsListQuery, EventsListResponse, Filters, ModelUsage,
  ModelUsageResponse, ModelUsageSummary, Pagination, PaginationQuery, Period, ProviderSpending,
  ProviderSpendingSummary, RequestUsageComparison, RequestUsageResponse, SpendingByAgentResponse,
  SpendingByProviderResponse, SpendingSummary, SpendingTotalComparison, SpendingTotalResponse,
  TokenUsageResponse, TokenUsageSummary,
};

pub use budget::get_budget_status;
pub use ingestion::{list_events, post_event};
pub use spending::{
  get_spending_avg, get_spending_by_agent, get_spending_by_provider, get_spending_total,
};
pub use usage::{get_usage_models, get_usage_requests, get_usage_tokens};
