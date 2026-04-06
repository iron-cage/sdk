//! LLM Router - Local proxy for LLM API requests
//!
//! Provides a local HTTP proxy that intercepts OpenAI/Anthropic API requests,
//! fetches real API keys from Iron Cage server, and forwards requests to providers.
//!
//! Core routing logic (provider detection, format translation, forwarding) is
//! provided by `iron_llm_core`. This module adds local concerns: port binding,
//! key fetching, budget handshake, and analytics sync.

mod error;
mod key_fetcher;
mod proxy;
mod router;

pub use error::LlmRouterError;
pub use key_fetcher::KeyFetcher;
pub use router::LlmRouter;

// Re-export proxy config and server entry point
pub use proxy::{run_proxy, ProxyConfig};
