# Migration Metrics Framework

## Purpose
Prove migration completeness by tracking measurable counts at each checkpoint. If ratios don't shift as planned, migration is incomplete.

## Metrics Definition

### M1: Adapter Function Counts
- **Orphaned Adapters**: Functions with no valid API endpoint
- **Correct Adapters**: Functions with valid API endpoint
- **Total Adapters**: All adapter functions in codebase

### M2: Routing Pattern Counts
- **Broken Routes**: Routes calling wrong/orphaned adapters
- **Correct Routes**: Routes calling correct adapters
- **Total Routes**: All command routes in binary

### M3: Code Quality Counts
- **Dead Code Lines**: Lines in orphaned adapters
- **Parameter Mismatches**: Routes where params don't match adapter
- **API Contract Violations**: Adapters calling non-existent endpoints

---

## Checkpoint 1: Initial State (Before Migration)

### M1: Adapter Function Counts
```
Orphaned Adapters:  6  (show_agent_usage, export_agent_usage,
                        reset_limit, show_agent_limits,
                        update_agent_limit, show_trace_stats)
Correct Adapters:   22 (all others)
Total Adapters:     28

Ratio: 21% orphaned (6/28)
```

### M2: Routing Pattern Counts
```
Broken Routes:      6  (.usage.by_agent, .usage.export_agent,
                       .limits.reset, .limits.by_agent, .limits.update_agent,
                       .traces.stats)
Correct Routes:     22 (all others)
Total Routes:       28

Ratio: 21% broken (6/28)
```

### M3: Code Quality Counts
```
Dead Code Lines:          384 (total in 6 orphaned adapters)
  - usage_adapters.rs:    140 lines (2 functions)
  - limits_adapters.rs:   188 lines (3 functions)
  - traces_adapters.rs:   56 lines (1 function)

Parameter Mismatches:     6 (all broken routes)
API Contract Violations:  6 (all orphaned adapters)
```

---

## Checkpoint 2: After Routing Fixes (Phase 2.1)

### M1: Adapter Function Counts
```
Orphaned Adapters:  6  (unchanged - functions still exist)
Correct Adapters:   22 (unchanged)
Total Adapters:     28

Ratio: 21% orphaned (6/28) ⚠️ NO CHANGE YET
```

### M2: Routing Pattern Counts
```
Broken Routes:      0  ✅ FIXED (routes removed from binary)
Correct Routes:     22 ✅ ALL CORRECT
Total Routes:       22

Ratio: 0% broken (0/22) ✅ TARGET ACHIEVED
```

### M3: Code Quality Counts
```
Dead Code Lines:          384 ⚠️ STILL PRESENT (orphaned code exists)
Parameter Mismatches:     0   ✅ FIXED (broken routes removed)
API Contract Violations:  6   ⚠️ STILL PRESENT (orphaned adapters exist)
```

**Analysis**: Routes fixed but dead code remains. Migration 50% complete.

---

## Checkpoint 3: After Adapter Deletion (Phase 2H)

### M1: Adapter Function Counts
```
Orphaned Adapters:  0  ✅ DELETED
Correct Adapters:   22 ✅ ALL VALID
Total Adapters:     22

Ratio: 0% orphaned (0/22) ✅ TARGET ACHIEVED
```

### M2: Routing Pattern Counts
```
Broken Routes:      0  ✅ MAINTAINED
Correct Routes:     22 ✅ MAINTAINED
Total Routes:       22

Ratio: 0% broken (0/22) ✅ TARGET ACHIEVED
```

### M3: Code Quality Counts
```
Dead Code Lines:          0   ✅ DELETED
Parameter Mismatches:     0   ✅ MAINTAINED
API Contract Violations:  0   ✅ DELETED
```

**Analysis**: All metrics at target. Migration 100% complete.

---

## Migration Trajectory Verification

### Expected Pattern Shift

| Metric | Initial | After 2.1 | After 2H | Expected Δ | Status |
|--------|---------|-----------|----------|------------|--------|
| Orphaned Adapters | 6 | 6 | 0 | -6 | ✅ |
| Broken Routes | 6 | 0 | 0 | -6 | ✅ |
| Dead Code Lines | 384 | 384 | 0 | -384 | ✅ |
| Correct Adapters | 22 | 22 | 22 | +0 | ✅ |
| API Violations | 6 | 6 | 0 | -6 | ✅ |

