//! Translate `OpenAI` chat completion request to Anthropic messages format

use serde_json::{json, Value};

/// Translate `OpenAI` `/v1/chat/completions` request to Anthropic `/v1/messages` format
///
/// Key transformations:
/// - Extract system prompt from messages array to separate `system` field
/// - Translate `role: "tool"` messages to Anthropic `tool_result` content blocks (wrapped in a `user` turn)
/// - Translate `role: "assistant"` messages with `tool_calls` to Anthropic `tool_use` content blocks
/// - Map `stop` to `stop_sequences`
/// - Ensure `max_tokens` is present (required by Anthropic)
/// - Forward the `stream` flag to the Anthropic request body
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
    } else if role == "tool" {
      // OpenAI tool result → Anthropic user message with tool_result content block
      // OpenAI: {"role":"tool","tool_call_id":"call_xyz","content":"result text"}
      // Anthropic: {"role":"user","content":[{"type":"tool_result","tool_use_id":"call_xyz","content":"result text"}]}
      user_messages.push(json!({
        "role": "user",
        "content": [{
          "type": "tool_result",
          "tool_use_id": msg["tool_call_id"],
          "content": msg["content"],
        }]
      }));
    } else if role == "assistant" {
      if let Some(tool_calls) = msg["tool_calls"].as_array() {
        // OpenAI assistant tool call → Anthropic assistant message with tool_use content blocks
        // OpenAI: {"role":"assistant","tool_calls":[{"id":"call_xyz","type":"function","function":{"name":"fn","arguments":"{\"k\":\"v\"}"}}]}
        // Anthropic: {"role":"assistant","content":[{"type":"tool_use","id":"call_xyz","name":"fn","input":{...}}]}
        let mut content_blocks = Vec::new();

        // Carry over any text content alongside the tool calls
        let text = extract_text_content(&msg["content"]);
        if !text.is_empty() {
          content_blocks.push(json!({"type": "text", "text": text}));
        }

        for call in tool_calls {
          let input: Value = call["function"]["arguments"]
            .as_str()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(Value::Object(serde_json::Map::default()));
          content_blocks.push(json!({
            "type": "tool_use",
            "id": call["id"],
            "name": call["function"]["name"],
            "input": input,
          }));
        }

        user_messages.push(json!({"role": "assistant", "content": content_blocks}));
      } else {
        // Plain assistant message — translate content blocks as usual
        let mut translated_msg = msg.clone();
        if let Some(content_array) = msg["content"].as_array() {
          translated_msg["content"] = translate_content_blocks(content_array);
        }
        user_messages.push(translated_msg);
      }
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

  // Forward stream flag — Anthropic controls streaming via the request body, not query params
  if let Some(stream) = openai.get("stream") {
    anthropic["stream"] = stream.clone();
  }

  // Translate tools: OpenAI function.parameters → Anthropic input_schema
  if let Some(tools) = openai.get("tools").and_then(|t| t.as_array()) {
    let anthropic_tools: Vec<Value> = tools
      .iter()
      .filter_map(|tool| {
        let func = tool.get("function")?;
        Some(json!({
          "name": func["name"],
          "description": func.get("description").cloned().unwrap_or(Value::Null),
          "input_schema": func.get("parameters").cloned().unwrap_or(json!({"type": "object"})),
        }))
      })
      .collect();
    anthropic["tools"] = json!(anthropic_tools);
  }

  // Translate tool_choice:
  // OpenAI "none" → Anthropic {"type":"none"} (via omission — Anthropic defaults to auto)
  // OpenAI "auto" → Anthropic {"type":"auto"}
  // OpenAI "required" → Anthropic {"type":"any"}
  // OpenAI {"type":"function","function":{"name":"X"}} → Anthropic {"type":"tool","name":"X"}
  if let Some(tc) = openai.get("tool_choice") {
    let anthropic_tc = if let Some(s) = tc.as_str() {
      match s {
        "auto" => Some(json!({"type": "auto"})),
        "required" => Some(json!({"type": "any"})),
        "none" => Some(json!({"type": "none"})),
        _ => None,
      }
    } else if tc.is_object() {
      // {"type":"function","function":{"name":"X"}}
      tc["function"]["name"]
        .as_str()
        .map(|name| json!({"type": "tool", "name": name}))
    } else {
      None
    };
    if let Some(atc) = anthropic_tc {
      anthropic["tool_choice"] = atc;
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

/// Translate `OpenAI` `/v1/chat/completions` request to Gemini `generateContent` format
///
/// Key transformations:
/// - `OpenAI` `messages` → Gemini `contents` (role mapping: `assistant` → `model`)
/// - `OpenAI` `system` message → Gemini `systemInstruction`
/// - `OpenAI` `tools[].function` → Gemini `tools[].functionDeclarations[]`
/// - `OpenAI` `tool_choice` → Gemini `toolConfig.functionCallingConfig`
/// - `OpenAI` `max_tokens` → Gemini `generationConfig.maxOutputTokens`
///
/// # Errors
///
/// Returns an error string if the input is not valid JSON or serialization fails.
pub fn translate_openai_to_gemini(openai_body: &[u8]) -> Result<Vec<u8>, String> {
  let openai =
    serde_json::from_slice::<Value>(openai_body).map_err(|e| format!("Invalid JSON: {e}"))?;

  let messages = openai["messages"]
    .as_array()
    .ok_or("Missing 'messages' array")?;

  // Separate system messages and content messages
  let mut system_parts = Vec::new();
  let mut contents = Vec::new();

  for msg in messages {
    let role = msg["role"].as_str().unwrap_or("");
    match role {
      "system" => {
        let text = extract_text_content(&msg["content"]);
        if !text.is_empty() {
          system_parts.push(json!({"text": text}));
        }
      }
      "tool" => {
        // OpenAI tool result → Gemini functionResponse (role is "user" per Gemini API)
        contents.push(json!({
          "role": "user",
          "parts": [{
            "functionResponse": {
              "name": msg.get("name").and_then(|n| n.as_str()).unwrap_or("unknown"),
              "response": {
                "content": msg["content"],
              }
            }
          }]
        }));
      }
      "assistant" => {
        if let Some(tool_calls) = msg["tool_calls"].as_array() {
          // Assistant with tool calls → Gemini model turn with functionCall parts
          let mut parts = Vec::new();
          let text = extract_text_content(&msg["content"]);
          if !text.is_empty() {
            parts.push(json!({"text": text}));
          }
          for call in tool_calls {
            let args: Value = call["function"]["arguments"]
              .as_str()
              .and_then(|s| serde_json::from_str(s).ok())
              .unwrap_or(json!({}));
            parts.push(json!({
              "functionCall": {
                "name": call["function"]["name"],
                "args": args,
              }
            }));
          }
          contents.push(json!({"role": "model", "parts": parts}));
        } else {
          // Plain assistant message
          let text = extract_text_content(&msg["content"]);
          contents.push(json!({
            "role": "model",
            "parts": [{"text": text}]
          }));
        }
      }
      _ => {
        // user and other roles
        let text = extract_text_content(&msg["content"]);
        contents.push(json!({
          "role": "user",
          "parts": [{"text": text}]
        }));
      }
    }
  }

  let mut gemini = json!({
    "contents": contents,
  });

  // System instruction
  if !system_parts.is_empty() {
    gemini["systemInstruction"] = json!({"parts": system_parts});
  }

  // Generation config
  let mut gen_config = serde_json::Map::new();
  if let Some(max_tokens) = openai
    .get("max_tokens")
    .or_else(|| openai.get("max_completion_tokens"))
  {
    gen_config.insert("maxOutputTokens".to_string(), max_tokens.clone());
  }
  if let Some(temp) = openai.get("temperature") {
    gen_config.insert("temperature".to_string(), temp.clone());
  }
  if let Some(top_p) = openai.get("top_p") {
    gen_config.insert("topP".to_string(), top_p.clone());
  }
  if let Some(stop) = openai.get("stop") {
    if stop.is_array() {
      gen_config.insert("stopSequences".to_string(), stop.clone());
    } else if stop.is_string() {
      gen_config.insert("stopSequences".to_string(), json!([stop]));
    }
  }
  if !gen_config.is_empty() {
    gemini["generationConfig"] = Value::Object(gen_config);
  }

  // Translate tools: OpenAI function → Gemini functionDeclarations
  if let Some(tools) = openai.get("tools").and_then(|t| t.as_array()) {
    let declarations: Vec<Value> = tools
      .iter()
      .filter_map(|tool| {
        let func = tool.get("function")?;
        let mut decl = json!({
          "name": func["name"],
        });
        if let Some(desc) = func.get("description") {
          decl["description"] = desc.clone();
        }
        if let Some(params) = func.get("parameters") {
          decl["parameters"] = params.clone();
        }
        Some(decl)
      })
      .collect();
    if !declarations.is_empty() {
      gemini["tools"] = json!([{"functionDeclarations": declarations}]);
    }
  }

  // Translate tool_choice → Gemini toolConfig.functionCallingConfig
  // OpenAI "auto" → Gemini mode: "AUTO"
  // OpenAI "required" → Gemini mode: "ANY"
  // OpenAI "none" → Gemini mode: "NONE"
  // OpenAI {"type":"function","function":{"name":"X"}} → mode: "ANY" + allowedFunctionNames: ["X"]
  if let Some(tc) = openai.get("tool_choice") {
    let tool_config = if let Some(s) = tc.as_str() {
      match s {
        "auto" => Some(json!({"functionCallingConfig": {"mode": "AUTO"}})),
        "required" => Some(json!({"functionCallingConfig": {"mode": "ANY"}})),
        "none" => Some(json!({"functionCallingConfig": {"mode": "NONE"}})),
        _ => None,
      }
    } else if tc.is_object() {
      tc["function"]["name"].as_str().map(|name| {
        json!({"functionCallingConfig": {"mode": "ANY", "allowedFunctionNames": [name]}})
      })
    } else {
      None
    };
    if let Some(tc_val) = tool_config {
      gemini["toolConfig"] = tc_val;
    }
  }

  serde_json::to_vec(&gemini).map_err(|e| format!("Serialization error: {e}"))
}

/// Extract the model name from an `OpenAI` request body
#[must_use]
pub fn extract_model(body: &[u8]) -> Option<String> {
  serde_json::from_slice::<Value>(body)
    .ok()
    .and_then(|v| v["model"].as_str().map(String::from))
}
