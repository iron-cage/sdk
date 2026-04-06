//! HTTP server setup with axum router and graceful shutdown.

use core::net::SocketAddr;

use axum::{routing, Router};

use crate::{config::Config, error::ServerError, proxy, state::AppState};

/// Build the axum router with all routes.
///
/// Routes:
/// - `GET /health` - public health check (no auth)
/// - `ANY /v1/*path` - LLM proxy endpoint (IC Token auth required)
fn build_router(state: AppState) -> Router {
  Router::new()
    .route("/health", routing::get(proxy::handle_health))
    .route("/v1/{*path}", routing::any(proxy::handle_proxy))
    .with_state(state)
}

/// Start the HTTP server and block until shutdown signal (Ctrl+C).
///
/// # Errors
///
/// Returns error if binding to the address fails or the server encounters a fatal error.
pub async fn start_server(config: &Config, state: AppState) -> Result<(), ServerError> {
  let router = build_router(state);

  let addr = format!("{}:{}", config.bind_addr, config.port);
  let listener = tokio::net::TcpListener::bind(&addr)
    .await
    .map_err(|e| ServerError::Server(format!("Failed to bind {addr}: {e}")))?;
  tracing::info!("iron_server_proxy listening on {addr}");

  // into_make_service_with_connect_info enables ConnectInfo<SocketAddr>
  // for extracting the real TCP peer IP (not spoofable via headers).
  axum::serve(
    listener,
    router.into_make_service_with_connect_info::<SocketAddr>(),
  )
  .with_graceful_shutdown(shutdown_signal())
  .await
  .map_err(|e| ServerError::Server(format!("Server runtime error: {e}")))?;
  tracing::info!("Server shut down gracefully");

  Ok(())
}

/// Wait for shutdown signal (Ctrl+C or SIGTERM).
///
/// Handles both interactive Ctrl+C and systemd's SIGTERM (`systemctl stop`)
/// so `axum::serve::with_graceful_shutdown` can drain in-flight requests.
async fn shutdown_signal() {
  let ctrl_c = tokio::signal::ctrl_c();
  let mut sigterm =
    tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
      .expect("Failed to install SIGTERM handler");

  tokio::select! {
    _ = ctrl_c => {}
    _ = sigterm.recv() => {}
  }

  tracing::info!("Shutdown signal received, stopping...");
}
