//! Shared types and state for analytics endpoints
//!
//! Contains common types, enums, query parameters, response structures,
//! and database state used across all analytics endpoints.

use chrono::{ DateTime, Datelike, Duration, Utc };
use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, SqlitePool, sqlite::SqlitePoolOptions };
use std::sync::Arc;
use crate::ic_token::IcTokenManager;

// ============================================================================
// Type Aliases (for complex query result types)
// ============================================================================

/// Row type for spending by agent: (agent_id, agent_name, spending_micros, request_count, budget)
pub type SpendingByAgentRow = ( i64, Option< String >, i64, i64, Option< f64 > );

/// Row type for token usage by agent: (agent_id, agent_name, input_tokens, output_tokens, request_count)
pub type TokensByAgentRow = ( i64, Option< String >, i64, i64, i64 );

/// Row type for model usage: (model, provider, request_count, spending_micros, input_tokens, output_tokens)
pub type ModelUsageRow = ( String, String, i64, i64, i64, i64 );

// ============================================================================
// Period Enum
// ============================================================================

/// Period filter for time-based queries
#[derive( Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq )]
#[serde( rename_all = "kebab-case" )]
pub enum Period
{
  Today,
  Yesterday,
  Last7Days,
  Last30Days,
  ThisMonth,
  LastMonth,
  #[default]
  AllTime,
}

impl Period
{
  /// Convert period to (start_ms, end_ms) range
  pub fn to_range( &self ) -> ( i64, i64 )
  {
    let now = Utc::now();
    let end_ms = now.timestamp_millis();

    let start = match self
    {
      Period::Today => now.date_naive().and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" ),
      Period::Yesterday =>
      {
        let yesterday = now - Duration::days( 1 );
        yesterday.date_naive().and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" )
      }
      Period::Last7Days => ( now - Duration::days( 7 ) ).naive_utc(),
      Period::Last30Days => ( now - Duration::days( 30 ) ).naive_utc(),
      Period::ThisMonth =>
      {
        let first_of_month = now.date_naive().with_day( 1 ).expect( "INVARIANT: day 1 is valid for all months" );
        first_of_month.and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" )
      }
      Period::LastMonth =>
      {
        let first_of_this_month = now.date_naive().with_day( 1 ).expect( "INVARIANT: day 1 is valid for all months" );
        let last_month = first_of_this_month - Duration::days( 1 );
        last_month.with_day( 1 ).expect( "INVARIANT: day 1 is valid for all months" ).and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" )
      }
      Period::AllTime => return ( 0, end_ms ),
    };

    let start_ms = DateTime::<Utc>::from_naive_utc_and_offset( start, Utc ).timestamp_millis();

    // For Yesterday, end at end of yesterday
    let end_ms = if *self == Period::Yesterday
    {
      let today_start = now.date_naive().and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" );
      DateTime::<Utc>::from_naive_utc_and_offset( today_start, Utc ).timestamp_millis() - 1
    }
    else if *self == Period::LastMonth
    {
      let first_of_this_month = now.date_naive().with_day( 1 ).expect( "INVARIANT: day 1 is valid for all months" );
      DateTime::<Utc>::from_naive_utc_and_offset(
        first_of_this_month.and_hms_opt( 0, 0, 0 ).expect( "INVARIANT: midnight (00:00:00) is always valid" ),
        Utc
      ).timestamp_millis() - 1
    }
    else
    {
      end_ms
    };

    ( start_ms, end_ms )
  }
}

// ============================================================================
// Query Parameter Types
// ============================================================================

/// Common query parameters for analytics endpoints
#[derive( Debug, Clone, Default, Deserialize )]
pub struct AnalyticsQuery
{
  #[serde( default )]
  pub period: Period,
  pub agent_id: Option< i64 >,
  pub provider_id: Option< String >,
}

/// Pagination parameters
#[derive( Debug, Clone, Deserialize )]
pub struct PaginationQuery
{
  #[serde( default = "default_page" )]
  pub page: u32,
  #[serde( default = "default_per_page" )]
  pub per_page: u32,
}

fn default_page() -> u32 { 1 }
fn default_per_page() -> u32 { 50 }

impl Default for PaginationQuery
{
  fn default() -> Self
  {
    Self { page: 1, per_page: 50 }
  }
}

/// Budget status query parameters
#[derive( Debug, Clone, Default, Deserialize )]
pub struct BudgetStatusQuery
{
  pub threshold: Option< u32 >,
  pub status: Option< String >,
  pub agent_id: Option< i64 >,
  #[serde( default = "default_page" )]
  pub page: u32,
  #[serde( default = "default_per_page" )]
  pub per_page: u32,
}

