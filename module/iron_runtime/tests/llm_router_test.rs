//! Unit tests for LLM Router module

use iron_runtime::llm_router::ProviderKey;
use iron_runtime::llm_router::detect_provider_from_model;
use iron_runtime::llm_router::strip_provider_prefix;

// =============================================================================
// ProviderKey::detect_provider_from_key tests
// =============================================================================

#[test]
fn test_detect_provider_from_key_anthropic()
{
  assert_eq!( ProviderKey::detect_provider_from_key( "sk-ant-api03-xxx" ), "anthropic" );
  assert_eq!( ProviderKey::detect_provider_from_key( "sk-ant-abc123" ), "anthropic" );
}

#[test]
fn test_detect_provider_from_key_openai()
{
  assert_eq!( ProviderKey::detect_provider_from_key( "sk-proj-xxx" ), "openai" );
  assert_eq!( ProviderKey::detect_provider_from_key( "sk-abc123" ), "openai" );
}

#[test]
fn test_detect_provider_from_key_unknown_defaults_to_openai()
{
  // Unknown formats default to OpenAI
  assert_eq!( ProviderKey::detect_provider_from_key( "unknown-key" ), "openai" );
  assert_eq!( ProviderKey::detect_provider_from_key( "" ), "openai" );
}

// =============================================================================
// strip_provider_prefix tests
// =============================================================================

#[test]
fn test_strip_provider_prefix_anthropic()
{
  let ( path, provider ) = strip_provider_prefix( "/anthropic/v1/messages" );
  assert_eq!( path, "/v1/messages" );
  assert_eq!( provider, Some( "anthropic" ) );
}

#[test]
fn test_strip_provider_prefix_anthropic_root()
{
  let ( path, provider ) = strip_provider_prefix( "/anthropic" );
  assert_eq!( path, "/" );
  assert_eq!( provider, Some( "anthropic" ) );
}

#[test]
fn test_strip_provider_prefix_openai()
{
  let ( path, provider ) = strip_provider_prefix( "/openai/v1/chat/completions" );
  assert_eq!( path, "/v1/chat/completions" );
  assert_eq!( provider, Some( "openai" ) );
}

#[test]
fn test_strip_provider_prefix_openai_root()
{
  let ( path, provider ) = strip_provider_prefix( "/openai" );
  assert_eq!( path, "/" );
  assert_eq!( provider, Some( "openai" ) );
}

#[test]
fn test_strip_provider_prefix_no_prefix()
{
  let ( path, provider ) = strip_provider_prefix( "/v1/chat/completions" );
  assert_eq!( path, "/v1/chat/completions" );
  assert_eq!( provider, None );
}

#[test]
fn test_strip_provider_prefix_root()
{
  let ( path, provider ) = strip_provider_prefix( "/" );
  assert_eq!( path, "/" );
  assert_eq!( provider, None );
}

// =============================================================================
// detect_provider_from_model tests
// =============================================================================

#[test]
fn test_detect_provider_from_model_claude()
{
  let body = br#"{"model": "claude-sonnet-4-20250514", "messages": []}"#;
  assert_eq!( detect_provider_from_model( body ), Some( "anthropic" ) );
}

#[test]
fn test_detect_provider_from_model_claude_variants()
{
  let body1 = br#"{"model": "claude-3-opus-20240229"}"#;
  let body2 = br#"{"model": "claude-3-haiku-20240307"}"#;

  assert_eq!( detect_provider_from_model( body1 ), Some( "anthropic" ) );
  assert_eq!( detect_provider_from_model( body2 ), Some( "anthropic" ) );
}

#[test]
fn test_detect_provider_from_model_gpt()
{
  let body = br#"{"model": "gpt-5-nano", "messages": []}"#;
  assert_eq!( detect_provider_from_model( body ), Some( "openai" ) );
}

#[test]
fn test_detect_provider_from_model_gpt_variants()
{
  let body1 = br#"{"model": "gpt-4o"}"#;
  let body2 = br#"{"model": "gpt-4o-mini"}"#;
  let body3 = br#"{"model": "gpt-3.5-turbo"}"#;

  assert_eq!( detect_provider_from_model( body1 ), Some( "openai" ) );
  assert_eq!( detect_provider_from_model( body2 ), Some( "openai" ) );
  assert_eq!( detect_provider_from_model( body3 ), Some( "openai" ) );
}

#[test]
fn test_detect_provider_from_model_o1_o3()
{
  let body1 = br#"{"model": "o1-preview"}"#;
  let body2 = br#"{"model": "o3-mini"}"#;

  assert_eq!( detect_provider_from_model( body1 ), Some( "openai" ) );
  assert_eq!( detect_provider_from_model( body2 ), Some( "openai" ) );
}

#[test]
fn test_detect_provider_from_model_unknown()
{
  let body = br#"{"model": "llama-3.1-70b"}"#;
  assert_eq!( detect_provider_from_model( body ), None );
}

#[test]
fn test_detect_provider_from_model_no_model_field()
{
  let body = br#"{"messages": [{"role": "user", "content": "hi"}]}"#;
  assert_eq!( detect_provider_from_model( body ), None );
}

#[test]
fn test_detect_provider_from_model_invalid_json()
{
  let body = b"not valid json";
  assert_eq!( detect_provider_from_model( body ), None );
}

#[test]
fn test_detect_provider_from_model_empty()
{
  let body = b"";
  assert_eq!( detect_provider_from_model( body ), None );
}
