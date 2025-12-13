# Tests

Tests for iron_test_db test database infrastructure.

## Organization

| File | Responsibility |
|------|----------------|
| infrastructure_tests.rs | Database creation, migration, and cleanup tests |
| wipe_test.rs | Table dependency discovery and data wiping tests |

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
