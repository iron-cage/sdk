//! LLM Router - Local proxy for LLM API requests
//!
//! Provides a local HTTP proxy that intercepts OpenAI/Anthropic API requests,
//! fetches real API keys from Iron Cage server, and forwards requests to providers.

mod error;
mod key_fetcher;
mod proxy;
mod router;

pub use error::LlmRouterError;
pub use router::LlmRouter;
