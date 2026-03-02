//! Iron Cage server-side LLM proxy.
//!
//! Accepts remote agent HTTP connections authenticated by IC Token,
//! resolves the assigned provider key from the database, decrypts it in memory,
//! and forwards requests to the LLM provider. No key material is ever sent to the agent.

use clap::Parser;

use iron_server_proxy::{server, AppState, Config, ServerError};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
  // Load .env file if present (for local development)
  dotenvy::dotenv().ok();

  // Initialize structured logging (RUST_LOG env filter)
  tracing_subscriber::fmt()
    .with_env_filter(
      tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
    )
    .init();

  // Parse config from CLI args + env vars
  let config = Config::parse();

  tracing::info!(
    port = config.port,
    bind_addr = %config.bind_addr,
    "Starting iron_server_proxy"
  );

  // Initialize shared state (DB pool, crypto, HTTP client, pricing)
  let state = AppState::new(&config).await?;

  tracing::info!("Database connected, migrations applied");

  // Start HTTP server (blocks until Ctrl+C)
  server::start_server(&config, state).await?;

  Ok(())
}
