//! Usage analytics endpoints
//!
//! Provides usage statistics:
//! - Request counts and success rates (GET /api/v1/analytics/usage/requests)
//! - Token usage by agent (GET /api/v1/analytics/usage/tokens/by-agent)
//! - Model usage statistics (GET /api/v1/analytics/usage/models)

use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use chrono::Utc;

use super::shared::{
  compute_change_percent, AgentTokenUsage, AnalyticsQuery, AnalyticsState, Filters, ModelUsage,
  ModelUsageResponse, ModelUsageRow, ModelUsageSummary, Pagination, PaginationQuery,
  RequestUsageComparison, RequestUsageResponse, TokenUsageResponse, TokenUsageSummary,
  TokensByAgentRow,
};
use crate::jwt_auth::AuthenticatedUser;

/// GET /api/v1/analytics/usage/requests
///
/// Requires JWT authentication.
/// Admins see all usage; regular users see only their owned agents' usage.
pub async fn get_usage_requests(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r"SELECT
         COUNT(*) as total,
         SUM(CASE WHEN event_type = 'llm_request_completed' THEN 1 ELSE 0 END) as successful,
         SUM(CASE WHEN event_type = 'llm_request_failed' THEN 1 ELSE 0 END) as failed
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?",
  );

  // Filter by owned agents for non-admins
  if !is_admin {
    query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

  if params.agent_id.is_some() {
    query.push_str(" AND agent_id = ?");
  }
  if params.provider_id.is_some() {
    query.push_str(" AND provider_id = ?");
  }
  if params.provider_key_id.is_some() {
    query.push_str(" AND provider_key_id = ?");
  }

  // Bind time range and all active filters to a usage query.
  // Defined once to guarantee both the current and previous-period queries
  // use an identical bind sequence (prevents silent mismatches).
  macro_rules! bind_usage_filters {
    ($start:expr, $end:expr) => {{
      let mut __q = sqlx::query_as::<_, (i64, i64, i64)>(&query)
        .bind($start)
        .bind($end);
      if !is_admin {
        __q = __q.bind(&user.0.sub);
      }
      if let Some(agent_id) = params.agent_id {
        __q = __q.bind(agent_id);
      }
      if let Some(ref provider_id) = params.provider_id {
        __q = __q.bind(provider_id);
      }
      if let Some(provider_key_id) = params.provider_key_id {
        __q = __q.bind(provider_key_id);
      }
      __q
    }};
  }

  match bind_usage_filters!(start_ms, end_ms)
    .fetch_one(&state.pool)
    .await
  {
    Ok((total, successful, failed)) => {
      let success_rate = if total > 0 {
        (successful as f64 / total as f64) * 100.0
      } else {
        0.0
      };

      // Compute previous period comparison if requested
      let previous_period = if params.compare {
        if let Some((prev_start, prev_end)) = params.period.previous_period_range() {
          match bind_usage_filters!(prev_start, prev_end)
            .fetch_one(&state.pool)
            .await
          {
            Ok((prev_total, prev_successful, prev_failed)) => {
              let prev_success_rate = if prev_total > 0 {
                (prev_successful as f64 / prev_total as f64) * 100.0
              } else {
                0.0
              };
              Some(RequestUsageComparison {
                total_requests: prev_total,
                successful_requests: prev_successful,
                failed_requests: prev_failed,
                success_rate: prev_success_rate,
                change_percent: compute_change_percent(total as f64, prev_total as f64),
              })
            }
            Err(e) => {
              tracing::warn!("Failed to query previous period request usage: {}", e);
              None
            }
          }
        } else {
          None // AllTime has no previous period
        }
      } else {
        None
      };

      (
        StatusCode::OK,
        Json(RequestUsageResponse {
          total_requests: total,
          successful_requests: successful,
          failed_requests: failed,
          success_rate,
          period: format!("{:?}", params.period)
            .to_lowercase()
            .replace('_', "-"),
          filters: Filters {
            agent_id: params.agent_id,
            provider_id: params.provider_id,
            provider_key_id: params.provider_key_id,
          },
          previous_period,
          calculated_at: Utc::now().to_rfc3339(),
        }),
      )
        .into_response()
    }
    Err(e) => {
      tracing::error!("Failed to query request usage: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query request usage"
        })),
      )
        .into_response()
    }
  }
}

