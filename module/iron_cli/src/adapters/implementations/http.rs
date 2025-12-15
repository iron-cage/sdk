//! HTTP adapter for production API integration
//!
//! This adapter connects to the Iron API server over HTTP, providing
//! full production functionality for token management, usage tracking,
//! and authentication.
//!
//! ## Purpose
//!
//! Replaces InMemoryAdapter for production use. InMemoryAdapter is now test-only
//! (enforced via compile_error! guard). HttpAdapter provides real API integration
//! for production deployments.
//!
//! ## Architecture
//!
//! - **HTTP Client**: Uses reqwest for async HTTP communication
//! - **Authentication**: JWT tokens stored in Arc<RwLock<>> for thread-safe access
//! - **Token Storage**: Persists auth tokens to `~/.iron/tokens.json`
//! - **Error Mapping**: HTTP status codes → ServiceError variants
//!
//! ## Design Decisions
//!
//! **Why reqwest?**
//! - Mature async HTTP client with good ecosystem support
//! - Built-in JSON serialization/deserialization
//! - Works well with tokio runtime
//!
//! **Why local token storage?**
//! - Simple persistence without external dependencies
//! - User can inspect/debug tokens manually
//! - Alternative considered: keyring (rejected for simplicity)
//!
//! **Why Arc<RwLock<>> for auth token?**
//! - Thread-safe token access across async tasks
//! - Allows token refresh without &mut self
//! - Matches InMemoryAdapter pattern for consistency
//!
//! ## Error Handling
//!
//! HTTP status codes map to ServiceError:
//! - 401 → Unauthorized (invalid/expired token)
//! - 403 → Forbidden (insufficient permissions)
//! - 404 → NotFound (resource doesn't exist)
//! - 409 → Conflict (resource already exists)
//! - Other → NetworkError (with detailed message)
//!
//! ## Migration Notes
//!
//! Migrated from InMemoryAdapter (2025-12-04):
//! - All 7 service traits implemented (Auth, Token, Usage, Limits, Traces, Health, Storage)
//! - InMemoryAdapter now behind compile_error! guard (test-only)
//! - 272 tests passing with HttpAdapter available
//! - Binary integration pending (stub blocker)

use super::super::error::ServiceError;
use super::super::services::*;
use async_trait::async_trait;
use reqwest::{ Client, Method, Response };
use serde::{ Deserialize, Serialize };
use std::sync::{ Arc, RwLock };

/// HTTP adapter using reqwest for API communication
pub struct HttpAdapter
{
  client: Client,
  base_url: String,
  auth_token: Arc<RwLock<Option<String>>>,
}

impl HttpAdapter
{
  /// Create new HTTP adapter with API base URL
  pub fn new( base_url: impl Into<String> ) -> Result<Self, ServiceError>
  {
    let base_url = base_url.into();

    // Validate URL format
    if !base_url.starts_with( "http://" ) && !base_url.starts_with( "https://" )
    {
      return Err( ServiceError::ValidationError(
        format!( "Invalid URL: must start with http:// or https://, got: {}", base_url )
      ) );
    }

    Ok( Self {
      client: Client::new(),
      base_url,
      auth_token: Arc::new( RwLock::new( None ) ),
    } )
  }

  /// Set authentication token for API requests
  pub fn set_auth_token( &self, token: String )
  {
    let mut auth = self.auth_token.write().unwrap();
    *auth = Some( token );
  }

  /// Clear authentication token
  pub fn clear_auth_token( &self )
  {
    let mut auth = self.auth_token.write().unwrap();
    *auth = None;
  }

  /// Build HTTP request with authentication
  fn request( &self, method: Method, path: &str ) -> reqwest::RequestBuilder
  {
    let url = format!( "{}{}", self.base_url, path );
    let mut builder = self.client.request( method, &url );

    // Add auth header if token is set
    if let Some( token ) = self.auth_token.read().unwrap().as_ref()
    {
      builder = builder.header( "Authorization", format!( "Bearer {}", token ) );
    }

    builder
  }

