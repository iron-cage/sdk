//! Server proxy configuration loaded from environment variables.

use std::env;

use zeroize::Zeroizing;

/// Error loading proxy configuration.
#[derive(Debug)]
pub enum ConfigError {
  /// A required environment variable is missing.
  MissingVar(&'static str),
  /// An environment variable has an invalid value.
  InvalidValue {
    /// Name of the environment variable with an invalid value.
    var: &'static str,
    /// Human-readable description of why the value is invalid.
    detail: String,
  },
}

impl core::fmt::Display for ConfigError {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    match self {
      Self::MissingVar(var) => write!(f, "required environment variable {var} is not set"),
      Self::InvalidValue { var, detail } => {
        write!(f, "invalid value for {var}: {detail}")
      }
    }
  }
}

impl core::error::Error for ConfigError {}

/// AES-256 master key read from the environment, stored as a `Zeroizing<String>` so the
/// base64-encoded key material is zeroed when `Config` is dropped.
#[derive(Clone)]
pub struct MasterKey(pub(crate) Zeroizing<String>);

impl core::fmt::Debug for MasterKey {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    f.write_str("<redacted>")
  }
}

impl core::ops::Deref for MasterKey {
  type Target = str;
  fn deref(&self) -> &str {
    self.0.as_str()
  }
}

/// Server-side LLM proxy configuration.
///
/// All values are loaded from environment variables.
#[derive(Debug)]
pub struct Config {
  /// Port to listen on. Default: 8081. Set via `PROXY_PORT`.
  pub port: u16,

  /// Bind address. Default: 127.0.0.1 (safe behind nginx).
  /// Use 0.0.0.0 only for Docker or direct-access deployments.
  /// Set via `BIND_ADDR`.
  pub bind_addr: String,

  /// `SQLite` database URL (shared with `iron_control_api`).
  /// Set via `DATABASE_URL`.
  pub database_url: String,

  /// Master encryption key for provider API keys (base64-encoded, 32 bytes for AES-256-GCM).
  /// Must match the `IRON_SECRETS_MASTER_KEY` used by `iron_control_api`.
  /// Set via `IRON_SECRETS_MASTER_KEY`.
  pub secrets_master_key: MasterKey,

  /// Whether to trust the `X-Real-IP` header for client IP detection.
  ///
  /// Set to `true` only when behind a trusted reverse proxy (nginx/HAProxy) that sets
  /// this header from `$remote_addr`. When `false`, the TCP connection IP is used.
  /// Default: `true` when `BIND_ADDR` is `127.0.0.1`, `false` otherwise.
  /// Override via `TRUST_PROXY_HEADERS=true|false`.
  pub trust_proxy_headers: bool,
}

impl Config {
  /// Load configuration from environment variables.
  ///
  /// # Errors
  /// Returns `ConfigError` if required variables are missing or have invalid values.
  pub fn from_env() -> Result<Self, ConfigError> {
    let bind_addr = env::var("BIND_ADDR").unwrap_or_else(|_| "127.0.0.1".to_string());

    let port = env::var("PROXY_PORT")
      .unwrap_or_else(|_| "8081".to_string())
      .parse::<u16>()
      .map_err(|e| ConfigError::InvalidValue {
        var: "PROXY_PORT",
        detail: e.to_string(),
      })?;

    let database_url =
      env::var("DATABASE_URL").map_err(|_| ConfigError::MissingVar("DATABASE_URL"))?;

    let master_key_str = env::var("IRON_SECRETS_MASTER_KEY")
      .map_err(|_| ConfigError::MissingVar("IRON_SECRETS_MASTER_KEY"))?;
    let secrets_master_key = MasterKey(Zeroizing::new(master_key_str));

    let trust_proxy_headers = env::var("TRUST_PROXY_HEADERS")
      .map_or_else(|_| bind_addr == "127.0.0.1", |v| v.eq_ignore_ascii_case("true") || v == "1");

    Ok(Self {
      port,
      bind_addr,
      database_url,
      secrets_master_key,
      trust_proxy_headers,
    })
  }
}