// ============================================================================
// Request/Response Types
// ============================================================================

/// POST /api/v1/analytics/events - Request body
#[derive( Debug, Clone, Deserialize )]
pub struct AnalyticsEventRequest
{
  /// IC Token for authentication (required) - proves agent identity
  pub ic_token: String,
  pub event_id: String,
  pub timestamp_ms: i64,
  pub event_type: String,
  pub model: String,
  pub provider: String,
  #[serde( default )]
  pub input_tokens: Option< i64 >,
  #[serde( default )]
  pub output_tokens: Option< i64 >,
  #[serde( default )]
  pub cost_micros: Option< i64 >,
  // agent_id is derived from ic_token claims, not provided by caller
  #[serde( default )]
  pub provider_id: Option< String >,
  #[serde( default )]
  pub error_code: Option< String >,
  #[serde( default )]
  pub error_message: Option< String >,
}

/// POST /api/v1/analytics/events - Response
#[derive( Debug, Serialize )]
pub struct EventResponse
{
  pub event_id: String,
  pub status: String,
}

/// Filter info in response
#[derive( Debug, Clone, Serialize )]
pub struct Filters
{
  pub agent_id: Option< i64 >,
  pub provider_id: Option< String >,
}

/// Pagination info in response
#[derive( Debug, Clone, Serialize )]
pub struct Pagination
{
  pub page: u32,
  pub per_page: u32,
  pub total: u32,
  pub total_pages: u32,
}

impl Pagination
{
  pub fn new( page: u32, per_page: u32, total: u32 ) -> Self
  {
    let total_pages = ( total + per_page - 1 ) / per_page;
    Self { page, per_page, total, total_pages }
  }
}

/// GET /api/v1/analytics/spending/total - Response
#[derive( Debug, Serialize )]
pub struct SpendingTotalResponse
{
  pub total_spend: f64,
  pub currency: String,
  pub period: String,
  pub filters: Filters,
  pub calculated_at: String,
}

/// Agent spending record
#[derive( Debug, Serialize )]
pub struct AgentSpending
{
  pub agent_id: i64,
  pub agent_name: String,
  pub spending: f64,
  pub budget: f64,
  pub percent_used: f64,
  pub request_count: i64,
}

/// GET /api/v1/analytics/spending/by-agent - Response
#[derive( Debug, Serialize )]
pub struct SpendingByAgentResponse
{
  pub data: Vec< AgentSpending >,
  pub summary: SpendingSummary,
  pub pagination: Pagination,
  pub period: String,
  pub calculated_at: String,
}

/// Spending summary
#[derive( Debug, Serialize )]
pub struct SpendingSummary
{
  pub total_spend: f64,
  pub total_budget: f64,
  pub total_agents: u32,
}

/// Provider spending record
#[derive( Debug, Serialize )]
pub struct ProviderSpending
{
  pub provider: String,
  pub spending: f64,
  pub request_count: i64,
  pub avg_cost_per_request: f64,
  pub agent_count: i64,
}

/// GET /api/v1/analytics/spending/by-provider - Response
#[derive( Debug, Serialize )]
pub struct SpendingByProviderResponse
{
  pub data: Vec< ProviderSpending >,
  pub summary: ProviderSpendingSummary,
  pub period: String,
  pub calculated_at: String,
}

/// Provider spending summary
#[derive( Debug, Serialize )]
pub struct ProviderSpendingSummary
{
  pub total_spend: f64,
  pub total_requests: i64,
  pub providers_count: u32,
}

/// GET /api/v1/analytics/spending/avg-per-request - Response
#[derive( Debug, Serialize )]
pub struct AvgCostResponse
{
  pub average_cost_per_request: f64,
  pub total_requests: i64,
  pub total_spend: f64,
  pub min_cost_per_request: f64,
  pub max_cost_per_request: f64,
  pub period: String,
  pub filters: Filters,
  pub calculated_at: String,
}

/// Budget status record
#[derive( Debug, Serialize )]
pub struct BudgetStatus
{
  pub agent_id: i64,
  pub agent_name: String,
  pub budget: f64,
  pub spent: f64,
  pub remaining: f64,
  pub percent_used: f64,
  pub status: String,
  pub risk_level: String,
}

/// Budget summary
#[derive( Debug, Serialize )]
pub struct BudgetSummary
{
  pub total_agents: u32,
  pub active: u32,
  pub exhausted: u32,
  pub critical: u32,
  pub high: u32,
  pub medium: u32,
  pub low: u32,
}

