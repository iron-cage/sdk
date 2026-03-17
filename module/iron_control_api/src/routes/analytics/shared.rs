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

/// Row type for spending by agent: (`agent_id`, `agent_name`, `spending_micros`, `request_count`, `budget_micros`)
pub type SpendingByAgentRow = (i64, Option<String>, i64, i64, Option<i64>);

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

/// Compute the percentage change between two values.
///
/// Returns `None` when the previous value is zero and current is non-zero
/// (infinite change). Returns `Some(0.0)` when both are zero.
#[must_use]
pub fn compute_change_percent(current: f64, previous: f64) -> Option<f64> {
  if previous == 0.0 {
    if current == 0.0 {
      Some(0.0)
    } else {
      None
    }
  } else {
    Some(((current - previous) / previous) * 100.0)
  }
}

impl Period {
  /// Compute the previous period's time range for period-over-period comparison.
  ///
  /// Returns `None` for `AllTime` (no meaningful previous period).
  ///
  /// # Panics
  ///
  /// May panic if time calculations produce invalid dates, which should never
  /// happen in practice (e.g., midnight is always valid, day 1 always exists).
  #[must_use]
  pub fn previous_period_range(&self) -> Option<(i64, i64)> {
    let now = Utc::now();

    match self {
      Period::AllTime => None,
      Period::Today => {
        // Previous = yesterday (midnight to midnight)
        let yesterday = (now - Duration::days(1)).date_naive();
        let start = yesterday
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let end = now
          .date_naive()
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let start_ms = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis();
        let end_ms =
          DateTime::<Utc>::from_naive_utc_and_offset(end, Utc).timestamp_millis() - 1;
        Some((start_ms, end_ms))
      }
      Period::Yesterday => {
        // Previous = day before yesterday
        let dby = (now - Duration::days(2)).date_naive();
        let start = dby
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let yesterday = (now - Duration::days(1)).date_naive();
        let end = yesterday
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let start_ms = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis();
        let end_ms =
          DateTime::<Utc>::from_naive_utc_and_offset(end, Utc).timestamp_millis() - 1;
        Some((start_ms, end_ms))
      }
      Period::Last7Days => {
        // Current: (now-7d, now). Previous: (now-14d, now-7d)
        let prev_start = (now - Duration::days(14)).timestamp_millis();
        let prev_end = (now - Duration::days(7)).timestamp_millis() - 1;
        Some((prev_start, prev_end))
      }
      Period::Last30Days => {
        // Current: (now-30d, now). Previous: (now-60d, now-30d)
        let prev_start = (now - Duration::days(60)).timestamp_millis();
        let prev_end = (now - Duration::days(30)).timestamp_millis() - 1;
        Some((prev_start, prev_end))
      }
      Period::ThisMonth => {
        // Previous = last month
        let first_of_this = now
          .date_naive()
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let last_month_any_day = first_of_this - Duration::days(1);
        let first_of_last = last_month_any_day
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let start = first_of_last
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let end = first_of_this
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let start_ms = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis();
        let end_ms =
          DateTime::<Utc>::from_naive_utc_and_offset(end, Utc).timestamp_millis() - 1;
        Some((start_ms, end_ms))
      }
      Period::LastMonth => {
        // Previous = month before last
        let first_of_this = now
          .date_naive()
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let last_month_any_day = first_of_this - Duration::days(1);
        let first_of_last = last_month_any_day
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let before_last_any_day = first_of_last - Duration::days(1);
        let first_of_before = before_last_any_day
          .with_day(1)
          .expect("INVARIANT: day 1 is valid for all months");
        let start = first_of_before
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let end = first_of_last
          .and_hms_opt(0, 0, 0)
          .expect("INVARIANT: midnight (00:00:00) is always valid");
        let start_ms = DateTime::<Utc>::from_naive_utc_and_offset(start, Utc).timestamp_millis();
        let end_ms =
          DateTime::<Utc>::from_naive_utc_and_offset(end, Utc).timestamp_millis() - 1;
        Some((start_ms, end_ms))
      }
    }
  }

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
  /// Optional provider key ID filter
  pub provider_key_id: Option<i64>,
  /// Include previous period comparison data
  #[serde(default)]
  pub compare: bool,
  /// Optional group-by field (e.g., "key" for by-provider endpoint)
  pub group_by: Option<String>,
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
  /// Optional provider key ID
  #[serde(default)]
  pub provider_key_id: Option<i64>,
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
  /// Provider key ID filter applied
  pub provider_key_id: Option<i64>,
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

/// Previous period comparison data for spending total
#[derive(Debug, Serialize)]
pub struct SpendingTotalComparison {
  /// Previous period total spending in dollars
  pub total_spend: f64,
  /// Percentage change from previous to current period (null if previous is zero)
  pub change_percent: Option<f64>,
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
  /// Previous period comparison (only present when compare=true)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub previous_period: Option<SpendingTotalComparison>,
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
  /// Provider key ID (when grouped by key)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub provider_key_id: Option<i64>,
  /// Provider key alias (when grouped by key)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub alias: Option<String>,
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
  /// Median cost per request in dollars
  pub median_cost_per_request: f64,
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

/// Previous period comparison data for request usage
#[derive(Debug, Serialize)]
pub struct RequestUsageComparison {
  /// Previous period total requests
  pub total_requests: i64,
  /// Previous period successful requests
  pub successful_requests: i64,
  /// Previous period failed requests
  pub failed_requests: i64,
  /// Previous period success rate as percentage
  pub success_rate: f64,
  /// Percentage change in total requests from previous to current period
  pub change_percent: Option<f64>,
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
  /// Previous period comparison (only present when compare=true)
  #[serde(skip_serializing_if = "Option::is_none")]
  pub previous_period: Option<RequestUsageComparison>,
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
  /// Optional provider ID filter
  pub provider_id: Option<String>,
  /// Optional provider key ID filter
  pub provider_key_id: Option<i64>,
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
  /// Optional provider key ID
  #[serde(default)]
  pub provider_key_id: Option<i64>,
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

#[cfg(test)]
mod tests {
  use super::*;
  use serde_json;

