//! Translate `OpenAI` chat completion request to Anthropic messages format

use serde_json::{json, Value};

/// Translate `OpenAI` `/v1/chat/completions` request to Anthropic `/v1/messages` format
///
/// Key transformations:
/// - Extract system prompt from messages array to separate `system` field
/// - Map `stop` to `stop_sequences`
/// - Ensure `max_tokens` is present (required by Anthropic)
///
/// # Errors
///
/// Returns an error string if the input is not valid JSON or serialization fails.
pub fn translate_openai_to_anthropic(openai_body: &[u8]) -> Result<Vec<u8>, String> {
  let openai =
    serde_json::from_slice::<Value>(openai_body).map_err(|e| format!("Invalid JSON: {e}"))?;

  let messages = openai["messages"]
    .as_array()
    .ok_or("Missing 'messages' array")?;

  // Extract system prompt and filter non-system messages
  let mut system_prompt = None;
  let mut user_messages = Vec::new();

  for msg in messages {
    let role = msg["role"].as_str().unwrap_or("");
    if role == "system" {
      // Concatenate multiple system messages if present
      let content = msg["content"].as_str().unwrap_or("");
      system_prompt = Some(match system_prompt {
        Some(existing) => format!("{existing}\n{content}"),
        None => content.to_string(),
      });
    } else {
      user_messages.push(msg.clone());
    }
  }

  // Build Anthropic request
  let mut anthropic = json!({
    "model": openai["model"],
    "messages": user_messages,
    "max_tokens": openai.get("max_tokens")
      .or_else(|| openai.get("max_completion_tokens"))
      .unwrap_or(&json!(4096)),
  });

  // Add system prompt if present
  if let Some(system) = system_prompt {
    anthropic["system"] = json!(system);
  }

  // Map optional parameters
  if let Some(temp) = openai.get("temperature") {
    anthropic["temperature"] = temp.clone();
  }

  if let Some(top_p) = openai.get("top_p") {
    anthropic["top_p"] = top_p.clone();
  }

  // Map stop -> stop_sequences
  if let Some(stop) = openai.get("stop") {
    if stop.is_array() {
      anthropic["stop_sequences"] = stop.clone();
    } else if stop.is_string() {
      anthropic["stop_sequences"] = json!([stop]);
    }
  }

  serde_json::to_vec(&anthropic).map_err(|e| format!("Serialization error: {e}"))
}
