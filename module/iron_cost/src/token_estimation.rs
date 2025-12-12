//! Helper utilities for cost estimation

use serde_json::Value;

/// Estimate input tokens from request JSON.
///
/// Counts characters in all message content and converts to tokens
/// using ~4 chars per token approximation. Adds 10% buffer for
/// message formatting overhead (role, system prompts, etc).
pub fn estimate_input_tokens(json: &Value) -> u64 {
    let mut total_chars = 0usize;

    // OpenAI format: messages array
    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            if let Some(content) = msg.get("content") {
                total_chars += count_content_chars(content);
            }
        }
    }

    // Anthropic format: system + messages
    if let Some(system) = json.get("system").and_then(|s| s.as_str()) {
        total_chars += system.len();
    }

    // If no messages found, use minimum buffer
    if total_chars == 0 {
        return 1000; // Minimum 1k tokens for unknown input
    }

    // Convert chars to tokens (~4 chars per token) with 10% buffer
    let estimated_tokens = (total_chars / 4) as u64;
    estimated_tokens + (estimated_tokens / 10) + 100 // +10% + 100 token overhead
}

/// Count characters in message content (handles string or array of content blocks)
pub fn count_content_chars(content: &Value) -> usize {
    match content {
        Value::String(s) => s.len(),
        Value::Array(arr) => {
            // Content blocks format (e.g., vision messages)
            arr.iter()
                .filter_map(|block| {
                    block.get("text").and_then(|t| t.as_str()).map(|s| s.len())
                })
                .sum()
        }
        _ => 0,
    }
}
