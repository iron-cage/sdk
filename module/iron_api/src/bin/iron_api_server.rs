//! Iron Cage REST API Server
//!
//! Provides REST API endpoints for token management, usage tracking,
//! and budget limits. Serves the frontend dashboard at localhost:3000.
//!
//! # Architecture
//!
//! This server uses a combined `AppState` pattern to manage multiple service states
//! (authentication, token management) while keeping routes modular. The `FromRef` trait
//! allows Axum extractors (like `AuthenticatedUser`) to access specific sub-states
//! without coupling to the full state structure.
//!
//! # Configuration
//!
//! - **DATABASE_URL**: SQLite connection string (default: `sqlite://./iron.db?mode=rwc`)
//!   - The `?mode=rwc` parameter is REQUIRED for SQLite to create the database file
//!   - Production should use PostgreSQL: `postgres://user:pass@host/database`
//! - **JWT_SECRET**: Secret key for JWT signing (default: dev-secret-change-in-production)
//!   - Production MUST use a cryptographically secure random value
//!   - Generate with: `openssl rand -base64 32`
//!
//! # Endpoints
//!
//! ## Authentication (Public)
//! - `POST /api/auth/login` - User login (returns access + refresh tokens)
//! - `POST /api/auth/refresh` - Refresh access token
//! - `POST /api/auth/logout` - Logout (blacklist refresh token)
//!
//! ## Token Management (Requires Authentication)
//! - `GET /api/tokens` - List user's tokens
//! - `POST /api/tokens` - Create new token
//! - `GET /api/tokens/:id` - Get specific token
//! - `POST /api/tokens/:id/rotate` - Rotate token (issue new value)
//! - `DELETE /api/tokens/:id` - Revoke token (soft delete)
//!
//! ## Health Check (Public)
//! - `GET /health` - Server health status
//!
//! # Known Pitfalls
//!
//! **Pitfall:** SQLite database creation fails without `?mode=rwc` parameter.
//!
//! **Root Cause:** SQLite requires explicit permission to create files. Without
//! `mode=rwc`, connection attempts to non-existent databases fail.
//!
//! **Fix:** Always include `?mode=rwc` in SQLite URLs: `sqlite://./path.db?mode=rwc`
//!
//! **Prevention:** Use environment variable `DATABASE_URL` with proper format, or
//! ensure default value includes the parameter (as implemented here).

use axum::{
  routing::{ get, post, delete },
  Router,
  http::{ Method, header },
};
use std::{ net::SocketAddr, env };
use tower_http::cors::CorsLayer;

/// Deployment mode classification for production safety warnings
///
/// Used to detect whether the server is running in:
/// - **Pilot**: Localhost development environment
/// - **ProductionUnconfirmed**: Production environment detected but not explicitly configured
/// - **Production**: Explicit production deployment (IRON_DEPLOYMENT_MODE=production)
enum DeploymentMode
{
  /// Localhost development environment (safe to use defaults)
  Pilot,

  /// Production environment detected but IRON_DEPLOYMENT_MODE not set
  /// (triggers warning to ensure conscious production deployment)
  ProductionUnconfirmed,

  /// Explicit production deployment (IRON_DEPLOYMENT_MODE=production set)
  Production,
}

/// Detect deployment mode using environment signals
///
/// Checks multiple signals to determine if running in production environment:
/// - **Explicit**: `IRON_DEPLOYMENT_MODE=production` environment variable
/// - **Kubernetes**: `KUBERNETES_SERVICE_HOST` present
/// - **AWS**: `AWS_EXECUTION_ENV` present (Lambda/ECS)
/// - **Heroku**: `DYNO` environment variable
/// - **Build Type**: Release build (debug_assertions disabled)
///
/// Returns:
/// - `Production`: Explicitly configured for production
/// - `ProductionUnconfirmed`: Detected production but not explicitly configured
/// - `Pilot`: Localhost development environment
fn detect_deployment_mode() -> DeploymentMode
{
  // Check for explicit deployment mode setting
  match env::var( "IRON_DEPLOYMENT_MODE" ).as_deref()
  {
    Ok( "production" ) => return DeploymentMode::Production,
    Ok( "pilot" ) => return DeploymentMode::Pilot,
    _ => {}
  }

  // Heuristics for unintentional production deployment
  let is_production =
    env::var( "KUBERNETES_SERVICE_HOST" ).is_ok() ||  // Kubernetes
    env::var( "AWS_EXECUTION_ENV" ).is_ok() ||        // AWS Lambda/ECS
    env::var( "DYNO" ).is_ok() ||                     // Heroku
    !cfg!( debug_assertions );                        // Release build

  if is_production
  {
    DeploymentMode::ProductionUnconfirmed
  }
  else
  {
    DeploymentMode::Pilot
  }
}

