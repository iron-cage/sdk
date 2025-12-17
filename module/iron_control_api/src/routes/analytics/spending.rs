//! Spending analytics endpoints
//!
//! Provides spending analytics queries:
//! - Total spending (GET /api/v1/analytics/spending/total)
//! - Spending by agent (GET /api/v1/analytics/spending/by-agent)
//! - Spending by provider (GET /api/v1/analytics/spending/by-provider)
//! - Average cost per request (GET /api/v1/analytics/spending/avg-per-request)

use axum::{
  extract::{ Query, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use chrono::Utc;
use crate::jwt_auth::AuthenticatedUser;
use super::shared::{
  AnalyticsQuery, AnalyticsState, PaginationQuery,
  SpendingTotalResponse, Filters, Pagination,
  SpendingByAgentResponse, AgentSpending, SpendingSummary, SpendingByAgentRow,
  SpendingByProviderResponse, ProviderSpending, ProviderSpendingSummary,
  AvgCostResponse,
};

/// GET /api/v1/analytics/spending/total
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
pub async fn get_spending_total(
  user: AuthenticatedUser,
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r#"SELECT COALESCE(SUM(cost_micros), 0) as total
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  );

  // Filter by owned agents for non-admins
  if !is_admin
  {
    query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

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

  // Bind owner_id for non-admins
  if !is_admin
  {
    q = q.bind( &user.0.sub );
  }

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
///
/// Requires JWT authentication.
/// Admins see all agents; regular users see only their owned agents.
pub async fn get_spending_by_agent(
  user: AuthenticatedUser,
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
  Query( page ): Query< PaginationQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let offset = ( page.page - 1 ) * page.per_page;
  let is_admin = user.0.role == "admin";

  // Build query with optional owner filter
  let base_query = if is_admin {
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
  } else {
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
       AND a.owner_id = ?
       GROUP BY e.agent_id
       ORDER BY spending_micros DESC
       LIMIT ? OFFSET ?"#
  };

  // Query spending by agent with budget info
  let rows: Result< Vec< SpendingByAgentRow >, _ > = if is_admin {
    sqlx::query_as( base_query )
      .bind( start_ms )
      .bind( end_ms )
      .bind( page.per_page as i64 )
      .bind( offset as i64 )
      .fetch_all( &state.pool )
      .await
  } else {
    sqlx::query_as( base_query )
      .bind( start_ms )
      .bind( end_ms )
      .bind( &user.0.sub )
      .bind( page.per_page as i64 )
      .bind( offset as i64 )
      .fetch_all( &state.pool )
      .await
  };

  // Query total count (filtered by owner for non-admins)
  let total_count: i64 = if is_admin {
    sqlx::query_scalar(
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
      .unwrap_or( 0 )
  } else {
    sqlx::query_scalar(
      r#"SELECT COUNT(DISTINCT e.agent_id)
         FROM analytics_events e
         INNER JOIN agents a ON e.agent_id = a.id
         WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
         AND e.event_type = 'llm_request_completed'
         AND e.agent_id IS NOT NULL
         AND a.owner_id = ?"#
    )
      .bind( start_ms )
      .bind( end_ms )
      .bind( &user.0.sub )
      .fetch_one( &state.pool )
      .await
      .unwrap_or( 0 )
  };

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
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
pub async fn get_spending_by_provider(
  user: AuthenticatedUser,
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r#"SELECT
         provider,
         COALESCE(SUM(cost_micros), 0) as spending_micros,
         COUNT(*) as request_count,
         COUNT(DISTINCT agent_id) as agent_count
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'"#
  );

  // Filter by owned agents for non-admins
  if !is_admin
  {
    query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

  if params.agent_id.is_some()
  {
    query.push_str( " AND agent_id = ?" );
  }

  query.push_str( " GROUP BY provider ORDER BY spending_micros DESC" );

  let mut q = sqlx::query_as::< _, ( String, i64, i64, i64 ) >( &query )
    .bind( start_ms )
    .bind( end_ms );

  // Bind owner_id for non-admins
  if !is_admin
  {
    q = q.bind( &user.0.sub );
  }

  if let Some( agent_id ) = params.agent_id
  {
    q = q.bind( agent_id );
  }

  let rows = q.fetch_all( &state.pool ).await;

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
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
pub async fn get_spending_avg(
  user: AuthenticatedUser,
  State( state ): State< AnalyticsState >,
  Query( params ): Query< AnalyticsQuery >,
) -> impl IntoResponse
{
  let ( start_ms, end_ms ) = params.period.to_range();
  let is_admin = user.0.role == "admin";

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

  // Filter by owned agents for non-admins
  if !is_admin
  {
    query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

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

  // Bind owner_id for non-admins
  if !is_admin
  {
    q = q.bind( &user.0.sub );
  }

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