  // ========================================================================
  // compute_change_percent tests
  // ========================================================================

  #[test]
  fn change_percent_both_zero() {
    assert_eq!(compute_change_percent(0.0, 0.0), Some(0.0));
  }

  #[test]
  fn change_percent_previous_zero_current_positive() {
    assert_eq!(compute_change_percent(5.0, 0.0), None);
  }

  #[test]
  fn change_percent_previous_zero_current_negative() {
    assert_eq!(compute_change_percent(-3.0, 0.0), None);
  }

  #[test]
  fn change_percent_normal_increase() {
    let result = compute_change_percent(13.0, 10.0).unwrap();
    assert!((result - 30.0).abs() < f64::EPSILON, "expected 30.0, got {result}");
  }

  #[test]
  fn change_percent_normal_decrease() {
    let result = compute_change_percent(7.0, 10.0).unwrap();
    assert!((result - (-30.0)).abs() < f64::EPSILON, "expected -30.0, got {result}");
  }

  #[test]
  fn change_percent_no_change() {
    let result = compute_change_percent(10.0, 10.0).unwrap();
    assert!((result - 0.0).abs() < f64::EPSILON, "expected 0.0, got {result}");
  }

  #[test]
  fn change_percent_large_change() {
    let result = compute_change_percent(100.0, 1.0).unwrap();
    assert!((result - 9900.0).abs() < f64::EPSILON, "expected 9900.0, got {result}");
  }

  // ========================================================================
  // previous_period_range tests
  // ========================================================================

  #[test]
  fn previous_period_alltime_returns_none() {
    assert!(Period::AllTime.previous_period_range().is_none());
  }

  #[test]
  fn previous_period_today_returns_valid_range() {
    let range = Period::Today.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end, "start ({start}) should be < end ({end})");

    let current = Period::Today.to_range();
    assert!(end < current.0, "previous end ({end}) should be < current start ({})", current.0);
  }

  #[test]
  fn previous_period_yesterday_returns_valid_range() {
    let range = Period::Yesterday.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end);

    let current = Period::Yesterday.to_range();
    assert!(end < current.0, "previous end ({end}) should be < current start ({})", current.0);
  }

  #[test]
  fn previous_period_last7days_returns_valid_range() {
    let range = Period::Last7Days.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end);

    let current = Period::Last7Days.to_range();
    assert!(end < current.0);

    // Duration should be ~7 days (7 * 86400 * 1000 ms = 604_800_000)
    let duration_ms = end - start;
    let seven_days_ms: i64 = 7 * 24 * 60 * 60 * 1000;
    // Allow 1 second tolerance for the -1 ms adjustment
    assert!(
      (duration_ms - seven_days_ms).abs() < 1000,
      "duration {duration_ms} should be ~{seven_days_ms}"
    );
  }

  #[test]
  fn previous_period_last30days_returns_valid_range() {
    let range = Period::Last30Days.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end);

    let current = Period::Last30Days.to_range();
    assert!(end < current.0);

