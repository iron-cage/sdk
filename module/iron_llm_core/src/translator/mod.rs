//! Request/Response translation between `OpenAI` and other provider formats
//!
//! Enables using `OpenAI` client SDK with Claude and Gemini models by translating
//! request/response formats transparently.

mod request;
mod response;

pub use request::{extract_model, translate_openai_to_anthropic, translate_openai_to_gemini};
pub use response::{translate_anthropic_to_openai, translate_gemini_to_openai};
