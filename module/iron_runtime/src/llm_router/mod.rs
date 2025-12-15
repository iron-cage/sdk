//! LLM Router - Local proxy for LLM API requests
//!
//! Provides a local HTTP proxy that intercepts OpenAI/Anthropic API requests,
//! fetches real API keys from Iron Cage server, and forwards requests to providers.

mod error;
mod key_fetcher;
mod proxy;
mod router;
mod translator;

pub use error::LlmRouterError;
pub use key_fetcher::ProviderKey;
pub use router::LlmRouter;

// Re-export proxy utilities for testing
pub use proxy::detect_provider_from_model;
pub use proxy::strip_provider_prefix;

// Re-export translator functions for testing
pub use translator::translate_anthropic_to_openai;
pub use translator::translate_openai_to_anthropic;