  /// Handle HTTP response and map errors
  async fn handle_response<T: for<'de> Deserialize<'de>>( response: Response ) -> Result<T, ServiceError>
  {
    let status = response.status();

    match status.as_u16()
    {
      200..=299 => {
        response.json::<T>().await.map_err( |e| {
          ServiceError::NetworkError( format!( "Failed to parse response: {}", e ) )
        } )
      }
      401 => Err( ServiceError::Unauthorized ),
      403 => Err( ServiceError::Forbidden ),
      404 => Err( ServiceError::NotFound ),
      409 => Err( ServiceError::Conflict ),
      _ => {
        let error_text = response.text().await.unwrap_or_else( |_| "Unknown error".to_string() );
        Err( ServiceError::NetworkError( format!( "HTTP {}: {}", status, error_text ) ) )
      }
    }
  }

  /// Handle HTTP response for operations with no body
  async fn handle_empty_response( response: Response ) -> Result<(), ServiceError>
  {
    let status = response.status();

    match status.as_u16()
    {
      200..=299 => Ok( () ),
      401 => Err( ServiceError::Unauthorized ),
      403 => Err( ServiceError::Forbidden ),
      404 => Err( ServiceError::NotFound ),
      409 => Err( ServiceError::Conflict ),
      _ => {
        let error_text = response.text().await.unwrap_or_else( |_| "Unknown error".to_string() );
        Err( ServiceError::NetworkError( format!( "HTTP {}: {}", status, error_text ) ) )
      }
    }
  }
}

// ============================================================================
// AuthService Implementation
// ============================================================================

#[ derive( Serialize ) ]
struct LoginRequest
{
  username: String,
  password: String,
}

#[ derive( Deserialize ) ]
struct TokensResponse
{
  access_token: String,
  refresh_token: String,
}

