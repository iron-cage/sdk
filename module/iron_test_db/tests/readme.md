# Tests

Tests for iron_test_db test database infrastructure.

## Responsibility Table

| File | Responsibility | Input→Output | Out of Scope |
|------|----------------|--------------|--------------|
| `infrastructure_tests.rs` | Test database creation, migration, and cleanup | DB lifecycle → Infrastructure validation | NOT wipe algorithm (wipe_test.rs) |
| `wipe_test.rs` | Test table dependency discovery and data wiping | Dependency graphs → Topological sort validation | NOT DB lifecycle (infrastructure_tests.rs) |

## Test Categories

- **Unit Tests:** Individual component validation
- **Integration Tests:** End-to-end database lifecycle
- **Algorithm Tests:** Topological sort and dependency resolution

## Running Tests

```bash
# All tests
cargo nextest run

# Infrastructure tests
cargo nextest run --test infrastructure_tests

# Wipe algorithm tests
cargo nextest run --test wipe_test
```

## Test Data

- SQLite in-memory databases
- Test migration scripts
- Mock table schemas
