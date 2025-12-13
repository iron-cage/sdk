# Tests

Tests for iron_runtime_state agent state management.

## Organization

| File | Responsibility |
|------|----------------|
| state_test.rs | Agent state lifecycle and persistence tests |

## Test Categories

- **Unit Tests:** State transition validation
- **Integration Tests:** State persistence and retrieval
- **Concurrency Tests:** Concurrent state access

## Running Tests

```bash
# All tests
cargo nextest run

# State tests
cargo nextest run --test state_test

# With SQLite feature
cargo nextest run --features sqlite
```

## Test Data

- In-memory state fixtures
- Test agent configurations
