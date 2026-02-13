//! Health check endpoint
//!
//! Phase 4 Day 29: REST API Endpoints - Health Check

use axum::{ http::StatusCode, response::{ IntoResponse, Json } };
use chrono::Utc;
use serde::{ Serialize };

/// Health check response
#[ derive( Debug, Serialize ) ]
pub struct HealthResponse
{
  /// Service health status
  pub status: String,
  /// Response Unix timestamp
  pub timestamp: i64,
}

/// GET /api/health
///
/// Health check endpoint for monitoring and load balancers
///
/// # Returns
///
/// Always returns 200 OK with service status
#[ must_use ]
pub async fn health_check() -> impl IntoResponse
{
  let now = Utc::now().timestamp();

  ( StatusCode::OK, Json( HealthResponse
  {
    status: "healthy".to_string(),
    timestamp: now,
  } ) )
}

