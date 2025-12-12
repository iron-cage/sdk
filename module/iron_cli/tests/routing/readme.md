# Routing Tests

Tests that verify CLI command routing correctness.

## Responsibility

Verify all CLI commands route to correct adapter functions and that no routes call orphaned adapters.

## Files

| File | Responsibility |
|------|----------------|
| correctness.rs | Routing correctness verification tests |

## Test Coverage

### correctness.rs (3 tests)

1. **test_all_commands_route_correctly**
   - Verifies all 22 commands have route entries
   - Ensures routing file contains each command pattern
   - NC-R.3: All commands must route to valid adapters

2. **test_no_orphaned_adapter_usage**
   - Verifies routing file doesn't reference deleted adapters
   - Checks for 6 orphaned adapter names in routing code
   - NC-R.1: Zero routes calling orphaned adapters

3. **test_routing_compilation_prevents_old_adapters**
   - Documents compile-time protection against old patterns
   - Verifies adapter modules exist
   - Proves syntactic layer of multi-layer defense

## Negative Criteria

- **NC-R.1**: Zero routes calling orphaned adapters
- **NC-R.2**: Zero routes with parameter mismatches
- **NC-R.3**: All 22 commands must route to valid adapters

## Orphaned Adapters (Deleted)

The following 6 adapters were deleted in migration because they had no matching API endpoints:

1. `show_agent_usage_adapter` (usage_adapters.rs)
2. `export_agent_usage_adapter` (usage_adapters.rs)
3. `reset_limit_adapter` (limits_adapters.rs)
4. `show_agent_limits_adapter` (limits_adapters.rs)
5. `update_agent_limit_adapter` (limits_adapters.rs)
6. `show_trace_stats_adapter` (traces_adapters.rs)

## Running Tests

```bash
# Run routing tests only
cargo test --test routing --package iron_cli

# Run with output
cargo test --test routing --package iron_cli -- --nocapture

# Run specific test
cargo test test_all_commands_route_correctly --package iron_cli
```

## Multi-Layer Defense

Routing tests verify the **syntactic layer** of protection:

1. **Syntactic**: Deleted adapters cannot compile (verified by these tests)
2. **Semantic**: Old API endpoints return 404 (runtime verification)
3. **Architectural**: Parameter contracts diverged (design verification)
4. **Operational**: Rollback requires coordination (process verification)