/// Combined application state containing all service states
///
/// This pattern allows routes to access only the state they need through Axum's
/// `FromRef` mechanism. Routes using `State<AuthState>` or `State<TokenState>`
/// automatically extract their sub-state from the combined `AppState`.
///
/// # Why This Pattern
///
/// Without combined state, each route would need to know the full application state
/// structure. This creates tight coupling and makes routes harder to test in isolation.
///
/// With `FromRef`, routes declare only their dependencies:
/// - Authentication routes: `State<AuthState>`
/// - Token routes: `State<TokenState>`
/// - Extractors (like `AuthenticatedUser`): Access `AuthState` via `FromRef`
///
/// # Example
///
/// ```rust
/// // Route only declares what it needs
/// async fn my_route( State(token_state): State<TokenState> ) {
///   // Automatically extracted from AppState.tokens
/// }
/// ```
#[ derive( Clone ) ]
struct AppState
{
  auth: iron_api::routes::auth::AuthState,
  tokens: iron_api::routes::tokens::TokenState,
  usage: iron_api::routes::usage::UsageState,
  limits: iron_api::routes::limits::LimitsState,
  traces: iron_api::routes::traces::TracesState,
  providers: iron_api::routes::providers::ProvidersState,
}

/// Enable auth routes and extractors to access AuthState from combined AppState
///
/// This implementation allows:
/// - Routes with `State<AuthState>` parameter to extract auth sub-state
/// - `AuthenticatedUser` extractor to access JWT secret for token verification
impl axum::extract::FromRef< AppState > for iron_api::routes::auth::AuthState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.auth.clone()
  }
}

/// Enable token routes to access TokenState from combined AppState
///
/// This implementation allows routes with `State<TokenState>` parameter to
/// extract the token management sub-state (database connection, token generator).
impl axum::extract::FromRef< AppState > for iron_api::routes::tokens::TokenState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.tokens.clone()
  }
}

/// Enable usage routes to access UsageState from combined AppState
impl axum::extract::FromRef< AppState > for iron_api::routes::usage::UsageState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.usage.clone()
  }
}

/// Enable limits routes to access LimitsState from combined AppState
impl axum::extract::FromRef< AppState > for iron_api::routes::limits::LimitsState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.limits.clone()
  }
}

/// Enable traces routes to access TracesState from combined AppState
impl axum::extract::FromRef< AppState > for iron_api::routes::traces::TracesState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.traces.clone()
  }
}

