//! Integration tests for `CostCalculator`
//!
//! Tests token-to-cost conversion using real provider pricing (`OpenAI`, Anthropic, Gemini).
//!
//! ## Test Matrix
//!
//! | Test Case | Scenario | Input | Expected | Status |
//! |-----------|----------|-------|----------|--------|
//! | `test_openai_gpt4_cost_calculation` | GPT-4 Turbo cost calculation | "openai", "gpt-4-turbo", 100k input, 50k output | 250 cents ($2.50) | ✅ |
//! | `test_openai_gpt35_cost_calculation` | GPT-3.5 Turbo cost calculation | "openai", "gpt-3.5-turbo", 200k input, 100k output | 25 cents ($0.25) | ✅ |
//! | `test_anthropic_claude_sonnet_cost_calculation` | Claude 3.5 Sonnet cost calculation | "anthropic", "claude-3-5-sonnet", 150k input, 75k output | 158 cents ($1.58, rounded) | ✅ |
//! | `test_anthropic_claude_opus_cost_calculation` | Claude 3 Opus cost calculation | "anthropic", "claude-3-opus", 50k input, 25k output | 263 cents ($2.63, rounded) | ✅ |
//! | `test_anthropic_claude_haiku_cost_calculation` | Claude 3 Haiku cost calculation | "anthropic", "claude-3-haiku", 400k input, 200k output | 35 cents ($0.35) | ✅ |
//! | `test_gemini_pro_cost_calculation` | Gemini 1.5 Pro cost calculation | "google", "gemini-1.5-pro", 300k input, 150k output | 113 cents ($1.13, rounded) | ✅ |
//! | `test_gemini_flash_cost_calculation` | Gemini 1.5 Flash cost calculation | "google", "gemini-1.5-flash", 500k input, 250k output | 11 cents ($0.11, rounded) | ✅ |
//! | `test_zero_tokens_returns_zero_cost` | Zero tokens produces zero cost | Any provider/model, 0 input, 0 output | 0 cents | ✅ |
//! | `test_unknown_provider_returns_zero` | Unknown provider produces zero cost | "`unknown_provider`", any model, 100k input, 50k output | 0 cents | ✅ |
//! | `test_unknown_model_returns_zero` | Unknown model produces zero cost | "`openai`", "`unknown_model`", 100k input, 50k output | 0 cents | ✅ |
//! | `test_get_provider_models` | Provider model enumeration | "openai" | Non-empty list with gpt-4-turbo, gpt-3.5-turbo | ✅ |
//! | `test_get_all_providers` | All providers enumeration | None | ≥3 providers (openai, anthropic, google) | ✅ |
//!
//! ## Corner Cases Covered
//!
//! **Happy Path:**
//! - ✅ `OpenAI` pricing (GPT-4 Turbo, GPT-3.5 Turbo)
//! - ✅ `Anthropic` pricing (Claude 3.5 Sonnet, Claude 3 Opus, Claude 3 Haiku)
//! - ✅ Gemini pricing (Gemini 1.5 Pro, Gemini 1.5 Flash)
//! - ✅ Provider/model enumeration
//!
//! **Boundary Conditions:**
//! - ✅ Zero tokens (0 input, 0 output) → 0 cost
//! - ✅ Large token counts (500k tokens, valid cost calculation)
//! - ✅ Fractional cents (rounding to nearest cent)
//!
//! **Error Conditions:**
//! - ✅ Unknown provider → 0 cost (graceful degradation)
//! - ✅ Unknown model → 0 cost (graceful degradation)
//!
//! **Edge Cases:**
//! - ✅ Rounding behavior (157.5 cents → 158, 262.5 cents → 263, 112.5 cents → 113, 11.25 cents → 11)
//! - ✅ Multiple providers with different pricing models
//! - ✅ Model name exact matching (full model IDs with dates)
//!
//! **State Transitions:** N/A (stateless calculator)
//! **Concurrent Access:** Not tested (calculator is stateless and thread-safe)
//! **Resource Limits:** Not applicable (arithmetic operations only)
//! **Precondition Violations:** None (all inputs valid, unknown providers/models handled gracefully)

use iron_token_manager::cost_calculator::CostCalculator;

#[ test ]
fn test_openai_gpt4_cost_calculation()
{
  let calculator = CostCalculator::new();

  // GPT-4 Turbo: $10.00/1M input, $30.00/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 100_000, 50_000 );

  // Expected: (100k * $10/1M) + (50k * $30/1M) = $1.00 + $1.50 = $2.50 = 250 cents
  assert_eq!( cost_cents, 250, "Should calculate GPT-4 Turbo cost correctly" );
}

