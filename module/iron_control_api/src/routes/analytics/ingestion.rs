//! Event ingestion endpoint
//!
//! Handles POST /api/v1/analytics/events for receiving analytics events from LlmRouter.
//! Validates IC tokens to authenticate agent identity and prevents event duplication.

use axum::{
  extract::State,
  http::StatusCode,
  response::{ IntoResponse, Json },
};
use chrono::Utc;
use super::shared::{ AnalyticsEventRequest, EventResponse, AnalyticsState };

/// POST /api/v1/analytics/events
///
/// Ingest analytics event from LlmRouter.
/// Requires valid IC Token for authentication - agent_id is derived from token claims.
/// Returns 202 Accepted for new events, 200 OK for duplicates.
pub async fn post_event(
  State( state ): State< AnalyticsState >,
  Json( event ): Json< AnalyticsEventRequest >,
) -> impl IntoResponse
{
  // Verify IC Token and extract agent identity
  let claims = match state.ic_token_manager.verify_token( &event.ic_token )
  {
    Ok( claims ) => claims,
    Err( e ) =>
    {
      tracing::warn!( "IC Token verification failed: {}", e );
      return (
        StatusCode::UNAUTHORIZED,
        Json( serde_json::json!({
          "error": "UNAUTHORIZED",
          "message": "Invalid or expired IC token"
        }) )
      ).into_response();
    }
  };

  // Extract agent_id from token claims (format: "agent_<id>")
  let agent_id: i64 = match claims.agent_id.strip_prefix( "agent_" )
  {
    Some( id_str ) => match id_str.parse()
    {
      Ok( id ) => id,
      Err( _ ) =>
      {
        return (
          StatusCode::BAD_REQUEST,
          Json( serde_json::json!({
            "error": "INVALID_TOKEN",
            "message": "Invalid agent_id format in token"
          }) )
        ).into_response();
      }
    },
    None =>
    {
      return (
        StatusCode::BAD_REQUEST,
        Json( serde_json::json!({
          "error": "INVALID_TOKEN",
          "message": "Invalid agent_id format in token"
        }) )
      ).into_response();
    }
  };

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
    && ( event.input_tokens.is_none() || event.output_tokens.is_none() )
  {
    return (
      StatusCode::BAD_REQUEST,
      Json( serde_json::json!({
        "error": "VALIDATION_ERROR",
        "message": "input_tokens and output_tokens required for completed events"
      }) )
    ).into_response();
  }

  let now_ms = Utc::now().timestamp_millis();

  // INSERT OR IGNORE for deduplication (agent_id from verified token)
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
    .bind( agent_id )  // From verified IC token, not request body
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
