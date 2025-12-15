# TDD Methodology for Migration Verification

**Purpose**: Test-Driven Development approach for code migrations

**Status**: Production methodology (extracted from Phase 2 Config Unification)

**Last Updated**: 2025-12-15

---

## Overview

This document describes a 5-phase TDD cycle specifically designed for verifying code migrations. Unlike traditional TDD (RED → GREEN → REFACTOR), this methodology adds BASELINE measurement and ANTI-REGRESSION validation phases.

### Traditional TDD vs Verification TDD

**Traditional TDD**: RED → GREEN → REFACTOR

**Verification TDD**: BASELINE → RED → GREEN → ANTI-REGRESSION → MEASUREMENT

---

## The 5-Phase Cycle

### Phase 0: Baseline Measurement

**Purpose**: Establish concrete starting point before any changes

**When**: Before writing any tests or code

**Actions**:
1. Measure current state (old pattern count, new pattern count)
2. Document baseline numbers in plan
3. Establish success criteria (what numbers should change and how)

**Example**:
```bash
# Measure baseline
grep -r 'env::var.*DATABASE_URL' module/iron_token_manager/src/ | wc -l
# Result: 4

# Document baseline
echo "Baseline: 4 manual env::var calls" >> -baseline.md

# Define success criteria
echo "Target: 0 manual env::var calls (100% elimination)" >> -baseline.md
```

**Why This Matters**:
- Provides quantitative evidence migration occurred
- Prevents "addition trap" (adding new code alongside old)
- Creates objective pass/fail criteria

---

### Phase 1: RED (Write Tests That Fail)

**Purpose**: Write impossibility tests BEFORE migration, expect them to FAIL

**Philosophy**: "If tests pass before migration, they're not testing the migration"

**When**: After baseline measurement, before any implementation

**Actions**:
1. Write Tier 2 impossibility tests
2. Run tests, verify they FAIL (because old code still exists)
3. Document expected failures

**Example**:
```rust
// Write test BEFORE migration
#[test]
fn test_old_way_manual_env_var_is_deleted() {
  let source = std::fs::read_to_string("src/config.rs").unwrap();

  // This will FAIL initially because old code exists
  assert!(!source.contains("env::var(\"DATABASE_URL\")"));
}

// Run test
cargo test test_old_way_manual_env_var_is_deleted
// Result: FAILED (expected - old code still exists) ✅ RED PHASE
```

**Critical Rule**: If test passes before migration, something is wrong!

**Common Pitfall**: Writing tests that pass immediately
```rust
// ❌ BAD: Test passes before migration
#[test]
fn test_new_config_works() {
  // This test passes even if old code exists!
  assert!(true);
}

// ✅ GOOD: Test fails before migration
#[test]
fn test_old_config_deleted() {
  // This test fails if old code exists
  let source = std::fs::read_to_string("src/config.rs").unwrap();
  assert!(!source.contains("old_pattern"));
}
```

---

### Phase 2: GREEN (Perform Migration)

**Purpose**: Implement changes to make tests pass

**When**: After RED phase tests are written and failing

**Actions**:
1. Perform actual migration (delete old code, add new code)
2. Run impossibility tests again
3. Verify tests now PASS

**Example**:
```rust
// BEFORE migration (src/config.rs):
let database_url = std::env::var("DATABASE_URL")?;

// AFTER migration (src/config.rs):
let config = iron_config::ConfigLoader::load()?;
let database_url = config.database_url;

// Run test again
cargo test test_old_way_manual_env_var_is_deleted
// Result: PASSED ✅ GREEN PHASE
```

**Important**: Complete deletion, not commenting out

```rust
// ❌ BAD: Commenting out old code
// let database_url = std::env::var("DATABASE_URL")?;
let config = iron_config::ConfigLoader::load()?;

// ✅ GOOD: Complete deletion
let config = iron_config::ConfigLoader::load()?;
```

---

### Phase 3: ANTI-REGRESSION (Attempt Rollback)

**Purpose**: Prove migration cannot be reversed (meta-testing)