#[ async_trait ]
impl AuthService for HttpAdapter
{
  async fn login( &self, username: &str, password: &str ) -> Result<Tokens, ServiceError>
  {
    let req_body = LoginRequest {
      username: username.to_string(),
      password: password.to_string(),
    };

    let response = self
      .request( Method::POST, "/api/auth/login" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Login request failed: {}", e ) ) )?;

    let tokens_resp: TokensResponse = Self::handle_response( response ).await?;

    Ok( Tokens {
      access_token: tokens_resp.access_token,
      refresh_token: tokens_resp.refresh_token,
    } )
  }

  async fn refresh( &self, refresh_token: &str ) -> Result<Tokens, ServiceError>
  {
    #[ derive( Serialize ) ]
    struct RefreshRequest { refresh_token: String }

    let req_body = RefreshRequest {
      refresh_token: refresh_token.to_string(),
    };

    let response = self
      .request( Method::POST, "/api/auth/refresh" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Refresh request failed: {}", e ) ) )?;

    let tokens_resp: TokensResponse = Self::handle_response( response ).await?;

    Ok( Tokens {
      access_token: tokens_resp.access_token,
      refresh_token: tokens_resp.refresh_token,
    } )
  }

  async fn logout( &self, access_token: &str ) -> Result<(), ServiceError>
  {
    #[ derive( Serialize ) ]
    struct LogoutRequest { access_token: String }

    let req_body = LogoutRequest {
      access_token: access_token.to_string(),
    };

    let response = self
      .request( Method::POST, "/api/auth/logout" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Logout request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }
}

// ============================================================================
// TokenService Implementation
// ============================================================================

#[ derive( Serialize ) ]
struct GenerateTokenRequest
{
  name: String,
  scope: String,
  ttl: Option<i64>,
}

#[ derive( Deserialize ) ]
struct TokenResponse
{
  id: String,
  name: String,
  scope: String,
  created_at: String,
  expires_at: Option<String>,
}

impl From<TokenResponse> for Token
{
  fn from( resp: TokenResponse ) -> Self
  {
    Token {
      id: resp.id,
      name: resp.name,
      scope: resp.scope,
      created_at: resp.created_at,
      expires_at: resp.expires_at,
    }
  }
}

#[ async_trait ]
impl TokenService for HttpAdapter
{
  async fn generate( &self, name: &str, scope: &str, ttl: Option<i64> ) -> Result<Token, ServiceError>
  {
    let req_body = GenerateTokenRequest {
      name: name.to_string(),
      scope: scope.to_string(),
      ttl,
    };

    let response = self
      .request( Method::POST, "/api/tokens" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Generate token request failed: {}", e ) ) )?;

    let token_resp: TokenResponse = Self::handle_response( response ).await?;
    Ok( token_resp.into() )
  }

  async fn list( &self, filter: Option<&str> ) -> Result<Vec<Token>, ServiceError>
  {
    let mut req = self.request( Method::GET, "/api/tokens" );

    if let Some( f ) = filter
    {
      req = req.query( &[ ( "filter", f ) ] );
    }

    let response = req
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "List tokens request failed: {}", e ) ) )?;

    let tokens_resp: Vec<TokenResponse> = Self::handle_response( response ).await?;
    Ok( tokens_resp.into_iter().map( Token::from ).collect() )
  }

  async fn get( &self, token_id: &str ) -> Result<Token, ServiceError>
  {
    let path = format!( "/api/tokens/{}", token_id );
    let response = self
      .request( Method::GET, &path )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get token request failed: {}", e ) ) )?;

    let token_resp: TokenResponse = Self::handle_response( response ).await?;
    Ok( token_resp.into() )
  }

  async fn rotate( &self, token_id: &str, new_ttl: Option<i64> ) -> Result<Token, ServiceError>
  {
    #[ derive( Serialize ) ]
    struct RotateRequest { ttl: Option<i64> }

    let req_body = RotateRequest { ttl: new_ttl };
    let path = format!( "/api/tokens/{}/rotate", token_id );

    let response = self
      .request( Method::POST, &path )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Rotate token request failed: {}", e ) ) )?;

    let token_resp: TokenResponse = Self::handle_response( response ).await?;
    Ok( token_resp.into() )
  }

  async fn revoke( &self, token_id: &str, reason: Option<&str> ) -> Result<(), ServiceError>
  {
    #[ derive( Serialize ) ]
    struct RevokeRequest { reason: Option<String> }

    let req_body = RevokeRequest {
      reason: reason.map( |s| s.to_string() ),
    };
    let path = format!( "/api/tokens/{}/revoke", token_id );

    let response = self
      .request( Method::POST, &path )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Revoke token request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }
}

// ============================================================================
// UsageService Implementation
// ============================================================================

#[ derive( Serialize ) ]
struct RecordUsageRequest
{
  project_id: String,
  provider: String,
  tokens: u64,
  cost: u64,
}

#[ derive( Deserialize ) ]
struct UsageRecordResponse
{
  project_id: String,
  provider: String,
  tokens_used: u64,
  cost: u64,
  timestamp: String,
}

impl From<UsageRecordResponse> for UsageRecord
{
  fn from( resp: UsageRecordResponse ) -> Self
  {
    UsageRecord {
      project_id: resp.project_id,
      provider: resp.provider,
      tokens_used: resp.tokens_used,
      cost: resp.cost,
      timestamp: resp.timestamp,
    }
  }
}

#[ async_trait ]
impl UsageService for HttpAdapter
{
  async fn record_usage( &self, project_id: &str, provider: &str, tokens: u64, cost: u64 ) -> Result<(), ServiceError>
  {
    let req_body = RecordUsageRequest {
      project_id: project_id.to_string(),
      provider: provider.to_string(),
      tokens,
      cost,
    };

    let response = self
      .request( Method::POST, "/api/usage" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Record usage request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }

  async fn get_usage( &self, start_date: Option<&str>, end_date: Option<&str> ) -> Result<Vec<UsageRecord>, ServiceError>
  {
    let mut req = self.request( Method::GET, "/api/usage" );

    let mut query_params = Vec::new();
    if let Some( start ) = start_date
    {
      query_params.push( ( "start_date", start ) );
    }
    if let Some( end ) = end_date
    {
      query_params.push( ( "end_date", end ) );
    }

    if !query_params.is_empty()
    {
      req = req.query( &query_params );
    }

    let response = req
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get usage request failed: {}", e ) ) )?;

    let usage_resp: Vec<UsageRecordResponse> = Self::handle_response( response ).await?;
    Ok( usage_resp.into_iter().map( UsageRecord::from ).collect() )
  }

  async fn get_usage_by_project( &self, project_id: &str, start_date: Option<&str> ) -> Result<Vec<UsageRecord>, ServiceError>
  {
    let path = format!( "/api/usage/project/{}", project_id );
    let mut req = self.request( Method::GET, &path );

    if let Some( start ) = start_date
    {
      req = req.query( &[ ( "start_date", start ) ] );
    }

    let response = req
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get usage by project request failed: {}", e ) ) )?;

    let usage_resp: Vec<UsageRecordResponse> = Self::handle_response( response ).await?;
    Ok( usage_resp.into_iter().map( UsageRecord::from ).collect() )
  }

  async fn get_usage_by_provider( &self, provider: &str, aggregation: Option<&str> ) -> Result<Vec<UsageRecord>, ServiceError>
  {
    let path = format!( "/api/usage/provider/{}", provider );
    let mut req = self.request( Method::GET, &path );

    if let Some( agg ) = aggregation
    {
      req = req.query( &[ ( "aggregation", agg ) ] );
    }

    let response = req
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get usage by provider request failed: {}", e ) ) )?;

    let usage_resp: Vec<UsageRecordResponse> = Self::handle_response( response ).await?;
    Ok( usage_resp.into_iter().map( UsageRecord::from ).collect() )
  }

  async fn export_usage( &self, output_path: &str, format: &str ) -> Result<(), ServiceError>
  {
    #[ derive( Serialize ) ]
    struct ExportRequest
    {
      output_path: String,
      format: String,
    }

    let req_body = ExportRequest {
      output_path: output_path.to_string(),
      format: format.to_string(),
    };

    let response = self
      .request( Method::POST, "/api/usage/export" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Export usage request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }
}

// ============================================================================
// LimitsService Implementation
// ============================================================================

#[ derive( Serialize ) ]
struct CreateLimitRequest
{
  resource_type: String,
  limit_value: u64,
}

#[ derive( Deserialize ) ]
struct LimitResponse
{
  id: String,
  resource_type: String,
  limit_value: u64,
  created_at: String,
  updated_at: String,
}

impl From<LimitResponse> for Limit
{
  fn from( resp: LimitResponse ) -> Self
  {
    Limit {
      id: resp.id,
      resource_type: resp.resource_type,
      limit_value: resp.limit_value,
      created_at: resp.created_at,
      updated_at: resp.updated_at,
    }
  }
}

#[ async_trait ]
impl LimitsService for HttpAdapter
{
  async fn create_limit( &self, resource_type: &str, limit_value: u64 ) -> Result<Limit, ServiceError>
  {
    let req_body = CreateLimitRequest {
      resource_type: resource_type.to_string(),
      limit_value,
    };

    let response = self
      .request( Method::POST, "/api/limits" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Create limit request failed: {}", e ) ) )?;

    let limit_resp: LimitResponse = Self::handle_response( response ).await?;
    Ok( limit_resp.into() )
  }

  async fn list_limits( &self ) -> Result<Vec<Limit>, ServiceError>
  {
    let response = self
      .request( Method::GET, "/api/limits" )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "List limits request failed: {}", e ) ) )?;

    let limits_resp: Vec<LimitResponse> = Self::handle_response( response ).await?;
    Ok( limits_resp.into_iter().map( Limit::from ).collect() )
  }