/// GET /api/v1/analytics/budget/status - Response
#[derive( Debug, Serialize )]
pub struct BudgetStatusResponse
{
  pub data: Vec< BudgetStatus >,
  pub summary: BudgetSummary,
  pub pagination: Pagination,
  pub calculated_at: String,
}

/// GET /api/v1/analytics/usage/requests - Response
#[derive( Debug, Serialize )]
pub struct RequestUsageResponse
{
  pub total_requests: i64,
  pub successful_requests: i64,
  pub failed_requests: i64,
  pub success_rate: f64,
  pub period: String,
  pub filters: Filters,
  pub calculated_at: String,
}

/// Agent token usage record
#[derive( Debug, Serialize )]
pub struct AgentTokenUsage
{
  pub agent_id: i64,
  pub agent_name: String,
  pub input_tokens: i64,
  pub output_tokens: i64,
  pub total_tokens: i64,
  pub request_count: i64,
  pub avg_tokens_per_request: i64,
}

/// Token usage summary
#[derive( Debug, Serialize )]
pub struct TokenUsageSummary
{
  pub total_input_tokens: i64,
  pub total_output_tokens: i64,
  pub total_tokens: i64,
}

/// GET /api/v1/analytics/usage/tokens/by-agent - Response
#[derive( Debug, Serialize )]
pub struct TokenUsageResponse
{
  pub data: Vec< AgentTokenUsage >,
  pub summary: TokenUsageSummary,
  pub pagination: Pagination,
  pub period: String,
  pub calculated_at: String,
}

/// Model usage record
#[derive( Debug, Serialize )]
pub struct ModelUsage
{
  pub model: String,
  pub provider: String,
  pub request_count: i64,
  pub spending: f64,
  pub input_tokens: i64,
  pub output_tokens: i64,
}

/// Model usage summary
#[derive( Debug, Serialize )]
pub struct ModelUsageSummary
{
  pub unique_models: u32,
  pub total_requests: i64,
  pub total_spend: f64,
}

/// GET /api/v1/analytics/usage/models - Response
#[derive( Debug, Serialize )]
pub struct ModelUsageResponse
{
  pub data: Vec< ModelUsage >,
  pub summary: ModelUsageSummary,
  pub pagination: Pagination,
  pub period: String,
  pub calculated_at: String,
}

/// Events list query parameters
#[derive( Debug, Clone, Default, Deserialize )]
pub struct EventsListQuery
{
  #[serde( default )]
  pub period: Period,
  pub agent_id: Option< i64 >,
  #[serde( default = "default_page" )]
  pub page: u32,
  #[serde( default = "default_events_per_page" )]
  pub per_page: u32,
}

fn default_events_per_page() -> u32 { 10 }

/// GET /api/v1/analytics/events - Response
#[derive( Debug, Serialize )]
pub struct EventsListResponse
{
  pub data: Vec< AnalyticsEventWithAgent >,
  pub pagination: Pagination,
  pub period: String,
  pub calculated_at: String,
}

/// Event with agent name
#[derive( Debug, Serialize, FromRow )]
pub struct AnalyticsEventWithAgent
{
  pub event_id: String,
  pub timestamp_ms: i64,
  pub event_type: String,
  pub model: String,
  pub provider: String,
  pub input_tokens: i64,
  pub output_tokens: i64,
  pub cost_micros: i64,
  pub agent_id: i64,
  pub agent_name: String,
  pub error_code: Option< String >,
  pub error_message: Option< String >,
}

// ============================================================================
// State
// ============================================================================

/// Analytics state containing database pool and IC token manager
#[derive( Clone )]
pub struct AnalyticsState
{
  pub pool: SqlitePool,
  pub ic_token_manager: Arc< IcTokenManager >,
}

impl AnalyticsState
{
  /// Create new analytics state
  ///
  /// # Arguments
  /// * `database_url` - SQLite database connection URL
  /// * `ic_token_secret` - Secret for verifying IC tokens
  pub async fn new( database_url: &str, ic_token_secret: String ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( 5 )
      .connect( database_url )
      .await?;

    // Run migration
    let migration = include_str!( "../../../../iron_token_manager/migrations/011_create_analytics_events.sql" );
    sqlx::raw_sql( migration ).execute( &pool ).await?;

    let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );

    Ok( Self { pool, ic_token_manager } )
  }
}

// ============================================================================
// Database Row Types
// ============================================================================

#[derive( Debug, FromRow )]
pub struct AgentBudgetRow
{
  pub agent_id: i64,
  pub agent_name: String,
  pub total_allocated: i64,
  pub total_spent: i64,
  pub budget_remaining: i64,
}
