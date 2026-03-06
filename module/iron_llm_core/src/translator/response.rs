//! Translate Anthropic messages response to `OpenAI` chat completion format

use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

/// Translate Anthropic `/v1/messages` response to `OpenAI` `/v1/chat/completions` format
///
/// Key transformations:
/// - Convert `content[0].text` to `choices[0].message.content`
/// - Convert `tool_use` blocks to `tool_calls` array
/// - Map `input_tokens`/`output_tokens` to `prompt_tokens`/`completion_tokens`
/// - Map `stop_reason` to `finish_reason`
/// - Add `OpenAI`-specific fields (`object`, `created`)
///
/// # Errors
///
/// Returns an error string if the input is not valid JSON, content is missing, or serialization fails.
pub fn translate_anthropic_to_openai(anthropic_body: &[u8]) -> Result<Vec<u8>, String> {
  let anthropic =
    serde_json::from_slice::<Value>(anthropic_body).map_err(|e| format!("Invalid JSON: {e}"))?;

  let content_array = anthropic["content"]
    .as_array()
    .ok_or("Missing 'content' array in response")?;

  // Extract text content and tool_use blocks separately
  let (text_content, tool_calls) = extract_content_and_tools(content_array);

  // Map stop_reason to finish_reason
  let finish_reason = match anthropic["stop_reason"].as_str() {
    Some("max_tokens") => "length",
    Some("tool_use") => "tool_calls",
    _ => "stop",
  };

  // Get current timestamp
  let created = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0);

  // Map usage tokens
  let usage = &anthropic["usage"];
  let prompt_tokens = usage["input_tokens"].as_i64().unwrap_or(0);
  let completion_tokens = usage["output_tokens"].as_i64().unwrap_or(0);

  // Build message object
  let mut message = json!({
    "role": "assistant",
    "content": text_content,
  });

  if !tool_calls.is_empty() {
    message["tool_calls"] = json!(tool_calls);
    // OpenAI convention: content is null when there are tool calls and no text
    if text_content.is_none() {
      message["content"] = Value::Null;
    }
  }

  // Build OpenAI response
  let openai = json!({
    "id": anthropic["id"],
    "object": "chat.completion",
    "created": created,
    "model": anthropic["model"],
    "choices": [{
      "index": 0,
      "message": message,
      "logprobs": null,
      "finish_reason": finish_reason
    }],
    "usage": {
      "prompt_tokens": prompt_tokens,
      "completion_tokens": completion_tokens,
      "total_tokens": prompt_tokens + completion_tokens
    },
    "system_fingerprint": null
  });

  serde_json::to_vec(&openai).map_err(|e| format!("Serialization error: {e}"))
}

/// Extract text content and `tool_use` blocks from Anthropic content array
///
/// Returns (optional text content, `tool_calls` vec in `OpenAI` format)
fn extract_content_and_tools(content_array: &[Value]) -> (Option<String>, Vec<Value>) {
  let mut text_parts = Vec::new();
  let mut tool_calls = Vec::new();

  for block in content_array {
    match block["type"].as_str() {
      Some("text") => {
        if let Some(text) = block["text"].as_str() {
          text_parts.push(text.to_string());
        }
      }
      Some("tool_use") => {
        let tool_call = json!({
          "id": block["id"],
          "type": "function",
          "function": {
            "name": block["name"],
            "arguments": block["input"].to_string(),
          }
        });
        tool_calls.push(tool_call);
      }
      _ => {}
    }
  }

  let text = if text_parts.is_empty() {
    None
  } else {
    Some(text_parts.join(""))
  };

  (text, tool_calls)
}
