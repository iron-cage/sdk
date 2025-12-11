# iron_runtime_analytics - Specification

**Module:** iron_runtime_analytics
**Layer:** 2 (Foundation)
**Status:** Planned

> **Specification Philosophy:** This specification focuses on architectural-level design and well-established knowledge. It describes what the module does and why, not implementation details or algorithms. Implementation constraints are minimal to allow flexibility.

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

**Out of Scope:**
- Server-side persistence (see iron_control_api)
- Dashboard WebSocket streaming (see iron_control_api)
- Agent name/budget lookups (server enrichment)
- Min/max/median computation (server computes)
- HTTP sync to server (Python layer responsibility)

---

## Dependencies

**Required External:**
- crossbeam - Lock-free data structures (ArrayQueue, channels)
- dashmap - Concurrent hashmap
- uuid - Event identifiers
- serde - Serialization for sync

**Optional:**
- pyo3 - Python bindings (when used from iron_runtime)

---

## Core Concepts

**Key Components:**
- **EventStore:** Lock-free bounded ring buffer with atomic counters
- **AnalyticsEvent:** Enum of event types (LlmRequestCompleted, LlmRequestFailed, etc.)
- **AtomicModelStats:** Per-model statistics with atomic operations
- **ComputedStats:** Snapshot of aggregated statistics

**Design Decisions:**
- Lock-free for async safety (no blocking in async contexts)
- Bounded buffer with eviction (oldest events evicted when full)
- Atomic counters for O(1) total stats
- Events as source of truth (aggregations computed on-demand)

---

## Protocol 012 Compatibility

**Event Fields (Protocol 012 compatible):**
- `timestamp_ms` - Unix timestamp in milliseconds
- `agent_id` - Optional agent identifier
- `provider_id` - Optional provider identifier
- `provider` - Provider name (openai, anthropic, etc.)
- `model` - Model name (gpt-4, claude-3, etc.)
- `input_tokens` - Input token count
- `output_tokens` - Output token count
- `cost_micros` - Cost in microdollars (1 USD = 1,000,000)

**Internal Fields (not in Protocol 012):**
- `event_id` - UUID for sync deduplication
- `synced` - Boolean sync state

---

## Integration Points

**Used by:**
- iron_runtime - LlmRouter proxy integration
- Python SDK - Analytics access via PyO3

**Depends on:**
- None (foundation module)

---

*For detailed implementation plan, see docs/features/007_python_analytics_implementation.md*
*For Protocol 012 specification, see docs/protocol/012_analytics_api.md*
