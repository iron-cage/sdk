//! Translate OpenAI chat completion request to Anthropic messages format

use serde_json::{json, Value};

/// Translate OpenAI /v1/chat/completions request to Anthropic /v1/messages format
///
/// Key transformations:
/// - Extract system prompt from messages array to separate `system` field
/// - Map `stop` to `stop_sequences`
/// - Ensure `max_tokens` is present (required by Anthropic)
pub fn translate_openai_to_anthropic( openai_body: &[ u8 ] ) -> Result< Vec< u8 >, String >
{
  let openai: Value =
    serde_json::from_slice( openai_body ).map_err( | e | format!( "Invalid JSON: {}", e ) )?;

  let messages = openai[ "messages" ]
    .as_array()
    .ok_or( "Missing 'messages' array" )?;

  // Extract system prompt and filter non-system messages
  let mut system_prompt: Option< String > = None;
  let mut user_messages: Vec< Value > = Vec::new();

  for msg in messages
  {
    let role = msg[ "role" ].as_str().unwrap_or( "" );
    if role == "system"
    {
      // Concatenate multiple system messages if present
      let content = msg[ "content" ].as_str().unwrap_or( "" );
      system_prompt = Some( match system_prompt
      {
        Some( existing ) => format!( "{}\n{}", existing, content ),
        None => content.to_string(),
      } );
    }
    else
    {
      user_messages.push( msg.clone() );
    }
  }

  // Build Anthropic request
  let mut anthropic = json!( {
    "model": openai[ "model" ],
    "messages": user_messages,
    "max_tokens": openai.get( "max_tokens" )
      .or_else( || openai.get( "max_completion_tokens" ) )
      .unwrap_or( &json!( 4096 ) ),
  } );

  // Add system prompt if present
  if let Some( system ) = system_prompt
  {
    anthropic[ "system" ] = json!( system );
  }

  // Map optional parameters
  if let Some( temp ) = openai.get( "temperature" )
  {
    anthropic[ "temperature" ] = temp.clone();
  }

  if let Some( top_p ) = openai.get( "top_p" )
  {
    anthropic[ "top_p" ] = top_p.clone();
  }

  // Map stop -> stop_sequences
  if let Some( stop ) = openai.get( "stop" )
  {
    if stop.is_array()
    {
      anthropic[ "stop_sequences" ] = stop.clone();
    }
    else if stop.is_string()
    {
      anthropic[ "stop_sequences" ] = json!( [ stop ] );
    }
  }

  serde_json::to_vec( &anthropic ).map_err( | e | format!( "Serialization error: {}", e ) )
}

#[ cfg( test ) ]
mod tests
{
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

    assert_eq!( anthropic[ "model" ], "claude-sonnet-4-20250514" );
    assert_eq!( anthropic[ "max_tokens" ], 100 );
    assert_eq!( anthropic[ "messages" ][ 0 ][ "role" ], "user" );
    assert!( anthropic.get( "system" ).is_none() );
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

    assert_eq!( anthropic[ "system" ], "You are helpful" );
    assert_eq!( anthropic[ "messages" ].as_array().unwrap().len(), 1 );
    assert_eq!( anthropic[ "messages" ][ 0 ][ "role" ], "user" );
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

    assert_eq!( anthropic[ "stop_sequences" ], json!( [ "END", "STOP" ] ) );
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

    assert_eq!( anthropic[ "stop_sequences" ], json!( [ "END" ] ) );
  }
}
