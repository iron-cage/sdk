//! Tests for `OpenAI` <-> Anthropic/Gemini request/response translation.

use serde_json::Value;

use iron_llm_core::translator::{
  translate_anthropic_to_openai, translate_gemini_to_openai, translate_openai_to_anthropic,
  translate_openai_to_gemini,
};

#[test]
fn basic_openai_to_anthropic() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["model"], "claude-3-opus-20240229");
  assert_eq!(anthropic["messages"][0]["role"], "user");
  assert_eq!(anthropic["messages"][0]["content"], "Hello!");
  // Default max_tokens should be set
  assert!(anthropic["max_tokens"].is_number());
}

#[test]
fn system_prompt_extracted() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [
      {"role": "system", "content": "You are a helpful assistant."},
      {"role": "user", "content": "Hello!"}
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  // System should be a top-level field, not in messages
  assert_eq!(anthropic["system"], "You are a helpful assistant.");

  // Messages should only contain the user message
  let messages = anthropic["messages"].as_array().unwrap();
  assert_eq!(messages.len(), 1);
  assert_eq!(messages[0]["role"], "user");
}

#[test]
fn multiple_system_messages_concatenated() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [
      {"role": "system", "content": "First instruction."},
      {"role": "system", "content": "Second instruction."},
      {"role": "user", "content": "Hello!"}
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  let system = anthropic["system"].as_str().unwrap();
  assert!(system.contains("First instruction."));
  assert!(system.contains("Second instruction."));
}

#[test]
fn stop_string_to_stop_sequences_array() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hi"}],
    "stop": "END"
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  let seqs = anthropic["stop_sequences"].as_array().unwrap();
  assert_eq!(seqs.len(), 1);
  assert_eq!(seqs[0], "END");
}

#[test]
fn stop_array_to_stop_sequences() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hi"}],
    "stop": ["END", "STOP"]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  let seqs = anthropic["stop_sequences"].as_array().unwrap();
  assert_eq!(seqs.len(), 2);
}

#[test]
fn temperature_and_top_p_forwarded() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hi"}],
    "temperature": 0.7,
    "top_p": 0.9
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["temperature"], 0.7);
  assert_eq!(anthropic["top_p"], 0.9);
}

#[test]
fn max_tokens_preserved() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_tokens": 1024
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["max_tokens"], 1024);
}

#[test]
fn max_completion_tokens_fallback() {
  let openai = serde_json::json!({
    "model": "claude-3-opus-20240229",
    "messages": [{"role": "user", "content": "Hi"}],
    "max_completion_tokens": 2048
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["max_tokens"], 2048);
}

#[test]
fn invalid_json_returns_error() {
  let result = translate_openai_to_anthropic(b"not json");
  assert!(result.is_err());
}

#[test]
fn missing_messages_returns_error() {
  let openai = serde_json::json!({"model": "claude-3-opus-20240229"});
  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap());
  assert!(result.is_err());
}

#[test]
fn basic_anthropic_to_openai() {
  let anthropic = serde_json::json!({
    "id": "msg_test123",
    "model": "claude-3-opus-20240229",
    "content": [{"type": "text", "text": "Hello!"}],
    "stop_reason": "end_turn",
    "usage": {
      "input_tokens": 10,
      "output_tokens": 5
    }
  });

  let result = translate_anthropic_to_openai(&serde_json::to_vec(&anthropic).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["object"], "chat.completion");
  assert_eq!(openai["choices"][0]["message"]["role"], "assistant");
  assert_eq!(openai["choices"][0]["message"]["content"], "Hello!");
  assert_eq!(openai["usage"]["prompt_tokens"], 10);
  assert_eq!(openai["usage"]["completion_tokens"], 5);
  assert_eq!(openai["usage"]["total_tokens"], 15);
}

#[test]
fn max_tokens_stop_reason_mapped_to_length() {
  let anthropic = serde_json::json!({
    "id": "msg_test",
    "model": "claude-3-opus-20240229",
    "content": [{"type": "text", "text": "truncated"}],
    "stop_reason": "max_tokens",
    "usage": {"input_tokens": 10, "output_tokens": 5}
  });

  let result = translate_anthropic_to_openai(&serde_json::to_vec(&anthropic).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["choices"][0]["finish_reason"], "length");
}

