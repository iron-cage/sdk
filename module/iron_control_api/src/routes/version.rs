//! Version endpoint - API version and build metadata
//!
//! ## Endpoint
//!
//! `GET /api/v1/version`
//!
//! ## Response
//!
//! Returns version info and build metadata from compile-time.
//! All data from build.rs (not runtime), ensuring static values.
//!
//! ## Migration Notes
//!
//! Replaces version field in health endpoint.
//! - Old pattern: health.version (removed)
//! - New pattern: dedicated /api/v1/version
//!
//! This separation ensures health endpoint is minimal (status, timestamp only)
//! while version information is available through proper discovery mechanism.

use axum::{ Json, response::IntoResponse };
use serde::{ Serialize, Deserialize };

/// API version response structure
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct VersionResponse
{
  pub current_version: String,
  pub supported_versions: Vec< String >,
  pub deprecated_versions: Vec< String >,
  pub latest_endpoint: String,
  pub build: BuildInfo,
}

/// Build metadata from compile-time
#[ derive( Debug, Serialize, Deserialize ) ]
pub struct BuildInfo
{
  pub commit: String,
  pub timestamp: String,
  pub environment: String,
}

/// GET /api/v1/version
///
/// Returns API version and build metadata.
/// All metadata from build-time (env! macros), not runtime.
///
/// ## Security
///
/// This endpoint is public (no authentication required) to
/// allow clients to discover API version before authentication.
///
/// ## Build Metadata
///
/// - commit: Git SHA from VERGEN_GIT_SHA (build.rs)
/// - timestamp: Build timestamp from VERGEN_BUILD_TIMESTAMP (build.rs)
/// - environment: Runtime environment from ENVIRONMENT var or "development"
#[ must_use ]
pub async fn get_version() -> impl IntoResponse
{
  let response = VersionResponse
  {
    current_version: "v1".to_string(),
    supported_versions: vec![ "v1".to_string() ],
    deprecated_versions: vec![],
    latest_endpoint: "/api/v1".to_string(),
    build: BuildInfo
    {
      commit: env!( "VERGEN_GIT_SHA" ).to_string(),
      timestamp: env!( "VERGEN_BUILD_TIMESTAMP" ).to_string(),
      environment: std::env::var( "ENVIRONMENT" )
        .unwrap_or_else( |_| "development".to_string() ),
    },
  };

  Json( response )
}
