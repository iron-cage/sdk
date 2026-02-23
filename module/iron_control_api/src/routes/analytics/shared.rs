//! Shared types and state for analytics endpoints
//!
//! Contains common types, enums, query parameters, response structures,
//! and database state used across all analytics endpoints.

use crate::ic_token::IcTokenManager;
use chrono::{DateTime, Datelike, Duration, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::sync::Arc;

// ============================================================================
// Type Aliases (for complex query result types)
// ============================================================================

/// Row type for spending by agent: (`agent_id`, `agent_name`, `spending_micros`, `request_count`, `budget`)
pub type SpendingByAgentRow = (i64, Option<String>, i64, i64, Option<f64>);

/// Row type for token usage by agent: (`agent_id`, `agent_name`, `input_tokens`, `output_tokens`, `request_count`)
pub type TokensByAgentRow = (i64, Option<String>, i64, i64, i64);

/// Row type for model usage: (`model`, `provider`, `request_count`, `spending_micros`, `input_tokens`, `output_tokens`)
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
  /// Previous day
  Yesterday,
  /// Rolling last 7 days
  Last7Days,
  /// Rolling last 30 days
  Last30Days,
  /// Current calendar month
  ThisMonth,
  /// Previous calendar month
  LastMonth,
  /// All recorded history
  #[default]
  AllTime,
}

impl Period {
  /// Convert period to (`start_ms`, `end_ms`) range
  ///
  /// # Panics
  ///
  /// Panics if midnight `NaiveDateTime` construction fails (should never happen).
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
  /// Page number (starts at 1)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Items per page
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
  /// Budget usage threshold percentage
  pub threshold: Option<u32>,
  /// Filter by budget status
  pub status: Option<String>,
  /// Optional agent ID filter
  pub agent_id: Option<i64>,
  /// Page number (starts at 1)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Items per page
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
  /// Type of analytics event
  pub event_type: String,
  /// Model name used in request
  pub model: String,
  /// Provider name (e.g. `OpenAI`, `Anthropic`)
  pub provider: String,
  /// Number of input tokens consumed
  #[serde(default)]
  pub input_tokens: Option<i64>,
  /// Number of output tokens generated
  #[serde(default)]
  pub output_tokens: Option<i64>,
  /// Cost in microdollars
  #[serde(default)]
  pub cost_micros: Option<i64>,
  // agent_id is derived from ic_token claims, not provided by caller
  /// Optional provider identifier
  #[serde(default)]
  pub provider_id: Option<String>,
  /// Error code if request failed
  #[serde(default)]
  pub error_code: Option<String>,
  /// Error description if request failed
  #[serde(default)]
  pub error_message: Option<String>,
}

/// POST /api/v1/analytics/events - Response
#[derive(Debug, Serialize)]
pub struct EventResponse {
  /// Recorded event identifier
  pub event_id: String,
  /// Ingestion result status
  pub status: String,
}

/// Filter info in response
#[derive(Debug, Clone, Serialize)]
pub struct Filters {
  /// Applied agent ID filter
  pub agent_id: Option<i64>,
  /// Applied provider ID filter
  pub provider_id: Option<String>,
}

/// Pagination info in response
#[derive(Debug, Clone, Serialize)]
pub struct Pagination {
  /// Current page number
  pub page: u32,
  /// Items per page
  pub per_page: u32,
  /// Total number of items
  pub total: u32,
  /// Total number of pages
  pub total_pages: u32,
}

