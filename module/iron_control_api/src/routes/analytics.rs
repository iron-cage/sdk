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

use axum::{
  extract::{ Query, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use chrono::{ DateTime, Datelike, Duration, Utc };
use serde::{ Deserialize, Serialize };
use sqlx::{ FromRow, SqlitePool, sqlite::SqlitePoolOptions };

// ============================================================================
// Types
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
      Period::Today => now.date_naive().and_hms_opt( 0, 0, 0 ).unwrap(),
      Period::Yesterday =>
      {
        let yesterday = now - Duration::days( 1 );
        yesterday.date_naive().and_hms_opt( 0, 0, 0 ).unwrap()
      }
      Period::Last7Days => ( now - Duration::days( 7 ) ).naive_utc(),
      Period::Last30Days => ( now - Duration::days( 30 ) ).naive_utc(),
      Period::ThisMonth =>
      {
        let first_of_month = now.date_naive().with_day( 1 ).unwrap();
        first_of_month.and_hms_opt( 0, 0, 0 ).unwrap()
      }
      Period::LastMonth =>
      {
        let first_of_this_month = now.date_naive().with_day( 1 ).unwrap();
        let last_month = first_of_this_month - Duration::days( 1 );
        last_month.with_day( 1 ).unwrap().and_hms_opt( 0, 0, 0 ).unwrap()
      }
      Period::AllTime => return ( 0, end_ms ),
    };

    let start_ms = DateTime::<Utc>::from_naive_utc_and_offset( start, Utc ).timestamp_millis();

    // For Yesterday, end at end of yesterday
    let end_ms = if *self == Period::Yesterday
    {
      let today_start = now.date_naive().and_hms_opt( 0, 0, 0 ).unwrap();
      DateTime::<Utc>::from_naive_utc_and_offset( today_start, Utc ).timestamp_millis() - 1
    }
    else if *self == Period::LastMonth
    {
      let first_of_this_month = now.date_naive().with_day( 1 ).unwrap();
      DateTime::<Utc>::from_naive_utc_and_offset(
        first_of_this_month.and_hms_opt( 0, 0, 0 ).unwrap(),
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
  #[serde( default )]
  pub agent_id: Option< i64 >,
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

// ============================================================================
// State
// ============================================================================

/// Analytics state containing database pool
#[derive( Clone )]
pub struct AnalyticsState
{
  pub pool: SqlitePool,
}

impl AnalyticsState
{
  /// Create new analytics state
  pub async fn new( database_url: &str ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let pool = SqlitePoolOptions::new()
      .max_connections( 5 )
      .connect( database_url )
      .await?;

    // Run migration
    let migration = include_str!( "../../../iron_token_manager/migrations/011_create_analytics_events.sql" );
    sqlx::raw_sql( migration ).execute( &pool ).await?;

    Ok( Self { pool } )
  }
}

// ============================================================================
// Database Row Types
// ============================================================================

#[derive( Debug, FromRow )]
struct AgentBudgetRow
{
  agent_id: i64,
  agent_name: String,
  total_allocated: f64,
  total_spent: f64,
  budget_remaining: f64,
}

// ============================================================================
// Handlers
// ============================================================================

/// POST /api/v1/analytics/events
///
/// Ingest analytics event from LlmRouter.
/// Returns 202 Accepted for new events, 200 OK for duplicates.
pub async fn post_event(
  State( state ): State< AnalyticsState >,
  Json( event ): Json< AnalyticsEventRequest >,
) -> impl IntoResponse
{
  // Validate event_type
  if event.event_type != "llm_request_completed" && event.event_type != "llm_request_failed"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "event_type must be 'llm_request_completed' or 'llm_request_failed'"
      }) )
    ).into_response();
  }

  // Validate provider
  if event.provider != "openai" && event.provider != "anthropic" && event.provider != "unknown"
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "provider must be 'openai', 'anthropic', or 'unknown'"
      }) )
    ).into_response();
  }

  // For completed events, require token counts
  if event.event_type == "llm_request_completed"
  {
    if event.input_tokens.is_none() || event.output_tokens.is_none()
    {
      return (
        StatusCode::BAD_REQUEST,
        Json( serde_json::json!({
          "error": "VALIDATION_ERROR",
          "message": "input_tokens and output_tokens required for completed events"
        }) )
      ).into_response();
    }
  }

  let now_ms = Utc::now().timestamp_millis();

  // INSERT OR IGNORE for deduplication
  let result = sqlx::query(
    r#"INSERT OR IGNORE INTO analytics_events
       (event_id, timestamp_ms, event_type, model, provider,
        input_tokens, output_tokens, cost_micros,
        agent_id, provider_id, error_code, error_message, received_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#
  )
    .bind( &event.event_id )
    .bind( event.timestamp_ms )
    .bind( &event.event_type )
    .bind( &event.model )
    .bind( &event.provider )
    .bind( event.input_tokens.unwrap_or( 0 ) )
    .bind( event.output_tokens.unwrap_or( 0 ) )
    .bind( event.cost_micros.unwrap_or( 0 ) )
    .bind( event.agent_id )
    .bind( &event.provider_id )
    .bind( &event.error_code )
    .bind( &event.error_message )
    .bind( now_ms )
    .execute( &state.pool )
    .await;

  match result
  {
    Ok( r ) =>
    {
      if r.rows_affected() > 0
      {
        ( StatusCode::ACCEPTED, Json( EventResponse {
          event_id: event.event_id,
          status: "accepted".to_string(),
        }) ).into_response()
      }
      else
      {
        ( StatusCode::OK, Json( EventResponse {
          event_id: event.event_id,
          status: "duplicate".to_string(),
        }) ).into_response()
      }
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to insert event: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to store event"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/total
pub async fn get_spending_total(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();

  let mut query = String::from(
    r#"SELECT COALESCE(SUM(cost_micros), 0) as total
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  );

  if params.agent_id.is_some()
  {
    query.push_str( " AND agent_id = ?" );
  }
  if params.provider_id.is_some()
  {
    query.push_str( " AND provider_id = ?" );
  }

  let mut q = sqlx::query_scalar::< _, i64 >( &query )
    .bind( start_ms )
    .bind( end_ms );

  if let Some( agent_id ) = params.agent_id
  {
    q = q.bind( agent_id );
  }
  if let Some( ref provider_id ) = params.provider_id
  {
    q = q.bind( provider_id );
  }

  match q.fetch_one( &state.pool ).await
  {
    Ok( total_micros ) =>
    {
      let total_usd = total_micros as f64 / 1_000_000.0;

      ( StatusCode::OK, Json( SpendingTotalResponse {
        total_spend: total_usd,
        currency: "USD".to_string(),
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        filters: Filters {
          agent_id: params.agent_id,
          provider_id: params.provider_id,
        },
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query spending total: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query spending"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/by-agent
pub async fn get_spending_by_agent(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
  Query( page ): Query< PaginationQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let offset = ( page.page - 1 ) * page.per_page;

  // Query spending by agent with budget info
  let rows: Result< Vec< ( i64, Option< String >, i64, i64, Option< f64 > ) >, _ > = sqlx::query_as(
    r#"SELECT
         e.agent_id,
         a.name as agent_name,
         COALESCE(SUM(e.cost_micros), 0) as spending_micros,
         COUNT(*) as request_count,
         ab.total_allocated as budget
       FROM analytics_events e
       LEFT JOIN agents a ON e.agent_id = a.id
       LEFT JOIN agent_budgets ab ON e.agent_id = ab.agent_id
       WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
       AND e.event_type = 'llm_request_completed'
       AND e.agent_id IS NOT NULL
       GROUP BY e.agent_id
       ORDER BY spending_micros DESC
       LIMIT ? OFFSET ?"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .bind( page.per_page as i64 )
    .bind( offset as i64 )
    .fetch_all( &state.pool )
    .await;

  // Query total count
  let total_count: i64 = sqlx::query_scalar(
    r#"SELECT COUNT(DISTINCT agent_id)
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'
       AND agent_id IS NOT NULL"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .fetch_one( &state.pool )
    .await
    .unwrap_or( 0 );

  match rows
  {
    Ok( rows ) =>
    {
      let mut total_spend = 0.0;
      let mut total_budget = 0.0;

      let data: Vec< AgentSpending > = rows.iter().map( |row| {
        let spending = row.2 as f64 / 1_000_000.0;
        let budget = row.4.unwrap_or( 0.0 );
        total_spend += spending;
        total_budget += budget;

        let percent_used = if budget > 0.0 { ( spending / budget ) * 100.0 } else { 0.0 };

        AgentSpending {
          agent_id: row.0,
          agent_name: row.1.clone().unwrap_or_else( || format!( "Agent {}", row.0 ) ),
          spending,
          budget,
          percent_used,
          request_count: row.3,
        }
      }).collect();

      ( StatusCode::OK, Json( SpendingByAgentResponse {
        data,
        summary: SpendingSummary {
          total_spend,
          total_budget,
          total_agents: total_count as u32,
        },
        pagination: Pagination::new( page.page, page.per_page, total_count as u32 ),
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query spending by agent: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query spending by agent"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/by-provider
pub async fn get_spending_by_provider(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();

  let rows: Result< Vec< ( String, i64, i64, i64 ) >, _ > = sqlx::query_as(
    r#"SELECT
         provider,
         COALESCE(SUM(cost_micros), 0) as spending_micros,
         COUNT(*) as request_count,
         COUNT(DISTINCT agent_id) as agent_count
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'
       GROUP BY provider
       ORDER BY spending_micros DESC"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .fetch_all( &state.pool )
    .await;

  match rows
  {
    Ok( rows ) =>
    {
      let mut total_spend = 0.0;
      let mut total_requests = 0i64;

      let data: Vec< ProviderSpending > = rows.iter().map( |row| {
        let spending = row.1 as f64 / 1_000_000.0;
        let request_count = row.2;
        total_spend += spending;
        total_requests += request_count;

        let avg_cost = if request_count > 0 { spending / request_count as f64 } else { 0.0 };

        ProviderSpending {
          provider: row.0.clone(),
          spending,
          request_count,
          avg_cost_per_request: avg_cost,
          agent_count: row.3,
        }
      }).collect();

      ( StatusCode::OK, Json( SpendingByProviderResponse {
        summary: ProviderSpendingSummary {
          total_spend,
          total_requests,
          providers_count: data.len() as u32,
        },
        data,
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query spending by provider: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query spending by provider"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/avg-per-request
pub async fn get_spending_avg(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();

  let mut query = String::from(
    r#"SELECT
         COALESCE(SUM(cost_micros), 0) as total_micros,
         COUNT(*) as request_count,
         COALESCE(MIN(cost_micros), 0) as min_micros,
         COALESCE(MAX(cost_micros), 0) as max_micros
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  );

  if params.agent_id.is_some()
  {
    query.push_str( " AND agent_id = ?" );
  }
  if params.provider_id.is_some()
  {
    query.push_str( " AND provider_id = ?" );
  }

  let mut q = sqlx::query_as::< _, ( i64, i64, i64, i64 ) >( &query )
    .bind( start_ms )
    .bind( end_ms );

  if let Some( agent_id ) = params.agent_id
  {
    q = q.bind( agent_id );
  }
  if let Some( ref provider_id ) = params.provider_id
  {
    q = q.bind( provider_id );
  }

  match q.fetch_one( &state.pool ).await
  {
    Ok( ( total_micros, request_count, min_micros, max_micros ) ) =>
    {
      let total_usd = total_micros as f64 / 1_000_000.0;
      let avg = if request_count > 0 { total_usd / request_count as f64 } else { 0.0 };
      let min_usd = min_micros as f64 / 1_000_000.0;
      let max_usd = max_micros as f64 / 1_000_000.0;

      ( StatusCode::OK, Json( AvgCostResponse {
        average_cost_per_request: avg,
        total_requests: request_count,
        total_spend: total_usd,
        min_cost_per_request: min_usd,
        max_cost_per_request: max_usd,
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        filters: Filters {
          agent_id: params.agent_id,
          provider_id: params.provider_id,
        },
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query avg cost: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query average cost"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/budget/status
pub async fn get_budget_status(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< BudgetStatusQuery >,
) -> impl IntoResponse
{
  let offset = ( params.page - 1 ) * params.per_page;

  // Query budget status from agent_budgets table
  let mut query = String::from(
    r#"SELECT
         a.id as agent_id,
         a.name as agent_name,
         COALESCE(ab.total_allocated, 0.0) as total_allocated,
         COALESCE(ab.total_spent, 0.0) as total_spent,
         COALESCE(ab.budget_remaining, 0.0) as budget_remaining
       FROM agents a
       LEFT JOIN agent_budgets ab ON a.id = ab.agent_id
       WHERE 1=1"#
  );

  if params.agent_id.is_some()
  {
    query.push_str( " AND a.id = ?" );
  }

  query.push_str( " ORDER BY total_spent DESC LIMIT ? OFFSET ?" );

  let mut q = sqlx::query_as::< _, AgentBudgetRow >( &query );

  if let Some( agent_id ) = params.agent_id
  {
    q = q.bind( agent_id );
  }
  q = q.bind( params.per_page as i64 ).bind( offset as i64 );

  let rows = q.fetch_all( &state.pool ).await;

  // Query total count
  let total_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM agents" )
    .fetch_one( &state.pool )
    .await
    .unwrap_or( 0 );

  match rows
  {
    Ok( rows ) =>
    {
      let mut summary = BudgetSummary {
        total_agents: rows.len() as u32,
        active: 0,
        exhausted: 0,
        critical: 0,
        high: 0,
        medium: 0,
        low: 0,
      };

      let mut data: Vec< BudgetStatus > = rows.iter().filter_map( |row| {
        let percent_used = if row.total_allocated > 0.0
        {
          ( row.total_spent / row.total_allocated ) * 100.0
        }
        else
        {
          0.0
        };

        // Apply threshold filter
        if let Some( threshold ) = params.threshold
        {
          if percent_used <= threshold as f64
          {
            return None;
          }
        }

        let ( status, risk_level ) = if percent_used >= 100.0
        {
          summary.exhausted += 1;
          ( "exhausted", "exhausted" )
        }
        else if percent_used >= 95.0
        {
          summary.critical += 1;
          summary.active += 1;
          ( "active", "critical" )
        }
        else if percent_used >= 80.0
        {
          summary.high += 1;
          summary.active += 1;
          ( "active", "high" )
        }
        else if percent_used >= 50.0
        {
          summary.medium += 1;
          summary.active += 1;
          ( "active", "medium" )
        }
        else
        {
          summary.low += 1;
          summary.active += 1;
          ( "active", "low" )
        };

        // Apply status filter
        if let Some( ref status_filter ) = params.status
        {
          if status != status_filter
          {
            return None;
          }
        }

        Some( BudgetStatus {
          agent_id: row.agent_id,
          agent_name: row.agent_name.clone(),
          budget: row.total_allocated,
          spent: row.total_spent,
          remaining: row.budget_remaining,
          percent_used,
          status: status.to_string(),
          risk_level: risk_level.to_string(),
        })
      }).collect();

      // Sort by percent_used descending
      data.sort_by( |a, b| b.percent_used.partial_cmp( &a.percent_used ).unwrap_or( std::cmp::Ordering::Equal ) );

      ( StatusCode::OK, Json( BudgetStatusResponse {
        data,
        summary,
        pagination: Pagination::new( params.page, params.per_page, total_count as u32 ),
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query budget status: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query budget status"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/usage/requests
pub async fn get_usage_requests(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();

  let mut query = String::from(
    r#"SELECT
         COUNT(*) as total,
         SUM(CASE WHEN event_type = 'llm_request_completed' THEN 1 ELSE 0 END) as successful,
         SUM(CASE WHEN event_type = 'llm_request_failed' THEN 1 ELSE 0 END) as failed
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?"#
  );

  if params.agent_id.is_some()
  {
    query.push_str( " AND agent_id = ?" );
  }
  if params.provider_id.is_some()
  {
    query.push_str( " AND provider_id = ?" );
  }

  let mut q = sqlx::query_as::< _, ( i64, i64, i64 ) >( &query )
    .bind( start_ms )
    .bind( end_ms );

  if let Some( agent_id ) = params.agent_id
  {
    q = q.bind( agent_id );
  }
  if let Some( ref provider_id ) = params.provider_id
  {
    q = q.bind( provider_id );
  }

  match q.fetch_one( &state.pool ).await
  {
    Ok( ( total, successful, failed ) ) =>
    {
      let success_rate = if total > 0 { ( successful as f64 / total as f64 ) * 100.0 } else { 0.0 };

      ( StatusCode::OK, Json( RequestUsageResponse {
        total_requests: total,
        successful_requests: successful,
        failed_requests: failed,
        success_rate,
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        filters: Filters {
          agent_id: params.agent_id,
          provider_id: params.provider_id,
        },
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query request usage: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query request usage"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/usage/tokens/by-agent
pub async fn get_usage_tokens(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
  Query( page ): Query< PaginationQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let offset = ( page.page - 1 ) * page.per_page;

  let rows: Result< Vec< ( i64, Option< String >, i64, i64, i64 ) >, _ > = sqlx::query_as(
    r#"SELECT
         e.agent_id,
         a.name as agent_name,
         COALESCE(SUM(e.input_tokens), 0) as input_tokens,
         COALESCE(SUM(e.output_tokens), 0) as output_tokens,
         COUNT(*) as request_count
       FROM analytics_events e
       LEFT JOIN agents a ON e.agent_id = a.id
       WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
       AND e.event_type = 'llm_request_completed'
       AND e.agent_id IS NOT NULL
       GROUP BY e.agent_id
       ORDER BY (input_tokens + output_tokens) DESC
       LIMIT ? OFFSET ?"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .bind( page.per_page as i64 )
    .bind( offset as i64 )
    .fetch_all( &state.pool )
    .await;

  // Query totals
  let totals: ( i64, i64 ) = sqlx::query_as(
    r#"SELECT
         COALESCE(SUM(input_tokens), 0),
         COALESCE(SUM(output_tokens), 0)
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .fetch_one( &state.pool )
    .await
    .unwrap_or( ( 0, 0 ) );

  // Query total count
  let total_count: i64 = sqlx::query_scalar(
    r#"SELECT COUNT(DISTINCT agent_id)
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'
       AND agent_id IS NOT NULL"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .fetch_one( &state.pool )
    .await
    .unwrap_or( 0 );

  match rows
  {
    Ok( rows ) =>
    {
      let data: Vec< AgentTokenUsage > = rows.iter().map( |row| {
        let total_tokens = row.2 + row.3;
        let avg = if row.4 > 0 { total_tokens / row.4 } else { 0 };

        AgentTokenUsage {
          agent_id: row.0,
          agent_name: row.1.clone().unwrap_or_else( || format!( "Agent {}", row.0 ) ),
          input_tokens: row.2,
          output_tokens: row.3,
          total_tokens,
          request_count: row.4,
          avg_tokens_per_request: avg,
        }
      }).collect();

      ( StatusCode::OK, Json( TokenUsageResponse {
        data,
        summary: TokenUsageSummary {
          total_input_tokens: totals.0,
          total_output_tokens: totals.1,
          total_tokens: totals.0 + totals.1,
        },
        pagination: Pagination::new( page.page, page.per_page, total_count as u32 ),
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query token usage: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query token usage"
      }) ) ).into_response()
    }
  }
}

/// GET /api/v1/analytics/usage/models
pub async fn get_usage_models(
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
  Query( page ): Query< PaginationQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let offset = ( page.page - 1 ) * page.per_page;

  let rows: Result< Vec< ( String, String, i64, i64, i64, i64 ) >, _ > = sqlx::query_as(
    r#"SELECT
         model,
         provider,
         COUNT(*) as request_count,
         COALESCE(SUM(cost_micros), 0) as spending_micros,
         COALESCE(SUM(input_tokens), 0) as input_tokens,
         COALESCE(SUM(output_tokens), 0) as output_tokens
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'
       GROUP BY model, provider
       ORDER BY request_count DESC
       LIMIT ? OFFSET ?"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .bind( page.per_page as i64 )
    .bind( offset as i64 )
    .fetch_all( &state.pool )
    .await;

  // Query totals
  let totals: ( i64, i64, i64 ) = sqlx::query_as(
    r#"SELECT
         COUNT(DISTINCT model || provider) as unique_models,
         COUNT(*) as total_requests,
         COALESCE(SUM(cost_micros), 0) as total_spend
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  )
    .bind( start_ms )
    .bind( end_ms )
    .fetch_one( &state.pool )
    .await
    .unwrap_or( ( 0, 0, 0 ) );

  match rows
  {
    Ok( rows ) =>
    {
      let data: Vec< ModelUsage > = rows.iter().map( |row| {
        ModelUsage {
          model: row.0.clone(),
          provider: row.1.clone(),
          request_count: row.2,
          spending: row.3 as f64 / 1_000_000.0,
          input_tokens: row.4,
          output_tokens: row.5,
        }
      }).collect();

      ( StatusCode::OK, Json( ModelUsageResponse {
        data,
        summary: ModelUsageSummary {
          unique_models: totals.0 as u32,
          total_requests: totals.1,
          total_spend: totals.2 as f64 / 1_000_000.0,
        },
        pagination: Pagination::new( page.page, page.per_page, totals.0 as u32 ),
        period: format!( "{:?}", params.period ).to_lowercase().replace( "_", "-" ),
        calculated_at: Utc::now().to_rfc3339(),
      }) ).into_response()
    }
    Err( e ) =>
    {
      tracing::error!( "Failed to query model usage: {}", e );
      ( StatusCode::INTERNAL_SERVER_ERROR, Json( serde_json::json!({
        "error": "DATABASE_ERROR",
        "message": "Failed to query model usage"
      }) ) ).into_response()
    }
  }
}