/// Enable providers routes to access ProvidersState from combined AppState
impl axum::extract::FromRef< AppState > for iron_api::routes::providers::ProvidersState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.providers.clone()
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load .env file if present (ignore if not found)
  match dotenvy::dotenv()
  {
    Ok( path ) => eprintln!( "Loaded .env from: {:?}", path ),
    Err( e ) => eprintln!( "No .env file loaded: {}", e ),
  }

  // Debug: Check if master key is set
  match std::env::var( "IRON_SECRETS_MASTER_KEY" )
  {
    Ok( _ ) => eprintln!( "IRON_SECRETS_MASTER_KEY is set" ),
    Err( _ ) => eprintln!( "IRON_SECRETS_MASTER_KEY is NOT set" ),
  }

  // Initialize tracing
  tracing_subscriber::fmt::init();

  // Detect deployment mode before starting server
  let mode = detect_deployment_mode();
  match mode
  {
    DeploymentMode::ProductionUnconfirmed =>
    {
      eprintln!( "⚠️  WARNING: Production environment detected but IRON_DEPLOYMENT_MODE not set" );
      eprintln!( "⚠️  Set IRON_DEPLOYMENT_MODE=production to confirm production deployment" );
      eprintln!( "⚠️  See docs/production_deployment.md for security checklist" );
      eprintln!();
      eprintln!( "Sleeping 10 seconds to ensure this warning is visible..." );
      std::thread::sleep( std::time::Duration::from_secs( 10 ) );
    }
    DeploymentMode::Production =>
    {
      eprintln!( "✓ Production mode confirmed (IRON_DEPLOYMENT_MODE=production)" );
    }
    DeploymentMode::Pilot =>
    {
      eprintln!( "✓ Pilot mode (localhost only)" );
    }
  }

  // Database URL (SQLite for pilot)
  let database_url = std::env::var( "DATABASE_URL" )
    .unwrap_or_else( |_| "sqlite://./iron.db?mode=rwc".to_string() );

  // JWT secret for authentication
  let jwt_secret = std::env::var( "JWT_SECRET" )
    .unwrap_or_else( |_| "dev-secret-change-in-production".to_string() );

  tracing::info!( "Initializing API server..." );
  tracing::info!( "Database: {}", database_url );

  // Initialize route states
  let auth_state = iron_api::routes::auth::AuthState::new( jwt_secret, &database_url )
    .await
    .expect( "Failed to initialize auth state" );

  let token_state = iron_api::routes::tokens::TokenState::new( &database_url )
    .await
    .expect( "Failed to initialize token state" );

  let usage_state = iron_api::routes::usage::UsageState::new( &database_url )
    .await
    .expect( "Failed to initialize usage state" );

  let limits_state = iron_api::routes::limits::LimitsState::new( &database_url )
    .await
    .expect( "Failed to initialize limits state" );

  let traces_state = iron_api::routes::traces::TracesState::new( &database_url )
    .await
    .expect( "Failed to initialize traces state" );

  let providers_state = iron_api::routes::providers::ProvidersState::new( &database_url )
    .await
    .expect( "Failed to initialize providers storage" );

  // Create combined app state
  let app_state = AppState
  {
    auth: auth_state,
    tokens: token_state,
    usage: usage_state,
    limits: limits_state,
    traces: traces_state,
    providers: providers_state,
  };

  // Build router with all endpoints
  let app = Router::new()
    // Health check (FR-2: Health endpoint at /api/health)
    .route( "/api/health", get( iron_api::routes::health::health_check ) )

    // Authentication endpoints
    .route( "/api/auth/login", post( iron_api::routes::auth::login ) )
    .route( "/api/auth/refresh", post( iron_api::routes::auth::refresh ) )
    .route( "/api/auth/logout", post( iron_api::routes::auth::logout ) )

    // Token management endpoints
    .route( "/api/tokens", post( iron_api::routes::tokens::create_token ) )
    .route( "/api/tokens", get( iron_api::routes::tokens::list_tokens ) )
    .route( "/api/tokens/:id", get( iron_api::routes::tokens::get_token ) )
    .route( "/api/tokens/:id/rotate", post( iron_api::routes::tokens::rotate_token ) )
    .route( "/api/tokens/:id", delete( iron_api::routes::tokens::revoke_token ) )

    // Usage analytics endpoints
    .route( "/api/usage/aggregate", get( iron_api::routes::usage::get_aggregate_usage ) )
    .route( "/api/usage/by-project/:project_id", get( iron_api::routes::usage::get_usage_by_project ) )
    .route( "/api/usage/by-provider/:provider", get( iron_api::routes::usage::get_usage_by_provider ) )

    // Limits management endpoints
    .route( "/api/limits", get( iron_api::routes::limits::list_limits ) )
    .route( "/api/limits", post( iron_api::routes::limits::create_limit ) )
    .route( "/api/limits/:id", get( iron_api::routes::limits::get_limit ) )
    .route( "/api/limits/:id", axum::routing::put( iron_api::routes::limits::update_limit ) )
    .route( "/api/limits/:id", axum::routing::delete( iron_api::routes::limits::delete_limit ) )

    // Traces endpoints
    .route( "/api/traces", get( iron_api::routes::traces::list_traces ) )
    .route( "/api/traces/:id", get( iron_api::routes::traces::get_trace ) )

    // Provider key management endpoints
    .route( "/api/providers", post( iron_api::routes::providers::create_provider_key ) )
    .route( "/api/providers", get( iron_api::routes::providers::list_provider_keys ) )
    .route( "/api/providers/:id", get( iron_api::routes::providers::get_provider_key ) )
    .route( "/api/providers/:id", axum::routing::put( iron_api::routes::providers::update_provider_key ) )
    .route( "/api/providers/:id", delete( iron_api::routes::providers::delete_provider_key ) )
    .route( "/api/projects/:project_id/provider", post( iron_api::routes::providers::assign_provider_to_project ) )

    // Apply combined state to all routes
    .with_state( app_state )

    // CORS middleware (FR-4: Restrict to frontend origin for pilot)
    // Pilot configuration: localhost:5173, 5174, 5175 (Vite dev server on any available port)
    // Allow methods: GET, POST, PUT, DELETE (all REST operations)
    // Allow headers: Content-Type (JSON requests), Authorization (Bearer tokens)
    .layer(
      CorsLayer::new()
        .allow_origin( [
          "http://localhost:5173".parse::<axum::http::HeaderValue>().unwrap(),
          "http://localhost:5174".parse::<axum::http::HeaderValue>().unwrap(),
          "http://localhost:5175".parse::<axum::http::HeaderValue>().unwrap(),
        ] )
        .allow_methods( [ Method::GET, Method::POST, Method::PUT, Method::DELETE ] )
        .allow_headers( [ header::CONTENT_TYPE, header::AUTHORIZATION ] )
    );

  // Server address
  let addr = SocketAddr::from( ( [127, 0, 0, 1], 3000 ) );
  
  tracing::info!( "API server listening on http://{}", addr );
  tracing::info!( "Endpoints:" );
  tracing::info!( "  GET  /api/health" );
  tracing::info!( "  POST /api/auth/login" );
  tracing::info!( "  POST /api/auth/refresh" );
  tracing::info!( "  POST /api/auth/logout" );
  tracing::info!( "  GET  /api/tokens" );
  tracing::info!( "  POST /api/tokens" );
  tracing::info!( "  GET  /api/tokens/:id" );
  tracing::info!( "  POST /api/tokens/:id/rotate" );
  tracing::info!( "  DELETE /api/tokens/:id" );
  tracing::info!( "  GET  /api/usage/aggregate" );
  tracing::info!( "  GET  /api/usage/by-project/:project_id" );
  tracing::info!( "  GET  /api/usage/by-provider/:provider" );
  tracing::info!( "  GET  /api/limits" );
  tracing::info!( "  POST /api/limits" );
  tracing::info!( "  GET  /api/limits/:id" );
  tracing::info!( "  PUT  /api/limits/:id" );
  tracing::info!( "  DELETE /api/limits/:id" );
  tracing::info!( "  GET  /api/traces" );
  tracing::info!( "  GET  /api/traces/:id" );
  tracing::info!( "  POST /api/providers" );
  tracing::info!( "  GET  /api/providers" );
  tracing::info!( "  GET  /api/providers/:id" );
  tracing::info!( "  PUT  /api/providers/:id" );
  tracing::info!( "  DELETE /api/providers/:id" );
  tracing::info!( "  POST /api/projects/:project_id/provider" );

  // Start server
  let listener = tokio::net::TcpListener::bind( addr ).await?;
  axum::serve( listener, app ).await?;

  Ok( () )
}

