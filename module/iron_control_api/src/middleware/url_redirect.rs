//! URL redirect middleware for backward compatibility
//!
//! Provides 308 Permanent Redirect from deprecated URL paths to new spec-compliant paths.
//!
//! ## Redirects
//! - `/api/tokens` → `/api/v1/api-tokens`
//! - `/api/tokens/:id` → `/api/v1/api-tokens/:id`
//! - `/api/tokens/:id/rotate` → `/api/v1/api-tokens/:id/rotate`
//!
//! ## Expiration
//! These redirects expire 3 months after deployment (2025-03-12).
//! After expiration, old paths return 404 Not Found.

use axum::
{
  body::Body,
  http::{ Request, Response, StatusCode, Uri },
  middleware::Next,
};
use std::time::{ SystemTime, UNIX_EPOCH };

/// Deployment date for redirect expiration calculation
const DEPLOYMENT_DATE: u64 = 1733961600; // 2024-12-12 00:00:00 UTC

/// Redirect expiration duration (3 months in seconds)
const EXPIRATION_SECONDS: u64 = 90 * 24 * 60 * 60; // 90 days

/// URL redirect middleware
///
/// Redirects deprecated `/api/tokens` paths to `/api/v1/api-tokens` with 308 status.
/// Expires 3 months after deployment.
pub async fn redirect_old_tokens_url( req: Request<Body>, next: Next ) -> Response<Body>
{
  let uri = req.uri();
  let path = uri.path();

  // Check if redirect has expired
  let now = SystemTime::now().duration_since( UNIX_EPOCH ).unwrap().as_secs();
  let redirect_expired = now > DEPLOYMENT_DATE + EXPIRATION_SECONDS;

  if redirect_expired
  {
    // After expiration, old paths return 404
    if path.starts_with( "/api/tokens" )
    {
      return Response::builder()
        .status( StatusCode::NOT_FOUND )
        .body( Body::empty() )
        .unwrap();
    }
  }
  else
  {
    // During transition period, redirect old paths
    if let Some( new_path ) = convert_old_path_to_new( path )
    {
      let new_uri = build_redirect_uri( uri, &new_path );
      return Response::builder()
        .status( StatusCode::PERMANENT_REDIRECT )
        .header( "Location", new_uri )
        .body( Body::empty() )
        .unwrap();
    }
  }

  // Not an old token path, continue to next handler
  next.run( req ).await
}

/// Convert old token path to new path
pub fn convert_old_path_to_new( path: &str ) -> Option<String>
{
  path.strip_prefix( "/api/tokens" ).map( |suffix| format!( "/api/v1/api-tokens{suffix}" ) )
}

/// Build redirect URI preserving query parameters
pub fn build_redirect_uri( original_uri: &Uri, new_path: &str ) -> String
{
  if let Some( query ) = original_uri.query()
  {
    format!( "{}?{}", new_path, query )
  }
  else
  {
    new_path.to_string()
  }
}