**When**: After GREEN phase passes

**Actions**:
1. Write rollback tests (Tier 4)
2. Attempt to inject old code back
3. Verify impossibility tests catch the injection

**Example**:
```bash
#!/bin/bash
# Rollback Test: Try to inject old code

# Inject old pattern back into code
sed -i '/ConfigLoader::load/a\    let _old = std::env::var("DATABASE_URL");' src/config.rs

# Run impossibility test
cargo test test_old_way_manual_env_var_is_deleted 2>&1 | tee /tmp/rollback.log

# Check if injection was caught
if grep -q "FAILED" /tmp/rollback.log; then
  echo "✅ ROLLBACK BLOCKED: Impossibility test caught injection"
  git checkout -- src/config.rs
  exit 0
else
  echo "❌ ROLLBACK SUCCEEDED: Test didn't catch injection (BUG!)"
  git checkout -- src/config.rs
  exit 1
fi
```

**Meta-Testing Concept**:
- **Phase 1 tests** verify old code doesn't exist
- **Phase 3 tests** verify Phase 1 tests actually work by injecting old code

This creates two-level verification:
1. Primary: Old code deleted (Phase 1)
2. Meta: Detection actually works (Phase 3)

---

### Phase 4: MEASUREMENT (Quantitative Verification)

**Purpose**: Prove shift occurred with concrete numbers

**When**: After migration is complete and all tests pass

**Actions**:
1. Measure current state (after migration)
2. Compare to baseline (Phase 0)
3. Calculate ratios
4. Verify shift matches expectations

**Example**:
```bash
# Measure current state
grep -r 'env::var.*DATABASE_URL' module/iron_token_manager/src/ | wc -l
# Result: 0

# Compare to baseline
echo "Baseline: 4 → Current: 0 (100% elimination)"

# Calculate replacement ratio
# Formula: new_usage / (old_usage + new_usage) × 100
# old=0, new=6
# ratio = 6/(0+6) = 100% ✅

# Verify targets achieved
echo "✅ QUANTITATIVE VERIFICATION: Shift confirmed"
```

**Key Metrics**:

| Metric | Formula | Target |
|--------|---------|--------|
| Replacement Ratio | new/(old+new) × 100 | 100% |
| Elimination Ratio | current_old/baseline_old × 100 | 0% |
| Adoption Ratio | current_new/expected_new × 100 | ≥100% |

---

## Complete TDD Example: Adding New Config Variable

**Scenario**: Add `MAX_RETRIES` configuration variable using ConfigLoader

### Phase 0: Baseline

```bash
# Measure current state
grep -r "MAX_RETRIES" module/ | wc -l
# Result: 0 (doesn't exist yet)

# Document baseline
echo "Baseline: 0 MAX_RETRIES references" >> -baseline.md

# Define target
echo "Target: 1 MAX_RETRIES in ConfigLoader, 0 manual env::var" >> -baseline.md
```

### Phase 1: RED

```rust
// Test 1: Impossibility test (write first, expect failure)
#[test]
fn test_max_retries_uses_config_loader() {
  let source = std::fs::read_to_string("src/config.rs").unwrap();

  // Should NOT use manual env::var
  assert!(!source.contains(r#"env::var("MAX_RETRIES")"#));

  // Should use ConfigLoader
  assert!(source.contains("ConfigLoader::load"));
}

// Run test
cargo test test_max_retries_uses_config_loader
// Result: FAILED (config doesn't support MAX_RETRIES yet) ✅ RED

// Test 2: Functionality test
#[test]
fn test_max_retries_from_env() {
  std::env::set_var("IRON_TOKEN_MAX_RETRIES", "5");
  let config = Config::load().unwrap();
  assert_eq!(config.max_retries, 5);
  std::env::remove_var("IRON_TOKEN_MAX_RETRIES");
}

// Run test
cargo test test_max_retries_from_env
// Result: FAILED (field doesn't exist) ✅ RED
```

### Phase 2: GREEN