#[ cfg( test ) ]
mod deployment_mode_tests
{
  use super::*;

  /// Helper to clear all production environment variables
  fn clear_production_env_vars()
  {
    env::remove_var( "IRON_DEPLOYMENT_MODE" );
    env::remove_var( "KUBERNETES_SERVICE_HOST" );
    env::remove_var( "AWS_EXECUTION_ENV" );
    env::remove_var( "DYNO" );
  }

  #[ test ]
  fn test_pilot_mode_default()
  {
    // Clear all production indicators
    clear_production_env_vars();

    let mode = detect_deployment_mode();

    // In debug builds with no env vars, should detect pilot mode
    #[ cfg( debug_assertions ) ]
    assert!( matches!( mode, DeploymentMode::Pilot ) );
  }

  #[ test ]
  fn test_production_kubernetes_detection()
  {
    clear_production_env_vars();
    env::set_var( "KUBERNETES_SERVICE_HOST", "10.0.0.1" );

    let mode = detect_deployment_mode();

    assert!( matches!( mode, DeploymentMode::ProductionUnconfirmed ) );

    env::remove_var( "KUBERNETES_SERVICE_HOST" );
  }

  #[ test ]
  fn test_production_aws_detection()
  {
    clear_production_env_vars();
    env::set_var( "AWS_EXECUTION_ENV", "AWS_ECS_FARGATE" );

    let mode = detect_deployment_mode();

    assert!( matches!( mode, DeploymentMode::ProductionUnconfirmed ) );

    env::remove_var( "AWS_EXECUTION_ENV" );
  }

