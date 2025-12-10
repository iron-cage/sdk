//! Request/Response translation between OpenAI and Anthropic formats
//!
//! Enables using OpenAI client SDK with Claude models by translating
//! request/response formats transparently.

mod request;
mod response;

pub use request::translate_openai_to_anthropic;
pub use response::translate_anthropic_to_openai;
