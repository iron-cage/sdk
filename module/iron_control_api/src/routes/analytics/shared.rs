//! Shared types and state for analytics endpoints
//!
//! Contains common types, enums, query parameters, response structures,
//! and database state used across all analytics endpoints.

use crate::ic_token::{IcTokenManager, IcTokenRateLimiter};
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::sync::Arc;

// ============================================================================
// Type Aliases (for complex query result types)
// ============================================================================

/// Row type for spending by agent: (`agent_id`, `agent_name`, `spending_micros`, `request_count`, budget)
pub type SpendingByAgentRow = (i64, Option<String>, i64, i64, Option<f64>);

/// Row type for token usage by agent: (`agent_id`, `agent_name`, `input_tokens`, `output_tokens`, `request_count`)
pub type TokensByAgentRow = (i64, Option<String>, i64, i64, i64);

/// Row type for model usage: (model, provider, `request_count`, `spending_micros`, `input_tokens`, `output_tokens`)
pub type ModelUsageRow = (String, String, i64, i64, i64, i64);

// ============================================================================
// Period Enum
// ============================================================================

/// Period filter for time-based queries
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Period {
  /// Current day from midnight
  Today,
  /// Previous day (midnight to midnight)
  Yesterday,
  /// Last 7 days from now
  Last7Days,
  /// Last 30 days from now
  Last30Days,
  /// Current calendar month from 1st
  ThisMonth,
  /// Previous calendar month
  LastMonth,
  /// All time (no date filter)
  #[default]
  AllTime,
}

