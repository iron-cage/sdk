//! Analytics event types and payloads.

use crate::provider_utils::current_time_ms;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

/// Unique event identifier for deduplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(Uuid);

impl EventId {
  /// Create a new random event identifier.
  #[must_use]
  pub fn new() -> Self {
    Self(Uuid::new_v4())
  }

  /// Get UUID as string
  #[must_use]
  pub fn to_uuid_string(&self) -> String {
    self.0.to_string()
  }
}

impl core::fmt::Display for EventId {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    write!(f, "{}", self.0)
  }
}

impl Default for EventId {
  fn default() -> Self {
    Self::new()
  }
}

/// Analytics event with metadata and typed payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
  #[serde(flatten)]
  metadata: EventMetadata,

  /// The event payload describing what happened.
  #[serde(flatten)]
  pub payload: EventPayload,
}

impl AnalyticsEvent {
  /// Create a new analytics event with the given payload.
  #[must_use]
  pub fn new(payload: EventPayload) -> Self {
    Self {
      metadata: EventMetadata::default(),
      payload,
    }
  }

  /// Builder: set `agent_id` on the event.
  #[must_use]
  pub fn with_agent_id(mut self, agent_id: Option<impl Into<Arc<str>>>) -> Self {
    self.metadata.agent_id = agent_id.map(Into::into);
    self
  }

  /// Returns the unique event identifier.
  #[must_use]
  pub fn event_id(&self) -> EventId {
    self.metadata.event_id
  }

  /// Returns the event timestamp in milliseconds since Unix epoch.
  #[must_use]
  pub fn timestamp_ms(&self) -> u64 {
    self.metadata.timestamp_ms
  }

  /// Returns whether this event has been synced to the dashboard.
  #[must_use]
  pub fn is_synced(&self) -> bool {
    self.metadata.synced
  }

  /// Set the synced flag on this event.
  pub fn set_synced(&mut self, val: bool) {
    self.metadata.synced = val;
  }

  /// Returns the agent identifier associated with this event, if any.
  #[must_use]
  pub fn agent_id(&self) -> Option<&Arc<str>> {
    self.metadata.agent_id.as_ref()
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventMetadata {
  #[serde(default = "EventId::new")]
  event_id: EventId,

  #[serde(skip)]
  synced: bool,

  #[serde(default = "current_time_ms")]
  timestamp_ms: u64,

  #[serde(skip_serializing_if = "Option::is_none")]
  agent_id: Option<Arc<str>>,
}

impl Default for EventMetadata {
  fn default() -> Self {
    Self {
      event_id: EventId::new(),
      synced: false,
      timestamp_ms: current_time_ms(),
      agent_id: None,
    }
  }
}

/// Common metadata for LLM requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmModelMeta {
  /// Optional provider-assigned identifier.
  pub provider_id: Option<Arc<str>>,
  /// Provider name (e.g. `"openai"`, `"anthropic"`).
  pub provider: Arc<str>,
  /// Model name (e.g. `"gpt-4"`, `"claude-3-opus"`).
  pub model: Arc<str>,
}

/// Data for successful LLM request completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmUsageData {
  /// Model and provider metadata.
  #[serde(flatten)]
  pub meta: LlmModelMeta,

  /// Number of input tokens consumed.
  pub input_tokens: u64,
  /// Number of output tokens generated.
  pub output_tokens: u64,
  /// Cost in microdollars (1 USD = `1_000_000` micros).
  pub cost_micros: u64,
}

/// Data for failed LLM request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmFailureData {
  /// Model and provider metadata.
  #[serde(flatten)]
  pub meta: LlmModelMeta,

  /// Optional machine-readable error code.
  pub error_code: Option<String>,
  /// Optional human-readable error message.
  pub error_message: Option<String>,
}

/// Event payload variants.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum EventPayload {
  /// A LLM request completed successfully.
  LlmRequestCompleted(LlmUsageData),

  /// A LLM request failed.
  LlmRequestFailed(LlmFailureData),

  /// A budget threshold was reached.
  BudgetThresholdReached {
    /// Threshold percentage that was reached (0–100).
    threshold_percent: u8,
    /// Current cumulative spend in microdollars.
    current_spend_micros: u64,
    /// Configured budget limit in microdollars.
    budget_micros: u64,
  },

  /// The proxy router started and is accepting connections.
  RouterStarted {
    /// Port the router is listening on.
    port: u16,
  },

  /// The proxy router stopped.
  RouterStopped {
    /// Total requests handled during the session.
    total_requests: u64,
    /// Total cost in microdollars during the session.
    total_cost_micros: u64,
  },
}