  async fn get_limit( &self, limit_id: &str ) -> Result<Limit, ServiceError>
  {
    let path = format!( "/api/limits/{}", limit_id );
    let response = self
      .request( Method::GET, &path )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get limit request failed: {}", e ) ) )?;

    let limit_resp: LimitResponse = Self::handle_response( response ).await?;
    Ok( limit_resp.into() )
  }

  async fn update_limit( &self, limit_id: &str, new_value: u64 ) -> Result<Limit, ServiceError>
  {
    #[ derive( Serialize ) ]
    struct UpdateLimitRequest { limit_value: u64 }

    let req_body = UpdateLimitRequest { limit_value: new_value };
    let path = format!( "/api/limits/{}", limit_id );

    let response = self
      .request( Method::PUT, &path )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Update limit request failed: {}", e ) ) )?;

    let limit_resp: LimitResponse = Self::handle_response( response ).await?;
    Ok( limit_resp.into() )
  }

  async fn delete_limit( &self, limit_id: &str ) -> Result<(), ServiceError>
  {
    let path = format!( "/api/limits/{}", limit_id );
    let response = self
      .request( Method::DELETE, &path )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Delete limit request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }
}

// ============================================================================
// TracesService Implementation
// ============================================================================

#[ derive( Serialize ) ]
struct RecordTraceRequest
{
  trace_id: String,
  request: String,
  duration_ms: u64,
}

#[ derive( Deserialize ) ]
struct TraceResponse
{
  id: String,
  request: String,
  duration_ms: u64,
  timestamp: String,
}

impl From<TraceResponse> for Trace
{
  fn from( resp: TraceResponse ) -> Self
  {
    Trace {
      id: resp.id,
      request: resp.request,
      duration_ms: resp.duration_ms,
      timestamp: resp.timestamp,
    }
  }
}

#[ async_trait ]
impl TracesService for HttpAdapter
{
  async fn record_trace( &self, trace_id: &str, request: &str, duration_ms: u64 ) -> Result<(), ServiceError>
  {
    let req_body = RecordTraceRequest {
      trace_id: trace_id.to_string(),
      request: request.to_string(),
      duration_ms,
    };

    let response = self
      .request( Method::POST, "/api/traces" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Record trace request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }

  async fn list_traces( &self, filter: Option<&str>, limit: Option<u32> ) -> Result<Vec<Trace>, ServiceError>
  {
    let mut req = self.request( Method::GET, "/api/traces" );

    let mut query_params = Vec::new();
    if let Some( f ) = filter
    {
      query_params.push( ( "filter", f.to_string() ) );
    }
    if let Some( l ) = limit
    {
      query_params.push( ( "limit", l.to_string() ) );
    }

    if !query_params.is_empty()
    {
      req = req.query( &query_params );
    }

    let response = req
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "List traces request failed: {}", e ) ) )?;

