# tests/

Contains all automated tests for iron_runtime_analytics.

## Pilot Strategy

The implementation follows a simple pilot strategy:

1. **Fixed Memory:** Bounded buffer (default 10,000 slots, ~2-5MB)
2. **Non-Blocking:** Drop new events when full (never block main thread)
3. **Observability:** `dropped_count` counter tracks lost events

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `event_store_test.rs` | Test EventStore basic operations | Store creation/recording → Operation validation | NOT concurrency (concurrency_test.rs), NOT stats (stats_test.rs), NOT sync (sync_test.rs) |
| `stats_test.rs` | Test atomic statistics and computed stats | Event recording → Stats calculation validation | NOT event storage (event_store_test.rs), NOT concurrency (concurrency_test.rs), NOT protocol schema (protocol_012_test.rs) |
| `concurrency_test.rs` | Test lock-free concurrent access safety | Multi-threaded operations → Thread safety validation | NOT storage operations (event_store_test.rs), NOT stats logic (stats_test.rs), NOT sync (sync_test.rs) |
| `protocol_012_test.rs` | Test Protocol 012 Analytics API compatibility | Event schema → Protocol compliance validation | NOT stats calculation (stats_test.rs), NOT storage (event_store_test.rs), NOT sync (sync_test.rs) |
| `recording_test.rs` | Test high-level recording API | Recording calls → Event creation validation | NOT low-level storage (event_store_test.rs), NOT stats (stats_test.rs), NOT sync (sync_test.rs) |
| `helpers_test.rs` | Test helper functions and Provider enum | Provider strings → Provider enum validation | NOT event storage (event_store_test.rs), NOT stats (stats_test.rs), NOT recording (recording_test.rs) |
| `sync_test.rs` | Test analytics sync to Control API | Sync operations → HTTP sync validation | NOT event storage (event_store_test.rs), NOT stats (stats_test.rs), NOT concurrency (concurrency_test.rs) |

## Running Tests

```bash
cd module/iron_runtime_analytics
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
- TDD approach: tests written before implementation

## Test Categories

### Event Store Operations
- Creation with default/custom capacity
- Event recording (LlmRequestCompleted, LlmRequestFailed)
- Buffer bounds and drop-on-full behavior
- `dropped_count` observability
- Event ID assignment

### Atomic Stats (O(1))
- Total counters (requests, tokens, cost) - tracks ALL events including dropped
- Per-model breakdown
- Per-provider breakdown
- Computed stats (success_rate, avg_cost)

### Concurrency Safety
- Multi-threaded recording
- Lock-free guarantees
- No data races under load

### Protocol 012 Compatibility
- Required fields present
- Cost in microdollars
- Timestamps in milliseconds