/// GET /api/v1/analytics/usage/tokens/by-agent
///
/// Requires JWT authentication.
/// Admins see all usage; regular users see only their owned agents' usage.
pub async fn get_usage_tokens(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
  Query(page): Query<PaginationQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let offset = (page.page - 1) * page.per_page;
  let is_admin = user.0.role == "admin";

  // Build dynamic data query
  let mut query = String::from(
    r"SELECT
         e.agent_id,
         a.name as agent_name,
         COALESCE(SUM(e.input_tokens), 0) as input_tokens,
         COALESCE(SUM(e.output_tokens), 0) as output_tokens,
         COUNT(*) as request_count
       FROM analytics_events e
       LEFT JOIN agents a ON e.agent_id = a.id
       WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
       AND e.event_type = 'llm_request_completed'
       AND e.agent_id IS NOT NULL",
  );

  if !is_admin {
    query.push_str(" AND a.owner_id = ?");
  }
  if params.provider_id.is_some() {
    query.push_str(" AND e.provider = ?");
  }
  if params.provider_key_id.is_some() {
    query.push_str(" AND e.provider_key_id = ?");
  }

  query.push_str(" GROUP BY e.agent_id ORDER BY (input_tokens + output_tokens) DESC LIMIT ? OFFSET ?");

  let mut q = sqlx::query_as::<_, TokensByAgentRow>(&query)
    .bind(start_ms)
    .bind(end_ms);

  if !is_admin {
    q = q.bind(&user.0.sub);
  }
  if let Some(ref provider_id) = params.provider_id {
    q = q.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    q = q.bind(provider_key_id);
  }

  let rows = q
    .bind(i64::from(page.per_page))
    .bind(i64::from(offset))
    .fetch_all(&state.pool)
    .await;

  // Build dynamic totals query
  let mut totals_query = String::from(
    r"SELECT
         COALESCE(SUM(e.input_tokens), 0),
         COALESCE(SUM(e.output_tokens), 0)
       FROM analytics_events e",
  );

  if !is_admin {
    totals_query.push_str(" INNER JOIN agents a ON e.agent_id = a.id");
  }

  totals_query.push_str(
    " WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ? \
     AND e.event_type = 'llm_request_completed'",
  );

  if !is_admin {
    totals_query.push_str(" AND a.owner_id = ?");
  }
  if params.provider_id.is_some() {
    totals_query.push_str(" AND e.provider = ?");
  }
  if params.provider_key_id.is_some() {
    totals_query.push_str(" AND e.provider_key_id = ?");
  }

  let mut tq = sqlx::query_as::<_, (i64, i64)>(&totals_query)
    .bind(start_ms)
    .bind(end_ms);

  if !is_admin {
    tq = tq.bind(&user.0.sub);
  }
  if let Some(ref provider_id) = params.provider_id {
    tq = tq.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    tq = tq.bind(provider_key_id);
  }

  let totals = tq.fetch_one(&state.pool).await.unwrap_or((0, 0));

  // Build dynamic count query
  let mut count_query = String::from(
    r"SELECT COUNT(DISTINCT e.agent_id)
       FROM analytics_events e",
  );

  if !is_admin {
    count_query.push_str(" INNER JOIN agents a ON e.agent_id = a.id");
  }

  count_query.push_str(
    " WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ? \
     AND e.event_type = 'llm_request_completed' AND e.agent_id IS NOT NULL",
  );

  if !is_admin {
    count_query.push_str(" AND a.owner_id = ?");
  }
  if params.provider_id.is_some() {
    count_query.push_str(" AND e.provider = ?");
  }
  if params.provider_key_id.is_some() {
    count_query.push_str(" AND e.provider_key_id = ?");
  }

  let mut cq = sqlx::query_scalar::<_, i64>(&count_query)
    .bind(start_ms)
    .bind(end_ms);

  if !is_admin {
    cq = cq.bind(&user.0.sub);
  }
  if let Some(ref provider_id) = params.provider_id {
    cq = cq.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    cq = cq.bind(provider_key_id);
  }

  let total_count: i64 = cq.fetch_one(&state.pool).await.unwrap_or(0);

  match rows {
    Ok(rows) => {
      let data: Vec<AgentTokenUsage> = rows
        .iter()
        .map(|row| {
          let total_tokens = row.2 + row.3;
          let avg = if row.4 > 0 { total_tokens / row.4 } else { 0 };

          AgentTokenUsage {
            agent_id: row.0,
            agent_name: row.1.clone().unwrap_or_else(|| format!("Agent {}", row.0)),
            input_tokens: row.2,
            output_tokens: row.3,
            total_tokens,
            request_count: row.4,
            avg_tokens_per_request: avg,
          }
        })
        .collect();

      (
        StatusCode::OK,
        Json(TokenUsageResponse {
          data,
          summary: TokenUsageSummary {
            total_input_tokens: totals.0,
            total_output_tokens: totals.1,
            total_tokens: totals.0 + totals.1,
          },
          pagination: Pagination::new(
            page.page,
            page.per_page,
            u32::try_from(total_count).unwrap_or(u32::MAX),
          ),
          period: format!("{:?}", params.period)
            .to_lowercase()
            .replace('_', "-"),
          calculated_at: Utc::now().to_rfc3339(),
        }),
      )
        .into_response()
    }
    Err(e) => {
      tracing::error!("Failed to query token usage: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query token usage"
        })),
      )
        .into_response()
    }
  }
}

