//! Budget protocol shared state
//!
//! Provides `BudgetState` which holds all managers and dependencies needed
//! for budget protocol endpoints.

use crate::{
  ic_token::{IcTokenManager, IcTokenRateLimiter},
  ip_token::IpTokenCrypto,
  jwt_auth::JwtSecret,
  routes::auth::AuthState,
};
use axum::extract::FromRef;
use iron_secrets::crypto::CryptoService;
use iron_secrets::ip_token::IpTokenKey;
use iron_token_manager::{
  agent_budget::AgentBudgetManager, lease_manager::LeaseManager,
  provider_key_storage::ProviderKeyStorage,
};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Budget protocol shared state
#[derive(Clone, Debug)]
pub struct BudgetState {
  /// IC token manager for authentication
  pub ic_token_manager: Arc<IcTokenManager>,
  /// Crypto service for IP token encryption
  pub ip_token_crypto: Arc<IpTokenCrypto>,
  /// Manager for budget leases
  pub lease_manager: Arc<LeaseManager>,
  /// Manager for agent budgets
  pub agent_budget_manager: Arc<AgentBudgetManager>,
  /// Storage for provider API keys
  pub provider_key_storage: Arc<ProviderKeyStorage>,
  /// Crypto service for provider key encryption/decryption
  pub provider_key_crypto: Arc<CryptoService>,
  /// `SQLite` database connection pool
  pub db_pool: SqlitePool,
  /// JWT secret for access token verification
  pub jwt_secret: Arc<JwtSecret>,
  /// Crypto service for decrypting provider keys (Feature 014)
  pub crypto_service: Option<Arc<CryptoService>>,
  /// Rate limiter for IC Token validation (H2)
  pub ic_token_rate_limiter: IcTokenRateLimiter,
}

/// Enable `AuthState` extraction from `BudgetState`
impl FromRef<BudgetState> for AuthState {
  fn from_ref(state: &BudgetState) -> Self {
    AuthState {
      jwt_secret: state.jwt_secret.clone(),
      db_pool: state.db_pool.clone(),
      rate_limiter: crate::rate_limiter::LoginRateLimiter::new(),
      // Budget endpoints use IC Token auth, not login rate limiting
      rate_limiting_enabled: false,
    }
  }
}

impl BudgetState {
  /// Create new budget state
  ///
  /// # Arguments
  ///
  /// * `ic_token_manager` - Shared IC token manager for authentication
  /// * `ip_token_key` - 32-byte encryption key for IP Token AES-256-GCM
  /// * `provider_key_master` - 32-byte master key for provider key encryption/decryption
  /// * `jwt_secret` - Secret key for JWT access token signing/verification
  /// * `database_url` - Database connection string
  /// * `crypto_service` - Optional crypto service for provider key decryption
  /// * `ic_token_rate_limiter` - Shared rate limiter for IC token validation
  ///
  /// # Errors
  ///
  /// Returns error if database connection or crypto initialization fails
  pub async fn new(
    ic_token_manager: Arc<IcTokenManager>,
    ip_token_key: &IpTokenKey,
    provider_key_master: &[u8],
    jwt_secret: Arc<JwtSecret>,
    database_url: &str,
    crypto_service: Option<Arc<CryptoService>>,
    ic_token_rate_limiter: IcTokenRateLimiter,
  ) -> Result<Self, Box<dyn core::error::Error>> {
    let db_pool = SqlitePool::connect(database_url).await?;

    // Enforce referential integrity on every connection in this pool.
    // SQLite disables foreign-key enforcement by default; the PRAGMA must be
    // re-issued per connection.  apply_all_migrations() does this for pools
    // that own migrations, but BudgetState shares the same database file and
    // creates its own pool, so we set it explicitly here.
    //qqq: [Info] PRAGMA foreign_keys is OFF by default in SQLite — this pragma must be set on every new connection; verify pool connection hooks also set it
    sqlx::query("PRAGMA foreign_keys = ON")
      .execute(&db_pool)
      .await
      .map_err(|e| format!("PRAGMA foreign_keys failed: {e}"))?;

    let ip_token_crypto = Arc::new(IpTokenCrypto::new(ip_token_key)?);
    let provider_key_crypto = Arc::new(CryptoService::new(provider_key_master)?);
    let lease_manager = Arc::new(LeaseManager::from_pool(db_pool.clone()));
    let agent_budget_manager = Arc::new(AgentBudgetManager::from_pool(db_pool.clone()));
    let provider_key_storage = Arc::new(ProviderKeyStorage::new(db_pool.clone()));

    Ok(Self {
      ic_token_manager,
      ip_token_crypto,
      lease_manager,
      agent_budget_manager,
      provider_key_storage,
      provider_key_crypto,
      db_pool,
      jwt_secret,
      crypto_service,
      ic_token_rate_limiter,
    })
  }
}
