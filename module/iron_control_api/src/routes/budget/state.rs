//! Budget protocol shared state
//!
//! Provides `BudgetState` which holds all managers and dependencies needed
//! for budget protocol endpoints.

use crate::{ ic_token::IcTokenManager, ip_token::IpTokenCrypto, jwt_auth::JwtSecret, routes::auth::AuthState };
use axum::extract::FromRef;
use iron_secrets::crypto::CryptoService;
use iron_token_manager::
{
  agent_budget::AgentBudgetManager,
  lease_manager::LeaseManager,
  provider_key_storage::ProviderKeyStorage,
};
use sqlx::SqlitePool;
use std::sync::Arc;

/// Budget protocol shared state
#[ derive( Clone ) ]
pub struct BudgetState
{
  pub ic_token_manager: Arc< IcTokenManager >,
  pub ip_token_crypto: Arc< IpTokenCrypto >,
  pub lease_manager: Arc< LeaseManager >,
  pub agent_budget_manager: Arc< AgentBudgetManager >,
  pub provider_key_storage: Arc< ProviderKeyStorage >,
  pub provider_key_crypto: Arc< CryptoService >,
  pub db_pool: SqlitePool,
  pub jwt_secret: Arc< JwtSecret >,
}

/// Enable AuthState extraction from BudgetState
impl FromRef< BudgetState > for AuthState
{
  fn from_ref( state: &BudgetState ) -> Self
  {
    AuthState
    {
      jwt_secret: state.jwt_secret.clone(),
      db_pool: state.db_pool.clone(),
      rate_limiter: crate::rate_limiter::LoginRateLimiter::new(),
    }
  }
}

impl BudgetState
{
  /// Create new budget state
  ///
  /// # Arguments
  ///
  /// * `ic_token_secret` - Secret key for IC Token JWT signing
  /// * `ip_token_key` - 32-byte encryption key for IP Token AES-256-GCM
  /// * `provider_key_master` - 32-byte master key for provider key encryption/decryption
  /// * `jwt_secret` - Secret key for JWT access token signing/verification
  /// * `database_url` - Database connection string
  ///
  /// # Errors
  ///
  /// Returns error if database connection or crypto initialization fails
  pub async fn new(
    ic_token_secret: String,
    ip_token_key: &[ u8 ],
    provider_key_master: &[ u8 ],
    jwt_secret: Arc< JwtSecret >,
    database_url: &str,
  ) -> Result< Self, Box< dyn std::error::Error > >
  {
    let db_pool = SqlitePool::connect( database_url ).await?;
    let ic_token_manager = Arc::new( IcTokenManager::new( ic_token_secret ) );
    let ip_token_crypto = Arc::new( IpTokenCrypto::new( ip_token_key )? );
    let provider_key_crypto = Arc::new( CryptoService::new( provider_key_master )? );
    let lease_manager = Arc::new( LeaseManager::from_pool( db_pool.clone() ) );
    let agent_budget_manager = Arc::new( AgentBudgetManager::from_pool( db_pool.clone() ) );
    let provider_key_storage = Arc::new( ProviderKeyStorage::new( db_pool.clone() ) );

    Ok( Self
    {
      ic_token_manager,
      ip_token_crypto,
      lease_manager,
      agent_budget_manager,
      provider_key_storage,
      provider_key_crypto,
      db_pool,
      jwt_secret,
    } )
  }
}
