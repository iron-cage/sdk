# iron_runtime_analytics - Specification

**Module:** iron_runtime_analytics
**Layer:** 2 (Foundation)
**Status:** Implemented (Phase 1 + Phase 2 Complete)

---

## Responsibility

Lock-free event-based analytics storage for Python LlmRouter. Provides async-safe event recording with bounded memory, atomic counters for O(1) stats, and Protocol 012 Analytics API compatibility.

---

## Scope

**In Scope:**
- Lock-free event buffer (crossbeam ArrayQueue)
- Atomic running totals (total_requests, total_cost, etc.)
- Per-model and per-provider stats (DashMap)
- Event streaming via crossbeam channels
- Sync state tracking (synced/unsynced events)
- Protocol 012 field compatibility
- High-level recording API with automatic provider inference
- Cost calculation via iron_cost integration

**Out of Scope:**
- Server-side persistence (see iron_control_api)
- Dashboard WebSocket streaming (see iron_control_api)
- Agent name/budget lookups (server enrichment)
- Min/max/median computation (server computes)
- HTTP sync to server (Python layer responsibility)

---

## Dependencies

**Required:**
- `crossbeam` - Lock-free data structures (ArrayQueue, channels)
- `dashmap` - Concurrent hashmap for per-model stats
- `uuid` - Event identifiers (v4)
- `serde` - Serialization for sync
- `iron_cost` - LLM pricing data and cost calculation

---

## Core Types

### EventStore

Lock-free bounded ring buffer with atomic counters.

```rust
pub struct EventStore {
    buffer: ArrayQueue<AnalyticsEvent>,      // Lock-free bounded buffer
    global: GlobalStats,                      // Atomic counters
    by_model: DashMap<Arc<str>, AtomicModelStats>,
    by_provider: DashMap<Arc<str>, AtomicModelStats>,
    event_sender: Option<Sender<AnalyticsEvent>>,
    unsynced_count: AtomicU64,
    dropped_events: AtomicU64,
}
```

### AnalyticsEvent

Event with metadata and typed payload.

```rust
pub struct AnalyticsEvent {
    metadata: EventMetadata,  // event_id, timestamp_ms, synced, agent_id
    pub payload: EventPayload,
}

pub enum EventPayload {
    LlmRequestCompleted(LlmUsageData),
    LlmRequestFailed(LlmFailureData),
    BudgetThresholdReached { threshold_percent, current_spend_micros, budget_micros },
    RouterStarted { port },
    RouterStopped { total_requests, total_cost_micros },
}
```

### Provider

LLM provider enumeration with inference support.

```rust
pub enum Provider {
    OpenAI,
    Anthropic,
    Unknown,  // Fallback for unknown providers
}

pub fn infer_provider(model: &str) -> Provider;  // gpt-* → OpenAI, claude-* → Anthropic
```

### ComputedStats

Snapshot of aggregated statistics.

```rust
pub struct ComputedStats {
    pub total_requests: u64,
    pub failed_requests: u64,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cost_micros: u64,
    pub by_model: HashMap<String, ModelStats>,
    pub by_provider: HashMap<String, ModelStats>,
}
```

---

## API Surface

### Constructors

| Method | Description |
|--------|-------------|
| `EventStore::new()` | Default capacity (10,000 events) |
| `EventStore::with_capacity(n)` | Custom capacity |
| `EventStore::with_streaming(cap, chan)` | With event streaming channel |

### High-Level Recording (recording.rs)

| Method | Description |
|--------|-------------|
| `record_llm_completed(pricing, model, in, out, agent, provider)` | Record successful request |
| `record_llm_completed_with_provider(...)` | With explicit provider (skip inference) |
| `record_llm_failed(model, agent, provider, code, msg)` | Record failed request |
| `record_budget_threshold(percent, current, budget, agent)` | Record budget alert |
| `record_router_started(port)` | Lifecycle: router started |
| `record_router_stopped()` | Lifecycle: router stopped (captures stats) |

### Low-Level Recording

| Method | Description |
|--------|-------------|
| `record(event)` | Record pre-constructed event |

### Statistics

| Method | Description |
|--------|-------------|
| `stats()` | Get ComputedStats snapshot |
| `dropped_count()` | Events dropped due to full buffer |
| `unsynced_count()` | Events pending server sync |

### Buffer Access