#[test]
fn multiple_content_blocks_concatenated() {
  let anthropic = serde_json::json!({
    "id": "msg_test",
    "model": "claude-3-opus-20240229",
    "content": [
      {"type": "text", "text": "First "},
      {"type": "text", "text": "Second"}
    ],
    "stop_reason": "end_turn",
    "usage": {"input_tokens": 10, "output_tokens": 5}
  });

  let result = translate_anthropic_to_openai(&serde_json::to_vec(&anthropic).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["choices"][0]["message"]["content"], "First Second");
}

#[test]
fn missing_content_returns_error() {
  let anthropic = serde_json::json!({
    "id": "msg_test",
    "model": "claude-3-opus-20240229",
    "stop_reason": "end_turn",
    "usage": {"input_tokens": 10, "output_tokens": 5}
  });

  let result = translate_anthropic_to_openai(&serde_json::to_vec(&anthropic).unwrap());
  assert!(result.is_err());
}

#[test]
fn anthropic_invalid_json_returns_error() {
  let result = translate_anthropic_to_openai(b"bad json");
  assert!(result.is_err());
}

#[test]
fn tool_role_message_translated_to_tool_result() {
  let openai = serde_json::json!({
    "model": "claude-3-5-sonnet-20241022",
    "messages": [
      {"role": "user", "content": "What's the weather?"},
      {
        "role": "assistant",
        "content": null,
        "tool_calls": [{
          "id": "call_abc",
          "type": "function",
          "function": {"name": "get_weather", "arguments": "{\"location\":\"SF\"}"}
        }]
      },
      {"role": "tool", "tool_call_id": "call_abc", "content": "72°F and sunny"}
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();
  let messages = anthropic["messages"].as_array().unwrap();

  // user message passes through as-is
  assert_eq!(messages[0]["role"], "user");

  // assistant message with tool_use block
  assert_eq!(messages[1]["role"], "assistant");
  let tool_use = &messages[1]["content"][0];
  assert_eq!(tool_use["type"], "tool_use");
  assert_eq!(tool_use["id"], "call_abc");
  assert_eq!(tool_use["name"], "get_weather");
  assert_eq!(tool_use["input"]["location"], "SF");

  // tool result translated to user message with tool_result block
  assert_eq!(messages[2]["role"], "user");
  let tool_result = &messages[2]["content"][0];
  assert_eq!(tool_result["type"], "tool_result");
  assert_eq!(tool_result["tool_use_id"], "call_abc");
  assert_eq!(tool_result["content"], "72°F and sunny");
}

#[test]
fn assistant_tool_calls_translated_to_tool_use_blocks() {
  let openai = serde_json::json!({
    "model": "claude-3-5-sonnet-20241022",
    "messages": [
      {
        "role": "assistant",
        "content": "Let me check that.",
        "tool_calls": [{
          "id": "call_xyz",
          "type": "function",
          "function": {"name": "lookup", "arguments": "{\"q\":\"rust\"}"}
        }]
      }
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();
  let content = anthropic["messages"][0]["content"].as_array().unwrap();

  // text block preserved before tool_use
  assert_eq!(content[0]["type"], "text");
  assert_eq!(content[0]["text"], "Let me check that.");
  assert_eq!(content[1]["type"], "tool_use");
  assert_eq!(content[1]["id"], "call_xyz");
  assert_eq!(content[1]["name"], "lookup");
  assert_eq!(content[1]["input"]["q"], "rust");
}

// ── Anthropic tools translation ──────────────────────────────────────

#[test]
fn tools_translated_to_anthropic_input_schema() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "What's the weather?"}],
    "tools": [{
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get current weather",
        "parameters": {
          "type": "object",
          "properties": {
            "location": {"type": "string", "description": "City name"}
          },
          "required": ["location"]
        }
      }
    }]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  let tools = anthropic["tools"].as_array().unwrap();
  assert_eq!(tools.len(), 1);
  assert_eq!(tools[0]["name"], "get_weather");
  assert_eq!(tools[0]["description"], "Get current weather");
  // OpenAI "parameters" → Anthropic "input_schema"
  assert_eq!(tools[0]["input_schema"]["type"], "object");
  assert_eq!(
    tools[0]["input_schema"]["properties"]["location"]["type"],
    "string"
  );
  assert_eq!(tools[0]["input_schema"]["required"][0], "location");
  // Must NOT have "parameters" key (that's OpenAI format)
  assert!(tools[0].get("parameters").is_none());
}

#[test]
fn tool_choice_auto_translated_to_anthropic() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "f", "parameters": {"type": "object"}}}],
    "tool_choice": "auto"
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["tool_choice"]["type"], "auto");
}

