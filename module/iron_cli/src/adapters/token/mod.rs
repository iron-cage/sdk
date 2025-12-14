//! Token Manager API adapters
//!
//! HTTP adapters for Token Manager API (iron_token_manager).
//!
//! ## Architecture
//!
//! ```text
//! Binary → Adapter → Handler → Formatter
//!   ↓         ↓         ↓          ↓
//! Route    HTTP I/O  Validate   Output
//! ```
//!
//! ## Modules
//!
//! - `config`: Token API configuration (URL, timeout)
//! - `http_client`: HTTP client for Token Manager API
//! - Auth adapters: Login, logout, refresh (in parent module)
//! - Token adapters: Generate, list, get, rotate, revoke
//! - Usage adapters: Show usage, export usage
//! - Limit adapters: Show limits, update limits, reset limit
//! - Trace adapters: List traces, get trace
//! - Health adapters: Health check, version
//!
//! ## Authentication
//!
//! Uses keyring-stored access tokens (not static API tokens).
//! See `super::keyring` module for token storage.

mod config;
mod http_client;

pub use config::TokenApiConfig;
pub use http_client::{ TokenApiClient, TokenApiError };
