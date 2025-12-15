# tests/

Contains all automated tests for iron_types.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `ids_test.rs` | Test ID types and validation | ID operations → Type validation | NOT readme examples (readme_example_test.rs) |
| `readme_example_test.rs` | Test readme code examples for correctness | Example code → Execution validation | NOT ID internals (ids_test.rs) |

## Running Tests

```bash
cd types
cargo test --all-features
```

## Test Principles

- All tests in tests/ directory (NO #[cfg(test)] in src/)
- Real implementations only (NO mocking)
- Tests fail loudly (NO silent failures)
- Domain-based organization (NOT methodology-based)