#[test]
fn tool_choice_required_translated_to_anthropic_any() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "f", "parameters": {"type": "object"}}}],
    "tool_choice": "required"
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["tool_choice"]["type"], "any");
}

#[test]
fn tool_choice_none_translated_to_anthropic() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "Hi"}],
    "tool_choice": "none"
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["tool_choice"]["type"], "none");
}

#[test]
fn tool_choice_specific_function_translated_to_anthropic() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "get_weather", "parameters": {"type": "object"}}}],
    "tool_choice": {"type": "function", "function": {"name": "get_weather"}}
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(anthropic["tool_choice"]["type"], "tool");
  assert_eq!(anthropic["tool_choice"]["name"], "get_weather");
}

#[test]
fn multiple_tools_translated_to_anthropic() {
  let openai = serde_json::json!({
    "model": "claude-opus-4-6",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [
      {"type": "function", "function": {"name": "tool_a", "description": "A", "parameters": {"type": "object"}}},
      {"type": "function", "function": {"name": "tool_b", "description": "B", "parameters": {"type": "object"}}}
    ]
  });

  let result = translate_openai_to_anthropic(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let anthropic = serde_json::from_slice::<Value>(&result).unwrap();

  let tools = anthropic["tools"].as_array().unwrap();
  assert_eq!(tools.len(), 2);
  assert_eq!(tools[0]["name"], "tool_a");
  assert_eq!(tools[1]["name"], "tool_b");
}

#[test]
fn anthropic_tool_use_response_translated_to_openai() {
  let anthropic = serde_json::json!({
    "id": "msg_tool",
    "model": "claude-opus-4-6",
    "content": [
      {"type": "tool_use", "id": "call_1", "name": "get_weather", "input": {"location": "SF"}}
    ],
    "stop_reason": "tool_use",
    "usage": {"input_tokens": 20, "output_tokens": 15}
  });

  let result = translate_anthropic_to_openai(&serde_json::to_vec(&anthropic).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["choices"][0]["finish_reason"], "tool_calls");
  let tool_calls = openai["choices"][0]["message"]["tool_calls"]
    .as_array()
    .unwrap();
  assert_eq!(tool_calls.len(), 1);
  assert_eq!(tool_calls[0]["id"], "call_1");
  assert_eq!(tool_calls[0]["type"], "function");
  assert_eq!(tool_calls[0]["function"]["name"], "get_weather");
  // content should be null when only tool_use blocks
  assert!(openai["choices"][0]["message"]["content"].is_null());
}

// ── Gemini request translation ───────────────────────────────────────

#[test]
fn basic_openai_to_gemini() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [
      {"role": "user", "content": "Hello!"}
    ],
    "max_tokens": 1024,
    "temperature": 0.7
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  let contents = gemini["contents"].as_array().unwrap();
  assert_eq!(contents.len(), 1);
  assert_eq!(contents[0]["role"], "user");
  assert_eq!(contents[0]["parts"][0]["text"], "Hello!");

  assert_eq!(gemini["generationConfig"]["maxOutputTokens"], 1024);
  assert_eq!(gemini["generationConfig"]["temperature"], 0.7);
}

#[test]
fn gemini_system_instruction() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [
      {"role": "system", "content": "You are helpful."},
      {"role": "user", "content": "Hi"}
    ]
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  // System should be in systemInstruction, not contents
  assert_eq!(
    gemini["systemInstruction"]["parts"][0]["text"],
    "You are helpful."
  );
  let contents = gemini["contents"].as_array().unwrap();
  assert_eq!(contents.len(), 1);
  assert_eq!(contents[0]["role"], "user");
}

#[test]
fn gemini_assistant_role_mapped_to_model() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [
      {"role": "user", "content": "Hi"},
      {"role": "assistant", "content": "Hello!"},
      {"role": "user", "content": "How are you?"}
    ]
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  let contents = gemini["contents"].as_array().unwrap();
  assert_eq!(contents[0]["role"], "user");
  assert_eq!(contents[1]["role"], "model"); // assistant → model
  assert_eq!(contents[1]["parts"][0]["text"], "Hello!");
  assert_eq!(contents[2]["role"], "user");
}