impl Pagination {
  /// Create pagination from page, size, and total count
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
  /// Total spending amount
  pub total_spend: f64,
  /// Currency code (e.g. USD)
  pub currency: String,
  /// Queried time period
  pub period: String,
  /// Applied query filters
  pub filters: Filters,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Agent spending record
#[derive(Debug, Serialize)]
pub struct AgentSpending {
  /// Agent identifier
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Total spending amount
  pub spending: f64,
  /// Allocated budget amount
  pub budget: f64,
  /// Budget usage percentage
  pub percent_used: f64,
  /// Total number of requests
  pub request_count: i64,
}

/// GET /api/v1/analytics/spending/by-agent - Response
#[derive(Debug, Serialize)]
pub struct SpendingByAgentResponse {
  /// Per-agent spending records
  pub data: Vec<AgentSpending>,
  /// Aggregated spending summary
  pub summary: SpendingSummary,
  /// Pagination metadata
  pub pagination: Pagination,
  /// Queried time period
  pub period: String,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Spending summary
#[derive(Debug, Serialize)]
pub struct SpendingSummary {
  /// Total spending across all agents
  pub total_spend: f64,
  /// Total budget across all agents
  pub total_budget: f64,
  /// Number of agents
  pub total_agents: u32,
}

/// Provider spending record
#[derive(Debug, Serialize)]
pub struct ProviderSpending {
  /// Provider name
  pub provider: String,
  /// Total spending for provider
  pub spending: f64,
  /// Number of requests to provider
  pub request_count: i64,
  /// Average cost per request
  pub avg_cost_per_request: f64,
  /// Number of agents using provider
  pub agent_count: i64,
}

/// GET /api/v1/analytics/spending/by-provider - Response
#[derive(Debug, Serialize)]
pub struct SpendingByProviderResponse {
  /// Per-provider spending records
  pub data: Vec<ProviderSpending>,
  /// Aggregated provider spending summary
  pub summary: ProviderSpendingSummary,
  /// Queried time period
  pub period: String,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Provider spending summary
#[derive(Debug, Serialize)]
pub struct ProviderSpendingSummary {
  /// Total spending across all providers
  pub total_spend: f64,
  /// Total request count across providers
  pub total_requests: i64,
  /// Number of distinct providers
  pub providers_count: u32,
}

/// GET /api/v1/analytics/spending/avg-per-request - Response
#[derive(Debug, Serialize)]
pub struct AvgCostResponse {
  /// Average cost per request
  pub average_cost_per_request: f64,
  /// Total number of requests
  pub total_requests: i64,
  /// Total spending amount
  pub total_spend: f64,
  /// Minimum single-request cost
  pub min_cost_per_request: f64,
  /// Maximum single-request cost
  pub max_cost_per_request: f64,
  /// Queried time period
  pub period: String,
  /// Applied query filters
  pub filters: Filters,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Budget status record
#[derive(Debug, Serialize)]
pub struct BudgetStatus {
  /// Agent identifier
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Allocated budget amount
  pub budget: f64,
  /// Amount already spent
  pub spent: f64,
  /// Remaining budget amount
  pub remaining: f64,
  /// Budget usage percentage
  pub percent_used: f64,
  /// Current budget status label
  pub status: String,
  /// Risk level classification
  pub risk_level: String,
}

/// Budget summary
#[derive(Debug, Serialize)]
pub struct BudgetSummary {
  /// Total number of agents
  pub total_agents: u32,
  /// Agents with active budget
  pub active: u32,
  /// Agents with exhausted budget
  pub exhausted: u32,
  /// Agents at critical usage level
  pub critical: u32,
  /// Agents at high usage level
  pub high: u32,
  /// Agents at medium usage level
  pub medium: u32,
  /// Agents at low usage level
  pub low: u32,
}

/// GET /api/v1/analytics/budget/status - Response
#[derive(Debug, Serialize)]
pub struct BudgetStatusResponse {
  /// Per-agent budget status records
  pub data: Vec<BudgetStatus>,
  /// Aggregated budget summary
  pub summary: BudgetSummary,
  /// Pagination metadata
  pub pagination: Pagination,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// GET /api/v1/analytics/usage/requests - Response
#[derive(Debug, Serialize)]
pub struct RequestUsageResponse {
  /// Total number of requests
  pub total_requests: i64,
  /// Count of successful requests
  pub successful_requests: i64,
  /// Count of failed requests
  pub failed_requests: i64,
  /// Success rate as percentage
  pub success_rate: f64,
  /// Queried time period
  pub period: String,
  /// Applied query filters
  pub filters: Filters,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Agent token usage record
#[derive(Debug, Serialize)]
pub struct AgentTokenUsage {
  /// Agent identifier
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Total input tokens consumed
  pub input_tokens: i64,
  /// Total output tokens generated
  pub output_tokens: i64,
  /// Combined input and output tokens
  pub total_tokens: i64,
  /// Total number of requests
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
  /// Combined total tokens
  pub total_tokens: i64,
}

/// GET /api/v1/analytics/usage/tokens/by-agent - Response
#[derive(Debug, Serialize)]
pub struct TokenUsageResponse {
  /// Per-agent token usage records
  pub data: Vec<AgentTokenUsage>,
  /// Aggregated token usage summary
  pub summary: TokenUsageSummary,
  /// Pagination metadata
  pub pagination: Pagination,
  /// Queried time period
  pub period: String,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Model usage record
#[derive(Debug, Serialize)]
pub struct ModelUsage {
  /// Model name
  pub model: String,
  /// Provider name
  pub provider: String,
  /// Number of requests using this model
  pub request_count: i64,
  /// Total spending for this model
  pub spending: f64,
  /// Total input tokens for this model
  pub input_tokens: i64,
  /// Total output tokens for this model
  pub output_tokens: i64,
}

/// Model usage summary
#[derive(Debug, Serialize)]
pub struct ModelUsageSummary {
  /// Number of distinct models used
  pub unique_models: u32,
  /// Total request count across models
  pub total_requests: i64,
  /// Total spending across models
  pub total_spend: f64,
}

/// GET /api/v1/analytics/usage/models - Response
#[derive(Debug, Serialize)]
pub struct ModelUsageResponse {
  /// Per-model usage records
  pub data: Vec<ModelUsage>,
  /// Aggregated model usage summary
  pub summary: ModelUsageSummary,
  /// Pagination metadata
  pub pagination: Pagination,
  /// Queried time period
  pub period: String,
  /// Response calculation timestamp
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
  /// Page number (starts at 1)
  #[serde(default = "default_page")]
  pub page: u32,
  /// Events per page
  #[serde(default = "default_events_per_page")]
  pub per_page: u32,
}

fn default_events_per_page() -> u32 {
  10
}

/// GET /api/v1/analytics/events - Response
#[derive(Debug, Serialize)]
pub struct EventsListResponse {
  /// List of analytics events
  pub data: Vec<AnalyticsEventWithAgent>,
  /// Pagination metadata
  pub pagination: Pagination,
  /// Queried time period
  pub period: String,
  /// Response calculation timestamp
  pub calculated_at: String,
}

/// Event with agent name
#[derive(Debug, Serialize, FromRow)]
pub struct AnalyticsEventWithAgent {
  /// Unique event identifier
  pub event_id: String,
  /// Event timestamp in milliseconds
  pub timestamp_ms: i64,
  /// Type of analytics event
  pub event_type: String,
  /// Model name used in request
  pub model: String,
  /// Provider name
  pub provider: String,
  /// Number of input tokens consumed
  pub input_tokens: i64,
  /// Number of output tokens generated
  pub output_tokens: i64,
  /// Cost in microdollars
  pub cost_micros: i64,
  /// Agent identifier
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Error code if request failed
  pub error_code: Option<String>,
  /// Error description if request failed
  pub error_message: Option<String>,
}

// ============================================================================
// State
// ============================================================================

/// Analytics state containing database pool and IC token manager
#[derive(Debug, Clone)]
pub struct AnalyticsState {
  /// `SQLite` connection pool
  pub pool: SqlitePool,
  /// Shared IC token manager
  pub ic_token_manager: Arc<IcTokenManager>,
}

impl AnalyticsState {
  /// Create new analytics state
  ///
  /// # Arguments
  /// * `database_url` - `SQLite` database connection URL
  /// * `ic_token_secret` - Secret for verifying IC tokens
  ///
  /// # Errors
  ///
  /// Returns an error if the database connection or migration fails.
  pub async fn new(
    database_url: &str,
    ic_token_secret: String,
  ) -> Result<Self, Box<dyn core::error::Error>> {
    let pool = SqlitePoolOptions::new()
      .max_connections(5)
      .connect(database_url)
      .await?;

    // Run migration
    let migration = include_str!("../../../migrations/011_create_analytics_events.sql");
    sqlx::raw_sql(migration).execute(&pool).await?;

    let ic_token_manager = Arc::new(IcTokenManager::new(ic_token_secret));

    Ok(Self {
      pool,
      ic_token_manager,
    })
  }
}

// ============================================================================
// Database Row Types
// ============================================================================

/// Database row for agent budget data
#[derive(Debug, FromRow)]
pub struct AgentBudgetRow {
  /// Agent identifier
  pub agent_id: i64,
  /// Agent display name
  pub agent_name: String,
  /// Total allocated budget in microdollars
  pub total_allocated: i64,
  /// Total spent in microdollars
  pub total_spent: i64,
  /// Remaining budget in microdollars
  pub budget_remaining: i64,
}
