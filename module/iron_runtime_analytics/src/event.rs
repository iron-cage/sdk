//! Analytics event types and payloads.

use serde::{ Deserialize, Serialize };
use std::sync::Arc;
use uuid::Uuid;
use crate::helpers::current_time_ms;

/// Unique event identifier for deduplication.
#[ derive( Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize ) ]
#[ serde( transparent ) ]
pub struct EventId( Uuid );

impl EventId
{
  pub fn new() -> Self { Self( Uuid::new_v4() ) }
}

impl Default for EventId
{
  fn default() -> Self { Self::new() }
}

/// Analytics event with metadata and typed payload.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct AnalyticsEvent
{
  #[ serde( flatten ) ]
  metadata : EventMetadata,

  #[ serde( flatten ) ]
  pub payload : EventPayload,
}

impl AnalyticsEvent
{
  pub fn new( payload : EventPayload ) -> Self
  {
    Self
    {
      metadata : EventMetadata::default(),
      payload,
    }
  }

  /// Builder: set agent_id on the event.
  #[ must_use ]
  pub fn with_agent_id( mut self, agent_id : Option< impl Into< Arc< str > > > ) -> Self
  {
    self.metadata.agent_id = agent_id.map( Into::into );
    self
  }

  pub fn event_id( &self ) -> EventId
  {
    self.metadata.event_id
  }

  pub fn timestamp_ms( &self ) -> u64
  {
    self.metadata.timestamp_ms
  }

  pub fn is_synced( &self ) -> bool
  {
    self.metadata.synced
  }

  pub fn set_synced( &mut self, val : bool )
  {
    self.metadata.synced = val;
  }

  pub fn agent_id( &self ) -> Option< &Arc< str > >
  {
    self.metadata.agent_id.as_ref()
  }
}

#[ derive( Debug, Clone, Serialize, Deserialize ) ]
struct EventMetadata
{
  #[ serde( default = "EventId::new" ) ]
  event_id : EventId,

  #[ serde( skip ) ]
  synced : bool,

  #[ serde( default = "current_time_ms" ) ]
  timestamp_ms : u64,

  #[ serde( skip_serializing_if = "Option::is_none" ) ]
  agent_id : Option< Arc< str > >,
}

impl Default for EventMetadata
{
  fn default() -> Self
  {
    Self
    {
      event_id : EventId::new(),
      synced : false,
      timestamp_ms : current_time_ms(),
      agent_id : None,
    }
  }
}

/// Common metadata for LLM requests.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct LlmModelMeta
{
  pub provider_id : Option< Arc< str > >,
  pub provider : Arc< str >,
  pub model : Arc< str >,
}

/// Data for successful LLM request completion.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct LlmUsageData
{
  #[ serde( flatten ) ]
  pub meta : LlmModelMeta,

  pub input_tokens : u64,
  pub output_tokens : u64,
  pub cost_micros : u64,
}

/// Data for failed LLM request.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
pub struct LlmFailureData
{
  #[ serde( flatten ) ]
  pub meta : LlmModelMeta,

  pub error_code : Option< String >,
  pub error_message : Option< String >,
}

/// Event payload variants.
#[ derive( Debug, Clone, Serialize, Deserialize ) ]
#[ serde( tag = "event_type" ) ]
pub enum EventPayload
{
  LlmRequestCompleted( LlmUsageData ),

  LlmRequestFailed( LlmFailureData ),

  BudgetThresholdReached
  {
    threshold_percent : u8,
    current_spend_micros : u64,
    budget_micros : u64,
  },

  RouterStarted
  {
    port : u16,
  },

  RouterStopped
  {
    total_requests : u64,
    total_cost_micros : u64,
  },
}