#[test]
fn gemini_tools_translated_to_function_declarations() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Weather?"}],
    "tools": [{
      "type": "function",
      "function": {
        "name": "get_weather",
        "description": "Get weather",
        "parameters": {
          "type": "object",
          "properties": {"location": {"type": "string"}},
          "required": ["location"]
        }
      }
    }]
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  let tools = gemini["tools"].as_array().unwrap();
  assert_eq!(tools.len(), 1);
  let decls = tools[0]["functionDeclarations"].as_array().unwrap();
  assert_eq!(decls.len(), 1);
  assert_eq!(decls[0]["name"], "get_weather");
  assert_eq!(decls[0]["description"], "Get weather");
  // Gemini uses "parameters" (same key as OpenAI function.parameters)
  assert_eq!(decls[0]["parameters"]["type"], "object");
  assert_eq!(
    decls[0]["parameters"]["properties"]["location"]["type"],
    "string"
  );
}

#[test]
fn gemini_tool_choice_auto() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "f", "parameters": {"type": "object"}}}],
    "tool_choice": "auto"
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(
    gemini["toolConfig"]["functionCallingConfig"]["mode"],
    "AUTO"
  );
}

#[test]
fn gemini_tool_choice_required() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "f", "parameters": {"type": "object"}}}],
    "tool_choice": "required"
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(
    gemini["toolConfig"]["functionCallingConfig"]["mode"],
    "ANY"
  );
}

#[test]
fn gemini_tool_choice_none() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Hi"}],
    "tool_choice": "none"
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(
    gemini["toolConfig"]["functionCallingConfig"]["mode"],
    "NONE"
  );
}

#[test]
fn gemini_tool_choice_specific_function() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Hi"}],
    "tools": [{"type": "function", "function": {"name": "get_weather", "parameters": {"type": "object"}}}],
    "tool_choice": {"type": "function", "function": {"name": "get_weather"}}
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(
    gemini["toolConfig"]["functionCallingConfig"]["mode"],
    "ANY"
  );
  assert_eq!(
    gemini["toolConfig"]["functionCallingConfig"]["allowedFunctionNames"][0],
    "get_weather"
  );
}

#[test]
fn gemini_stop_sequences_in_generation_config() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [{"role": "user", "content": "Hi"}],
    "stop": ["END", "STOP"]
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  let seqs = gemini["generationConfig"]["stopSequences"]
    .as_array()
    .unwrap();
  assert_eq!(seqs.len(), 2);
}

#[test]
fn gemini_tool_result_uses_user_role() {
  let openai = serde_json::json!({
    "model": "gemini-2.0-flash",
    "messages": [
      {"role": "user", "content": "Weather?"},
      {
        "role": "assistant",
        "content": null,
        "tool_calls": [{"id": "c1", "type": "function", "function": {"name": "get_weather", "arguments": "{\"loc\":\"SF\"}"}}]
      },
      {"role": "tool", "name": "get_weather", "tool_call_id": "c1", "content": "72°F"}
    ]
  });

  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap()).unwrap();
  let gemini = serde_json::from_slice::<Value>(&result).unwrap();

  let contents = gemini["contents"].as_array().unwrap();
  // Assistant with tool call → model role with functionCall
  assert_eq!(contents[1]["role"], "model");
  assert_eq!(
    contents[1]["parts"][0]["functionCall"]["name"],
    "get_weather"
  );
  assert_eq!(contents[1]["parts"][0]["functionCall"]["args"]["loc"], "SF");

  // Tool result → user role with functionResponse (per Gemini API spec)
  assert_eq!(contents[2]["role"], "user");
  assert_eq!(
    contents[2]["parts"][0]["functionResponse"]["name"],
    "get_weather"
  );
}

// ── Gemini response translation ──────────────────────────────────────

