//! Event ingestion endpoint
//!
//! Handles POST /api/v1/analytics/events for receiving analytics events from `LlmRouter`.
//! Validates IC tokens to authenticate agent identity and prevents event duplication.

use super::shared::{
  AnalyticsEventRequest, AnalyticsEventWithAgent, AnalyticsState, EventResponse, EventsListQuery,
  EventsListResponse, Pagination,
};
use crate::ic_token;
use axum::{
  extract::{Query, State},
  http::StatusCode,
  response::{IntoResponse, Json},
};
use chrono::Utc;

/// POST /api/v1/analytics/events
///
/// Ingest analytics event from `LlmRouter`.
/// Requires valid IC Token for authentication - `agent_id` is derived from token claims.
/// Returns 202 Accepted for new events, 200 OK for duplicates.
pub async fn post_event(
  State(state): State<AnalyticsState>,
  Json(event): Json<AnalyticsEventRequest>,
) -> impl IntoResponse {
  // Verify IC Token and extract agent identity (JWT signature + hash-check against database)
  let (agent_id, _claims) = match ic_token::validate_ic_token_for_endpoint(
    &state.ic_token_manager,
    &event.ic_token,
    &state.pool,
    &state.ic_token_rate_limiter,
  )
  .await
  {
    Ok(result) => result,
    Err(response) => return response,
  };

  // Validate event_type
  if event.event_type != "llm_request_completed" && event.event_type != "llm_request_failed" {
    return (
      StatusCode::BAD_REQUEST,
      Json(serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "event_type must be 'llm_request_completed' or 'llm_request_failed'"
      })),
    )
      .into_response();
  }

  // Validate provider
  if event.provider != "openai" && event.provider != "anthropic" && event.provider != "unknown" {
    return (
      StatusCode::BAD_REQUEST,
      Json(serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "provider must be 'openai', 'anthropic', or 'unknown'"
      })),
    )
      .into_response();
  }

  // For completed events, require token counts
  if event.event_type == "llm_request_completed"
    && (event.input_tokens.is_none() || event.output_tokens.is_none())
  {
    return (
      StatusCode::BAD_REQUEST,
      Json(serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "input_tokens and output_tokens required for completed events"
      })),
    )
      .into_response();
  }

  let now_ms = Utc::now().timestamp_millis();

  // INSERT OR IGNORE for deduplication (agent_id from verified token)
  let result = sqlx::query(
    r"INSERT OR IGNORE INTO analytics_events
       (event_id, timestamp_ms, event_type, model, provider,
        input_tokens, output_tokens, cost_micros,
        agent_id, provider_id, error_code, error_message, provider_key_id, received_at)
       VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
  )
  .bind(&event.event_id)
  .bind(event.timestamp_ms)
  .bind(&event.event_type)
  .bind(&event.model)
  .bind(&event.provider)
  .bind(event.input_tokens.unwrap_or(0))
  .bind(event.output_tokens.unwrap_or(0))
  .bind(event.cost_micros.unwrap_or(0))
  .bind(agent_id) // From verified IC token, not request body
  .bind(&event.provider_id)
  .bind(&event.error_code)
  .bind(&event.error_message)
  .bind(event.provider_key_id)
  .bind(now_ms)
  .execute(&state.pool)
  .await;

  match result {
    Ok(r) => {
      if r.rows_affected() > 0 {
        (
          StatusCode::ACCEPTED,
          Json(EventResponse {
            event_id: event.event_id,
            status: "accepted".to_string(),
          }),
        )
          .into_response()
      } else {
        (
          StatusCode::OK,
          Json(EventResponse {
            event_id: event.event_id,
            status: "duplicate".to_string(),
          }),
        )
          .into_response()
      }
    }
    Err(e) => {
      tracing::error!("Failed to insert event: {}", e);
      (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to store event"
        })),
      )
        .into_response()
    }
  }
}

/// GET /api/v1/analytics/events
///
/// List analytics events with pagination and filtering.
/// Returns recent events sorted by timestamp descending.
pub async fn list_events(
  State(state): State<AnalyticsState>,
  Query(params): Query<EventsListQuery>,
) -> impl IntoResponse {
  let (start_ms, end_ms) = params.period.to_range();
  let offset = (params.page.saturating_sub(1)) * params.per_page;

  // Build count query
  let count_result: Result<i64, _> = if let Some(agent_id) = params.agent_id {
    sqlx::query_scalar(
      "SELECT COUNT(*) FROM analytics_events WHERE timestamp_ms >= ? AND timestamp_ms <= ? AND agent_id = ?"
    )
      .bind( start_ms )
      .bind( end_ms )
      .bind( agent_id )
      .fetch_one( &state.pool )
      .await
  } else {
    sqlx::query_scalar(
      "SELECT COUNT(*) FROM analytics_events WHERE timestamp_ms >= ? AND timestamp_ms <= ?",
    )
    .bind(start_ms)
    .bind(end_ms)
    .fetch_one(&state.pool)
    .await
  };

  let total = match count_result {
    Ok(c) => u32::try_from(c).unwrap_or(u32::MAX),
    Err(e) => {
      tracing::error!("Failed to count events: {}", e);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to count events"
        })),
      )
        .into_response();
    }
  };

  // Build data query with agent join
  let query = if params.agent_id.is_some() {
    r"SELECT
         e.event_id, e.timestamp_ms, e.event_type, e.model, e.provider,
         e.input_tokens, e.output_tokens, e.cost_micros, e.agent_id,
         COALESCE(a.name, 'Unknown') as agent_name,
         e.error_code, e.error_message, e.provider_key_id
       FROM analytics_events e
       LEFT JOIN agents a ON e.agent_id = a.id
       WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ? AND e.agent_id = ?
       ORDER BY e.timestamp_ms DESC
       LIMIT ? OFFSET ?"
  } else {
    r"SELECT
         e.event_id, e.timestamp_ms, e.event_type, e.model, e.provider,
         e.input_tokens, e.output_tokens, e.cost_micros, e.agent_id,
         COALESCE(a.name, 'Unknown') as agent_name,
         e.error_code, e.error_message, e.provider_key_id
       FROM analytics_events e
       LEFT JOIN agents a ON e.agent_id = a.id
       WHERE e.timestamp_ms >= ? AND e.timestamp_ms <= ?
       ORDER BY e.timestamp_ms DESC
       LIMIT ? OFFSET ?"
  };

  let data: Vec<AnalyticsEventWithAgent> = match if let Some(agent_id) = params.agent_id {
    sqlx::query_as(query)
      .bind(start_ms)
      .bind(end_ms)
      .bind(agent_id)
      .bind(i64::from(params.per_page))
      .bind(i64::from(offset))
      .fetch_all(&state.pool)
      .await
  } else {
    sqlx::query_as(query)
      .bind(start_ms)
      .bind(end_ms)
      .bind(i64::from(params.per_page))
      .bind(i64::from(offset))
      .fetch_all(&state.pool)
      .await
  } {
    Ok(rows) => rows,
    Err(e) => {
      tracing::error!("Failed to fetch events: {}", e);
      return (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
          "error": "DATABASE_ERROR",
          "message": "Failed to fetch events"
        })),
      )
        .into_response();
    }
  };

  let response = EventsListResponse {
    data,
    pagination: Pagination::new(params.page, params.per_page, total),
    period: format!("{:?}", params.period)
      .to_lowercase()
      .replace("days", "-days"),
    calculated_at: Utc::now().to_rfc3339(),
  };

  (StatusCode::OK, Json(response)).into_response()
}
