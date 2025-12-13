//! Lock-free event-based analytics for Iron Runtime LLM proxy.
//!
//! Provides real-time usage tracking, cost analytics, and operational metrics
//! for the Iron Runtime dashboard. Uses atomic operations for lock-free event
//! recording and aggregation.
//!
//! # Purpose
//!
//! This crate handles all analytics operations for Iron Runtime:
//! - Real-time event capture (LLM requests, failures, system events)
//! - Lock-free statistical aggregation (per-model, per-provider)
//! - Event storage with deduplication
//! - Dashboard sync for real-time visualization
//! - Cost tracking and budget threshold alerts
//!
//! # Architecture
//!
//! Analytics follows an event-driven, lock-free design:
//!
//! 1. **Event Recording**: Capture typed events with automatic metadata (timestamp, IDs)
//! 2. **Storage**: Append-only event store with atomic operations
//! 3. **Aggregation**: Compute statistics on-demand from event stream
//! 4. **Sync**: Background thread sends events to dashboard (optional)
//!
//! All operations use atomic primitives to avoid locks in the critical path
//! of LLM request handling.
//!
//! # Key Types
//!
//! - [`AnalyticsEvent`] - Typed event with metadata (timestamp, event_id, agent_id)
//! - [`EventPayload`] - Event variants (LlmRequestCompleted, LlmRequestFailed, etc.)
//! - [`EventStore`] - Lock-free append-only event storage with deduplication
//! - [`ComputedStats`] - Aggregated statistics snapshot (totals, by-model, by-provider)
//! - [`ModelStats`] - Per-model usage statistics (requests, tokens, cost)
//! - [`SyncClient`] - Background sync to dashboard (feature: sync)
//!
//! # Public API
//!
//! ## Record Events
//!
//! ```rust,ignore
//! use iron_runtime_analytics::{ EventStore, EventPayload, LlmUsageData, LlmModelMeta };
//! use std::sync::Arc;
//!
//! let store = EventStore::new();
//!
//! // Record successful LLM request
//! let meta = LlmModelMeta {
//!   provider_id: Some(Arc::from("openai")),
//!   provider: Arc::from("openai"),
//!   model: Arc::from("gpt-4"),
//! };
//!
//! let usage = LlmUsageData {
//!   meta,
//!   input_tokens: 1000,
//!   output_tokens: 500,
//!   cost_micros: 15_000_000, // $0.015
//! };
//!
//! store.record_event(EventPayload::LlmRequestCompleted(usage));
//! ```
//!
//! ## Compute Statistics
//!
//! ```rust,ignore
//! use iron_runtime_analytics::EventStore;
//!
//! let store = EventStore::new();
//! // ... record events ...
//!
//! let stats = store.compute_stats();
//! println!("Total requests: {}", stats.total_requests);
//! println!("Success rate: {:.1}%", stats.success_rate());
//! println!("Total cost: ${:.4}", stats.total_cost_usd());
//! println!("Avg cost/request: ${:.6}", stats.avg_cost_per_request_usd());
//!
//! // Per-model breakdown
//! for (model, stats) in &stats.by_model {
//!   println!("{}: {} requests, ${:.4}", model, stats.request_count, stats.cost_usd());
//! }
//! ```
//!
//! ## Dashboard Sync
//!
//! ```rust,ignore
//! # #[cfg(feature = "sync")]
//! # {
//! use iron_runtime_analytics::{ SyncClient, SyncConfig };
//!
//! let config = SyncConfig {
//!   dashboard_url: "http://localhost:8080".to_string(),
//!   sync_interval_ms: 5000,
//! };
//!
//! let sync_handle = SyncClient::start(config)?;
//!
//! // Client runs in background, syncing events every 5 seconds
//! // Stop when done:
//! sync_handle.stop();
//! # Ok::<(), anyhow::Error>(())
//! # }
//! ```
//!
//! # Feature Flags
//!
//! - `enabled` - Enable analytics collection (disabled for minimal builds)
//! - `sync` - Enable background dashboard sync (requires `enabled`)
//!
//! # Performance
//!
//! Event recording is lock-free and allocation-minimal:
//! - Event append: Single atomic operation
//! - Deduplication: Lock-free concurrent hash map
//! - Statistics: Computed on-demand from event stream
//!
//! Typical overhead: <1μs per event in hot path.

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
