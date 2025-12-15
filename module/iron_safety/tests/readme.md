# tests/

Contains all automated tests for iron_cage_safety.

## Organization

Tests organized by functional area (PII detection, redaction, audit logging).

Flat structure maintained (< 20 test files expected).

## Running Tests

```bash
cd safety
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