impl Period {
  /// Convert period to (`start_ms`, `end_ms`) range
  ///
  /// # Panics
  ///
  /// May panic if time calculations produce invalid dates, which should never
  /// happen in practice (e.g., midnight is always valid, day 1 always exists).
  #[must_use]
  pub fn to_range(&self) -> (i64, i64) {
    let now = Utc::now();
    let end_ms = now.timestamp_millis();

    let start = match self {
      Period::Today => now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("INVARIANT: midnight (00:00:00) is always valid"),
      Period::Yesterday => {
        let yesterday = now - Duration::days(1);
        yesterday
          .date_naive()
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid")
      }
      Period::Last7Days => (now - Duration::days(7)).naive_utc(),
      Period::Last30Days => (now - Duration::days(30)).naive_utc(),
      Period::ThisMonth => {
        let first_of_month = now
          .date_naive()
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        first_of_month
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid")
      }
      Period::LastMonth => {
        let first_of_this_month = now
          .date_naive()
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let last_month = first_of_this_month - Duration::days(1);
        last_month
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months")
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid")
      }
      Period::AllTime => return (0, end_ms),
    };

    let start_ms = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis();

    // For Yesterday, end at end of yesterday
    let end_ms = if *self == Period::Yesterday {
      let today_start = now
        .date_naive()
        .and_hms_opt(0, 0, 0)
        .expect("INVARIANT: midnight (00:00:00) is always valid");
      DateTime::<Utc>::from_naive_utc_and_offset(today_start, Utc).timestamp_millis() - 1
    } else if *self == Period::LastMonth {
      let first_of_this_month = now
        .date_naive()
        .with_day(1)
        .expect("INVARIANT: day 1 is valid for all months");
      DateTime::<Utc>::from_naive_utc_and_offset(
        first_of_this_month
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid"),
        Utc,
      )
      .timestamp_millis()
        - 1
    } else {
      end_ms
    };

    (start_ms, end_ms)
  }
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Common query parameters for analytics endpoints
#[derive(Debug, Clone, Default, Deserialize)]
pub struct AnalyticsQuery {
  /// Time period filter
  #[serde(default)]
  pub period: Period,
  /// Optional agent ID filter
  pub agent_id: Option<i64>,
  /// Optional provider ID filter
  pub provider_id: Option<String>,
}

/// Pagination parameters
#[derive(Debug, Clone, Deserialize)]
pub struct PaginationQuery {
  /// Current page number (1-indexed)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Number of items per page
  #[serde(default = "default_per_page")]
  pub per_page: u32,
}

fn default_page() -> u32 {
  1
}
fn default_per_page() -> u32 {
  50
}

impl Default for PaginationQuery {
  fn default() -> Self {
    Self {
      page: 1,
      per_page: 50,
    }
  }
}

/// Budget status query parameters
#[derive(Debug, Clone, Default, Deserialize)]
pub struct BudgetStatusQuery {
  /// Filter by budget usage threshold percentage
  pub threshold: Option<u32>,
  /// Filter by budget status (e.g., "active", "exhausted")
  pub status: Option<String>,
  /// Optional agent ID filter
  pub agent_id: Option<i64>,
  /// Current page number (1-indexed)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Number of items per page
  #[serde(default = "default_per_page")]
  pub per_page: u32,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// POST /api/v1/analytics/events - Request body
#[derive(Debug, Clone, Deserialize)]
pub struct AnalyticsEventRequest {
  /// IC Token for authentication (required) - proves agent identity
  pub ic_token: String,
  /// Unique event identifier
  pub event_id: String,
  /// Event timestamp in milliseconds
  pub timestamp_ms: i64,
  /// Type of event (e.g., `llm_call`, `error`)
  pub event_type: String,
  /// Model name used
  pub model: String,
  /// Provider name (e.g., "openai", "anthropic")
  pub provider: String,
  /// Number of input tokens consumed
  #[serde(default)]
  pub input_tokens: Option<i64>,
  /// Number of output tokens generated
  #[serde(default)]
  pub output_tokens: Option<i64>,
  /// Cost in microdollars ($0.000001)
  #[serde(default)]
  pub cost_micros: Option<i64>,
  /// Optional provider-specific identifier
  // agent_id is derived from ic_token claims, not provided by caller
  #[serde(default)]
  pub provider_id: Option<String>,
  /// Error code if event represents an error
  #[serde(default)]
  pub error_code: Option<String>,
  /// Error message if event represents an error
  #[serde(default)]
  pub error_message: Option<String>,
}

/// POST /api/v1/analytics/events - Response
#[derive(Debug, Serialize)]
pub struct EventResponse {
  /// Event identifier echoed back
  pub event_id: String,
  /// Processing status (e.g., "recorded")
  pub status: String,
}

/// Filter info in response
#[derive(Debug, Clone, Serialize)]
pub struct Filters {
  /// Agent ID filter applied
  pub agent_id: Option<i64>,
  /// Provider ID filter applied
  pub provider_id: Option<String>,
}

/// Pagination info in response
#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
  /// Current page number (1-indexed)
  pub page: u32,
  /// Items per page
  pub per_page: u32,
  /// Total number of items
  pub total: u32,
  /// Total number of pages
  pub total_pages: u32,
}

impl Pagination {
  /// Create new pagination info
  #[must_use]
  pub fn new(page: u32, per_page: u32, total: u32) -> Self {
    let total_pages = total.div_ceil(per_page);
    Self {
      page,
      per_page,
      total,
      total_pages,
    }
  }
}

/// GET /api/v1/analytics/spending/total - Response
#[derive(Debug, Serialize)]
pub struct SpendingTotalResponse {
  /// Total spending in dollars
  pub total_spend: f64,
  /// Currency code (e.g., "USD")
  pub currency: String,
  /// Period description (e.g., "last-7-days")
  pub period: String,
  /// Filters applied to query
  pub filters: Filters,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Agent spending record
#[derive(Debug, Serialize)]
pub struct AgentSpending {
  /// Agent database ID
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Total spending in dollars
  pub spending: f64,
  /// Allocated budget in dollars
  pub budget: f64,
  /// Percentage of budget used
  pub percent_used: f64,
  /// Number of API requests made
  pub request_count: i64,
}

/// GET /api/v1/analytics/spending/by-agent - Response
#[derive(Debug, Serialize)]
pub struct SpendingByAgentResponse {
  /// List of agent spending records
  pub data: Vec<AgentSpending>,
  /// Aggregated summary statistics
  pub summary: SpendingSummary,
  /// Pagination information
  pub pagination: Pagination,
  /// Period description
  pub period: String,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Spending summary
#[derive(Debug, Serialize)]
pub struct SpendingSummary {
  /// Total spending across all agents in dollars
  pub total_spend: f64,
  /// Total budget allocated across all agents in dollars
  pub total_budget: f64,
  /// Number of agents included
  pub total_agents: u32,
}

/// Provider spending record
#[derive(Debug, Serialize)]
pub struct ProviderSpending {
  /// Provider name (e.g., "openai", "anthropic")
  pub provider: String,
  /// Total spending in dollars
  pub spending: f64,
  /// Number of API requests made
  pub request_count: i64,
  /// Average cost per request in dollars
  pub avg_cost_per_request: f64,
  /// Number of agents using this provider
  pub agent_count: i64,
}

/// GET /api/v1/analytics/spending/by-provider - Response
#[derive(Debug, Serialize)]
pub struct SpendingByProviderResponse {
  /// List of provider spending records
  pub data: Vec<ProviderSpending>,
  /// Aggregated summary statistics
  pub summary: ProviderSpendingSummary,
  /// Period description
  pub period: String,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Provider spending summary
#[derive(Debug, Serialize)]
pub struct ProviderSpendingSummary {
  /// Total spending across all providers in dollars
  pub total_spend: f64,
  /// Total requests across all providers
  pub total_requests: i64,
  /// Number of unique providers
  pub providers_count: u32,
}

/// GET /api/v1/analytics/spending/avg-per-request - Response
#[derive(Debug, Serialize)]
pub struct AvgCostResponse {
  /// Average cost per request in dollars
  pub average_cost_per_request: f64,
  /// Total number of requests
  pub total_requests: i64,
  /// Total spending in dollars
  pub total_spend: f64,
  /// Minimum cost per request in dollars
  pub min_cost_per_request: f64,
  /// Maximum cost per request in dollars
  pub max_cost_per_request: f64,
  /// Period description
  pub period: String,
  /// Filters applied to query
  pub filters: Filters,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Budget status record
#[derive(Debug, Serialize)]
pub struct BudgetStatus {
  /// Agent database ID
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Allocated budget in dollars
  pub budget: f64,
  /// Amount spent in dollars
  pub spent: f64,
  /// Remaining budget in dollars
  pub remaining: f64,
  /// Percentage of budget used
  pub percent_used: f64,
  /// Status label (e.g., "active", "exhausted")
  pub status: String,
  /// Risk level (e.g., "low", "medium", "high", "critical")
  pub risk_level: String,
}

/// Budget summary
#[derive(Debug, Serialize)]
pub struct BudgetSummary {
  /// Total number of agents
  pub total_agents: u32,
  /// Number of active agents (budget remaining)
  pub active: u32,
  /// Number of exhausted agents (budget depleted)
  pub exhausted: u32,
  /// Number of agents at critical risk level (>90% used)
  pub critical: u32,
  /// Number of agents at high risk level (75-90% used)
  pub high: u32,
  /// Number of agents at medium risk level (50-75% used)
  pub medium: u32,
  /// Number of agents at low risk level (<50% used)
  pub low: u32,
}

/// GET /api/v1/analytics/budget/status - Response
#[derive(Debug, Serialize)]
pub struct BudgetStatusResponse {
  /// List of budget status records
  pub data: Vec<BudgetStatus>,
  /// Aggregated summary statistics
  pub summary: BudgetSummary,
  /// Pagination information
  pub pagination: Pagination,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// GET /api/v1/analytics/usage/requests - Response
#[derive(Debug, Serialize)]
pub struct RequestUsageResponse {
  /// Total number of API requests
  pub total_requests: i64,
  /// Number of successful requests
  pub successful_requests: i64,
  /// Number of failed requests
  pub failed_requests: i64,
  /// Success rate as percentage (0.0-1.0)
  pub success_rate: f64,
  /// Period description
  pub period: String,
  /// Filters applied to query
  pub filters: Filters,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Agent token usage record
#[derive(Debug, Serialize)]
pub struct AgentTokenUsage {
  /// Agent database ID
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Total input tokens consumed
  pub input_tokens: i64,
  /// Total output tokens generated
  pub output_tokens: i64,
  /// Total tokens (input + output)
  pub total_tokens: i64,
  /// Number of API requests made
  pub request_count: i64,
  /// Average tokens per request
  pub avg_tokens_per_request: i64,
}

/// Token usage summary
#[derive(Debug, Serialize)]
pub struct TokenUsageSummary {
  /// Total input tokens across all agents
  pub total_input_tokens: i64,
  /// Total output tokens across all agents
  pub total_output_tokens: i64,
  /// Total tokens (input + output) across all agents
  pub total_tokens: i64,
}

/// GET /api/v1/analytics/usage/tokens/by-agent - Response
#[derive(Debug, Serialize)]
pub struct TokenUsageResponse {
  /// List of agent token usage records
  pub data: Vec<AgentTokenUsage>,
  /// Aggregated summary statistics
  pub summary: TokenUsageSummary,
  /// Pagination information
  pub pagination: Pagination,
  /// Period description
  pub period: String,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Model usage record
#[derive(Debug, Serialize)]
pub struct ModelUsage {
  /// Model name
  pub model: String,
  /// Provider name (e.g., "openai", "anthropic")
  pub provider: String,
  /// Number of API requests made
  pub request_count: i64,
  /// Total spending in dollars
  pub spending: f64,
  /// Total input tokens consumed
  pub input_tokens: i64,
  /// Total output tokens generated
  pub output_tokens: i64,
}

/// Model usage summary
#[derive(Debug, Serialize)]
pub struct ModelUsageSummary {
  /// Number of unique models used
  pub unique_models: u32,
  /// Total requests across all models
  pub total_requests: i64,
  /// Total spending across all models in dollars
  pub total_spend: f64,
}

/// GET /api/v1/analytics/usage/models - Response
#[derive(Debug, Serialize)]
pub struct ModelUsageResponse {
  /// List of model usage records
  pub data: Vec<ModelUsage>,
  /// Aggregated summary statistics
  pub summary: ModelUsageSummary,
  /// Pagination information
  pub pagination: Pagination,
  /// Period description
  pub period: String,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Events list query parameters
#[derive(Debug, Clone, Default, Deserialize)]
pub struct EventsListQuery {
  /// Time period filter
  #[serde(default)]
  pub period: Period,
  /// Optional agent ID filter
  pub agent_id: Option<i64>,
  /// Current page number (1-indexed)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Number of items per page
  #[serde(default = "default_events_per_page")]
  pub per_page: u32,
}

fn default_events_per_page() -> u32 {
  10
}

/// GET /api/v1/analytics/events - Response
#[derive(Debug, Serialize)]
pub struct EventsListResponse {
  /// List of analytics events with agent information
  pub data: Vec<AnalyticsEventWithAgent>,
  /// Pagination information
  pub pagination: Pagination,
  /// Period description
  pub period: String,
  /// ISO 8601 timestamp when calculated
  pub calculated_at: String,
}

/// Event with agent name
#[derive(Debug, Serialize, FromRow)]
pub struct AnalyticsEventWithAgent {
  /// Unique event identifier
  pub event_id: String,
  /// Event timestamp in milliseconds
  pub timestamp_ms: i64,
  /// Type of event (e.g., `llm_call`, `error`)
  pub event_type: String,
  /// Model name used
  pub model: String,
  /// Provider name (e.g., "openai", "anthropic")
  pub provider: String,
  /// Number of input tokens consumed
  pub input_tokens: i64,
  /// Number of output tokens generated
  pub output_tokens: i64,
  /// Cost in microdollars ($0.000001)
  pub cost_micros: i64,
  /// Agent database ID
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Error code if event represents an error
  pub error_code: Option<String>,
  /// Error message if event represents an error
  pub error_message: Option<String>,
}

// ============================================================================
// State
// ============================================================================

/// Analytics state containing database pool and IC token manager
#[derive(Clone, Debug)]
pub struct AnalyticsState {
  /// `SQLite` database connection pool
  pub pool: SqlitePool,
  /// IC token manager for authentication
  pub ic_token_manager: Arc<IcTokenManager>,
  /// Rate limiter for IC token validation
  pub ic_token_rate_limiter: IcTokenRateLimiter,
}

impl AnalyticsState {
  /// Create new analytics state
  ///
  /// # Arguments
  /// * `database_url` - `SQLite` database connection URL
  /// * `ic_token_manager` - Shared IC token manager for authentication
  /// * `ic_token_rate_limiter` - Shared rate limiter for IC token validation
  ///
  /// # Errors
  ///
  /// Returns error if:
  /// - Database connection fails
  /// - Migration execution fails
  pub async fn new(
    database_url: &str,
    ic_token_manager: Arc<IcTokenManager>,
    ic_token_rate_limiter: IcTokenRateLimiter,
  ) -> Result<Self, Box<dyn core::error::Error>> {
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect(database_url)
      .await?;

    // Apply all migrations from the single source of truth (iron_token_manager)
    iron_token_manager::migrations::apply_all_migrations(&pool).await?;

    Ok(Self {
      pool,
      ic_token_manager,
      ic_token_rate_limiter,
    })
  }
}

// ============================================================================
// Database Row Types
// ============================================================================

#[derive(Debug, FromRow)]
pub struct AgentBudgetRow {
  pub agent_id: i64,
  pub agent_name: String,
  pub total_allocated: i64,
  pub total_spent: i64,
  pub budget_remaining: i64,
}