```rust
// Step 1: Add field to Config struct (src/config.rs)
pub struct Config {
  pub max_retries: u32,
  // ... other fields
}

// Step 2: Add to ConfigLoader implementation
impl ConfigLoader {
  pub fn load() -> Result<Config> {
    let loader = iron_config::ConfigLoader::with_defaults("iron_token.toml")?;

    Ok(Config {
      max_retries: loader.get_var("MAX_RETRIES")
        .unwrap_or_else(|| "3".to_string())
        .parse()
        .unwrap_or(3),
      // ... other fields
    })
  }
}

// Run tests
cargo test test_max_retries_uses_config_loader
// Result: PASSED ✅ GREEN

cargo test test_max_retries_from_env
// Result: PASSED ✅ GREEN
```

### Phase 3: ANTI-REGRESSION

```bash
#!/bin/bash
# -rollback_test_max_retries.sh

# Inject manual env::var usage (attempt rollback)
sed -i '/max_retries:/a\    let _manual = std::env::var("MAX_RETRIES").unwrap_or_default();' \
  src/config.rs

# Run impossibility test
cargo test test_max_retries_uses_config_loader 2>&1 | tee /tmp/rollback.log

# Check if caught
if grep -q "FAILED" /tmp/rollback.log; then
  echo "✅ ROLLBACK BLOCKED: Test caught injection"
  git checkout -- src/config.rs
  exit 0
else
  echo "❌ ROLLBACK SUCCEEDED: Test didn't catch it (BUG in detection!)"
  git checkout -- src/config.rs
  exit 1
fi
```

```bash
# Execute rollback test
bash -rollback_test_max_retries.sh
# Expected: ✅ ROLLBACK BLOCKED
```

### Phase 4: MEASUREMENT

```bash
# Count manual env::var for MAX_RETRIES
grep -r 'env::var.*MAX_RETRIES' src/ | wc -l
# Result: 0 ✅

# Count ConfigLoader usage
grep -r 'max_retries' src/config.rs | grep -c 'get_var'
# Result: 1 ✅

# Calculate replacement ratio
# Old: 0, New: 1
# Replacement ratio: 1/(0+1) = 100% ✅

# Verify targets
echo "Baseline: 0 → Current: 1 ConfigLoader usage"
echo "Replacement ratio: 100%"
echo "✅ TDD CYCLE COMPLETE: MAX_RETRIES properly integrated"
```

---

## Bug Discovery During TDD

### How TDD Catches Bugs

The TDD cycle, especially Phase 3 (ANTI-REGRESSION), often reveals bugs in the verification system itself:

**Example: Bug #1 (issue-001) - Pattern Detection Too Strict**

#### Discovery Timeline

```
1. Phase 1: Write impossibility test
   Test uses exact string matching: contains("env::var(\"DATABASE_URL\")")
   Test PASSES (no exact match found in current code)

2. Phase 2: Perform migration
   Delete manual env::var calls
   Test still PASSES

3. Phase 3: Attempt rollback (THIS IS WHERE BUG WAS FOUND)
   Inject: std::env::var("DATABASE_URL").ok()
   Run impossibility test
   Expected: FAIL (should catch injection)
   Actual: PASS (didn't catch injection!) ❌

4. Investigation
   Pattern detection used exact match
   Looked for: env::var("DATABASE_URL")
   Actual code: env::var("DATABASE_URL").ok()
   No match due to .ok() suffix

5. Fix
   Implement proximity-based detection
   Check if "env::var" and "DATABASE_URL" appear within ±2 lines

6. Verification
   Re-run rollback test
   Inject: std::env::var("DATABASE_URL").ok()
   Run fixed test
   Expected: FAIL (catches injection)
   Actual: FAIL ✅ BUG FIXED
```

#### Key Insight

**Without Phase 3 rollback tests, this bug would never have been discovered.** The impossibility test appeared to work (Phase 1 FAIL → Phase 2 PASS), but it only worked for the specific code formatting that existed, not for variations.

#### Lesson Learned

**Rollback tests validate that impossibility tests actually work.** This is meta-testing: tests that test the tests.

---

## TDD Benefits for Migrations

