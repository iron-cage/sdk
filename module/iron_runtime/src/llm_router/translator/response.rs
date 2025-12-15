//! Translate Anthropic messages response to OpenAI chat completion format

use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

/// Translate Anthropic /v1/messages response to OpenAI /v1/chat/completions format
///
/// Key transformations:
/// - Convert `content[0].text` to `choices[0].message.content`
/// - Map `input_tokens`/`output_tokens` to `prompt_tokens`/`completion_tokens`
/// - Map `stop_reason` to `finish_reason`
/// - Add OpenAI-specific fields (`object`, `created`)
pub fn translate_anthropic_to_openai( anthropic_body: &[ u8 ] ) -> Result< Vec< u8 >, String >
{
  let anthropic: Value =
    serde_json::from_slice( anthropic_body ).map_err( | e | format!( "Invalid JSON: {}", e ) )?;

  // Extract content - handle multiple content blocks by concatenating
  let content = extract_content( &anthropic )?;

  // Map stop_reason to finish_reason
  let finish_reason = match anthropic[ "stop_reason" ].as_str()
  {
    Some( "end_turn" ) => "stop",
    Some( "max_tokens" ) => "length",
    Some( "stop_sequence" ) => "stop",
    _ => "stop",
  };

  // Get current timestamp
  let created = SystemTime::now()
    .duration_since( UNIX_EPOCH )
    .map( | d | d.as_secs() )
    .unwrap_or( 0 );

  // Map usage tokens
  let usage = &anthropic[ "usage" ];
  let prompt_tokens = usage[ "input_tokens" ].as_i64().unwrap_or( 0 );
  let completion_tokens = usage[ "output_tokens" ].as_i64().unwrap_or( 0 );

  // Build OpenAI response
  let openai = json!( {
    "id": anthropic[ "id" ],
    "object": "chat.completion",
    "created": created,
    "model": anthropic[ "model" ],
    "choices": [ {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": content
      },
      "logprobs": null,
      "finish_reason": finish_reason
    } ],
    "usage": {
      "prompt_tokens": prompt_tokens,
      "completion_tokens": completion_tokens,
      "total_tokens": prompt_tokens + completion_tokens
    },
    "system_fingerprint": null
  } );

  serde_json::to_vec( &openai ).map_err( | e | format!( "Serialization error: {}", e ) )
}

/// Extract text content from Anthropic content blocks
fn extract_content( anthropic: &Value ) -> Result< String, String >
{
  let content_array = anthropic[ "content" ]
    .as_array()
    .ok_or( "Missing 'content' array in response" )?;

  let mut text_parts: Vec< &str > = Vec::new();

  for block in content_array
  {
    if block[ "type" ].as_str() == Some( "text" )
    {
      if let Some( text ) = block[ "text" ].as_str()
      {
        text_parts.push( text );
      }
    }
  }

  if text_parts.is_empty()
  {
    return Err( "No text content in response".to_string() );
  }

  Ok( text_parts.join( "" ) )
}

