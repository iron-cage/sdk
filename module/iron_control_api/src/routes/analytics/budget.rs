//! Budget status endpoint
//!
//! Provides budget monitoring and status tracking:
//! - GET /api/v1/analytics/budget/status

use axum::{
  extract::{ Query, State },
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use chrono::Utc;
use crate::jwt_auth::AuthenticatedUser;
use super::shared::{
  BudgetStatusQuery, AnalyticsState, AgentBudgetRow,
  BudgetStatusResponse, BudgetStatus, BudgetSummary, Pagination,
};

/// GET /api/v1/analytics/budget/status
///
/// Requires JWT authentication.
pub async fn get_budget_status(
  _user: AuthenticatedUser,
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