### 1. Prevents False Positives

**Problem**: Tests that always pass
```rust
// ❌ Test always passes, doesn't verify anything
#[test]
fn test_migration_complete() {
  assert!(true);  // Useless test
}
```

**Solution**: RED phase ensures tests fail before migration
```rust
// ✅ Test fails before migration, passes after
#[test]
fn test_old_code_deleted() {
  let source = std::fs::read_to_string("src/old.rs").unwrap();
  assert!(!source.contains("old_pattern"));  // Fails if old code exists
}
```

### 2. Provides Quantitative Evidence

**Baseline → Measurement** proves migration with numbers:
- Before: 5 old patterns, 0 new patterns
- After: 0 old patterns, 6 new patterns
- Shift: -5 old, +6 new (quantitative proof)

### 3. Detects Incomplete Migrations

**Addition Trap**: Adding new code without deleting old

```bash
# Without BASELINE measurement
# Could add new code and think migration is done

# With BASELINE measurement
Baseline: old=5, new=0
Current: old=5, new=5  # Old code still there!
Replacement ratio: 5/(5+5) = 50%  # ❌ Incomplete
```

### 4. Validates Verification System

**Meta-testing** (Phase 3) proves verification actually works:
- Tests verify code state
- Rollback tests verify tests work

### 5. Documents Bug Discovery

When bugs are found during TDD, the timeline shows:
- Which phase revealed the bug
- Why it wasn't caught earlier
- How it was fixed
- How to prevent similar bugs

---

## Common TDD Pitfalls

### Pitfall 1: Tests That Always Pass

```rust
// ❌ BAD
#[test]
fn test_migration_works() {
  assert!(true);  // This is useless
}

// ✅ GOOD
#[test]
fn test_old_pattern_removed() {
  let source = std::fs::read_to_string("src/file.rs").unwrap();
  assert!(!source.contains("old_pattern"));  // Fails if pattern exists
}
```

### Pitfall 2: Skipping RED Phase

```rust
// ❌ BAD: Write test after migration
// Migration already done → test passes immediately → didn't verify anything

// ✅ GOOD: Write test before migration
// Test fails (RED) → migrate → test passes (GREEN) → verified!
```

### Pitfall 3: Ignoring Metrics

```bash
# ❌ BAD: No quantitative verification
# "I think I migrated everything..."

# ✅ GOOD: Measure before and after
Baseline: 5 old patterns
Current: 0 old patterns
Shift: -5 (100% elimination confirmed)
```

### Pitfall 4: No Meta-Testing

```bash
# ❌ BAD: Trust tests without validation
# Tests pass, assume they work

# ✅ GOOD: Validate tests with rollback
# Inject old code → tests should fail → proves detection works
```

---

## Quick Reference

### 5-Phase Checklist

- [ ] **Phase 0: BASELINE** - Measure current state, define targets
- [ ] **Phase 1: RED** - Write tests, verify they FAIL before migration
- [ ] **Phase 2: GREEN** - Perform migration, verify tests now PASS
- [ ] **Phase 3: ANTI-REGRESSION** - Attempt rollback, verify tests catch it
- [ ] **Phase 4: MEASUREMENT** - Measure final state, calculate ratios, verify targets

### Success Criteria

Migration TDD cycle is complete when:
- ✅ Phase 1 tests initially fail (RED)
- ✅ Phase 2 implementation makes tests pass (GREEN)
- ✅ Phase 3 rollback attempts fail (anti-regression validated)
- ✅ Phase 4 metrics achieve targets (quantitative proof)

### Key Formulas

```bash
# Replacement Ratio (target: 100%)
new_usage / (old_usage + new_usage) × 100

# Elimination Ratio (target: 0%)
current_old_usage / baseline_old_usage × 100

# Adoption Ratio (target: ≥100%)
current_new_usage / expected_new_usage × 100
```

---

**Related Documents**:
- `five_tier_pyramid.md` - Architectural verification pattern
- `migration_guide.md` - Step-by-step replication template
- `lessons_learned.md` - Insights from real migrations
