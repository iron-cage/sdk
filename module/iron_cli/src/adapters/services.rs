//! Service trait definitions for adapter layer
//!
//! These traits define the interfaces for async I/O operations.
//! Implementations include:
//! - `InMemoryAdapter`: Fast, predictable, for tests
//! - `SqlxAdapter`: Real `PostgreSQL`, for production

use super::error::ServiceError;
use async_trait::async_trait;

/// Authentication tokens
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Tokens {
  /// Short-lived JWT used to authenticate API requests
  pub access_token: String,
  /// Long-lived token used to obtain new access tokens
  pub refresh_token: String,
}

/// Token metadata
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Token {
  /// Unique token identifier
  pub id: String,
  /// Human-readable token name
  pub name: String,
  /// Permission scope (e.g., `read:tokens`)
  pub scope: String,
  /// ISO 8601 creation timestamp
  pub created_at: String,
  /// ISO 8601 expiry timestamp, or `None` if the token never expires
  pub expires_at: Option<String>,
}

/// Authentication service
#[async_trait]
pub trait AuthService: Send + Sync {
  /// Login with username/password
  async fn login(&self, username: &str, password: &str) -> Result<Tokens, ServiceError>;

  /// Refresh access token using refresh token
  async fn refresh(&self, refresh_token: &str) -> Result<Tokens, ServiceError>;

  /// Logout (invalidate tokens)
  async fn logout(&self, access_token: &str) -> Result<(), ServiceError>;
}

/// Token management service
#[async_trait]
pub trait TokenService: Send + Sync {
  /// Generate new token
  async fn generate(
    &self,
    name: &str,
    scope: &str,
    ttl: Option<i64>,
  ) -> Result<Token, ServiceError>;

  /// List all tokens
  async fn list(&self, filter: Option<&str>) -> Result<Vec<Token>, ServiceError>;

  /// Get token by ID
  async fn get(&self, token_id: &str) -> Result<Token, ServiceError>;

  /// Rotate token (generate new value, preserve scope)
  async fn rotate(&self, token_id: &str, new_ttl: Option<i64>) -> Result<Token, ServiceError>;

  /// Revoke token
  async fn revoke(&self, token_id: &str, reason: Option<&str>) -> Result<(), ServiceError>;
}

/// Usage data record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UsageRecord {
  /// Project that incurred the usage
  pub project_id: String,
  /// Provider that served the request
  pub provider: String,
  /// Number of tokens consumed
  pub tokens_used: u64,
  /// Cost in the smallest currency unit (e.g., micro-cents)
  pub cost: u64,
  /// ISO 8601 timestamp of the usage event
  pub timestamp: String,
}

/// Usage management service
#[async_trait]
pub trait UsageService: Send + Sync {
  /// Record usage data
  async fn record_usage(
    &self,
    project_id: &str,
    provider: &str,
    tokens: u64,
    cost: u64,
  ) -> Result<(), ServiceError>;

  /// Get all usage records
  async fn get_usage(
    &self,
    start_date: Option<&str>,
    end_date: Option<&str>,
  ) -> Result<Vec<UsageRecord>, ServiceError>;

  /// Get usage by project
  async fn get_usage_by_project(
    &self,
    project_id: &str,
    start_date: Option<&str>,
  ) -> Result<Vec<UsageRecord>, ServiceError>;

  /// Get usage by provider
  async fn get_usage_by_provider(
    &self,
    provider: &str,
    aggregation: Option<&str>,
  ) -> Result<Vec<UsageRecord>, ServiceError>;

  /// Export usage data
  async fn export_usage(&self, output_path: &str, format: &str) -> Result<(), ServiceError>;
}

/// Limit record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Limit {
  /// Unique limit identifier
  pub id: String,
  /// Resource category this limit applies to (e.g., `tokens`, `cost`)
  pub resource_type: String,
  /// Maximum allowed value for the resource
  pub limit_value: u64,
  /// ISO 8601 creation timestamp
  pub created_at: String,
  /// ISO 8601 last-update timestamp
  pub updated_at: String,
}

