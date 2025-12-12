# tests/

Contains all automated tests for iron_runtime_analytics.

## Pilot Strategy

The implementation follows a simple pilot strategy:

1. **Fixed Memory:** Bounded buffer (default 10,000 slots, ~2-5MB)
2. **Non-Blocking:** Drop new events when full (never block main thread)
3. **Observability:** `dropped_count` counter tracks lost events

## Organization

Tests organized by functional area:

- `event_store_test.rs` - EventStore basic operations (create, record, buffer, drops)
- `stats_test.rs` - Atomic stats and computed statistics
- `concurrency_test.rs` - Lock-free concurrent access safety
- `protocol_012_test.rs` - Protocol 012 Analytics API compatibility

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
