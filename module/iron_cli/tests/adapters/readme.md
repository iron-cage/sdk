# Adapter Tests

Tests that verify adapter layer functionality and coverage.

## Responsibility

Verify adapter implementations and ensure all adapters have valid API endpoints with no orphaned adapters.

## Files

| File | Responsibility |
|------|----------------|
| auth_adapters_test.rs | Authentication adapter integration tests |
| token_adapters_test.rs | Token management adapter integration tests |
| usage_adapters_test.rs | Usage tracking adapter integration tests |
| limits_adapters_test.rs | Rate limiting adapter integration tests |
| traces_adapters_test.rs | Trace management adapter integration tests |
| health_adapters_test.rs | Health check adapter integration tests |
| coverage.rs | Adapter coverage verification tests |

## Test Coverage

### Functional Tests (110 tests total)

Integration tests for each adapter function across 22 commands:
- auth_adapters_test.rs (3 commands)
- token_adapters_test.rs (5 commands)
- usage_adapters_test.rs (4 commands)
- limits_adapters_test.rs (5 commands)
- traces_adapters_test.rs (3 commands)
- health_adapters_test.rs (2 commands)

### Coverage Tests (3 tests)

1. **test_all_adapters_have_valid_endpoints**
   - Verifies adapter count per module
   - Ensures total of 19 adapters exist
   - NC-A.2: Zero API contract violations

2. **test_no_orphaned_adapters_exist**
   - Verifies no orphaned adapter references in source
   - Checks for 6 deleted adapter names
   - NC-A.1: Zero orphaned adapters

3. **test_adapter_count_metrics**
   - Verifies migration metrics (6→0 orphaned)
   - Calculates orphaned percentage (27%→0%)
   - NC-A.3: Orphaned percentage = 0%

## Negative Criteria

- **NC-A.1**: Zero orphaned adapters
- **NC-A.2**: Zero adapters calling non-existent endpoints
- **NC-A.3**: Orphaned percentage must be 0%

## Migration Metrics

### Before Migration
- Total adapters: 25
- Orphaned: 6 (27%)
- Correct: 19 (73%)

### After Migration
- Total adapters: 19
- Orphaned: 0 (0%)
- Correct: 19 (100%)

### Deleted Orphaned Adapters (6)

1. show_agent_usage_adapter (usage_adapters.rs)
2. export_agent_usage_adapter (usage_adapters.rs)
3. reset_limit_adapter (limits_adapters.rs)
4. show_agent_limits_adapter (limits_adapters.rs)
5. update_agent_limit_adapter (limits_adapters.rs)
6. show_trace_stats_adapter (traces_adapters.rs)

## Running Tests

```bash
# Run all adapter tests
cargo test --test adapters --package iron_cli

# Run coverage tests only
cargo test coverage --package iron_cli

# Run specific coverage test
cargo test test_no_orphaned_adapters_exist --package iron_cli
```

## Adapter Count by Module

| Module | Adapters | Before Migration | Deleted |
|--------|----------|------------------|---------|
| auth_adapters | 3 | 3 | 0 |
| token_adapters | 5 | 5 | 0 |
| usage_adapters | 4 | 6 | 2 |
| limits_adapters | 5 | 8 | 3 |
| traces_adapters | 2 | 3 | 1 |
| health_adapters | 2 | 2 | 0 |
| **Total** | **19** | **25** | **6** |
