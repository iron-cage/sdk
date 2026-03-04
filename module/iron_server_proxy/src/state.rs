//! Shared application state for axum handlers.

use std::sync::Arc;

use base64::{engine::general_purpose::STANDARD, Engine};
use reqwest::Client;
use sqlx::SqlitePool;

use crate::{config::Config, error::ServerError, rate_limiter::AuthRateLimiter};
use iron_cost::pricing::PricingManager;
use iron_secrets::crypto::{CryptoService, KEY_SIZE};
use iron_token_manager::ProviderKeyStorage;

/// Shared application state passed to every axum handler via `State<AppState>`.
///
/// Contains database pool, crypto services, HTTP client, and pricing data
/// needed to authenticate agents, decrypt provider keys, and forward LLM requests.
#[derive(Clone, Debug)]
pub struct AppState {
  /// `SQLite` connection pool (shared with `iron_control_api`).
  pub db_pool: SqlitePool,
  /// Storage layer for provider key CRUD and spending tracking.
  pub provider_key_storage: ProviderKeyStorage,
  /// AES-256-GCM crypto service for decrypting provider API keys from DB.
  pub crypto_service: Arc<CryptoService>,
  /// Reusable HTTP client for forwarding requests to LLM providers.
  pub http_client: Client,
  /// Model pricing data for cost calculation.
  pub pricing_manager: Arc<PricingManager>,
  /// Per-IP rate limiter for failed auth attempts.
  pub auth_rate_limiter: AuthRateLimiter,
}

impl AppState {
  /// Initialize application state from configuration.
  ///
  /// This connects to the database, applies all pending migrations,
  /// initializes the crypto service, and loads pricing data.
  ///
  /// # Errors
  ///
  /// Returns error if database connection, migration, crypto init, or pricing load fails.
  pub async fn new(config: &Config) -> Result<Self, ServerError> {
    // Connect to shared SQLite database
    let db_pool = SqlitePool::connect(&config.database_url)
      .await
      .map_err(|e| ServerError::Database(format!("Connection failed: {e}")))?;

    // Apply all pending migrations (idempotent, safe to call multiple times)
    iron_token_manager::apply_all_migrations(&db_pool)
      .await
      .map_err(|e| ServerError::Database(format!("Migration failed: {e}")))?;

    // Initialize provider key storage
    let provider_key_storage = ProviderKeyStorage::new(db_pool.clone());

    // Decode base64 master key and create crypto service
    let key_bytes = STANDARD
      .decode(&config.secrets_master_key)
      .map_err(|e| ServerError::Crypto(format!("Invalid base64 master key: {e}")))?;
    if key_bytes.len() != KEY_SIZE {
      return Err(ServerError::Crypto(format!(
        "IRON_SECRETS_MASTER_KEY must be exactly {KEY_SIZE} bytes (AES-256), got {} bytes",
        key_bytes.len()
      )));
    }
    let crypto_service = Arc::new(
      CryptoService::new(&key_bytes)
        .map_err(|e| ServerError::Crypto(format!("CryptoService init failed: {e}")))?,
    );

    // HTTP client with 5-minute timeout for slow LLM responses
    let http_client = Client::builder()
      .timeout(core::time::Duration::from_secs(300))
      .build()
      .map_err(|e| ServerError::HttpClient(e.to_string()))?;

    // Load embedded pricing data for cost calculation
    let pricing_manager =
      Arc::new(PricingManager::new().map_err(|e| ServerError::Pricing(e.to_string()))?);

    Ok(Self {
      db_pool,
      provider_key_storage,
      crypto_service,
      http_client,
      pricing_manager,
      auth_rate_limiter: AuthRateLimiter::new(),
    })
  }
}
