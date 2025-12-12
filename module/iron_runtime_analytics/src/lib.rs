//! Lock-free event-based analytics for Python LlmRouter.

#![ cfg_attr( not( feature = "enabled" ), allow( unused_imports, unused_variables, dead_code ) ) ]

#[ cfg( feature = "enabled" ) ]
pub mod event;

#[ cfg( feature = "enabled" ) ]
pub mod provider_utils;

#[ cfg( feature = "enabled" ) ]
pub mod stats;

#[ cfg( feature = "enabled" ) ]
pub mod event_storage;

#[ cfg( feature = "enabled" ) ]
pub mod recording;

#[ cfg( feature = "sync" ) ]
pub mod sync;

// Re-exports: Flat access to common types

#[ cfg( feature = "enabled" ) ]
pub use event::{ AnalyticsEvent, EventId, EventPayload };

#[ cfg( feature = "enabled" ) ]
pub use event::{ LlmModelMeta, LlmUsageData, LlmFailureData };

#[ cfg( feature = "enabled" ) ]
pub use event_storage::EventStore;

#[ cfg( feature = "enabled" ) ]
pub use stats::{ ComputedStats, ModelStats };

#[ cfg( feature = "enabled" ) ]
pub use provider_utils::{ Provider, infer_provider, current_time_ms };

#[ cfg( feature = "sync" ) ]
pub use sync::{ SyncClient, SyncConfig, SyncHandle };