    let traces_resp: Vec<TraceResponse> = Self::handle_response( response ).await?;
    Ok( traces_resp.into_iter().map( Trace::from ).collect() )
  }

  async fn get_trace( &self, trace_id: &str ) -> Result<Trace, ServiceError>
  {
    let path = format!( "/api/traces/{}", trace_id );
    let response = self
      .request( Method::GET, &path )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Get trace request failed: {}", e ) ) )?;

    let trace_resp: TraceResponse = Self::handle_response( response ).await?;
    Ok( trace_resp.into() )
  }

  async fn export_traces( &self, output_path: &str, format: &str ) -> Result<(), ServiceError>
  {
    #[ derive( Serialize ) ]
    struct ExportRequest
    {
      output_path: String,
      format: String,
    }

    let req_body = ExportRequest {
      output_path: output_path.to_string(),
      format: format.to_string(),
    };

    let response = self
      .request( Method::POST, "/api/traces/export" )
      .json( &req_body )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Export traces request failed: {}", e ) ) )?;

    Self::handle_empty_response( response ).await
  }
}

// ============================================================================
// HealthService Implementation
// ============================================================================

#[ derive( Deserialize ) ]
struct HealthStatusResponse
{
  status: String,
  timestamp: i64,
}

impl From<HealthStatusResponse> for HealthStatus
{
  fn from( resp: HealthStatusResponse ) -> Self
  {
    HealthStatus {
      status: resp.status,
      uptime_seconds: resp.timestamp as u64,
    }
  }
}

