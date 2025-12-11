# iron_runtime_analytics

Lock-free event-based analytics for Python LlmRouter.

## Pilot Strategy

Simple, predictable behavior for the pilot:

1. **Fixed Memory:** Bounded buffer (default 10,000 slots, ~2-5MB)
2. **Non-Blocking:** Drop new events when buffer full (never block)
3. **Observability:** `dropped_count()` tracks lost events

### Scope

**Responsibilities:**
Provides lock-free event storage for LLM request analytics. Events are stored in a bounded buffer with atomic counters for O(1) stats access. Designed for high-throughput concurrent LLM calls in async contexts. Full compatibility with Protocol 012 Analytics API.

**In Scope:**
- Lock-free event buffer (crossbeam ArrayQueue)
- Atomic running totals (AtomicU64)
- Per-model/provider stats (DashMap)
- Dropped event counter for observability
- Event streaming via channels
- Protocol 012 field compatibility
- PyO3 bindings for Python access

**Out of Scope:**
- Server-side event persistence (see iron_control_api)
- Dashboard WebSocket streaming (see iron_control_api)
- Agent name/budget lookups (server-side enrichment)
- Min/max/median computation (server computes from synced events)

## Installation

```toml
[dependencies]
iron_runtime_analytics = { path = "../iron_runtime_analytics" }
```

## Example

```rust
use iron_runtime_analytics::EventStore;

// Create event store with default capacity (10,000 events)
let store = EventStore::new();

// Record LLM request completion - lock-free, O(1)
store.record(AnalyticsEvent::LlmRequestCompleted {
    event_id: Uuid::nil(),
    synced: false,
    timestamp_ms: 1734000000000,
    agent_id: Some("agent_abc123".into()),
    provider_id: Some("ip_openai-001".into()),
    provider: "openai".into(),
    model: "gpt-4".into(),
    input_tokens: 150,
    output_tokens: 50,
    cost_micros: 6000,
});

// Get stats - O(1) for totals
let stats = store.get_stats();
println!("Total cost: ${:.4}", stats.total_cost_usd());

// Check for dropped events (observability)
if store.dropped_count() > 0 {
    println!("Warning: {} events dropped (buffer full)", store.dropped_count());
}
```

## License

Apache-2.0