/// GET /api/v1/analytics/usage/models
///
/// Requires JWT authentication.
/// Admins see all usage; regular users see only their owned agents' usage.
pub async fn get_usage_models(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
  Query(page): Query<PaginationQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let offset = (page.page - 1) * page.per_page;
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r"SELECT
         model,
         provider,
         COUNT(*) as request_count,
         COALESCE(SUM(cost_micros), 0) as spending_micros,
         COALESCE(SUM(input_tokens), 0) as input_tokens,
         COALESCE(SUM(output_tokens), 0) as output_tokens
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'",
  );

  // Filter by owned agents for non-admins
  if !is_admin {
    query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

  if params.agent_id.is_some() {
    query.push_str(" AND agent_id = ?");
  }
  if params.provider_id.is_some() {
    query.push_str(" AND provider = ?");
  }
  if params.provider_key_id.is_some() {
    query.push_str(" AND provider_key_id = ?");
  }

  query.push_str(" GROUP BY model, provider ORDER BY request_count DESC LIMIT ? OFFSET ?");

  let mut q = sqlx::query_as::<_, ModelUsageRow>(&query)
    .bind(start_ms)
    .bind(end_ms);

  // Bind owner_id for non-admins
  if !is_admin {
    q = q.bind(&user.0.sub);
  }

  if let Some(agent_id) = params.agent_id {
    q = q.bind(agent_id);
  }
  if let Some(ref provider_id) = params.provider_id {
    q = q.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    q = q.bind(provider_key_id);
  }

  let rows = q
    .bind(i64::from(page.per_page))
    .bind(i64::from(offset))
    .fetch_all(&state.pool)
    .await;

  // Query totals (filtered by owner for non-admins)
  let mut totals_query = String::from(
    r"SELECT
         COUNT(DISTINCT model || provider) as unique_models,
         COUNT(*) as total_requests,
         COALESCE(SUM(cost_micros), 0) as total_spend
       FROM analytics_events
       WHERE timestamp_ms >= ? AND timestamp_ms <= ?
       AND event_type = 'llm_request_completed'",
  );

  // Filter by owned agents for non-admins
  if !is_admin {
    totals_query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }

  if params.agent_id.is_some() {
    totals_query.push_str(" AND agent_id = ?");
  }
  if params.provider_id.is_some() {
    totals_query.push_str(" AND provider = ?");
  }
  if params.provider_key_id.is_some() {
    totals_query.push_str(" AND provider_key_id = ?");
  }

  let mut tq = sqlx::query_as::<_, (i64, i64, i64)>(&totals_query)
    .bind(start_ms)
    .bind(end_ms);

  // Bind owner_id for non-admins
  if !is_admin {
    tq = tq.bind(&user.0.sub);
  }

  if let Some(agent_id) = params.agent_id {
    tq = tq.bind(agent_id);
  }
  if let Some(ref provider_id) = params.provider_id {
    tq = tq.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    tq = tq.bind(provider_key_id);
  }

  let totals = tq.fetch_one(&state.pool).await.unwrap_or((0, 0, 0));

  match rows {
    Ok(rows) => {
      let data: Vec<ModelUsage> = rows
        .iter()
        .map(|row| ModelUsage {
          model: row.0.clone(),
          provider: row.1.clone(),
          request_count: row.2,
          spending: row.3 as f64 / 1_000_000.0,
          input_tokens: row.4,
          output_tokens: row.5,
        })
        .collect();

      (
        StatusCode::OK,
        Json(ModelUsageResponse {
          data,
          summary: ModelUsageSummary {
            unique_models: u32::try_from(totals.0).unwrap_or(u32::MAX),
            total_requests: totals.1,
            total_spend: totals.2 as f64 / 1_000_000.0,
          },
          pagination: Pagination::new(
            page.page,
            page.per_page,
            u32::try_from(totals.0).unwrap_or(u32::MAX),
          ),
          period: format!("{:?}", params.period)
            .to_lowercase()
            .replace('_', "-"),
          calculated_at: Utc::now().to_rfc3339(),
        }),
      )
        .into_response()
    }
    Err(e) => {
      tracing::error!("Failed to query model usage: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query model usage"
        })),
      )
        .into_response()
    }
  }
}
