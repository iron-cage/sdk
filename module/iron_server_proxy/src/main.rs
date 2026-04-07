//! Iron Cage server-side LLM proxy.
//!
//! Accepts remote agent HTTP connections authenticated by IC Token,
//! resolves the assigned provider key from the database, decrypts it in memory,
//! and forwards requests to the LLM provider. No key material is ever sent to the agent.

use tracing_subscriber::EnvFilter;

use iron_server_proxy::{server, AppState, Config, ServerError};

#[tokio::main]
async fn main() -> Result<(), ServerError> {
  // Load .env file if present (for local development)
  dotenvy::dotenv().ok();

  // Initialize structured logging (RUST_LOG env filter)
  tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
    .init();

  // Load config from environment variables. Startup failure prints and exits
  // immediately — no useful recovery is possible without valid configuration.
  let config = Config::from_env().unwrap_or_else(|e| {
    eprintln!("Configuration error: {e}");
    std::process::exit(1);
  });

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
