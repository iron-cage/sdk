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

/// Translate Gemini `generateContent` response to `OpenAI` `/v1/chat/completions` format
///
/// Key transformations:
/// - Gemini `candidates[0].content.parts` → `choices[0].message.content` / `tool_calls`
/// - Gemini `functionCall` parts → `OpenAI` `tool_calls` array
/// - Gemini `usageMetadata` → `OpenAI` `usage`
/// - Gemini `finishReason` → `OpenAI` `finish_reason`
///
/// # Errors
///
/// Returns an error string if the input is not valid JSON or required fields are missing.
pub fn translate_gemini_to_openai(gemini_body: &[u8]) -> Result<Vec<u8>, String> {
  let gemini =
    serde_json::from_slice::<Value>(gemini_body).map_err(|e| format!("Invalid JSON: {e}"))?;

  let candidate = gemini["candidates"]
    .as_array()
    .and_then(|c| c.first())
    .ok_or("Missing 'candidates' array in Gemini response")?;

  let parts = candidate["content"]["parts"]
    .as_array()
    .ok_or("Missing 'content.parts' in Gemini response")?;

  // Extract text and function calls from parts
  let mut text_parts = Vec::new();
  let mut tool_calls = Vec::new();
  let mut call_counter = 0u32;

  for part in parts {
    if let Some(text) = part["text"].as_str() {
      text_parts.push(text.to_string());
    }
    if let Some(fc) = part.get("functionCall") {
      let tool_call = json!({
        "id": format!("call_{call_counter}"),
        "type": "function",
        "function": {
          "name": fc["name"],
          "arguments": fc.get("args").map_or_else(|| "{}".to_string(), ToString::to_string),
        }
      });
      tool_calls.push(tool_call);
      call_counter += 1;
    }
  }

  // Map finish reason — function calls take priority since Gemini may
  // return finishReason: "STOP" even when the response contains functionCall parts
  let finish_reason = if tool_calls.is_empty() {
    match candidate["finishReason"].as_str() {
      Some("MAX_TOKENS") => "length",
      _ => "stop",
    }
  } else {
    "tool_calls"
  };

  // Build message
  let text_content = if text_parts.is_empty() {
    None
  } else {
    Some(text_parts.join(""))
  };

  let mut message = json!({
    "role": "assistant",
    "content": text_content,
  });

  if !tool_calls.is_empty() {
    message["tool_calls"] = json!(tool_calls);
    if text_content.is_none() {
      message["content"] = Value::Null;
    }
  }

  // Map usage
  let usage_meta = &gemini["usageMetadata"];
  let prompt_tokens = usage_meta["promptTokenCount"].as_i64().unwrap_or(0);
  let completion_tokens = usage_meta["candidatesTokenCount"].as_i64().unwrap_or(0);

  let created = std::time::SystemTime::now()
    .duration_since(std::time::UNIX_EPOCH)
    .map(|d| d.as_secs())
    .unwrap_or(0);

  let openai = json!({
    "id": gemini.get("responseId").cloned().unwrap_or(json!("gemini-response")),
    "object": "chat.completion",
    "created": created,
    "model": candidate["content"]["model"].as_str()
      .or_else(|| gemini["modelVersion"].as_str())
      .unwrap_or("gemini"),
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
