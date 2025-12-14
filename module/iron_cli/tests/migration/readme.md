# Migration Tests

Tests that verify migration from orphaned adapters to correct adapters completed successfully.

## Responsibility

Verify migration metrics are at target values and migration trajectory is correct.

## Files

| File | Responsibility |
|------|----------------|
| metrics.rs | Verify migration metrics and trajectory through tests |
| framework.md | Document migration metrics framework and methodology |

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

## Migration Results

**Final Metrics:**
- Total adapters: 28 → 22 (6 deleted)
- Orphaned adapters: 6 → 0 (100% elimination)
- Broken routes: 6 → 0 (100% fixed)
- Dead code: ~384 lines → 0 (100% removed)

**Deleted Orphaned Adapters (6):**
1. show_agent_usage_adapter (usage_adapters.rs)
2. export_agent_usage_adapter (usage_adapters.rs)
3. reset_limit_adapter (limits_adapters.rs)
4. show_agent_limits_adapter (limits_adapters.rs)
5. update_agent_limit_adapter (limits_adapters.rs)
6. show_trace_stats_adapter (traces_adapters.rs)

## Migration Framework

For complete migration metrics framework, checkpoint methodology, measurement commands, and verification criteria, see framework.md.

## Running Tests

```bash
# Run all migration tests
cargo test --test migration --package iron_cli

# Run metrics tests
cargo test metrics --package iron_cli

# Run specific test
cargo test test_migration_metrics_at_target --package iron_cli
```