#[ test ]
fn test_openai_gpt35_cost_calculation()
{
  let calculator = CostCalculator::new();

  // GPT-3.5 Turbo: $0.50/1M input, $1.50/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "openai", "gpt-3.5-turbo", 200_000, 100_000 );

  // Expected: (200k * $0.50/1M) + (100k * $1.50/1M) = $0.10 + $0.15 = $0.25 = 25 cents
  assert_eq!( cost_cents, 25, "Should calculate GPT-3.5 Turbo cost correctly" );
}

#[ test ]
fn test_anthropic_claude_sonnet_cost_calculation()
{
  let calculator = CostCalculator::new();

  // Claude 3.5 Sonnet: $3.00/1M input, $15.00/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "anthropic", "claude-3-5-sonnet-20241022", 150_000, 75_000 );

  // Expected: (150k * $3/1M) + (75k * $15/1M) = $0.45 + $1.125 = $1.575 = 157.5 cents
  // Round to 158 cents
  assert_eq!( cost_cents, 158, "Should calculate Claude 3.5 Sonnet cost correctly" );
}

#[ test ]
fn test_anthropic_claude_opus_cost_calculation()
{
  let calculator = CostCalculator::new();

  // Claude 3 Opus: $15.00/1M input, $75.00/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "anthropic", "claude-3-opus-20240229", 50_000, 25_000 );

  // Expected: (50k * $15/1M) + (25k * $75/1M) = $0.75 + $1.875 = $2.625 = 262.5 cents
  // Round to 263 cents
  assert_eq!( cost_cents, 263, "Should calculate Claude 3 Opus cost correctly" );
}

#[ test ]
fn test_anthropic_claude_haiku_cost_calculation()
{
  let calculator = CostCalculator::new();

  // Claude 3 Haiku: $0.25/1M input, $1.25/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "anthropic", "claude-3-haiku-20240307", 400_000, 200_000 );

  // Expected: (400k * $0.25/1M) + (200k * $1.25/1M) = $0.10 + $0.25 = $0.35 = 35 cents
  assert_eq!( cost_cents, 35, "Should calculate Claude 3 Haiku cost correctly" );
}

#[ test ]
fn test_gemini_pro_cost_calculation()
{
  let calculator = CostCalculator::new();

  // Gemini 1.5 Pro: $1.25/1M input, $5.00/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "google", "gemini-1.5-pro", 300_000, 150_000 );

  // Expected: (300k * $1.25/1M) + (150k * $5/1M) = $0.375 + $0.75 = $1.125 = 112.5 cents
  // Round to 113 cents
  assert_eq!( cost_cents, 113, "Should calculate Gemini 1.5 Pro cost correctly" );
}

#[ test ]
fn test_gemini_flash_cost_calculation()
{
  let calculator = CostCalculator::new();

  // Gemini 1.5 Flash: $0.075/1M input, $0.30/1M output (as of Dec 2025)
  let cost_cents = calculator.calculate_cost( "google", "gemini-1.5-flash", 500_000, 250_000 );

  // Expected: (500k * $0.075/1M) + (250k * $0.30/1M) = $0.0375 + $0.075 = $0.1125 = 11.25 cents
  // Round to 11 cents
  assert_eq!( cost_cents, 11, "Should calculate Gemini 1.5 Flash cost correctly" );
}

#[ test ]
fn test_zero_tokens_returns_zero_cost()
{
  let calculator = CostCalculator::new();

  let cost_cents = calculator.calculate_cost( "openai", "gpt-4-turbo", 0, 0 );
  assert_eq!( cost_cents, 0, "Zero tokens should return zero cost" );
}

#[ test ]
fn test_unknown_provider_returns_zero()
{
  let calculator = CostCalculator::new();

  let cost_cents = calculator.calculate_cost( "unknown_provider", "unknown_model", 100_000, 50_000 );
  assert_eq!( cost_cents, 0, "Unknown provider should return zero cost" );
}

#[ test ]
fn test_unknown_model_returns_zero()
{
  let calculator = CostCalculator::new();

  let cost_cents = calculator.calculate_cost( "openai", "unknown_model", 100_000, 50_000 );
  assert_eq!( cost_cents, 0, "Unknown model should return zero cost" );
}

#[ test ]
fn test_get_provider_models()
{
  let calculator = CostCalculator::new();

  let openai_models = calculator.get_provider_models( "openai" );
  assert!( !openai_models.is_empty(), "OpenAI should have models" );
  assert!( openai_models.contains( &"gpt-4-turbo".to_string() ), "Should include gpt-4-turbo" );
  assert!( openai_models.contains( &"gpt-3.5-turbo".to_string() ), "Should include gpt-3.5-turbo" );
}

#[ test ]
fn test_get_all_providers()
{
  let calculator = CostCalculator::new();

  let providers = calculator.get_providers();
  assert!( providers.len() >= 3, "Should have at least 3 providers" );
  assert!( providers.contains( &"openai".to_string() ), "Should include openai" );
  assert!( providers.contains( &"anthropic".to_string() ), "Should include anthropic" );
  assert!( providers.contains( &"google".to_string() ), "Should include google" );
}
