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
//!   - Canonical path: `iron.db` for standalone module use
//!   - Project Makefile overrides with module-specific paths (dev_control.db)
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
//! - `GET /api/v1/api-tokens` - List user's tokens
//! - `POST /api/v1/api-tokens` - Create new token
//! - `GET /api/v1/api-tokens/:id` - Get specific token
//! - `POST /api/v1/api-tokens/:id/rotate` - Rotate token (issue new value)
//! - `DELETE /api/v1/api-tokens/:id` - Revoke token (soft delete)
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
  Router, http::{ Method, header }, routing::{ delete, get, post, put }
};
use std::{ net::SocketAddr, env };
use tower_http::cors::CorsLayer;
use workspace_tools::workspace;

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

  /// Explicit development deployment (IRON_DEPLOYMENT_MODE=development set)
  Development,
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
    Ok( "development" ) => return DeploymentMode::Development,
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

/// Get database URL with workspace-relative path resolution
///
/// Resolves database path using workspace_tools for context-independent paths:
/// - **Pilot mode**: {workspace_root}/iron.db
/// - **Development mode**: {workspace_root}/data/dev_control.db
/// - **Production mode**: {workspace_root}/data/iron_production.db
///
/// Respects DATABASE_URL environment variable if set (highest priority).
/// All paths are workspace-relative and work regardless of execution directory.
///
/// Returns SQLite URL with ?mode=rwc parameter for database creation.
fn get_database_url() -> Result< String, Box< dyn std::error::Error > >
{
  // Check for explicit DATABASE_URL override (highest priority)
  if let Ok( url ) = env::var( "DATABASE_URL" )
  {
    return Ok( url );
  }

  // Detect workspace root
  let ws = workspace()
    .map_err( | e | format!( "Failed to detect workspace: {}", e ) )?;

  // Get deployment mode
  let mode = detect_deployment_mode();

  // Determine database path based on mode
  let db_path = match mode
  {
    DeploymentMode::Pilot =>
    {
      // Pilot: {workspace}/iron.db (canonical path)
      ws.root().join( "iron.db" )
    }
    DeploymentMode::Development =>
    {
      // Development: {workspace}/data/dev_control.db
      let data_dir = ws.data_dir();
      std::fs::create_dir_all( &data_dir )?;
      data_dir.join( "dev_control.db" )
    }
    DeploymentMode::Production | DeploymentMode::ProductionUnconfirmed =>
    {
      // Production: {workspace}/data/iron_production.db
      // Note: Production should use PostgreSQL in real deployment
      let data_dir = ws.data_dir();
      std::fs::create_dir_all( &data_dir )?;
      data_dir.join( "iron_production.db" )
    }
  };

  // Construct SQLite URL with mode=rwc for database creation
  Ok( format!( "sqlite://{}?mode=rwc", db_path.display() ) )
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
  auth: iron_control_api::routes::auth::AuthState,
  tokens: iron_control_api::routes::tokens::TokenState,
  usage: iron_control_api::routes::usage::UsageState,
  limits: iron_control_api::routes::limits::LimitsState,
  providers: iron_control_api::routes::providers::ProvidersState,
  keys: iron_control_api::routes::keys::KeysState,
  users: iron_control_api::routes::users::UserManagementState,
  agents: sqlx::SqlitePool,
  budget: iron_control_api::routes::budget::BudgetState,
  analytics: iron_control_api::routes::analytics::AnalyticsState,
}

/// Enable auth routes and extractors to access AuthState from combined AppState
///
/// This implementation allows:
/// - Routes with `State<AuthState>` parameter to extract auth sub-state
/// - `AuthenticatedUser` extractor to access JWT secret for token verification
impl axum::extract::FromRef< AppState > for iron_control_api::routes::auth::AuthState
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
impl axum::extract::FromRef< AppState > for iron_control_api::routes::tokens::TokenState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.tokens.clone()
  }
}

/// Enable usage routes to access UsageState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::usage::UsageState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.usage.clone()
  }
}

/// Enable limits routes to access LimitsState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::limits::LimitsState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.limits.clone()
  }
}

/// Enable providers routes to access ProvidersState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::providers::ProvidersState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.providers.clone()
  }
}

/// Enable keys routes to access KeysState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::keys::KeysState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.keys.clone()
  }
}

/// Enable user management routes to access UserManagementState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::users::UserManagementState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.users.clone()
  }
}

