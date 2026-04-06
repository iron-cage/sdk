//! Shared LLM routing core for Iron Cage
//!
//! This crate provides the common logic for forwarding LLM requests to providers
//! (`OpenAI`, Anthropic), including:
//!
//! - **Provider detection** from request path prefix or model name
//! - **Format translation** between `OpenAI` and Anthropic request/response formats
//! - **Request forwarding** with correct auth headers for each provider
//! - **Cost calculation** from request/response token usage
//!
//! Both `iron_runtime` (local embedded proxy) and `iron_server_proxy` (centralized
//! server proxy) depend on this crate, eliminating code duplication.

pub mod cost;
pub mod error;
pub mod forward;
pub mod provider;
pub mod translator;

// Re-export primary types for convenience
pub use cost::{calculate_request_cost, extract_model_from_body, CostInfo};
pub use error::{create_openai_error, LlmCoreError, OpenAiErrorDetail, OpenAiErrorResponse};
pub use forward::{forward_request, ForwardBody, ForwardRequest, ForwardResponse};
pub use provider::{detect_provider_from_model, strip_provider_prefix};
pub use translator::{
  translate_anthropic_to_openai, translate_gemini_to_openai, translate_openai_to_anthropic,
  translate_openai_to_gemini,
};
