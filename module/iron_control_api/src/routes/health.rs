//! Health check endpoint
//!
//! Phase 4 Day 29: REST API Endpoints - Health Check

use axum::{ http::StatusCode, response::{ IntoResponse, Json } };
use serde::{ Serialize };

/// Health check response
#[ derive( Debug, Serialize ) ]
pub struct HealthResponse
{
  pub status: String,
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
  let now = std::time::SystemTime::now()
    .duration_since( std::time::UNIX_EPOCH )
    .expect( "LOUD FAILURE: Time went backwards" )
    .as_secs() as i64;

  ( StatusCode::OK, Json( HealthResponse
  {
    status: "healthy".to_string(),
    timestamp: now,
  } ) )
}

