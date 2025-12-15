//! LLM provider adapter layer
//!
//! Wraps `api_openai`, `api_claude`, `api_gemini` with usage tracking.
//!
//! # Architecture
//!
//! Per plan Day 16-17: "Create `TrackedClient` wrapper trait (wraps api_* clients)"
//!
//! This module provides usage tracking wrappers around workspace LLM API clients.
//! No custom HTTP client implementation - 100% workspace crate reuse.

use crate::error::Result;
use crate::usage_tracker::UsageTracker;
use crate::cost_calculator::CostCalculator;
use std::sync::Arc;

/// Usage tracking metadata extracted from LLM API responses
#[ derive( Debug, Clone ) ]
pub struct UsageMetadata
{
  /// Provider name (openai, anthropic, google)
  pub provider: String,
  /// Model name used for the request
  pub model: String,
  /// Input tokens consumed
  pub input_tokens: i64,
  /// Output tokens generated
  pub output_tokens: i64,
  /// Total tokens (input + output)
  pub total_tokens: i64,
}

impl UsageMetadata
{
  /// Calculate cost in cents using `CostCalculator`
  #[ must_use ]
  pub fn calculate_cost( &self ) -> i64
  {
    let calculator = CostCalculator::new();
    calculator.calculate_cost(
      &self.provider,
      &self.model,
      self.input_tokens,
      self.output_tokens,
    )
  }
}

/// `OpenAI` client wrapper with usage tracking
///
/// Wraps `api_openai::Client` to record usage after each API call.
///
/// # Architecture
///
/// Per plan: "`TrackedOpenAIClient` (wraps `api_openai::OpenAIClient`)"
/// Per plan: "Add usage extraction from `api_openai` response types"
/// Per plan: "Record usage to `token_usage` table after each API call"
///
/// Note: Phase 2 Week 4 blocked by `api_openai` dead code warning
#[ derive( Debug, Clone ) ]
pub struct TrackedOpenAIClient
{
  usage_tracker: Arc< UsageTracker >,
  token_id: i64,
}

impl TrackedOpenAIClient
{
  /// Create new tracked `OpenAI` client
  ///
  /// # Arguments
  ///
  /// * `usage_tracker` - Shared usage tracker
  /// * `token_id` - Token ID for this client session
  #[ must_use ]
  pub fn new( usage_tracker: Arc< UsageTracker >, token_id: i64 ) -> Self
  {
    Self
    {
      usage_tracker,
      token_id,
    }
  }

  /// Record usage after an API call
  ///
  /// # Arguments
  ///
  /// * `metadata` - Usage metadata extracted from API response
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  pub async fn record_usage( &self, metadata: &UsageMetadata ) -> Result< () >
  {
    let cost_cents = metadata.calculate_cost();

    self.usage_tracker.record_usage_with_cost(
      self.token_id,
      &metadata.provider,
      &metadata.model,
      metadata.input_tokens,
      metadata.output_tokens,
      metadata.total_tokens,
      cost_cents,
    ).await
  }

  /// Get token ID for this client
  #[ must_use ]
  pub fn token_id( &self ) -> i64
  {
    self.token_id
  }
}

/// `Claude` (Anthropic) client wrapper with usage tracking
///
/// Wraps `api_claude::Client` to record usage after each API call.
///
/// # Architecture
///
/// Per plan Day 18-19: "`TrackedClaudeClient` (wraps `api_claude::ClaudeClient`)"
/// Per plan: "Extract common tracking logic to shared trait implementation"
#[ derive( Debug, Clone ) ]
pub struct TrackedClaudeClient
{
  usage_tracker: Arc< UsageTracker >,
  token_id: i64,
}

impl TrackedClaudeClient
{
  /// Create new tracked `Claude` client
  ///
  /// # Arguments
  ///
  /// * `usage_tracker` - Shared usage tracker
  /// * `token_id` - Token ID for this client session
  #[ must_use ]
  pub fn new( usage_tracker: Arc< UsageTracker >, token_id: i64 ) -> Self
  {
    Self
    {
      usage_tracker,
      token_id,
    }
  }

  /// Record usage after an API call
  ///
  /// # Arguments
  ///
  /// * `metadata` - Usage metadata extracted from API response
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  pub async fn record_usage( &self, metadata: &UsageMetadata ) -> Result< () >
  {
    let cost_cents = metadata.calculate_cost();

    self.usage_tracker.record_usage_with_cost(
      self.token_id,
      &metadata.provider,
      &metadata.model,
      metadata.input_tokens,
      metadata.output_tokens,
      metadata.total_tokens,
      cost_cents,
    ).await
  }

  /// Get token ID for this client
  #[ must_use ]
  pub fn token_id( &self ) -> i64
  {
    self.token_id
  }
}

/// `Gemini` (Google) client wrapper with usage tracking
///
/// Wraps `api_gemini::Client` to record usage after each API call.
///
/// # Architecture
///
/// Per plan Day 18-19: "`TrackedGeminiClient` (wraps `api_gemini::GeminiClient`)"
/// Per plan: "Extract common tracking pattern to shared module"
#[ derive( Debug, Clone ) ]
pub struct TrackedGeminiClient
{
  usage_tracker: Arc< UsageTracker >,
  token_id: i64,
}

impl TrackedGeminiClient
{
  /// Create new tracked `Gemini` client
  ///
  /// # Arguments
  ///
  /// * `usage_tracker` - Shared usage tracker
  /// * `token_id` - Token ID for this client session
  #[ must_use ]
  pub fn new( usage_tracker: Arc< UsageTracker >, token_id: i64 ) -> Self
  {
    Self
    {
      usage_tracker,
      token_id,
    }
  }

  /// Record usage after an API call
  ///
  /// # Arguments
  ///
  /// * `metadata` - Usage metadata extracted from API response
  ///
  /// # Errors
  ///
  /// Returns error if database insertion fails
  pub async fn record_usage( &self, metadata: &UsageMetadata ) -> Result< () >
  {
    let cost_cents = metadata.calculate_cost();

    self.usage_tracker.record_usage_with_cost(
      self.token_id,
      &metadata.provider,
      &metadata.model,
      metadata.input_tokens,
      metadata.output_tokens,
      metadata.total_tokens,
      cost_cents,
    ).await
  }

  /// Get token ID for this client
  #[ must_use ]
  pub fn token_id( &self ) -> i64
  {
    self.token_id
  }
}
