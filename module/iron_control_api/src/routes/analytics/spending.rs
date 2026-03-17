//! Spending analytics endpoints
//!
//! Provides spending analytics queries:
//! - Total spending (GET /api/v1/analytics/spending/total)
//! - Spending by agent (GET /api/v1/analytics/spending/by-agent)
//! - Spending by provider (GET /api/v1/analytics/spending/by-provider)
//! - Average cost per request (GET /api/v1/analytics/spending/avg-per-request)

use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use chrono::Utc;

use super::shared::{
  compute_change_percent, AgentSpending, AnalyticsQuery, AnalyticsState, AvgCostResponse, Filters,
  Pagination, PaginationQuery, ProviderSpending, ProviderSpendingSummary,
  SpendingByAgentResponse, SpendingByAgentRow, SpendingByProviderResponse, SpendingSummary,
  SpendingTotalComparison, SpendingTotalResponse,
};
use crate::jwt_auth::AuthenticatedUser;

/// GET /api/v1/analytics/spending/total
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
pub async fn get_spending_total(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r"SELECT COALESCE(SUM(cost_micros), 0) as total
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

  // Bind time range and all active filters to a spending scalar query.
  // Defined once to guarantee both the current and previous-period queries
  // use an identical bind sequence (prevents silent mismatches).
  macro_rules! bind_spending_filters {
    ($start:expr, $end:expr) => {{
      let mut __q = sqlx::query_scalar::<_, i64>(&query)
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

  match bind_spending_filters!(start_ms, end_ms)
    .fetch_one(&state.pool)
    .await
  {
    Ok(total_micros) => {
      let total_usd = total_micros as f64 / 1_000_000.0;

      // Compute previous period comparison if requested
      let previous_period = if params.compare {
        if let Some((prev_start, prev_end)) = params.period.previous_period_range() {
          match bind_spending_filters!(prev_start, prev_end)
            .fetch_one(&state.pool)
            .await
          {
            Ok(prev_micros) => {
              let prev_usd = prev_micros as f64 / 1_000_000.0;
              Some(SpendingTotalComparison {
                total_spend: prev_usd,
                change_percent: compute_change_percent(total_usd, prev_usd),
              })
            }
            Err(e) => {
              tracing::warn!("Failed to query previous period spending: {}", e);
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
        Json(SpendingTotalResponse {
          total_spend: total_usd,
          currency: "USD".to_string(),
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
      tracing::error!("Failed to query spending total: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query spending"
        })),
      )
        .into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/by-agent
///
/// Requires JWT authentication.
/// Admins see all agents; regular users see only their owned agents.
pub async fn get_spending_by_agent(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
  Query(page): Query<PaginationQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let offset = (page.page - 1) * page.per_page;
  let is_admin = user.0.role == "admin";

  // Build dynamic query
  let mut query = String::from(
    r"SELECT
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

  query.push_str(" GROUP BY e.agent_id ORDER BY spending_micros DESC LIMIT ? OFFSET ?");

  let mut q = sqlx::query_as::<_, SpendingByAgentRow>(&query)
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
      let mut total_spend = 0.0;
      let mut total_budget = 0.0;

      let data: Vec<AgentSpending> = rows
        .iter()
        .map(|row| {
          let spending = row.2 as f64 / 1_000_000.0;
          let budget = row.4.unwrap_or(0) as f64 / 1_000_000.0;
          total_spend += spending;
          total_budget += budget;

          let percent_used = if budget > 0.0 {
            (spending / budget) * 100.0
          } else {
            0.0
          };

          AgentSpending {
            agent_id: row.0,
            agent_name: row.1.clone().unwrap_or_else(|| format!("Agent {}", row.0)),
            spending,
            budget,
            percent_used,
            request_count: row.3,
          }
        })
        .collect();

      (
        StatusCode::OK,
        Json(SpendingByAgentResponse {
          data,
          summary: SpendingSummary {
            total_spend,
            total_budget,
            total_agents: u32::try_from(total_count).unwrap_or(u32::MAX),
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
      tracing::error!("Failed to query spending by agent: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query spending by agent"
        })),
      )
        .into_response()
    }
  }
}

/// GET /api/v1/analytics/spending/by-provider
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
/// Supports `group_by=key` to group by provider key instead of provider.
pub async fn get_spending_by_provider(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let is_admin = user.0.role == "admin";
  let group_by_key = params.group_by.as_deref() == Some("key");

  let mut query = if group_by_key {
    String::from(
      r"SELECT
           e.provider,
           COALESCE(SUM(e.cost_micros), 0) as spending_micros,
           COUNT(*) as request_count,
           COUNT(DISTINCT e.agent_id) as agent_count,
           e.provider_key_id,
           pk.alias
         FROM analytics_events e
         LEFT JOIN ai_provider_keys pk ON e.provider_key_id = pk.id
         WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
         AND e.event_type = 'llm_request_completed'",
    )
  } else {
    String::from(
      r"SELECT
           provider,
           COALESCE(SUM(cost_micros), 0) as spending_micros,
           COUNT(*) as request_count,
           COUNT(DISTINCT agent_id) as agent_count
         FROM analytics_events
         WHERE timestamp_ms >= ? AND timestamp_ms <= ?
         AND event_type = 'llm_request_completed'",
    )
  };

  // Filter by owned agents for non-admins
  if !is_admin {
    if group_by_key {
      query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = e.agent_id AND a.owner_id = ?)" );
    } else {
      query.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
    }
  }

  if params.agent_id.is_some() {
    if group_by_key {
      query.push_str(" AND e.agent_id = ?");
    } else {
      query.push_str(" AND agent_id = ?");
    }
  }
  if params.provider_id.is_some() {
    if group_by_key {
      query.push_str(" AND e.provider = ?");
    } else {
      query.push_str(" AND provider = ?");
    }
  }
  if params.provider_key_id.is_some() {
    if group_by_key {
      query.push_str(" AND e.provider_key_id = ?");
    } else {
      query.push_str(" AND provider_key_id = ?");
    }
  }

  if group_by_key {
    query.push_str(" GROUP BY e.provider_key_id ORDER BY spending_micros DESC");
  } else {
    query.push_str(" GROUP BY provider ORDER BY spending_micros DESC");
  }

  if group_by_key {
    // Query returns 6 columns when grouped by key
    let mut q = sqlx::query_as::<_, (String, i64, i64, i64, Option<i64>, Option<String>)>(&query)
      .bind(start_ms)
      .bind(end_ms);

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

    match q.fetch_all(&state.pool).await {
      Ok(rows) => {
        let mut total_spend = 0.0;
        let mut total_requests = 0i64;

        let data: Vec<ProviderSpending> = rows
          .iter()
          .map(|row| {
            let spending = row.1 as f64 / 1_000_000.0;
            let request_count = row.2;
            total_spend += spending;
            total_requests += request_count;

            let avg_cost = if request_count > 0 {
              spending / request_count as f64
            } else {
              0.0
            };

            ProviderSpending {
              provider: row.0.clone(),
              spending,
              request_count,
              avg_cost_per_request: avg_cost,
              agent_count: row.3,
              provider_key_id: row.4,
              alias: row.5.clone(),
            }
          })
          .collect();

        (
          StatusCode::OK,
          Json(SpendingByProviderResponse {
            summary: ProviderSpendingSummary {
              total_spend,
              total_requests,
              providers_count: u32::try_from(data.len()).unwrap_or(u32::MAX),
            },
            data,
            period: format!("{:?}", params.period)
              .to_lowercase()
              .replace('_', "-"),
            calculated_at: Utc::now().to_rfc3339(),
          }),
        )
          .into_response()
      }
      Err(e) => {
        tracing::error!("Failed to query spending by provider (key): {}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({
            "error": "DATABASE_ERROR",
            "message": "Failed to query spending by provider"
          })),
        )
          .into_response()
      }
    }
  } else {
    // Standard group-by-provider query (4 columns)
    let mut q = sqlx::query_as::<_, (String, i64, i64, i64)>(&query)
      .bind(start_ms)
      .bind(end_ms);

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

    match q.fetch_all(&state.pool).await {
      Ok(rows) => {
        let mut total_spend = 0.0;
        let mut total_requests = 0i64;

        let data: Vec<ProviderSpending> = rows
          .iter()
          .map(|row| {
            let spending = row.1 as f64 / 1_000_000.0;
            let request_count = row.2;
            total_spend += spending;
            total_requests += request_count;

            let avg_cost = if request_count > 0 {
              spending / request_count as f64
            } else {
              0.0
            };

            ProviderSpending {
              provider: row.0.clone(),
              spending,
              request_count,
              avg_cost_per_request: avg_cost,
              agent_count: row.3,
              provider_key_id: None,
              alias: None,
            }
          })
          .collect();

        (
          StatusCode::OK,
          Json(SpendingByProviderResponse {
            summary: ProviderSpendingSummary {
              total_spend,
              total_requests,
              providers_count: u32::try_from(data.len()).unwrap_or(u32::MAX),
            },
            data,
            period: format!("{:?}", params.period)
              .to_lowercase()
              .replace('_', "-"),
            calculated_at: Utc::now().to_rfc3339(),
          }),
        )
          .into_response()
      }
      Err(e) => {
        tracing::error!("Failed to query spending by provider: {}", e);
        (
          StatusCode::INTERNAL_SERVER_ERROR,
          Json(serde_json::json!({
            "error": "DATABASE_ERROR",
            "message": "Failed to query spending by provider"
          })),
        )
          .into_response()
      }
    }
  }
}

/// GET /api/v1/analytics/spending/avg-per-request
///
/// Requires JWT authentication.
/// Admins see all spending; regular users see only their owned agents' spending.
pub async fn get_spending_avg(
  user: AuthenticatedUser,
  State(state): State<AnalyticsState>,
  Query(params): Query<AnalyticsQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let is_admin = user.0.role == "admin";

  let mut query = String::from(
    r"SELECT
         COALESCE(SUM(cost_micros), 0) as total_micros,
         COUNT(*) as request_count,
         COALESCE(MIN(cost_micros), 0) as min_micros,
         COALESCE(MAX(cost_micros), 0) as max_micros
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

  let mut q = sqlx::query_as::<_, (i64, i64, i64, i64)>(&query)
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

  match q.fetch_one(&state.pool).await {
    Ok((total_micros, request_count, min_micros, max_micros)) => {
      let total_usd = total_micros as f64 / 1_000_000.0;
      let avg = if request_count > 0 {
        total_usd / request_count as f64
      } else {
        0.0
      };
      let min_usd = min_micros as f64 / 1_000_000.0;
      let max_usd = max_micros as f64 / 1_000_000.0;

      // Compute median cost (Task 5)
      let median_cost = compute_median(&state, start_ms, end_ms, is_admin, &user.0.sub, &params).await;

      (
        StatusCode::OK,
        Json(AvgCostResponse {
          average_cost_per_request: avg,
          total_requests: request_count,
          total_spend: total_usd,
          min_cost_per_request: min_usd,
          max_cost_per_request: max_usd,
          median_cost_per_request: median_cost,
          period: format!("{:?}", params.period)
            .to_lowercase()
            .replace('_', "-"),
          filters: Filters {
            agent_id: params.agent_id,
            provider_id: params.provider_id,
            provider_key_id: params.provider_key_id,
          },
          calculated_at: Utc::now().to_rfc3339(),
        }),
      )
        .into_response()
    }
    Err(e) => {
      tracing::error!("Failed to query avg cost: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to query average cost"
        })),
      )
        .into_response()
    }
  }
}

/// Compute median `cost_micros`, returning USD.
async fn compute_median(
  state: &AnalyticsState,
  start_ms: i64,
  end_ms: i64,
  is_admin: bool,
  user_sub: &str,
  params: &AnalyticsQuery,
) -> f64 {
  // Step 1: Build the WHERE clause (shared between count and value queries)
  let mut where_clause = String::from(
    "WHERE timestamp_ms >= ? AND timestamp_ms <= ? \
     AND event_type = 'llm_request_completed'",
  );

  if !is_admin {
    where_clause.push_str( " AND EXISTS (SELECT 1 FROM agents a WHERE a.id = analytics_events.agent_id AND a.owner_id = ?)" );
  }
  if params.agent_id.is_some() {
    where_clause.push_str(" AND agent_id = ?");
  }
  if params.provider_id.is_some() {
    where_clause.push_str(" AND provider = ?");
  }
  if params.provider_key_id.is_some() {
    where_clause.push_str(" AND provider_key_id = ?");
  }

  // Step 2: Query the count of matching rows
  let count_query = format!("SELECT COUNT(*) FROM analytics_events {}", where_clause);

  let mut cq = sqlx::query_scalar::<_, i64>(&count_query)
    .bind(start_ms)
    .bind(end_ms);

  if !is_admin {
    cq = cq.bind(user_sub);
  }
  if let Some(agent_id) = params.agent_id {
    cq = cq.bind(agent_id);
  }
  if let Some(ref provider_id) = params.provider_id {
    cq = cq.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    cq = cq.bind(provider_key_id);
  }

  let count = match cq.fetch_one(&state.pool).await {
    Ok(c) => c,
    Err(e) => {
      tracing::error!("Failed to query median count: {}", e);
      return 0.0;
    }
  };

  if count == 0 {
    return 0.0;
  }

  // Step 3: Fetch only the 1-2 middle values needed for the median
  let mid = count as usize / 2;
  let (offset, limit) = if count % 2 == 0 {
    (mid - 1, 2i64)
  } else {
    (mid, 1i64)
  };

  let value_query = format!(
    "SELECT cost_micros FROM analytics_events {} ORDER BY cost_micros LIMIT ? OFFSET ?",
    where_clause
  );

  let mut vq = sqlx::query_scalar::<_, i64>(&value_query)
    .bind(start_ms)
    .bind(end_ms);

  if !is_admin {
    vq = vq.bind(user_sub);
  }
  if let Some(agent_id) = params.agent_id {
    vq = vq.bind(agent_id);
  }
  if let Some(ref provider_id) = params.provider_id {
    vq = vq.bind(provider_id);
  }
  if let Some(provider_key_id) = params.provider_key_id {
    vq = vq.bind(provider_key_id);
  }

  let values = match vq
    .bind(limit)
    .bind(offset as i64)
    .fetch_all(&state.pool)
    .await
  {
    Ok(v) => v,
    Err(e) => {
      tracing::error!("Failed to query median values: {}", e);
      return 0.0;
    }
  };

  if values.is_empty() {
    return 0.0;
  }

  let median_micros = if count % 2 == 0 && values.len() == 2 {
    (values[0] + values[1]) as f64 / 2.0
  } else {
    values[0] as f64
  };

  median_micros / 1_000_000.0
}
