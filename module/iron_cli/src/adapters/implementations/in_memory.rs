//! InMemoryAdapter - Fast, predictable implementation for tests
//!
//! ## Purpose
//!
//! Provides REAL implementations of service traits using in-memory storage.
//! No mocking - this is a genuine alternative implementation used in tests.
//!
//! ## Architecture
//!
//! ```text
//! InMemoryAdapter
//!   ├── users: HashMap<username, password>
//!   ├── tokens: HashMap<user, Tokens>
//!   └── token_store: HashMap<token_id, Token>
//! ```
//!
//! ## Thread Safety
//!
//! Uses RwLock for concurrent access (same as production would use).
//!
//! ## Obsolescence Guard Pattern
//!
//! This module demonstrates the obsolescence proof pattern for enforcing
//! test-only code. The pattern combines three mechanisms:
//!
//! **1. compile_error! Guard** (below):
//! ```rust,ignore
//! #[cfg(not(any(test, feature = "test-adapter")))]
//! compile_error!("InMemoryAdapter is only for tests...");
//! ```
//! Prevents production code from compiling if it tries to use InMemoryAdapter.
//!
//! **2. Cfg Guards on Module Export** (in mod.rs):
//! ```rust,ignore
//! #[cfg(any(test, feature = "test-adapter"))]
//! pub use in_memory::InMemoryAdapter;
//! ```
//! Makes InMemoryAdapter unavailable to production code at module level.
//!
//! **3. Feature Flag for Integration Tests**:
//! Integration tests in tests/ directory compile as separate crates and don't
//! get automatic cfg(test). They need feature = "test-adapter" to access
//! InMemoryAdapter. Production builds never enable this feature.
//!
//! **Why This Pattern Works:**
//! - compile_error! provides hard enforcement (compile failure)
//! - Cfg guards make the type unavailable in production
//! - Feature flag solves integration test problem without weakening enforcement
//! - Combined, they make rollback/workarounds impossible without explicit action
//!
//! **Migration Context** (2025-12-04):
//! - Replaced by HttpAdapter for production use
//! - 272 tests continue passing using InMemoryAdapter
//! - Production code MUST use HttpAdapter (enforced at compile time)

// OBSOLESCENCE GUARD: InMemoryAdapter is test-only
// Production code must use HttpAdapter for real API integration
#[ cfg( not( any( test, feature = "test-adapter" ) ) ) ]
compile_error!(
  "InMemoryAdapter is only for tests. Use HttpAdapter for production. \
   See: module/iron_cli/src/adapters/implementations/http.rs"
);

use crate::adapters::{ ServiceError, Tokens, Token, UsageRecord, Limit, Trace, HealthStatus };
use crate::adapters::services::{ AuthService, TokenService, StorageService, UsageService, LimitsService, TracesService, HealthService };
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{ Arc, RwLock };

/// In-memory adapter for testing
pub struct InMemoryAdapter
{
  users: Arc<RwLock<HashMap<String, String>>>, // username -> password
  tokens: Arc<RwLock<Option<Tokens>>>,         // current user's tokens
  token_store: Arc<RwLock<HashMap<String, Token>>>, // token_id -> Token
  usage_store: Arc<RwLock<Vec<UsageRecord>>>,  // usage records
  limits_store: Arc<RwLock<HashMap<String, Limit>>>, // limit_id -> Limit
  traces_store: Arc<RwLock<HashMap<String, Trace>>>, // trace_id -> Trace
  failure_mode: Arc<RwLock<Option<String>>>,   // simulate failures
  expired: Arc<RwLock<bool>>,                  // simulate token expiration
}

impl Default for InMemoryAdapter
{
  fn default() -> Self
  {
    Self::new()
  }
}

impl InMemoryAdapter
{
  /// Create new empty adapter
  pub fn new() -> Self
  {
    Self {
      users: Arc::new( RwLock::new( HashMap::new() ) ),
      tokens: Arc::new( RwLock::new( None ) ),
      token_store: Arc::new( RwLock::new( HashMap::new() ) ),
      usage_store: Arc::new( RwLock::new( Vec::new() ) ),
      limits_store: Arc::new( RwLock::new( HashMap::new() ) ),
      traces_store: Arc::new( RwLock::new( HashMap::new() ) ),
      failure_mode: Arc::new( RwLock::new( None ) ),
      expired: Arc::new( RwLock::new( false ) ),
    }
  }

