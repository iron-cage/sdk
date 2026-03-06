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
      let content = extract_text_content(&msg["content"]);
      system_prompt = Some(match system_prompt {
        Some(existing) => format!("{existing}\n{content}"),
        None => content,
      });
    } else {
      // Translate message content (handles both string and multimodal array formats)
      let mut translated_msg = msg.clone();
      if let Some(content_array) = msg["content"].as_array() {
        translated_msg["content"] = translate_content_blocks(content_array);
      }
      user_messages.push(translated_msg);
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

/// Extract text from content field (handles both string and array formats)
fn extract_text_content(content: &Value) -> String {
  if let Some(s) = content.as_str() {
    return s.to_string();
  }
  if let Some(blocks) = content.as_array() {
    let texts: Vec<&str> = blocks
      .iter()
      .filter(|b| b["type"].as_str() == Some("text"))
      .filter_map(|b| b["text"].as_str())
      .collect();
    return texts.join("");
  }
  String::new()
}

/// Translate `OpenAI` multimodal content blocks to Anthropic format
///
/// Maps:
/// - `{"type":"text","text":"..."}` -> `{"type":"text","text":"..."}`
/// - `{"type":"image_url","image_url":{"url":"data:image/png;base64,..."}}` -> Anthropic image block
fn translate_content_blocks(blocks: &[Value]) -> Value {
  let mut anthropic_blocks = Vec::new();

  for block in blocks {
    match block["type"].as_str() {
      Some("text") => {
        anthropic_blocks.push(json!({
          "type": "text",
          "text": block["text"]
        }));
      }
      Some("image_url") => {
        if let Some(url) = block["image_url"]["url"].as_str() {
          if let Some((media_type, data)) = parse_data_url(url) {
            anthropic_blocks.push(json!({
              "type": "image",
              "source": {
                "type": "base64",
                "media_type": media_type,
                "data": data,
              }
            }));
          } else {
            // External URL - Anthropic supports url source type
            anthropic_blocks.push(json!({
              "type": "image",
              "source": {
                "type": "url",
                "url": url,
              }
            }));
          }
        }
      }
      _ => {
        // Pass through unknown block types
        anthropic_blocks.push(block.clone());
      }
    }
  }

  json!(anthropic_blocks)
}

/// Parse a data URL into (`media_type`, `base64_data`)
fn parse_data_url(url: &str) -> Option<(&str, &str)> {
  let rest = url.strip_prefix("data:")?;
  let (meta, data) = rest.split_once(',')?;
  let media_type = meta.strip_suffix(";base64")?;
  Some((media_type, data))
}
