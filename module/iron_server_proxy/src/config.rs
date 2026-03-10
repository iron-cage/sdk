//! Server proxy configuration from CLI args and environment variables.

use clap::Parser;
use zeroize::Zeroizing;

/// AES-256 master key read from the environment, stored as a `Zeroizing<String>` so the
/// base64-encoded key material is zeroed when `Config` is dropped.
///
/// Implements `FromStr` for clap parsing and `Debug` as `<redacted>` to prevent accidental leaks.
#[derive(Clone)]
pub struct MasterKey(pub(crate) Zeroizing<String>);

impl core::str::FromStr for MasterKey {
  type Err = core::convert::Infallible;
  fn from_str(s: &str) -> Result<Self, Self::Err> {
    Ok(Self(Zeroizing::new(s.to_owned())))
  }
}

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
/// Values can be provided via CLI arguments or environment variables.
/// CLI arguments take priority over environment variables (clap default).
#[derive(Parser, Debug)]
#[command(
  name = "iron_server_proxy",
  about = "Server-side LLM proxy for Iron Cage"
)]
pub struct Config {
  /// Port to listen on.
  #[arg(long, env = "PROXY_PORT", default_value = "8081")]
  pub port: u16,

  /// Bind address (0.0.0.0 for external access, 127.0.0.1 for localhost only).
  #[arg(long, env = "BIND_ADDR", default_value = "0.0.0.0")]
  pub bind_addr: String,

  /// `SQLite` database URL (shared with `iron_control_api`).
  #[arg(long, env = "DATABASE_URL")]
  pub database_url: String,

  /// Master encryption key for provider API keys (base64-encoded, 32 bytes for AES-256-GCM).
  /// Must match the `IRON_SECRETS_MASTER_KEY` used by `iron_control_api`.
  #[arg(long, env = "IRON_SECRETS_MASTER_KEY")]
  pub secrets_master_key: MasterKey,
}