    // Duration should be ~30 days
    let duration_ms = end - start;
    let thirty_days_ms: i64 = 30 * 24 * 60 * 60 * 1000;
    assert!(
      (duration_ms - thirty_days_ms).abs() < 1000,
      "duration {duration_ms} should be ~{thirty_days_ms}"
    );
  }

  #[test]
  fn previous_period_this_month_returns_valid_range() {
    let range = Period::ThisMonth.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end);

    let current = Period::ThisMonth.to_range();
    assert!(end < current.0);
  }

  #[test]
  fn previous_period_last_month_returns_valid_range() {
    let range = Period::LastMonth.previous_period_range();
    assert!(range.is_some());
    let (start, end) = range.unwrap();
    assert!(start < end);

    let current = Period::LastMonth.to_range();
    assert!(end < current.0);
  }

  #[test]
  fn previous_period_non_negative_durations() {
    let periods = [
      Period::Today,
      Period::Yesterday,
      Period::Last7Days,
      Period::Last30Days,
      Period::ThisMonth,
      Period::LastMonth,
    ];
    for period in &periods {
      if let Some((start, end)) = period.previous_period_range() {
        assert!(end >= start, "{period:?}: end ({end}) >= start ({start})");
      }
    }
  }

  #[test]
  fn today_previous_matches_yesterday_range() {
    let today_prev = Period::Today.previous_period_range().unwrap();
    let yesterday_range = Period::Yesterday.to_range();
    assert_eq!(
      today_prev, yesterday_range,
      "Today's previous period {today_prev:?} should equal Yesterday's range {yesterday_range:?}"
    );
  }

  // ========================================================================
  // Serde serialization tests
  // ========================================================================

  fn make_filters() -> Filters {
    Filters {
      agent_id: None,
      provider_id: None,
      provider_key_id: None,
    }
  }

  #[test]
  fn spending_total_response_without_previous_period() {
    let resp = SpendingTotalResponse {
      total_spend: 42.5,
      currency: "USD".to_string(),
      period: "last-7-days".to_string(),
      filters: make_filters(),
      previous_period: None,
      calculated_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert!(
      json.get("previous_period").is_none(),
      "previous_period should be absent when None"
    );
  }

  #[test]
  fn spending_total_response_with_previous_period() {
    let resp = SpendingTotalResponse {
      total_spend: 42.5,
      currency: "USD".to_string(),
      period: "last-7-days".to_string(),
      filters: make_filters(),
      previous_period: Some(SpendingTotalComparison {
        total_spend: 30.0,
        change_percent: Some(41.67),
      }),
      calculated_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    let prev = json.get("previous_period").expect("previous_period should be present");
    assert_eq!(prev["total_spend"], 30.0);
    assert_eq!(prev["change_percent"], 41.67);
  }

  #[test]
  fn request_usage_response_without_previous_period() {
    let resp = RequestUsageResponse {
      total_requests: 100,
      successful_requests: 95,
      failed_requests: 5,
      success_rate: 0.95,
      period: "today".to_string(),
      filters: make_filters(),
      previous_period: None,
      calculated_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    assert!(
      json.get("previous_period").is_none(),
      "previous_period should be absent when None"
    );
  }

  #[test]
  fn request_usage_response_with_previous_period() {
    let resp = RequestUsageResponse {
      total_requests: 100,
      successful_requests: 95,
      failed_requests: 5,
      success_rate: 0.95,
      period: "today".to_string(),
      filters: make_filters(),
      previous_period: Some(RequestUsageComparison {
        total_requests: 80,
        successful_requests: 75,
        failed_requests: 5,
        success_rate: 0.9375,
        change_percent: Some(25.0),
      }),
      calculated_at: "2026-01-01T00:00:00Z".to_string(),
    };
    let json = serde_json::to_value(&resp).unwrap();
    let prev = json.get("previous_period").expect("previous_period should be present");
    assert_eq!(prev["total_requests"], 80);
    assert_eq!(prev["change_percent"], 25.0);
  }

  #[test]
  fn change_percent_null_in_json() {
    let comp = SpendingTotalComparison {
      total_spend: 10.0,
      change_percent: None,
    };
    let json = serde_json::to_value(&comp).unwrap();
    assert!(
      json["change_percent"].is_null(),
      "change_percent: None should serialize as null"
    );
  }

  // ========================================================================
  // AnalyticsQuery deserialization tests
  // ========================================================================

  #[test]
  fn analytics_query_default_compare_false() {
    // Empty JSON object should use defaults
    let q: AnalyticsQuery = serde_json::from_str("{}").unwrap();
    assert!(!q.compare, "default compare should be false");
  }

  #[test]
  fn analytics_query_compare_true() {
    let q: AnalyticsQuery = serde_json::from_str(r#"{"compare": true}"#).unwrap();
    assert!(q.compare);
  }

  #[test]
  fn analytics_query_compare_false() {
    let q: AnalyticsQuery = serde_json::from_str(r#"{"compare": false}"#).unwrap();
    assert!(!q.compare);
  }
}
