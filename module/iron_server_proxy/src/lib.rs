//! Iron Cage server-side LLM proxy library.
//!
//! Provides the core components for running a server-side LLM proxy:
//! - [`Config`] - CLI/env configuration parsing
//! - [`AppState`] - shared application state (DB, crypto, HTTP client)
//! - [`server`] - HTTP server setup and graceful shutdown
//! - [`ServerError`] - initialization and runtime error types

pub mod config;
pub mod error;
pub mod proxy;
pub mod server;
pub mod state;

pub use config::Config;
pub use error::{ProxyError, ServerError};
pub use state::AppState;
