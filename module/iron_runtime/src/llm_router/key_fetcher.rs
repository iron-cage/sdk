//! Fetch and cache provider API keys from Iron Cage server

use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::llm_router::error::LlmRouterError;

/// Provider key information returned from Iron Cage server
#[derive(Clone, Debug)]
pub struct ProviderKey
{
  /// Provider type: "openai" or "anthropic"
  pub provider: String,
  /// The actual API key (e.g., sk-xxx for OpenAI)
  pub api_key: String,
  /// Optional custom base URL for the provider
  pub base_url: Option<String>,
}

impl ProviderKey
{
  /// Auto-detect provider from API key format
  pub fn detect_provider_from_key(api_key: &str) -> &'static str
  {
    if api_key.starts_with("sk-ant-")
    {
      "anthropic"
    }
    else
    {
      "openai" // Default to OpenAI for sk-* and other formats
    }
  }
}

/// Cached key entry
struct CachedKey
{
  key: ProviderKey,
  fetched_at: Instant,
}

/// Fetches and caches provider API keys from Iron Cage server
pub struct KeyFetcher
{
  server_url: String,
  ic_token: String,
  client: Client,
  /// Single cached key (auto-detected provider)
  cache: Arc<RwLock<Option<CachedKey>>>,
  cache_ttl: Duration,
  /// Static key mode - if set, always return this key without fetching
  static_key: Option<ProviderKey>,
}

impl KeyFetcher
{
  /// Create a new KeyFetcher
  ///
  /// # Arguments
  ///
  /// * `server_url` - Iron Cage server URL (e.g., "https://api.iron-cage.io")
  /// * `ic_token` - Iron Cage API token
  /// * `cache_ttl_seconds` - How long to cache the key (in seconds)
  pub fn new(server_url: String, ic_token: String, cache_ttl_seconds: u64) -> Self
  {
    let client = Client::builder()
      .timeout(Duration::from_secs(30))
      .build()
      .expect("LOUD FAILURE: Failed to create HTTP client");

    Self {
      server_url,
      ic_token,
      client,
      cache: Arc::new(RwLock::new(None)),
      cache_ttl: Duration::from_secs(cache_ttl_seconds),
      static_key: None,
    }
  }

  /// Create a KeyFetcher with a static API key (bypasses server)
  ///
  /// # Arguments
  ///
  /// * `api_key` - The provider API key (e.g., sk-xxx for OpenAI, sk-ant-xxx for Anthropic)
  /// * `base_url` - Optional custom base URL for the provider
  pub fn new_static(api_key: String, base_url: Option<String>) -> Self
  {
    let provider = ProviderKey::detect_provider_from_key(&api_key).to_string();
    let static_key = ProviderKey {
      provider,
      api_key,
      base_url,
    };

    Self {
      server_url: String::new(),
      ic_token: String::new(),
      client: Client::new(),
      cache: Arc::new(RwLock::new(None)),
      cache_ttl: Duration::from_secs(0),
      static_key: Some(static_key),
    }
  }

  /// Get provider key (from cache or fetch from server)
  /// Provider is auto-detected from API key format
  pub async fn get_key(&self) -> Result<ProviderKey, LlmRouterError>
  {
    // Return static key if set (bypass server)
    if let Some(ref key) = self.static_key {
      return Ok(key.clone());
    }

    // Check cache first
    {
      let cache = self.cache.read().await;
      if let Some(cached) = cache.as_ref()
      {
        if cached.fetched_at.elapsed() < self.cache_ttl
        {
          return Ok(cached.key.clone());
        }
      }
    }

    // Fetch from server
    let key = self.fetch_from_server().await?;

    // Update cache
    {
      let mut cache = self.cache.write().await;
      *cache = Some(CachedKey {
        key: key.clone(),
        fetched_at: Instant::now(),
      });
    }

    Ok(key)
  }

  /// Fetch key from Iron Cage server (auto-detect provider from key format)
  async fn fetch_from_server(&self) -> Result<ProviderKey, LlmRouterError>
  {
    let url = format!("{}/api/keys", self.server_url);

    let response = self
      .client
      .get(&url)
      .header("Authorization", format!("Bearer {}", self.ic_token))
      .send()
      .await
      .map_err(|e| LlmRouterError::KeyFetch(e.to_string()))?;

    if !response.status().is_success()
    {
      return Err(LlmRouterError::KeyFetch(format!(
        "Server returned status {}",
        response.status()
      )));
    }

    #[derive(serde::Deserialize)]
    struct KeyResponse
    {
      #[serde(default)]
      provider: Option<String>,
      api_key: String,
      #[serde(default)]
      base_url: Option<String>,
    }

    let data: KeyResponse = response
      .json()
      .await
      .map_err(|e| LlmRouterError::KeyFetch(e.to_string()))?;

    // Auto-detect provider from key format if not specified
    let provider = data
      .provider
      .unwrap_or_else(|| ProviderKey::detect_provider_from_key(&data.api_key).to_string());

    Ok(ProviderKey {
      provider,
      api_key: data.api_key,
      base_url: data.base_url,
    })
  }
}
