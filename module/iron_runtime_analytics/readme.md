# iron_runtime_analytics

Lock-free event-based analytics for Python LlmRouter.

## Overview

This crate provides high-performance analytics storage for LLM request tracking. Designed for async contexts with concurrent LLM calls, it uses lock-free data structures to ensure non-blocking operation.

## Features

- **Lock-free storage** - crossbeam ArrayQueue for bounded event buffer
- **Atomic counters** - O(1) stats access without locks
- **Per-model/provider stats** - DashMap for concurrent aggregation
- **High-level recording API** - automatic provider inference and cost calculation
- **Protocol 012 compatible** - field compatibility with analytics API
- **Background sync** - server sync with auto-flush on shutdown (feature: `sync`)

## Background Sync

When enabled with the `sync` feature, events can be automatically synced to Control API:

```rust,ignore
use iron_runtime_analytics::{EventStore, SyncClient, SyncConfig};
use std::sync::Arc;

let store = Arc::new(EventStore::new());
let config = SyncConfig::new("http://localhost:3001", "ic_token_here")
    .with_interval(Duration::from_secs(30))  // Sync every 30s
    .with_batch_threshold(10);               // Or when 10 events pending

// Start background sync (requires tokio runtime handle)
let handle = client.start(&runtime_handle);

// ... use store normally ...

// Stop and flush remaining events
handle.stop();
```

### SyncConfig Options

| Option | Default | Description |
|--------|---------|-------------|
| `sync_interval` | 30s | How often to sync events |
| `batch_threshold` | 10 | Sync immediately when this many events pending |
| `timeout` | 30s | HTTP request timeout |

### Sync Behavior

- Events are synced in batches to `/api/v1/analytics/events`
- Only `llm_request_completed` and `llm_request_failed` events are synced
- On shutdown, remaining events are flushed before stopping
- Failed syncs are retried on next interval (except 4xx errors)

## Pilot Strategy

Simple, predictable behavior:

1. **Fixed Memory:** Bounded buffer (default 10,000 slots, ~2-5MB)
2. **Non-Blocking:** Drop new events when buffer full (never block)
3. **Observability:** `dropped_count()` tracks lost events

## Installation

```toml,ignore
[dependencies]
iron_runtime_analytics = { path = "../iron_runtime_analytics" }
iron_cost = { path = "../iron_cost" }  # For pricing
```

## Quick Start

### High-Level API (Recommended)

The high-level API handles provider inference and cost calculation automatically:

```rust,ignore
use iron_runtime_analytics::EventStore;
use iron_cost::pricing::PricingManager;

let store = EventStore::new();
let pricing = PricingManager::new().unwrap();

// Record successful LLM request - provider inferred from model name
store.record_llm_completed(&pricing, "gpt-4", 150, 50, None, None);

// Record with agent attribution
store.record_llm_completed(
    &pricing,
    "claude-3-opus-20240229",
    200,
    100,
    Some("agent_123"),      // agent_id
    Some("ip_anthropic-001"), // provider_id
);

// Record failed request
store.record_llm_failed("gpt-4", None, None, Some("rate_limit"), None);

// Record budget threshold
store.record_budget_threshold(80, 80_000_000, 100_000_000, None);

// Lifecycle events
store.record_router_started(8080);
store.record_router_stopped();  // Captures final stats automatically
```

### Low-Level API

For full control over event construction:

```rust,ignore
use iron_runtime_analytics::{EventStore, AnalyticsEvent, EventPayload};
use iron_runtime_analytics::event::{LlmUsageData, LlmModelMeta};

let store = EventStore::new();

store.record(AnalyticsEvent::new(EventPayload::LlmRequestCompleted(LlmUsageData {
    meta: LlmModelMeta {
        provider_id: Some("ip_openai-001".into()),
        provider: "openai".into(),
        model: "gpt-4".into(),
    },
    input_tokens: 150,
    output_tokens: 50,
    cost_micros: 6000,  // $0.006
})));
```

### Accessing Statistics

```rust,ignore
let stats = store.stats();

// Totals (O(1) access)
println!("Requests: {}", stats.total_requests);
println!("Cost: ${:.4}", stats.total_cost_usd());
println!("Success rate: {:.1}%", stats.success_rate() * 100.0);

// Per-model breakdown
for (model, model_stats) in &stats.by_model {
    println!("{}: {} requests, ${:.4}", model, model_stats.request_count, model_stats.cost_usd());
}

// Per-provider breakdown
for (provider, provider_stats) in &stats.by_provider {
    println!("{}: {} tokens", provider, provider_stats.input_tokens + provider_stats.output_tokens);
}
```

### Observability

```rust,ignore
// Check for dropped events (buffer overflow)
if store.dropped_count() > 0 {
    eprintln!("Warning: {} events dropped (buffer full)", store.dropped_count());
}

// Check unsynced events (pending server sync)
println!("Unsynced events: {}", store.unsynced_count());
```

### Event Streaming

```rust,ignore
use std::thread;

// Create store with streaming channel
let (store, receiver) = EventStore::with_streaming(10_000, 100);

// Spawn consumer thread
thread::spawn(move || {
    while let Ok(event) = receiver.recv() {
        // Process event (e.g., send to server)
        println!("Event: {:?}", event.event_id());
    }
});

// Events are automatically sent to channel when recorded
store.record_llm_completed(&pricing, "gpt-4", 100, 50, None, None);
```

## Module Structure

```text
src/
├── lib.rs           # Re-exports
├── event.rs         # AnalyticsEvent, EventPayload, LlmUsageData
├── event_storage.rs # EventStore (lock-free buffer + atomic counters)
├── stats.rs         # AtomicModelStats, ModelStats, ComputedStats
├── recording.rs     # High-level record_* methods
└── helpers.rs       # Provider enum, infer_provider, current_time_ms
```

## Scope

**In Scope:**
- Lock-free event buffer (crossbeam ArrayQueue)
- Atomic running totals (AtomicU64)
- Per-model/provider stats (DashMap)
- Dropped event counter for observability
- Event streaming via channels
- Protocol 012 field compatibility

**Out of Scope:**
- Server-side event persistence (see iron_control_api)
- Dashboard WebSocket streaming (see iron_control_api)
- Agent name/budget lookups (server-side enrichment)
- Min/max/median computation (server computes from synced events)

## License

Apache-2.0

## Directory Structure

### Source Files

| File | Responsibility |
|------|----------------|
| lib.rs | Lock-free event-based analytics for Iron Runtime LLM proxy. |
| event.rs | Analytics event types and payloads. |
| event_storage.rs | Lock-free event storage with atomic counters. |
| provider_utils.rs | Utility functions and types for analytics. |
| recording.rs | High-level recording API for EventStore. |
| stats.rs | Statistics types for analytics aggregation. |
| sync.rs | Analytics sync - background sync of events to Control API. |

**Notes:**
- Entries marked 'TBD' require manual documentation
- Entries marked '⚠️ ANTI-PATTERN' should be renamed to specific responsibilities

