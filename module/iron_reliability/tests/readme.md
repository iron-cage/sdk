# tests/

Contains all automated tests for iron_reliability.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `circuit_breaker_test.rs` | Test circuit breaker state transitions and failure thresholds | Failure scenarios → Circuit state validation | NOT retry logic (future), NOT health monitoring (future) |
| `readme_example_test.rs` | Test readme code examples for correctness | Example code → Execution validation | NOT circuit breaker internals (circuit_breaker_test.rs) |

## Running Tests

```bash
cd reliability
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
