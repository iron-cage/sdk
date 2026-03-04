//! Tests for `OpenAI` <-> Anthropic request/response translation.

use serde_json::Value;

use iron_llm_core::translator::{translate_anthropic_to_openai, translate_openai_to_anthropic};

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