/// Enable agent routes to access SqlitePool from combined AppState
impl axum::extract::FromRef< AppState > for sqlx::SqlitePool
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.agents.clone()
  }
}

/// Enable budget routes to access BudgetState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::budget::BudgetState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.budget.clone()
  }
}

/// Enable API token authentication extractor to access ApiTokenState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::token_auth::ApiTokenState
{
  fn from_ref( state: &AppState ) -> Self
  {
    iron_control_api::token_auth::ApiTokenState
    {
      token_storage: state.keys.token_storage.clone(),
    }
  }
}

/// Enable analytics routes to access AnalyticsState from combined AppState
impl axum::extract::FromRef< AppState > for iron_control_api::routes::analytics::AnalyticsState
{
  fn from_ref( state: &AppState ) -> Self
  {
    state.analytics.clone()
  }
}

#[ tokio::main ]
async fn main() -> Result< (), Box< dyn std::error::Error > >
{
  // Load .env file if present (ignore if not found)
  let dotenv_result = dotenvy::dotenv();

  // Initialize tracing
  tracing_subscriber::fmt::init();

  // Log .env loading result (after tracing is initialized)
  match dotenv_result
  {
    Ok( path ) => tracing::debug!( "Loaded .env from: {:?}", path ),
    Err( _ ) => tracing::debug!( "No .env file loaded (not required)" ),
  }

  // Database URL (workspace-relative paths via workspace_tools)
  // Load this BEFORE deployment mode actions (needed for database wiping)
  let database_url = get_database_url()?;
  tracing::info!( "Database: {}", database_url );

  // Extract database file path from SQLite URL for development mode wiping
  let extract_sqlite_path = | url: &str | -> Option< String >
  {
    if let Some( path_with_query ) = url.strip_prefix( "sqlite://" )
    {
      // Remove query parameters
      let path = path_with_query.split( '?' ).next()?;
      Some( path.to_string() )
    }
    else
    {
      None
    }
  };

  // Detect deployment mode before starting server
  let mode = detect_deployment_mode();
  match mode
  {
    DeploymentMode::ProductionUnconfirmed =>
    {
      tracing::warn!( "⚠️  WARNING: Production environment detected but IRON_DEPLOYMENT_MODE not set" );
      tracing::warn!( "⚠️  Set IRON_DEPLOYMENT_MODE=production to confirm production deployment" );
      tracing::warn!( "⚠️  See docs/production_deployment.md for security checklist" );
      tracing::warn!( "" );
      tracing::warn!( "Sleeping 10 seconds to ensure this warning is visible..." );
      std::thread::sleep( std::time::Duration::from_secs( 10 ) );
    }
    DeploymentMode::Production =>
    {
      tracing::info!( "✓ Production mode confirmed (IRON_DEPLOYMENT_MODE=production)" );
    }
    DeploymentMode::Development =>
    {
      tracing::info!( "✓ Development mode (clearing database)" );

      // Extract database path from DATABASE_URL and delete it for clean state
      if let Some( db_path ) = extract_sqlite_path( &database_url )
      {
        if std::path::Path::new( &db_path ).exists()
        {
          if let Err( e ) = std::fs::remove_file( &db_path )
          {
            tracing::warn!( "⚠️  Failed to delete {}: {}", db_path, e );
          }
          else
          {
            tracing::info!( "✓ Cleared {}", db_path );
          }
        }
        else
        {
          tracing::info!( "✓ Database file doesn't exist (will be created fresh)" );
        }
      }
      else
      {
        tracing::warn!( "⚠️  Non-SQLite database detected - database wiping only works with SQLite URLs" );
      }
    }

    DeploymentMode::Pilot =>
    {
      tracing::info!( "✓ Pilot mode (localhost only)" );
    }
  }

  // JWT secret for authentication
  let jwt_secret = std::env::var( "JWT_SECRET" )
    .unwrap_or_else( |_| "dev-secret-change-in-production".to_string() );

  // Protocol 005: Budget Control Protocol secrets
  let ic_token_secret = std::env::var( "IC_TOKEN_SECRET" )
    .unwrap_or_else( |_| "dev-ic-token-secret-change-in-production".to_string() );

  // IP Token encryption key (32 bytes for AES-256-GCM)
  let ip_token_key_hex = std::env::var( "IP_TOKEN_KEY" )
    .unwrap_or_else( |_| "0000000000000000000000000000000000000000000000000000000000000000".to_string() );

  // Fix(production-secret-validation): Block server startup if insecure defaults detected in production
  // Root cause: Server allowed startup with default secrets (dev-secret-change-in-production,
  //             all-zeros encryption keys) in production environments. This creates multiple attack
  //             vectors: JWT tokens forged using known default secret, IC Tokens forged for budget
  //             bypass, IP Tokens decrypted/forged using all-zeros key, session hijacking via
  //             predictable tokens.
  // Pitfall: Never allow fallback secrets in production. Production environments MUST have unique,
  //          cryptographically secure secrets configured. Using defaults is a CRITICAL security
  //          vulnerability - any attacker with knowledge of defaults can forge authentication tokens,
  //          bypass budgets, decrypt session data, and impersonate users.
  // Test coverage: See tests/production_secret_validation_test.rs
  //
  // Validate production secrets before server initialization
  match mode
  {
    DeploymentMode::Production | DeploymentMode::ProductionUnconfirmed =>
    {
      let mut insecure_secrets = Vec::new();

      // Check JWT_SECRET
      if jwt_secret == "dev-secret-change-in-production"
      {
        insecure_secrets.push( "JWT_SECRET" );
      }

      // Check IC_TOKEN_SECRET
      if ic_token_secret == "dev-ic-token-secret-change-in-production"
      {
        insecure_secrets.push( "IC_TOKEN_SECRET" );
      }

      // Check IP_TOKEN_KEY (all zeros)
      if ip_token_key_hex == "0000000000000000000000000000000000000000000000000000000000000000"
      {
        insecure_secrets.push( "IP_TOKEN_KEY" );
      }

      // Check DATABASE_URL (SQLite defaults)
      if database_url.contains( "sqlite://" ) && !database_url.contains( "/var/lib/iron" )
      {
        tracing::warn!( "⚠️  WARNING: Using SQLite in production (DATABASE_URL={})", database_url );
        tracing::warn!( "⚠️  Production deployments SHOULD use PostgreSQL for reliability" );
      }

      // Block startup if any insecure defaults detected
      if !insecure_secrets.is_empty()
      {
        tracing::error!( "❌ CRITICAL SECURITY ERROR: Production deployment with insecure default secrets" );
        tracing::error!( "❌ The following secrets are using INSECURE DEFAULT VALUES:" );
        for secret in &insecure_secrets
        {
          tracing::error!( "❌   - {}", secret );
        }
        tracing::error!( "" );
        tracing::error!( "❌ REFUSING TO START SERVER" );
        tracing::error!( "❌ Generate secure secrets with:" );
        tracing::error!( "❌   JWT_SECRET=$(openssl rand -hex 32)" );
        tracing::error!( "❌   IC_TOKEN_SECRET=$(openssl rand -hex 32)" );
        tracing::error!( "❌   IP_TOKEN_KEY=$(openssl rand -hex 32)" );
        tracing::error!( "" );
        tracing::error!( "❌ See secret/readme.md for complete setup instructions" );
        panic!( "Production deployment blocked: {} insecure default secret(s) detected", insecure_secrets.len() );
      }

      tracing::info!( "✓ Production secret validation passed" );
    }
    _ =>
    {
      // Development/Pilot mode - defaults are acceptable
    }
  }

  // Decode hex string to bytes
  let ip_token_key = hex::decode( &ip_token_key_hex )
    .expect( "LOUD FAILURE: IP_TOKEN_KEY must be a valid 64-character hex string (32 bytes)" );

  if ip_token_key.len() != 32
  {
    panic!( "IP_TOKEN_KEY must be exactly 32 bytes (64 hex characters), got {} bytes", ip_token_key.len() );
  }

  tracing::info!( "Initializing API server..." );
  tracing::info!( "Database: {}", database_url );

  // Initialize route states
  let auth_state = iron_control_api::routes::auth::AuthState::new( jwt_secret, &database_url )
    .await
    .expect( "LOUD FAILURE: Failed to initialize auth state" );

  let token_state = iron_control_api::routes::tokens::TokenState::new( &database_url )
    .await
    .expect( "LOUD FAILURE: Failed to initialize token state" );

  let usage_state = iron_control_api::routes::usage::UsageState::new( &database_url )
    .await
    .expect( "LOUD FAILURE: Failed to initialize usage state" );

  let limits_state = iron_control_api::routes::limits::LimitsState::new( &database_url )
    .await
    .expect( "LOUD FAILURE: Failed to initialize limits state" );

  let providers_state = iron_control_api::routes::providers::ProvidersState::new( &database_url )
    .await
    .expect( "LOUD FAILURE: Failed to initialize providers storage" );

  // Initialize keys state for /api/keys endpoint (requires crypto)
  // Read provider key master key from environment (used for both keys API and budget protocol)
  let provider_key_master_b64 = std::env::var( "IRON_SECRETS_MASTER_KEY" )
    .expect( "LOUD FAILURE: IRON_SECRETS_MASTER_KEY required for provider key encryption" );

  let provider_key_master_bytes = base64::Engine::decode(
    &base64::engine::general_purpose::STANDARD,
    &provider_key_master_b64
  )
    .expect( "LOUD FAILURE: IRON_SECRETS_MASTER_KEY must be valid base64" );

  let crypto_service = std::sync::Arc::new(
    iron_secrets::crypto::CryptoService::new( &provider_key_master_bytes )
      .expect( "LOUD FAILURE: Failed to create crypto service" )
  );

  // Rate limiter for /api/keys endpoint: 10 requests per minute per user/project
  let key_rate_limiter = iron_token_manager::rate_limiter::RateLimiter::new(
    10,
    std::time::Duration::from_secs( 60 ),
  );

  // Clone crypto_service for BudgetState (Feature 014: Agent Provider Key)
  let crypto_service_for_budget = crypto_service.clone();

  let keys_state = iron_control_api::routes::keys::KeysState
  {
    token_storage: token_state.storage.clone(),
    provider_storage: providers_state.storage.clone(),
    crypto: crypto_service,
    rate_limiter: key_rate_limiter,
  };

  // Initialize user management state
  let permission_checker = std::sync::Arc::new( iron_control_api::rbac::PermissionChecker::new() );
  let user_management_state = iron_control_api::routes::users::UserManagementState::new(
    auth_state.db_pool.clone(),
    permission_checker,
  );

  // Initialize analytics state (Protocol 012)
  // Uses same IC_TOKEN_SECRET as budget module for consistent agent authentication
  let analytics_state = iron_control_api::routes::analytics::AnalyticsState::new(
    &database_url,
    ic_token_secret.clone(),
  )
    .await
    .expect( "LOUD FAILURE: Failed to initialize analytics state" );

  // Get database pool for agents (before moving token_state)
  let agents_pool = token_state.storage.pool().clone();

  // Seed database with test data if empty (development convenience)
  let user_count: i64 = sqlx::query_scalar( "SELECT COUNT(*) FROM users" )
    .fetch_one( &agents_pool )
    .await
    .unwrap_or( 0 );

  if user_count == 0 && iron_token_manager::seed::is_demo_seed_enabled()
  {
    tracing::info!( "Seeding database with demo data (ENABLE_DEMO_SEED=true)..." );
    iron_token_manager::seed::seed_all( &agents_pool )
      .await
      .expect( "LOUD FAILURE: Failed to seed database" );
    tracing::info!( "✓ Database seeded with demo accounts (see docs/demo_credentials.md)" );
  }

  // Initialize budget state (Protocol 005: Budget Control Protocol)
  // crypto_service_for_budget enables Feature 014: Agent Provider Key retrieval
  let budget_state = iron_control_api::routes::budget::BudgetState::new(
    ic_token_secret,
    &ip_token_key,
    &provider_key_master_bytes,
    auth_state.jwt_secret.clone(),
    &database_url,
    Some( crypto_service_for_budget ),
  )
  .await
  .expect( "LOUD FAILURE: Failed to initialize budget state" );

  // Create combined app state
  let app_state = AppState
  {
    auth: auth_state,
    tokens: token_state,
    usage: usage_state,
    limits: limits_state,
    providers: providers_state,
    keys: keys_state,
    users: user_management_state,
    agents: agents_pool,
    budget: budget_state,
    analytics: analytics_state,
  };

  // Fix(ironcage-migration): Replace hardcoded CORS with ALLOWED_ORIGINS env var
  // Root cause: Hardcoded origins prevented multi-domain production deployment
  // Pitfall: Never hardcode deployment-specific config (origins, ports, URLs)
  let allowed_origins_str = std::env::var( "ALLOWED_ORIGINS" )
    .expect( "ALLOWED_ORIGINS environment variable required (comma-separated URLs)" );

  let allowed_origins: Vec< axum::http::HeaderValue > = allowed_origins_str
    .split( ',' )
    .map( |origin| {
      origin.trim().parse::<axum::http::HeaderValue>()
        .unwrap_or_else( |_| panic!( "Invalid origin in ALLOWED_ORIGINS: {}", origin ) )
    } )
    .collect();

  tracing::info!( "✅ Configured CORS for {} origins", allowed_origins.len() );
  for origin in &allowed_origins
  {
    tracing::info!( "   - {}", origin.to_str().unwrap() );
  }

  // Build router with all endpoints
  let app = Router::new()
    // Health check (FR-2: Health endpoint at /api/health)
    .route( "/api/health", get( iron_control_api::routes::health::health_check ) )

    // Version endpoint (API version discovery)
    .route( "/api/v1/version", get( iron_control_api::routes::version::get_version ) )

    // Authentication endpoints
    .route( "/api/v1/auth/login", post( iron_control_api::routes::auth::login ) )
    .route( "/api/v1/auth/refresh", post( iron_control_api::routes::auth::refresh ) )
    .route( "/api/v1/auth/logout", post( iron_control_api::routes::auth::logout ) )
    .route( "/api/v1/auth/validate", post( iron_control_api::routes::auth::validate ) )

    // User management endpoints
    .route( "/api/v1/users", post( iron_control_api::routes::users::create_user ) )
    .route( "/api/v1/users", get( iron_control_api::routes::users::list_users ) )
    .route( "/api/v1/users/:id", get( iron_control_api::routes::users::get_user ) )
    .route( "/api/v1/users/:id", delete( iron_control_api::routes::users::delete_user ) )
    .route( "/api/v1/users/:id/suspend", axum::routing::put( iron_control_api::routes::users::suspend_user ) )
    .route( "/api/v1/users/:id/activate", axum::routing::put( iron_control_api::routes::users::activate_user ) )
    .route( "/api/v1/users/:id/role", axum::routing::put( iron_control_api::routes::users::change_user_role ) )
    .route( "/api/v1/users/:id/reset-password", post( iron_control_api::routes::users::reset_password ) )

    // Token management endpoints
    .route( "/api/v1/api-tokens", post( iron_control_api::routes::tokens::create_token ) )
    .route( "/api/v1/api-tokens/validate", post( iron_control_api::routes::tokens::validate_token ) )
    .route( "/api/v1/api-tokens", get( iron_control_api::routes::tokens::list_tokens ) )
    .route( "/api/v1/api-tokens/:id", get( iron_control_api::routes::tokens::get_token ) )
    .route( "/api/v1/api-tokens/:id/rotate", post( iron_control_api::routes::tokens::rotate_token ) )
    .route( "/api/v1/api-tokens/:id", delete( iron_control_api::routes::tokens::revoke_token ) )
    .route( "/api/v1/api-tokens/:id", put( iron_control_api::routes::tokens::update_token ) )

    // Usage analytics endpoints
    .route( "/api/v1/usage/aggregate", get( iron_control_api::routes::usage::get_aggregate_usage ) )
    .route( "/api/v1/usage/by-project/:project_id", get( iron_control_api::routes::usage::get_usage_by_project ) )
    .route( "/api/v1/usage/by-provider/:provider", get( iron_control_api::routes::usage::get_usage_by_provider ) )

    // Limits management endpoints
    .route( "/api/v1/limits", get( iron_control_api::routes::limits::list_limits ) )
    .route( "/api/v1/limits", post( iron_control_api::routes::limits::create_limit ) )
    .route( "/api/v1/limits/:id", get( iron_control_api::routes::limits::get_limit ) )
    .route( "/api/v1/limits/:id", axum::routing::put( iron_control_api::routes::limits::update_limit ) )
    .route( "/api/v1/limits/:id", axum::routing::delete( iron_control_api::routes::limits::delete_limit ) )

    // Provider key management endpoints
    .route( "/api/v1/providers", post( iron_control_api::routes::providers::create_provider_key ) )
    .route( "/api/v1/providers", get( iron_control_api::routes::providers::list_provider_keys ) )
    .route( "/api/v1/providers/:id", get( iron_control_api::routes::providers::get_provider_key ) )
    .route( "/api/v1/providers/:id", axum::routing::put( iron_control_api::routes::providers::update_provider_key ) )
    .route( "/api/v1/providers/:id", delete( iron_control_api::routes::providers::delete_provider_key ) )
    .route( "/api/v1/projects/:project_id/provider", post( iron_control_api::routes::providers::assign_provider_to_project ) )
    .route( "/api/v1/projects/:project_id/provider", delete( iron_control_api::routes::providers::unassign_provider_from_project ) )

    // Key fetch endpoint (API token authentication)
    .route( "/api/v1/keys", get( iron_control_api::routes::keys::get_key ) )

    // Agent management endpoints
    .route( "/api/v1/agents", get( iron_control_api::routes::agents::list_agents ) )
    .route( "/api/v1/agents", post( iron_control_api::routes::agents::create_agent ) )
    // Agent Provider Key endpoint (Feature 014) - must be before :id routes
    .route( "/api/v1/agents/provider-key", post( iron_control_api::routes::agent_provider_key::get_provider_key ) )
    .route( "/api/v1/agents/:id", get( iron_control_api::routes::agents::get_agent ) )
    .route( "/api/v1/agents/:id", axum::routing::put( iron_control_api::routes::agents::update_agent ) )
    .route( "/api/v1/agents/:id", delete( iron_control_api::routes::agents::delete_agent ) )
    .route( "/api/v1/agents/:id/tokens", get( iron_control_api::routes::agents::get_agent_tokens ) )

    // Budget Control Protocol endpoints (Protocol 005)
    .route( "/api/v1/budget/handshake", post( iron_control_api::routes::budget::handshake ) )
    .route( "/api/v1/budget/report", post( iron_control_api::routes::budget::report_usage ) )
    .route( "/api/v1/budget/refresh", post( iron_control_api::routes::budget::refresh_budget ) )
    .route( "/api/v1/budget/return", post( iron_control_api::routes::budget::return_budget ) )

    // Budget Request Workflow endpoints (Protocol 012)
    .route( "/api/v1/budget/requests", post( iron_control_api::routes::budget::create_budget_request ) )
    .route( "/api/v1/budget/requests/:id", get( iron_control_api::routes::budget::get_budget_request ) )
    .route( "/api/v1/budget/requests", get( iron_control_api::routes::budget::list_budget_requests ) )
    .route( "/api/v1/budget/requests/:id/approve", axum::routing::patch( iron_control_api::routes::budget::approve_budget_request ) )
    .route( "/api/v1/budget/requests/:id/reject", axum::routing::patch( iron_control_api::routes::budget::reject_budget_request ) )

    // Analytics endpoints (Protocol 012)
    .route( "/api/v1/analytics/events", post( iron_control_api::routes::analytics::post_event ) )
    .route( "/api/v1/analytics/spending/total", get( iron_control_api::routes::analytics::get_spending_total ) )
    .route( "/api/v1/analytics/spending/by-agent", get( iron_control_api::routes::analytics::get_spending_by_agent ) )
    .route( "/api/v1/analytics/spending/by-provider", get( iron_control_api::routes::analytics::get_spending_by_provider ) )
    .route( "/api/v1/analytics/spending/avg-per-request", get( iron_control_api::routes::analytics::get_spending_avg ) )
    .route( "/api/v1/analytics/budget/status", get( iron_control_api::routes::analytics::get_budget_status ) )
    .route( "/api/v1/analytics/usage/requests", get( iron_control_api::routes::analytics::get_usage_requests ) )
    .route( "/api/v1/analytics/usage/tokens/by-agent", get( iron_control_api::routes::analytics::get_usage_tokens ) )
    .route( "/api/v1/analytics/usage/models", get( iron_control_api::routes::analytics::get_usage_models ) )

    // Apply combined state to all routes
    .with_state( app_state )

    // CORS middleware (configured from ALLOWED_ORIGINS environment variable)
    // Allow methods: GET, POST, PUT, DELETE, PATCH (all REST operations)
    // Allow headers: Content-Type (JSON requests), Authorization (Bearer tokens)
    .layer(
      CorsLayer::new()
        .allow_origin( allowed_origins )
        .allow_methods( [ Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::PATCH ] )
        .allow_headers( [ header::CONTENT_TYPE, header::AUTHORIZATION ] )
    );

  // Fix(ironcage-migration): Replace hardcoded port with SERVER_PORT env var
  // Root cause: Hardcoded port prevented multi-environment deployment
  // Pitfall: Never hardcode deployment-specific config (ports, hosts, URLs)
  let server_port_str = std::env::var( "SERVER_PORT" )
    .expect( "SERVER_PORT environment variable required (port number 1-65535)" );

  let server_port: u16 = server_port_str.parse::< u16 >()
    .unwrap_or_else( |_| panic!( "Invalid SERVER_PORT: {} (must be 1-65535)", server_port_str ) );

  // Server address (0.0.0.0 for Docker container networking)
  let addr = SocketAddr::from( ( [0, 0, 0, 0], server_port ) );

  tracing::info!( "✅ API server configured on port {}", server_port );
  tracing::info!( "API server listening on http://{}", addr );
  tracing::info!( "Endpoints:" );
  tracing::info!( "  GET  /api/health" );
  tracing::info!( "  POST /api/auth/login" );
  tracing::info!( "  POST /api/auth/refresh" );
  tracing::info!( "  POST /api/auth/logout" );
  tracing::info!( "  POST /api/users" );
  tracing::info!( "  GET  /api/users" );
  tracing::info!( "  GET  /api/v1/api-tokens" );
  tracing::info!( "  POST /api/v1/api-tokens" );
  tracing::info!( "  GET  /api/v1/api-tokens/:id" );
  tracing::info!( "  POST /api/v1/api-tokens/:id/rotate" );
  tracing::info!( "  DELETE /api/v1/api-tokens/:id" );
  tracing::info!( "  GET  /api/usage/aggregate" );
  tracing::info!( "  GET  /api/usage/by-project/:project_id" );
  tracing::info!( "  GET  /api/usage/by-provider/:provider" );
  tracing::info!( "  GET  /api/limits" );
  tracing::info!( "  POST /api/limits" );
  tracing::info!( "  GET  /api/limits/:id" );
  tracing::info!( "  PUT  /api/limits/:id" );
  tracing::info!( "  DELETE /api/limits/:id" );
  tracing::info!( "  POST /api/providers" );
  tracing::info!( "  GET  /api/providers" );
  tracing::info!( "  GET  /api/providers/:id" );
  tracing::info!( "  PUT  /api/providers/:id" );
  tracing::info!( "  DELETE /api/providers/:id" );
  tracing::info!( "  POST /api/projects/:project_id/provider" );
  tracing::info!( "  DELETE /api/projects/:project_id/provider" );
  tracing::info!( "  GET  /api/keys" );
  tracing::info!( "  POST /api/budget/handshake" );
  tracing::info!( "  POST /api/budget/report" );
  tracing::info!( "  POST /api/budget/refresh" );
  tracing::info!( "  POST /api/v1/budget/requests" );
  tracing::info!( "  GET  /api/v1/budget/requests" );
  tracing::info!( "  GET  /api/v1/budget/requests/:id" );
  tracing::info!( "  PATCH /api/v1/budget/requests/:id/approve" );
  tracing::info!( "  PATCH /api/v1/budget/requests/:id/reject" );
  tracing::info!( "  POST /api/v1/analytics/events" );
  tracing::info!( "  GET  /api/v1/analytics/spending/total" );
  tracing::info!( "  GET  /api/v1/analytics/spending/by-agent" );
  tracing::info!( "  GET  /api/v1/analytics/spending/by-provider" );
  tracing::info!( "  GET  /api/v1/analytics/spending/avg-per-request" );
  tracing::info!( "  GET  /api/v1/analytics/budget/status" );
  tracing::info!( "  GET  /api/v1/analytics/usage/requests" );
  tracing::info!( "  GET  /api/v1/analytics/usage/tokens/by-agent" );
  tracing::info!( "  GET  /api/v1/analytics/usage/models" );

  // Fix(login-connect-info): Enable ConnectInfo extraction for per-IP rate limiting
  // Root cause: Login handler uses ConnectInfo<SocketAddr> for per-IP rate limiting
  //             (Fix issue-GAP-006), but axum::serve() doesnt provide ConnectInfo by
  //             default. Must explicitly opt-in via into_make_service_with_connect_info.
  // Pitfall: Never use axum::serve(listener, app) when handlers need ConnectInfo.
  //          Always use app.into_make_service_with_connect_info::<SocketAddr>() to
  //          make client addresses available. Without this, requests fail with 500
  //          "Missing request extension: ConnectInfo<SocketAddr>".
  //
  // Start server with ConnectInfo support
  let listener = tokio::net::TcpListener::bind( addr ).await?;
  axum::serve(
    listener,
    app.into_make_service_with_connect_info::<SocketAddr>()
  ).await?;

  Ok( () )
}