#[ async_trait ]
impl HealthService for HttpAdapter
{
  async fn get_health( &self ) -> Result<HealthStatus, ServiceError>
  {
    let response = self
      .request( Method::GET, "/api/health" )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Health check request failed: {}", e ) ) )?;

    let health_resp: HealthStatusResponse = Self::handle_response( response ).await?;
    Ok( health_resp.into() )
  }

  async fn get_version( &self ) -> Result<String, ServiceError>
  {
    #[ derive( Deserialize ) ]
    struct VersionResponse { current_version: String }

    let response = self
      .request( Method::GET, "/api/version" )
      .send()
      .await
      .map_err( |e| ServiceError::NetworkError( format!( "Version request failed: {}", e ) ) )?;

    let version_resp: VersionResponse = Self::handle_response( response ).await?;
    Ok( version_resp.current_version )
  }
}

// ============================================================================
// StorageService Implementation
// ============================================================================

#[ async_trait ]
impl StorageService for HttpAdapter
{
  async fn save_tokens( &self, tokens: &Tokens ) -> Result<(), ServiceError>
  {
    // Store tokens to local filesystem (~/.iron/tokens.json)
    let tokens_dir = dirs::home_dir()
      .ok_or_else( || ServiceError::StorageError( "Could not find home directory".to_string() ) )?
      .join( ".iron" );

    tokio::fs::create_dir_all( &tokens_dir )
      .await
      .map_err( |e| ServiceError::StorageError( format!( "Failed to create .iron directory: {}", e ) ) )?;

    let tokens_path = tokens_dir.join( "tokens.json" );
    let tokens_json = serde_json::to_string_pretty( tokens )
      .map_err( |e| ServiceError::StorageError( format!( "Failed to serialize tokens: {}", e ) ) )?;

    tokio::fs::write( &tokens_path, tokens_json )
      .await
      .map_err( |e| ServiceError::StorageError( format!( "Failed to write tokens file: {}", e ) ) )?;

    Ok( () )
  }

  async fn load_tokens( &self ) -> Result<Option<Tokens>, ServiceError>
  {
    let tokens_path = dirs::home_dir()
      .ok_or_else( || ServiceError::StorageError( "Could not find home directory".to_string() ) )?
      .join( ".iron" )
      .join( "tokens.json" );

    if !tokens_path.exists()
    {
      return Ok( None );
    }

    let tokens_json = tokio::fs::read_to_string( &tokens_path )
      .await
      .map_err( |e| ServiceError::StorageError( format!( "Failed to read tokens file: {}", e ) ) )?;

    let tokens: Tokens = serde_json::from_str( &tokens_json )
      .map_err( |e| ServiceError::StorageError( format!( "Failed to parse tokens file: {}", e ) ) )?;

    Ok( Some( tokens ) )
  }

  async fn clear_tokens( &self ) -> Result<(), ServiceError>
  {
    let tokens_path = dirs::home_dir()
      .ok_or_else( || ServiceError::StorageError( "Could not find home directory".to_string() ) )?
      .join( ".iron" )
      .join( "tokens.json" );

    if tokens_path.exists()
    {
      tokio::fs::remove_file( &tokens_path )
        .await
        .map_err( |e| ServiceError::StorageError( format!( "Failed to remove tokens file: {}", e ) ) )?;
    }

    Ok( () )
  }
}
