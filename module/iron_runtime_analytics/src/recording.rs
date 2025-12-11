//! High-level recording API for EventStore.
//!
//! Provides ergonomic methods that handle timestamp generation, provider inference,
//! and cost calculation internally. Callers provide minimal required data.

use std::sync::Arc;

use iron_cost::pricing::PricingManager;

use crate::event::{ AnalyticsEvent, EventPayload, LlmFailureData, LlmModelMeta, LlmUsageData };
use crate::event_storage::EventStore;
use crate::helpers::{ infer_provider, Provider };

/// Build LLM metadata with automatic provider inference.
fn build_meta( model : &str, provider_id : Option< &str > ) -> LlmModelMeta
{
  let provider = infer_provider( model );
  LlmModelMeta
  {
    provider_id : provider_id.map( Arc::from ),
    provider : provider.to_arc_str(),
    model : Arc::from( model ),
  }
}

/// Build LLM metadata with explicit provider (skips inference).
fn build_meta_with_provider
(
  model : &str,
  provider : Provider,
  provider_id : Option< &str >,
)
-> LlmModelMeta
{
  LlmModelMeta
  {
    provider_id : provider_id.map( Arc::from ),
    provider : provider.to_arc_str(),
    model : Arc::from( model ),
  }
}

impl EventStore
{
  /// Record a successful LLM request completion.
  ///
  /// Automatically handles:
  /// - Provider inference from model name (gpt-* → openai, claude-* → anthropic)
  /// - Cost calculation via PricingManager (0 if model unknown)
  /// - Timestamp generation (via EventMetadata)
  /// - Event ID generation (UUID v4)
  pub fn record_llm_completed
  (
    &self,
    pricing : &PricingManager,
    model : &str,
    input_tokens : u64,
    output_tokens : u64,
    agent_id : Option< &str >,
    provider_id : Option< &str >,
  )
  {
    // Cost defaults to 0 for unknown models (safe for analytics)
    let cost = pricing
    .get( model )
    .map( | p | p.calculate_cost_micros( input_tokens, output_tokens ) )
    .unwrap_or( 0 );

    let event = AnalyticsEvent::new( EventPayload::LlmRequestCompleted( LlmUsageData
    {
      meta : build_meta( model, provider_id ),
      input_tokens,
      output_tokens,
      cost_micros : cost,
    }))
    .with_agent_id( agent_id.map( Arc::from ) );

    self.record( event );
  }

  /// Record LLM completion with explicit provider (skips inference).
  ///
  /// Use when provider is already known from context (e.g., router configuration).
  pub fn record_llm_completed_with_provider
  (
    &self,
    pricing : &PricingManager,
    model : &str,
    provider : Provider,
    input_tokens : u64,
    output_tokens : u64,
    agent_id : Option< &str >,
    provider_id : Option< &str >,
  )
  {
    let cost = pricing
    .get( model )
    .map( | p | p.calculate_cost_micros( input_tokens, output_tokens ) )
    .unwrap_or( 0 );

    let event = AnalyticsEvent::new( EventPayload::LlmRequestCompleted( LlmUsageData
    {
      meta : build_meta_with_provider( model, provider, provider_id ),
      input_tokens,
      output_tokens,
      cost_micros : cost,
    }))
    .with_agent_id( agent_id.map( Arc::from ) );

    self.record( event );
  }

  /// Record a failed LLM request.
  ///
  /// Failed requests increment request counters but not token/cost counters.
  pub fn record_llm_failed
  (
    &self,
    model : &str,
    agent_id : Option< &str >,
    provider_id : Option< &str >,
    error_code : Option< &str >,
    error_message : Option< &str >,
  )
  {
    let event = AnalyticsEvent::new( EventPayload::LlmRequestFailed( LlmFailureData
    {
      meta : build_meta( model, provider_id ),
      error_code : error_code.map( String::from ),
      error_message : error_message.map( String::from ),
    }))
    .with_agent_id( agent_id.map( Arc::from ) );

    self.record( event );
  }

  /// Record a budget threshold being reached.
  ///
  /// Emitted when spending crosses a configured threshold (e.g., 80%, 90%, 100%).
  pub fn record_budget_threshold
  (
    &self,
    threshold_percent : u8,
    current_spend_micros : u64,
    budget_micros : u64,
    agent_id : Option< &str >,
  )
  {
    let event = AnalyticsEvent::new( EventPayload::BudgetThresholdReached
    {
      threshold_percent,
      current_spend_micros,
      budget_micros,
    })
    .with_agent_id( agent_id.map( Arc::from ) );

    self.record( event );
  }

  /// Record router startup.
  pub fn record_router_started( &self, port : u16 )
  {
    self.record( AnalyticsEvent::new( EventPayload::RouterStarted { port } ) );
  }

  /// Record router shutdown with final statistics.
  pub fn record_router_stopped( &self )
  {
    let stats = self.stats();
    self.record( AnalyticsEvent::new( EventPayload::RouterStopped
    {
      total_requests : stats.total_requests,
      total_cost_micros : stats.total_cost_micros,
    }));
  }
}
