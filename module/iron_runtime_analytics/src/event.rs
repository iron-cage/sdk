use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use crate::helpers::current_time_ms;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EventId(Uuid);

impl EventId {
    pub fn new() -> Self { Self(Uuid::new_v4()) }
}

impl Default for EventId {
    fn default() -> Self { Self::new() }
}

/// The main event structure.
/// serialization: "flattens" header and payload into one single JSON object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    #[serde(flatten)]
    metadata: EventMetadata,

    #[serde(flatten)]
    pub payload: EventPayload,
}

impl AnalyticsEvent {
    /// Convenience constructor
    pub fn new(payload: EventPayload) -> Self {
        Self {
            metadata: EventMetadata::default(),
            payload,
        }
    }

    pub fn event_id(&self) -> EventId {
        self.metadata.event_id
    }
    pub fn timestamp_ms(&self) -> u64 {
        self.metadata.timestamp_ms
    }

    pub fn is_synced(&self) -> bool {
        self.metadata.synced
    }

    pub fn set_synced(&mut self, val: bool) {
        self.metadata.synced = val;
    }

    pub fn agent_id(&self) -> Option<&Arc<str>> {
        self.metadata.agent_id.as_ref()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventMetadata {
    #[serde(default = "EventId::new")]
    pub event_id: EventId,

    #[serde(skip)]
    pub synced: bool,

    #[serde(default = "current_time_ms")]
    pub timestamp_ms: u64,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<Arc<str>>,
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


#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type")]
pub enum EventPayload {
    LlmRequestCompleted {
        provider_id: Option<Arc<str>>,
        provider: Arc<str>,
        model: Arc<str>,
        input_tokens: u64,
        output_tokens: u64,
        cost_micros: u64,
    },

    LlmRequestFailed {
        provider_id: Option<Arc<str>>,
        provider: Arc<str>,
        model: Arc<str>,
        error_code: Option<String>,
        error_message: Option<String>,
    },

    BudgetThresholdReached {
        threshold_percent: u8,
        current_spend_micros: u64,
        budget_micros: u64,
    },

    RouterStarted {
        port: u16,
    },

    RouterStopped {
        total_requests: u64,
        total_cost_micros: u64,
    },
}