#[test]
fn basic_gemini_to_openai() {
  let gemini = serde_json::json!({
    "candidates": [{
      "content": {
        "role": "model",
        "parts": [{"text": "Hello from Gemini!"}]
      },
      "finishReason": "STOP"
    }],
    "usageMetadata": {
      "promptTokenCount": 10,
      "candidatesTokenCount": 5,
      "totalTokenCount": 15
    },
    "modelVersion": "gemini-2.0-flash"
  });

  let result = translate_gemini_to_openai(&serde_json::to_vec(&gemini).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["object"], "chat.completion");
  assert_eq!(openai["choices"][0]["message"]["role"], "assistant");
  assert_eq!(
    openai["choices"][0]["message"]["content"],
    "Hello from Gemini!"
  );
  assert_eq!(openai["choices"][0]["finish_reason"], "stop");
  assert_eq!(openai["usage"]["prompt_tokens"], 10);
  assert_eq!(openai["usage"]["completion_tokens"], 5);
  assert_eq!(openai["usage"]["total_tokens"], 15);
}

#[test]
fn gemini_max_tokens_finish_reason() {
  let gemini = serde_json::json!({
    "candidates": [{
      "content": {"role": "model", "parts": [{"text": "truncated"}]},
      "finishReason": "MAX_TOKENS"
    }],
    "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 5}
  });

  let result = translate_gemini_to_openai(&serde_json::to_vec(&gemini).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["choices"][0]["finish_reason"], "length");
}

#[test]
fn gemini_function_call_response_translated_to_openai() {
  let gemini = serde_json::json!({
    "candidates": [{
      "content": {
        "role": "model",
        "parts": [{
          "functionCall": {
            "name": "get_weather",
            "args": {"location": "San Francisco"}
          }
        }]
      },
      "finishReason": "STOP"
    }],
    "usageMetadata": {"promptTokenCount": 15, "candidatesTokenCount": 10}
  });

  let result = translate_gemini_to_openai(&serde_json::to_vec(&gemini).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  assert_eq!(openai["choices"][0]["finish_reason"], "tool_calls");
  let tool_calls = openai["choices"][0]["message"]["tool_calls"]
    .as_array()
    .unwrap();
  assert_eq!(tool_calls.len(), 1);
  assert_eq!(tool_calls[0]["type"], "function");
  assert_eq!(tool_calls[0]["function"]["name"], "get_weather");
  // arguments should be JSON string
  let args: Value =
    serde_json::from_str(tool_calls[0]["function"]["arguments"].as_str().unwrap()).unwrap();
  assert_eq!(args["location"], "San Francisco");
  // content should be null when only function calls
  assert!(openai["choices"][0]["message"]["content"].is_null());
}

#[test]
fn gemini_mixed_text_and_function_call() {
  let gemini = serde_json::json!({
    "candidates": [{
      "content": {
        "role": "model",
        "parts": [
          {"text": "Let me check. "},
          {"functionCall": {"name": "lookup", "args": {"q": "test"}}}
        ]
      },
      "finishReason": "STOP"
    }],
    "usageMetadata": {"promptTokenCount": 10, "candidatesTokenCount": 8}
  });

  let result = translate_gemini_to_openai(&serde_json::to_vec(&gemini).unwrap()).unwrap();
  let openai = serde_json::from_slice::<Value>(&result).unwrap();

  // Both text and tool_calls should be present
  assert_eq!(
    openai["choices"][0]["message"]["content"],
    "Let me check. "
  );
  let tool_calls = openai["choices"][0]["message"]["tool_calls"]
    .as_array()
    .unwrap();
  assert_eq!(tool_calls[0]["function"]["name"], "lookup");
}

#[test]
fn gemini_missing_candidates_returns_error() {
  let gemini = serde_json::json!({"usageMetadata": {}});
  let result = translate_gemini_to_openai(&serde_json::to_vec(&gemini).unwrap());
  assert!(result.is_err());
}

#[test]
fn gemini_invalid_json_returns_error() {
  let result = translate_gemini_to_openai(b"not json");
  assert!(result.is_err());
}

#[test]
fn gemini_openai_to_gemini_invalid_json_returns_error() {
  let result = translate_openai_to_gemini(b"bad");
  assert!(result.is_err());
}

#[test]
fn gemini_openai_to_gemini_missing_messages_returns_error() {
  let openai = serde_json::json!({"model": "gemini-2.0-flash"});
  let result = translate_openai_to_gemini(&serde_json::to_vec(&openai).unwrap());
  assert!(result.is_err());
}