  #[ test ]
  fn test_production_heroku_detection()
  {
    clear_production_env_vars();
    env::set_var( "DYNO", "web.1" );

    let mode = detect_deployment_mode();

    assert!( matches!( mode, DeploymentMode::ProductionUnconfirmed ) );

    env::remove_var( "DYNO" );
  }

  #[ test ]
  fn test_explicit_production_mode()
  {
    clear_production_env_vars();
    env::set_var( "IRON_DEPLOYMENT_MODE", "production" );

    let mode = detect_deployment_mode();

    assert!( matches!( mode, DeploymentMode::Production ) );

    env::remove_var( "IRON_DEPLOYMENT_MODE" );
  }

  #[ test ]
  fn test_explicit_production_overrides_heuristics()
  {
    clear_production_env_vars();

    // Set multiple production indicators
    env::set_var( "KUBERNETES_SERVICE_HOST", "10.0.0.1" );
    env::set_var( "AWS_EXECUTION_ENV", "AWS_ECS_FARGATE" );

    // But explicit mode should take precedence
    env::set_var( "IRON_DEPLOYMENT_MODE", "production" );

    let mode = detect_deployment_mode();

    assert!( matches!( mode, DeploymentMode::Production ) );

    // Cleanup
    env::remove_var( "IRON_DEPLOYMENT_MODE" );
    env::remove_var( "KUBERNETES_SERVICE_HOST" );
    env::remove_var( "AWS_EXECUTION_ENV" );
  }

  #[ test ]
  fn test_release_build_detection()
  {
    clear_production_env_vars();

    let mode = detect_deployment_mode();

    // In release builds (debug_assertions disabled), should detect production
    #[ cfg( not( debug_assertions ) ) ]
    assert!( matches!( mode, DeploymentMode::ProductionUnconfirmed ) );

    // In debug builds, should detect pilot
    #[ cfg( debug_assertions ) ]
    assert!( matches!( mode, DeploymentMode::Pilot ) );
  }

  #[ test ]
  fn test_multiple_production_indicators()
  {
    clear_production_env_vars();

    // Set multiple production environment variables
    env::set_var( "KUBERNETES_SERVICE_HOST", "10.0.0.1" );
    env::set_var( "AWS_EXECUTION_ENV", "AWS_ECS_FARGATE" );
    env::set_var( "DYNO", "web.1" );

    let mode = detect_deployment_mode();

    // Should still detect as ProductionUnconfirmed (not explicitly set)
    assert!( matches!( mode, DeploymentMode::ProductionUnconfirmed ) );

    // Cleanup
    env::remove_var( "KUBERNETES_SERVICE_HOST" );
    env::remove_var( "AWS_EXECUTION_ENV" );
    env::remove_var( "DYNO" );
  }
}