  /// Test helper: Pre-seed a user (for tests)
  pub fn seed_user(&self, username: &str, password: &str)
  {
    let mut users = self.users.write().unwrap();
    users.insert( username.to_string(), password.to_string() );
  }

  /// Test helper: Simulate failure modes
  pub fn set_failure_mode(&self, mode: &str)
  {
    let mut failure = self.failure_mode.write().unwrap();
    *failure = Some( mode.to_string() );
  }

  /// Test helper: Check if tokens exist
  pub fn has_tokens(&self) -> bool
  {
    let tokens = self.tokens.read().unwrap();
    tokens.is_some()
  }

  /// Test helper: Get current tokens
  pub fn get_tokens(&self) -> Option<Tokens>
  {
    let tokens = self.tokens.read().unwrap();
    tokens.clone()
  }

  /// Test helper: Expire tokens
  pub fn expire_tokens(&self)
  {
    let mut expired = self.expired.write().unwrap();
    *expired = true;
  }

  /// Check for simulated failures
  fn check_failure(&self) -> Result<(), ServiceError>
  {
    let failure = self.failure_mode.read().unwrap();
    match failure.as_ref().map( |s| s.as_str() )
    {
      Some( "network_error" ) => Err( ServiceError::NetworkError( "Simulated network error".to_string() ) ),
      Some( "database_error" ) => Err( ServiceError::DatabaseError( "Simulated database error".to_string() ) ),
      Some( "storage_error" ) => Err( ServiceError::StorageError( "Simulated storage error".to_string() ) ),
      _ => Ok( () ),
    }
  }
}

#[ async_trait ]
impl AuthService for InMemoryAdapter
{
  async fn login(&self, username: &str, password: &str) -> Result<Tokens, ServiceError>
  {
    self.check_failure()?;

    let users = self.users.read().unwrap();
    let stored_password = users.get( username ).ok_or( ServiceError::Unauthorized )?;

    if stored_password != password
    {
      return Err( ServiceError::Unauthorized );
    }

    // Generate tokens
    let tokens = Tokens {
      access_token: format!( "access_token_{}", username ),
      refresh_token: format!( "refresh_token_{}", username ),
    };

    // Store tokens
    let mut token_storage = self.tokens.write().unwrap();
    *token_storage = Some( tokens.clone() );

    Ok( tokens )
  }

  async fn refresh(&self, refresh_token: &str) -> Result<Tokens, ServiceError>
  {
    self.check_failure()?;

    // Check if expired
    let expired = self.expired.read().unwrap();
    if *expired
    {
      return Err( ServiceError::Unauthorized );
    }

    // Verify refresh token exists and matches
    let current_tokens = self.tokens.read().unwrap();
    let stored_tokens = current_tokens.as_ref().ok_or( ServiceError::NotFound )?;

    if stored_tokens.refresh_token != refresh_token
    {
      return Err( ServiceError::Unauthorized );
    }

    // Generate new tokens (extract username from old token)
    let username = refresh_token.strip_prefix( "refresh_token_" )
      .ok_or( ServiceError::Unauthorized )?;

    let new_tokens = Tokens {
      access_token: format!( "access_token_new_{}", username ),
      refresh_token: format!( "refresh_token_new_{}", username ),
    };

    // Store new tokens
    drop( current_tokens ); // Release read lock
    let mut token_storage = self.tokens.write().unwrap();
    *token_storage = Some( new_tokens.clone() );

    Ok( new_tokens )
  }

  async fn logout(&self, _access_token: &str) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    // Clear tokens
    let mut tokens = self.tokens.write().unwrap();
    *tokens = None;

    Ok( () )
  }
}

#[ async_trait ]
impl TokenService for InMemoryAdapter
{
  async fn generate(&self, name: &str, scope: &str, ttl: Option<i64>) -> Result<Token, ServiceError>
  {
    self.check_failure()?;

    let token_id = format!( "tok_{}", name.replace( " ", "_" ) );

    let token = Token {
      id: token_id.clone(),
      name: name.to_string(),
      scope: scope.to_string(),
      created_at: "2025-12-03T00:00:00Z".to_string(),
      expires_at: ttl.map( |_| "2025-12-04T00:00:00Z".to_string() ),
    };

    let mut store = self.token_store.write().unwrap();
    store.insert( token_id, token.clone() );

    Ok( token )
  }

