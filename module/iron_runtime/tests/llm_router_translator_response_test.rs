use iron_runtime::*;
  use super::*;

  #[ test ]
  fn test_basic_response_translation()
  {
    let anthropic = json!( {
      "id": "msg_123",
      "type": "message",
      "role": "assistant",
      "model": "claude-sonnet-4-20250514",
      "content": [ {
        "type": "text",
        "text": "Hello! How can I help?"
      } ],
      "stop_reason": "end_turn",
      "usage": {
        "input_tokens": 10,
        "output_tokens": 8
      }
    } );

    let result = translate_anthropic_to_openai( anthropic.to_string().as_bytes() ).unwrap();
    let openai: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( openai[ "id" ], "msg_123", "Response ID should be preserved from Anthropic format" );
    assert_eq!( openai[ "object" ], "chat.completion", "Object type must be 'chat.completion' for OpenAI compatibility" );
    assert_eq!( openai[ "model" ], "claude-sonnet-4-20250514", "Model name should be preserved from Anthropic response" );
    assert_eq!( openai[ "choices" ][ 0 ][ "message" ][ "role" ], "assistant", "Message role must be 'assistant' for assistant responses" );
    assert_eq!( openai[ "choices" ][ 0 ][ "message" ][ "content" ], "Hello! How can I help?", "Message content should be extracted from Anthropic text block" );
    assert_eq!( openai[ "choices" ][ 0 ][ "finish_reason" ], "stop", "Anthropic 'end_turn' should translate to OpenAI 'stop'" );
    assert_eq!( openai[ "usage" ][ "prompt_tokens" ], 10, "Input tokens should map to prompt_tokens" );
    assert_eq!( openai[ "usage" ][ "completion_tokens" ], 8, "Output tokens should map to completion_tokens" );
    assert_eq!( openai[ "usage" ][ "total_tokens" ], 18, "Total tokens should be sum of prompt and completion tokens (10 + 8)" );
  }

  #[ test ]
  fn test_max_tokens_finish_reason()
  {
    let anthropic = json!( {
      "id": "msg_123",
      "model": "claude-sonnet-4-20250514",
      "content": [ { "type": "text", "text": "Truncated..." } ],
      "stop_reason": "max_tokens",
      "usage": { "input_tokens": 5, "output_tokens": 100 }
    } );

    let result = translate_anthropic_to_openai( anthropic.to_string().as_bytes() ).unwrap();
    let openai: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( openai[ "choices" ][ 0 ][ "finish_reason" ], "length", "Anthropic 'max_tokens' should translate to OpenAI 'length' finish reason" );
  }

  #[ test ]
  fn test_multiple_content_blocks()
  {
    let anthropic = json!( {
      "id": "msg_123",
      "model": "claude-sonnet-4-20250514",
      "content": [
        { "type": "text", "text": "First part. " },
        { "type": "text", "text": "Second part." }
      ],
      "stop_reason": "end_turn",
      "usage": { "input_tokens": 5, "output_tokens": 10 }
    } );

    let result = translate_anthropic_to_openai( anthropic.to_string().as_bytes() ).unwrap();
    let openai: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( openai[ "choices" ][ 0 ][ "message" ][ "content" ], "First part. Second part.", "Multiple Anthropic text blocks should be concatenated into single OpenAI content string" );
  }
