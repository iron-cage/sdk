use iron_runtime::*;
  use super::*;

  #[ test ]
  fn test_basic_translation()
  {
    let openai = json!( {
      "model": "claude-sonnet-4-20250514",
      "messages": [
        { "role": "user", "content": "Hello" }
      ],
      "max_tokens": 100
    } );

    let result = translate_openai_to_anthropic( openai.to_string().as_bytes() ).unwrap();
    let anthropic: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( anthropic[ "model" ], "claude-sonnet-4-20250514", "Model name should be preserved in translation" );
    assert_eq!( anthropic[ "max_tokens" ], 100, "max_tokens should be preserved in translation" );
    assert_eq!( anthropic[ "messages" ][ 0 ][ "role" ], "user", "Message role should be preserved in translation" );
    assert!( anthropic.get( "system" ).is_none(), "System prompt should not be present when no system message provided" );
  }

  #[ test ]
  fn test_system_prompt_extraction()
  {
    let openai = json!( {
      "model": "claude-sonnet-4-20250514",
      "messages": [
        { "role": "system", "content": "You are helpful" },
        { "role": "user", "content": "Hello" }
      ],
      "max_tokens": 100
    } );

    let result = translate_openai_to_anthropic( openai.to_string().as_bytes() ).unwrap();
    let anthropic: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( anthropic[ "system" ], "You are helpful", "System message should be extracted to separate 'system' field for Anthropic API" );
    assert_eq!( anthropic[ "messages" ].as_array().unwrap().len(), 1, "System message should be removed from messages array after extraction" );
    assert_eq!( anthropic[ "messages" ][ 0 ][ "role" ], "user", "First message should be user message after system extraction" );
  }

  #[ test ]
  fn test_stop_sequences_array()
  {
    let openai = json!( {
      "model": "claude-sonnet-4-20250514",
      "messages": [ { "role": "user", "content": "Hi" } ],
      "stop": [ "END", "STOP" ]
    } );

    let result = translate_openai_to_anthropic( openai.to_string().as_bytes() ).unwrap();
    let anthropic: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( anthropic[ "stop_sequences" ], json!( [ "END", "STOP" ] ), "OpenAI 'stop' array should translate to Anthropic 'stop_sequences' array" );
  }

  #[ test ]
  fn test_stop_sequences_string()
  {
    let openai = json!( {
      "model": "claude-sonnet-4-20250514",
      "messages": [ { "role": "user", "content": "Hi" } ],
      "stop": "END"
    } );

    let result = translate_openai_to_anthropic( openai.to_string().as_bytes() ).unwrap();
    let anthropic: Value = serde_json::from_slice( &result ).unwrap();

    assert_eq!( anthropic[ "stop_sequences" ], json!( [ "END" ] ), "OpenAI 'stop' string should translate to Anthropic 'stop_sequences' array with single element" );
  }