  async fn list(&self, _filter: Option<&str>) -> Result<Vec<Token>, ServiceError>
  {
    self.check_failure()?;

    let store = self.token_store.read().unwrap();
    Ok( store.values().cloned().collect() )
  }

  async fn get(&self, token_id: &str) -> Result<Token, ServiceError>
  {
    self.check_failure()?;

    let store = self.token_store.read().unwrap();
    store.get( token_id )
      .cloned()
      .ok_or( ServiceError::NotFound )
  }

  async fn rotate(&self, token_id: &str, new_ttl: Option<i64>) -> Result<Token, ServiceError>
  {
    self.check_failure()?;

    let mut store = self.token_store.write().unwrap();
    let token = store.get_mut( token_id )
      .ok_or( ServiceError::NotFound )?;

    // Update token (preserve scope, update value)
    token.expires_at = new_ttl.map( |_| "2025-12-05T00:00:00Z".to_string() );

    Ok( token.clone() )
  }

  async fn revoke(&self, token_id: &str, _reason: Option<&str>) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let mut store = self.token_store.write().unwrap();
    store.remove( token_id ).ok_or( ServiceError::NotFound )?;

    Ok( () )
  }
}

#[ async_trait ]
impl StorageService for InMemoryAdapter
{
  async fn save_tokens(&self, tokens: &Tokens) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let mut storage = self.tokens.write().unwrap();
    *storage = Some( tokens.clone() );

    Ok( () )
  }

  async fn load_tokens(&self) -> Result<Option<Tokens>, ServiceError>
  {
    self.check_failure()?;

    let storage = self.tokens.read().unwrap();
    Ok( storage.clone() )
  }

  async fn clear_tokens(&self) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let mut storage = self.tokens.write().unwrap();
    *storage = None;

    Ok( () )
  }
}

// Implement traits for Arc<InMemoryAdapter> (delegate to inner implementation)
#[ async_trait ]
impl AuthService for Arc<InMemoryAdapter>
{
  async fn login(&self, username: &str, password: &str) -> Result<Tokens, ServiceError>
  {
    self.as_ref().login( username, password ).await
  }

  async fn refresh(&self, refresh_token: &str) -> Result<Tokens, ServiceError>
  {
    self.as_ref().refresh( refresh_token ).await
  }

  async fn logout(&self, access_token: &str) -> Result<(), ServiceError>
  {
    self.as_ref().logout( access_token ).await
  }
}

#[ async_trait ]
impl TokenService for Arc<InMemoryAdapter>
{
  async fn generate(&self, name: &str, scope: &str, ttl: Option<i64>) -> Result<Token, ServiceError>
  {
    self.as_ref().generate( name, scope, ttl ).await
  }

  async fn list(&self, filter: Option<&str>) -> Result<Vec<Token>, ServiceError>
  {
    self.as_ref().list( filter ).await
  }

  async fn get(&self, token_id: &str) -> Result<Token, ServiceError>
  {
    self.as_ref().get( token_id ).await
  }

  async fn rotate(&self, token_id: &str, new_ttl: Option<i64>) -> Result<Token, ServiceError>
  {
    self.as_ref().rotate( token_id, new_ttl ).await
  }

  async fn revoke(&self, token_id: &str, reason: Option<&str>) -> Result<(), ServiceError>
  {
    self.as_ref().revoke( token_id, reason ).await
  }
}

#[ async_trait ]
impl UsageService for InMemoryAdapter
{
  async fn record_usage(&self, project_id: &str, provider: &str, tokens: u64, cost: u64) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let record = UsageRecord {
      project_id: project_id.to_string(),
      provider: provider.to_string(),
      tokens_used: tokens,
      cost,
      timestamp: "2025-01-01T00:00:00Z".to_string(), // Simple timestamp for testing
    };

    let mut usage = self.usage_store.write().unwrap();
    usage.push( record );

