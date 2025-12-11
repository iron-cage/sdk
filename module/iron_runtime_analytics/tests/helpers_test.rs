//! Tests for helper functions and Provider enum

use iron_runtime_analytics::helpers::{infer_provider, Provider};
use std::sync::Arc;

// ============================================================================
// Provider Enum
// ============================================================================

#[test]
fn test_provider_as_str() {
    assert_eq!(Provider::OpenAI.as_str(), "openai");
    assert_eq!(Provider::Anthropic.as_str(), "anthropic");
    assert_eq!(Provider::Unknown.as_str(), "unknown");
}

#[test]
fn test_provider_parse() {
    // Standard FromStr via .parse()
    assert_eq!("openai".parse::<Provider>().unwrap(), Provider::OpenAI);
    assert_eq!("OpenAI".parse::<Provider>().unwrap(), Provider::OpenAI);
    assert_eq!("OPENAI".parse::<Provider>().unwrap(), Provider::OpenAI);
    assert_eq!("anthropic".parse::<Provider>().unwrap(), Provider::Anthropic);
    assert_eq!("Anthropic".parse::<Provider>().unwrap(), Provider::Anthropic);
    assert_eq!("other".parse::<Provider>().unwrap(), Provider::Unknown);
    assert_eq!("".parse::<Provider>().unwrap(), Provider::Unknown);
}

#[test]
fn test_provider_from_str_conversion() {
    // From<&str> conversion
    let p: Provider = "openai".into();
    assert_eq!(p, Provider::OpenAI);

    let p: Provider = "unknown_provider".into();
    assert_eq!(p, Provider::Unknown);
}

#[test]
fn test_provider_display() {
    assert_eq!(format!("{}", Provider::OpenAI), "openai");
    assert_eq!(format!("{}", Provider::Anthropic), "anthropic");
    assert_eq!(format!("{}", Provider::Unknown), "unknown");
}

#[test]
fn test_provider_default() {
    assert_eq!(Provider::default(), Provider::Unknown);
}

#[test]
fn test_provider_to_arc_str() {
    let arc: Arc<str> = Provider::OpenAI.to_arc_str();
    assert_eq!(arc.as_ref(), "openai");

    let arc: Arc<str> = Provider::Anthropic.into();
    assert_eq!(arc.as_ref(), "anthropic");
}

// ============================================================================
// infer_provider - OpenAI Models
// ============================================================================

#[test]
fn test_infer_provider_openai_gpt() {
    assert_eq!(infer_provider("gpt-4"), Provider::OpenAI);
    assert_eq!(infer_provider("gpt-4-turbo"), Provider::OpenAI);
    assert_eq!(infer_provider("gpt-4o"), Provider::OpenAI);
    assert_eq!(infer_provider("gpt-4o-mini"), Provider::OpenAI);
    assert_eq!(infer_provider("gpt-3.5-turbo"), Provider::OpenAI);
    assert_eq!(infer_provider("GPT-4"), Provider::OpenAI);
}

#[test]
fn test_infer_provider_openai_reasoning() {
    assert_eq!(infer_provider("o1-preview"), Provider::OpenAI);
    assert_eq!(infer_provider("o1-mini"), Provider::OpenAI);
    assert_eq!(infer_provider("o3-mini"), Provider::OpenAI);
}

#[test]
fn test_infer_provider_openai_chatgpt() {
    assert_eq!(infer_provider("chatgpt-4o-latest"), Provider::OpenAI);
}

// ============================================================================
// infer_provider - Anthropic Models
// ============================================================================

#[test]
fn test_infer_provider_anthropic_claude3() {
    assert_eq!(infer_provider("claude-3-opus-20240229"), Provider::Anthropic);
    assert_eq!(infer_provider("claude-3-sonnet-20240229"), Provider::Anthropic);
    assert_eq!(infer_provider("claude-3-haiku-20240307"), Provider::Anthropic);
    assert_eq!(infer_provider("claude-3-5-sonnet-20241022"), Provider::Anthropic);
}

#[test]
fn test_infer_provider_anthropic_legacy() {
    assert_eq!(infer_provider("claude-2.1"), Provider::Anthropic);
    assert_eq!(infer_provider("claude-instant-1.2"), Provider::Anthropic);
}

#[test]
fn test_infer_provider_anthropic_case_insensitive() {
    assert_eq!(infer_provider("Claude-3-Opus"), Provider::Anthropic);
    assert_eq!(infer_provider("CLAUDE-3-SONNET"), Provider::Anthropic);
}

// ============================================================================
// infer_provider - Unknown Models
// ============================================================================

#[test]
fn test_infer_provider_unknown() {
    assert_eq!(infer_provider("llama-2-70b"), Provider::Unknown);
    assert_eq!(infer_provider("mistral-7b"), Provider::Unknown);
    assert_eq!(infer_provider("gemini-pro"), Provider::Unknown);
    assert_eq!(infer_provider("custom-model"), Provider::Unknown);
    assert_eq!(infer_provider(""), Provider::Unknown);
}