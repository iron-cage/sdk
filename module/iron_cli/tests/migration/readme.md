# Migration Tests

Tests that verify migration from orphaned adapters to correct adapters completed successfully.

## Responsibility

Verify migration metrics are at target values and migration trajectory is correct.

## Files

| File | Responsibility |
|------|----------------|
| metrics.rs | Migration metrics verification tests |

## Test Coverage

### metrics.rs (3 tests)

1. **test_migration_metrics_at_target**
   - Verifies M1 (adapters), M2 (routing), M3 (quality) metrics
   - All metrics must be at target values
   - NC-M.1/2/3: All counts at target (0 orphaned, 22 correct)

2. **test_migration_trajectory_correctness**
   - Verifies pattern shift occurred as expected
   - Orphaned: 6→0 (Δ -6), Correct: 22→22 (Δ +0)
   - Orphaned %: 21%→0% (Δ -21%)

3. **test_ratios_at_target**
   - Verifies all critical ratios at target
   - NC-M.3: Orphaned 0%, Correct 100%
   - Both adapters and routes verified

## Negative Criteria

- **NC-M.1**: Orphaned adapter count must be 0
- **NC-M.2**: Broken route count must be 0
- **NC-M.3**: All ratios must match targets (0% orphaned, 100% correct)

## Migration Metrics

### M1: Adapter Function Counts
- Before: Total 28, Orphaned 6 (21%), Correct 22 (79%)
- After: Total 22, Orphaned 0 (0%), Correct 22 (100%)
- Delta: Orphaned -6 (-100%), Correct +0 (maintained)

### M2: Routing Pattern Counts
- Before: Broken 6 (routing to orphaned), Correct 16
- After: Broken 0, Correct 22
- Delta: Broken -6 (-100%), Correct +6 (+37%)

### M3: Code Quality Counts
- Before: Dead code indicators 6, API violations 6
- After: Dead code 0, API violations 0
- Delta: Dead code -6 (-100%), Violations -6 (-100%)

## Migration Trajectory

| Metric | Initial | Final | Delta | Status |
|--------|---------|-------|-------|--------|
| Orphaned adapters | 6 | 0 | -6 | ✓ TARGET |
| Broken routes | 6 | 0 | -6 | ✓ TARGET |
| Dead code lines | ~384 | 0 | -384 | ✓ TARGET |
| Orphaned % | 21% | 0% | -21% | ✓ TARGET |
| Correct % | 79% | 100% | +21% | ✓ TARGET |

## Deleted Orphaned Adapters (6)

1. show_agent_usage_adapter (usage_adapters.rs)
2. export_agent_usage_adapter (usage_adapters.rs)
3. reset_limit_adapter (limits_adapters.rs)
4. show_agent_limits_adapter (limits_adapters.rs)
5. update_agent_limit_adapter (limits_adapters.rs)
6. show_trace_stats_adapter (traces_adapters.rs)

## Running Tests

```bash
# Run all migration tests
cargo test --test migration --package iron_cli

# Run metrics tests
cargo test metrics --package iron_cli

# Run specific test
cargo test test_migration_metrics_at_target --package iron_cli
```

## Migration Framework

For complete migration methodology and checkpoint verification, see the migration framework documentation (migrated from `-phase2_migration_metrics.md`).

### Checkpoints

**Checkpoint 1: Initial State (Before Migration)**
- Orphaned adapters: 6 (21%)
- Broken routes: 6 (27%)
- Dead code: ~384 lines

**Checkpoint 2: After Routing Fixes**
- Orphaned adapters: 6 (still exist)
- Broken routes: 0 (fixed)
- Migration: 50% complete

**Checkpoint 3: After Adapter Deletion (Final)**
- Orphaned adapters: 0 (deleted)
- Broken routes: 0 (maintained)
- Migration: 100% complete ✓