    Ok( () )
  }

  async fn get_usage(&self, _start_date: Option<&str>, _end_date: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.check_failure()?;

    let usage = self.usage_store.read().unwrap();
    Ok( usage.clone() )
  }

  async fn get_usage_by_project(&self, project_id: &str, _start_date: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.check_failure()?;

    let usage = self.usage_store.read().unwrap();
    let filtered: Vec<UsageRecord> = usage
      .iter()
      .filter( |r| r.project_id == project_id )
      .cloned()
      .collect();

    Ok( filtered )
  }

  async fn get_usage_by_provider(&self, provider: &str, _aggregation: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.check_failure()?;

    let usage = self.usage_store.read().unwrap();
    let filtered: Vec<UsageRecord> = usage
      .iter()
      .filter( |r| r.provider == provider )
      .cloned()
      .collect();

    Ok( filtered )
  }

  async fn export_usage(&self, _output_path: &str, _format: &str) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    // For in-memory adapter, we just simulate the export
    // Real implementation would write to file
    Ok( () )
  }
}

#[ async_trait ]
impl UsageService for Arc<InMemoryAdapter>
{
  async fn record_usage(&self, project_id: &str, provider: &str, tokens: u64, cost: u64) -> Result<(), ServiceError>
  {
    self.as_ref().record_usage( project_id, provider, tokens, cost ).await
  }

  async fn get_usage(&self, start_date: Option<&str>, end_date: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.as_ref().get_usage( start_date, end_date ).await
  }

  async fn get_usage_by_project(&self, project_id: &str, start_date: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.as_ref().get_usage_by_project( project_id, start_date ).await
  }

  async fn get_usage_by_provider(&self, provider: &str, aggregation: Option<&str>) -> Result<Vec<UsageRecord>, ServiceError>
  {
    self.as_ref().get_usage_by_provider( provider, aggregation ).await
  }

  async fn export_usage(&self, output_path: &str, format: &str) -> Result<(), ServiceError>
  {
    self.as_ref().export_usage( output_path, format ).await
  }
}

#[ async_trait ]
impl LimitsService for InMemoryAdapter
{
  async fn create_limit(&self, resource_type: &str, limit_value: u64) -> Result<Limit, ServiceError>
  {
    self.check_failure()?;

    let limit_id = format!( "lim_{}", resource_type );
    let timestamp = "2025-01-01T00:00:00Z".to_string();

    let limit = Limit {
      id: limit_id.clone(),
      resource_type: resource_type.to_string(),
      limit_value,
      created_at: timestamp.clone(),
      updated_at: timestamp,
    };

    let mut limits = self.limits_store.write().unwrap();
    limits.insert( limit_id, limit.clone() );

    Ok( limit )
  }

  async fn list_limits(&self) -> Result<Vec<Limit>, ServiceError>
  {
    self.check_failure()?;

    let limits = self.limits_store.read().unwrap();
    Ok( limits.values().cloned().collect() )
  }

  async fn get_limit(&self, limit_id: &str) -> Result<Limit, ServiceError>
  {
    self.check_failure()?;

    let limits = self.limits_store.read().unwrap();
    limits.get( limit_id )
      .cloned()
      .ok_or( ServiceError::NotFound )
  }

  async fn update_limit(&self, limit_id: &str, new_value: u64) -> Result<Limit, ServiceError>
  {
    self.check_failure()?;

    let mut limits = self.limits_store.write().unwrap();
    let limit = limits.get_mut( limit_id )
      .ok_or( ServiceError::NotFound )?;

    limit.limit_value = new_value;
    limit.updated_at = "2025-01-01T00:00:00Z".to_string();

    Ok( limit.clone() )
  }

  async fn delete_limit(&self, limit_id: &str) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let mut limits = self.limits_store.write().unwrap();
    limits.remove( limit_id )
      .ok_or( ServiceError::NotFound )?;

    Ok( () )
  }
}

#[ async_trait ]
impl LimitsService for Arc<InMemoryAdapter>
{
  async fn create_limit(&self, resource_type: &str, limit_value: u64) -> Result<Limit, ServiceError>
  {
    self.as_ref().create_limit( resource_type, limit_value ).await
  }