| Method | Description |
|--------|-------------|
| `len()` | Current buffer size |
| `is_empty()` | Buffer empty check |
| `drain_all()` | Remove and return all events |
| `snapshot_events()` | Copy all events (non-destructive) |
| `unsynced_events()` | Get unsynced events only |
| `mark_synced(ids)` | Mark events as synced |

---

## Design Decisions

### Pilot Strategy

- **Fixed Memory:** Bounded buffer (default 10,000 slots, ~2-5MB)
- **Non-Blocking:** Drop new events when full (never block main thread)
- **Observability:** `dropped_count` counter tracks lost events

### Lock-Free Design

- ArrayQueue for O(1) push/pop without locks
- AtomicU64 counters updated before buffer push (source of truth)
- DashMap for concurrent per-model/provider aggregation

### Cost Calculation

- Uses `iron_cost::PricingManager` for model pricing lookup
- Returns 0 for unknown models (safe default for analytics)
- All costs in microdollars (1 USD = 1,000,000 micros)

### Provider Inference

- `gpt-*`, `o1-*`, `o3-*`, `chatgpt-*` → OpenAI
- `claude-*` → Anthropic
- Unknown models → Provider::Unknown

---

## Protocol 012 Compatibility

**Event Fields (Protocol 012 compatible):**
- `timestamp_ms` - Unix timestamp in milliseconds
- `agent_id` - Optional agent identifier
- `provider_id` - Optional provider key identifier
- `provider` - Provider name (openai, anthropic, unknown)
- `model` - Model name (gpt-4, claude-3-opus, etc.)
- `input_tokens` - Input token count
- `output_tokens` - Output token count
- `cost_micros` - Cost in microdollars

**Internal Fields (not in Protocol 012):**
- `event_id` - UUID for sync deduplication
- `synced` - Boolean sync state

---

## File Structure

```
module/iron_runtime_analytics/
├── Cargo.toml
├── readme.md
├── spec.md
├── src/
│   ├── lib.rs           # Module exports and re-exports
│   ├── event.rs         # AnalyticsEvent, EventPayload, LlmUsageData
│   ├── event_storage.rs # EventStore implementation
│   ├── stats.rs         # AtomicModelStats, ModelStats, ComputedStats
│   ├── recording.rs     # High-level record_* methods
│   └── helpers.rs       # Provider, infer_provider, current_time_ms
└── tests/
    ├── event_store_test.rs   # Basic operations (23 tests)
    ├── stats_test.rs         # Statistics (23 tests)
    ├── concurrency_test.rs   # Multi-threaded safety (11 tests)
    ├── protocol_012_test.rs  # API compatibility (14 tests)
    ├── helpers_test.rs       # Provider inference (13 tests)
    └── recording_test.rs     # High-level API (20 tests)
```

---

## Test Coverage

| Test File | Tests | Coverage |
|-----------|-------|----------|
| event_store_test.rs | 23 | Core buffer operations, drops, streaming |
| stats_test.rs | 23 | Atomic counters, computed stats, aggregation |
| concurrency_test.rs | 11 | Multi-threaded safety, no deadlocks |
| protocol_012_test.rs | 14 | Field compatibility, serialization |
| helpers_test.rs | 13 | Provider enum, inference, traits |
| recording_test.rs | 20 | High-level API, cost calculation |
| **Total** | **104** | |

---

## Integration Points

**Used by:**
- iron_runtime/LlmRouter - Proxy integration for automatic event recording and logging

**Depends on:**
- iron_cost - Pricing data and cost calculation

---

## Python Integration (Phase 2)

Analytics is integrated into LlmRouter proxy and enabled by default via `full` feature.

**Automatic Event Recording:**
- `LlmRequestCompleted` - Recorded after each successful LLM request
- `LlmRequestFailed` - Recorded on non-2xx responses
- `RouterStarted` / `RouterStopped` - Lifecycle events

**Console Logging:**
All events are logged via `iron_telemetry`:
```
INFO LlmRouter proxy listening on http://127.0.0.1:45297
INFO LLM request completed model=gpt-4o-mini input_tokens=11 output_tokens=1 cost_usd=0.000001
INFO LlmRouter proxy shutting down
```

**Budget Tracking:**
Budget status is available via `router.budget_status` (uses CostController from iron_cost).

**Internal Storage:**
Events are stored in EventStore for future server sync (Phase 4 - BACKLOG).

---