/// Limits management service
#[async_trait]
pub trait LimitsService: Send + Sync {
  /// Create new limit
  async fn create_limit(
    &self,
    resource_type: &str,
    limit_value: u64,
  ) -> Result<Limit, ServiceError>;

  /// List all limits
  async fn list_limits(&self) -> Result<Vec<Limit>, ServiceError>;

  /// Get limit by ID
  async fn get_limit(&self, limit_id: &str) -> Result<Limit, ServiceError>;

  /// Update limit value
  async fn update_limit(&self, limit_id: &str, new_value: u64) -> Result<Limit, ServiceError>;

  /// Delete limit
  async fn delete_limit(&self, limit_id: &str) -> Result<(), ServiceError>;
}

/// Trace record
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Trace {
  /// Unique trace identifier
  pub id: String,
  /// Serialized request payload or summary
  pub request: String,
  /// End-to-end request duration in milliseconds
  pub duration_ms: u64,
  /// ISO 8601 timestamp when the trace was recorded
  pub timestamp: String,
}

/// Traces management service
#[async_trait]
pub trait TracesService: Send + Sync {
  /// Record new trace
  async fn record_trace(
    &self,
    trace_id: &str,
    request: &str,
    duration_ms: u64,
  ) -> Result<(), ServiceError>;

  /// List all traces
  async fn list_traces(
    &self,
    filter: Option<&str>,
    limit: Option<u32>,
  ) -> Result<Vec<Trace>, ServiceError>;

  /// Get trace by ID
  async fn get_trace(&self, trace_id: &str) -> Result<Trace, ServiceError>;

  /// Export traces
  async fn export_traces(&self, output_path: &str, format: &str) -> Result<(), ServiceError>;
}

/// Health check data
///
/// Note: Version information moved to dedicated /api/version endpoint.
/// Use `HealthService::get_version()` for version data.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthStatus {
  /// Service health status (e.g., `ok`, `degraded`, `down`)
  pub status: String,
  /// Seconds elapsed since the service started
  pub uptime_seconds: u64,
}

/// Health check service
#[async_trait]
pub trait HealthService: Send + Sync {
  /// Get health status
  async fn get_health(&self) -> Result<HealthStatus, ServiceError>;

  /// Get version information
  async fn get_version(&self) -> Result<String, ServiceError>;
}

/// Storage service (credentials, config, cache)
#[async_trait]
pub trait StorageService: Send + Sync {
  /// Save authentication tokens
  async fn save_tokens(&self, tokens: &Tokens) -> Result<(), ServiceError>;

  /// Load authentication tokens
  async fn load_tokens(&self) -> Result<Option<Tokens>, ServiceError>;

  /// Clear authentication tokens
  async fn clear_tokens(&self) -> Result<(), ServiceError>;
}

/// Combined services (all service traits in one)
pub struct Services {
  /// Authentication service (login, refresh, logout)
  pub auth: Box<dyn AuthService>,
  /// Token management service (generate, list, get, rotate, revoke)
  pub tokens: Box<dyn TokenService>,
  /// Storage service (credentials, config, cache)
  pub storage: Box<dyn StorageService>,
}

impl core::fmt::Debug for Services {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.debug_struct("Services")
      .field("auth", &"<dyn AuthService>")
      .field("tokens", &"<dyn TokenService>")
      .field("storage", &"<dyn StorageService>")
      .finish()
  }
}

impl Services {
  /// Create new `Services` instance with the given service implementations
  #[must_use]
  pub fn new(
    auth: Box<dyn AuthService>,
    tokens: Box<dyn TokenService>,
    storage: Box<dyn StorageService>,
  ) -> Self {
    Self {
      auth,
      tokens,
      storage,
    }
  }
}