  async fn list_limits(&self) -> Result<Vec<Limit>, ServiceError>
  {
    self.as_ref().list_limits().await
  }

  async fn get_limit(&self, limit_id: &str) -> Result<Limit, ServiceError>
  {
    self.as_ref().get_limit( limit_id ).await
  }

  async fn update_limit(&self, limit_id: &str, new_value: u64) -> Result<Limit, ServiceError>
  {
    self.as_ref().update_limit( limit_id, new_value ).await
  }

  async fn delete_limit(&self, limit_id: &str) -> Result<(), ServiceError>
  {
    self.as_ref().delete_limit( limit_id ).await
  }
}

#[ async_trait ]
impl TracesService for InMemoryAdapter
{
  async fn record_trace(&self, trace_id: &str, request: &str, duration_ms: u64) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    let trace = Trace {
      id: trace_id.to_string(),
      request: request.to_string(),
      duration_ms,
      timestamp: "2025-01-01T00:00:00Z".to_string(),
    };

    let mut traces = self.traces_store.write().unwrap();
    traces.insert( trace_id.to_string(), trace );

    Ok( () )
  }

  async fn list_traces(&self, filter: Option<&str>, _limit: Option<u32>) -> Result<Vec<Trace>, ServiceError>
  {
    self.check_failure()?;

    let traces = self.traces_store.read().unwrap();
    let mut result: Vec<Trace> = traces.values().cloned().collect();

    if let Some(f) = filter
    {
      result.retain( |t| t.request.contains( f ) );
    }

    Ok( result )
  }

  async fn get_trace(&self, trace_id: &str) -> Result<Trace, ServiceError>
  {
    self.check_failure()?;

    let traces = self.traces_store.read().unwrap();
    traces.get( trace_id )
      .cloned()
      .ok_or( ServiceError::NotFound )
  }

  async fn export_traces(&self, _output_path: &str, _format: &str) -> Result<(), ServiceError>
  {
    self.check_failure()?;

    // Simulate export (in-memory adapter doesn't write files)
    Ok( () )
  }
}

#[ async_trait ]
impl TracesService for Arc<InMemoryAdapter>
{
  async fn record_trace(&self, trace_id: &str, request: &str, duration_ms: u64) -> Result<(), ServiceError>
  {
    self.as_ref().record_trace( trace_id, request, duration_ms ).await
  }

  async fn list_traces(&self, filter: Option<&str>, limit: Option<u32>) -> Result<Vec<Trace>, ServiceError>
  {
    self.as_ref().list_traces( filter, limit ).await
  }

  async fn get_trace(&self, trace_id: &str) -> Result<Trace, ServiceError>
  {
    self.as_ref().get_trace( trace_id ).await
  }

  async fn export_traces(&self, output_path: &str, format: &str) -> Result<(), ServiceError>
  {
    self.as_ref().export_traces( output_path, format ).await
  }
}

#[ async_trait ]
impl HealthService for InMemoryAdapter
{
  async fn get_health(&self) -> Result<HealthStatus, ServiceError>
  {
    self.check_failure()?;

    Ok( HealthStatus {
      status: "ok".to_string(),
      uptime_seconds: 0,
    })
  }

  async fn get_version(&self) -> Result<String, ServiceError>
  {
    self.check_failure()?;
    Ok( env!( "CARGO_PKG_VERSION" ).to_string() )
  }
}

#[ async_trait ]
impl HealthService for Arc<InMemoryAdapter>
{
  async fn get_health(&self) -> Result<HealthStatus, ServiceError>
  {
    self.as_ref().get_health().await
  }

  async fn get_version(&self) -> Result<String, ServiceError>
  {
    self.as_ref().get_version().await
  }
}

#[ async_trait ]
impl StorageService for Arc<InMemoryAdapter>
{
  async fn save_tokens(&self, tokens: &Tokens) -> Result<(), ServiceError>
  {
    self.as_ref().save_tokens( tokens ).await
  }

  async fn load_tokens(&self) -> Result<Option<Tokens>, ServiceError>
  {
    self.as_ref().load_tokens().await
  }

  async fn clear_tokens(&self) -> Result<(), ServiceError>
  {
    self.as_ref().clear_tokens().await
  }
}