### Critical Ratios

| Ratio | Initial | Target | Actual | Status |
|-------|---------|--------|--------|--------|
| Orphaned % | 21% | 0% | 0% | ✅ |
| Broken Routes % | 21% | 0% | 0% | ✅ |
| Correct Adapters % | 79% | 100% | 100% | ✅ |

---

## Verification Criteria

### ✅ Migration is COMPLETE if:
1. Orphaned adapters: 6 → 0 (100% reduction)
2. Broken routes: 6 → 0 (100% reduction)
3. Dead code lines: 384 → 0 (100% reduction)
4. Correct adapters: 22 → 22 (maintained)
5. All ratios at target (0% orphaned, 100% correct)

### ❌ Migration is INCOMPLETE if:
1. Any orphaned adapters remain (count > 0)
2. Any broken routes remain (count > 0)
3. Any dead code remains (lines > 0)
4. Ratios don't match targets

### ⚠️ Migration is REGRESSED if:
1. Orphaned adapters increase (count goes up)
2. Broken routes increase (count goes up)
3. Dead code increases (lines go up)
4. Ratios shift away from targets

---

## Measurement Commands

### Count Orphaned Adapters
```bash
# Functions with no valid API endpoint (should all return 0)
grep -c "pub async fn show_agent_usage_adapter" src/adapters/usage_adapters.rs || echo 0
grep -c "pub async fn export_agent_usage_adapter" src/adapters/usage_adapters.rs || echo 0
grep -c "pub async fn reset_limit_adapter" src/adapters/limits_adapters.rs || echo 0
grep -c "pub async fn show_agent_limits_adapter" src/adapters/limits_adapters.rs || echo 0
grep -c "pub async fn update_agent_limit_adapter" src/adapters/limits_adapters.rs || echo 0
grep -c "pub async fn show_trace_stats_adapter" src/adapters/traces_adapters.rs || echo 0
```

### Count Total Adapters
```bash
# All adapter functions
grep -c "pub async fn.*_adapter" src/adapters/auth_adapters.rs
grep -c "pub async fn.*_adapter" src/adapters/token_adapters.rs
grep -c "pub async fn.*_adapter" src/adapters/usage_adapters.rs
grep -c "pub async fn.*_adapter" src/adapters/limits_adapters.rs
grep -c "pub async fn.*_adapter" src/adapters/traces_adapters.rs
grep -c "pub async fn.*_adapter" src/adapters/health_adapters.rs
```

### Count Broken Routes
```bash
# Routes calling orphaned adapters (should all return 0)
grep -c "show_agent_usage_adapter" src/bin/iron_token_unilang.rs || echo 0
grep -c "export_agent_usage_adapter" src/bin/iron_token_unilang.rs || echo 0
grep -c "show_agent_limits_adapter" src/bin/iron_token_unilang.rs || echo 0
grep -c "reset_limit_adapter" src/bin/iron_token_unilang.rs || echo 0
grep -c "update_agent_limit_adapter" src/bin/iron_token_unilang.rs || echo 0
grep -c "show_trace_stats_adapter" src/bin/iron_token_unilang.rs || echo 0
```

---

## Success Criteria

Migration is VERIFIED COMPLETE when:

1. **Zero-Count Verification**: All "must be 0" counts are 0
   - Orphaned adapters: 0
   - Broken routes: 0
   - Dead code lines: 0
   - API violations: 0

2. **Ratio Verification**: All ratios at target
   - Orphaned %: 0%
   - Broken routes %: 0%
   - Correct adapters %: 100%

3. **Direction Verification**: All metrics moved in expected direction
   - Old patterns decreased (6 → 0)
   - New patterns maintained (22 → 22)
   - Quality improved (384 dead lines → 0)

4. **Stability Verification**: Counts don't regress
   - After deletion, orphaned count stays 0
   - After fixes, broken count stays 0
   - No new violations introduced
