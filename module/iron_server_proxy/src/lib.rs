//! Iron Cage server-side LLM proxy library.
//!
//! Provides the core components for running a server-side LLM proxy:
//! - [`Config`] - CLI/env configuration parsing
//! - [`AppState`] - shared application state (DB, crypto, HTTP client)
//! - [`server`] - HTTP server setup and graceful shutdown
//! - [`ServerError`] - initialization and runtime error types

#![cfg_attr(not(feature = "enabled"), allow(unused_variables, dead_code))]

#[cfg(feature = "enabled")]
pub mod config;
#[cfg(feature = "enabled")]
pub mod error;
#[cfg(feature = "enabled")]
pub mod proxy;
#[cfg(feature = "enabled")]
pub mod rate_limiter;
#[cfg(feature = "enabled")]
pub mod server;
#[cfg(feature = "enabled")]
pub mod state;

#[cfg(feature = "enabled")]
pub use config::Config;
#[cfg(feature = "enabled")]
pub use error::{ProxyError, ServerError};
#[cfg(feature = "enabled")]
pub use rate_limiter::AuthRateLimiter;
#[cfg(feature = "enabled")]
pub use state::AppState;
