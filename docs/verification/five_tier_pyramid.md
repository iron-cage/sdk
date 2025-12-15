# The 5-Tier Verification Pyramid

**Purpose**: Architectural pattern for verifying code migrations are complete, irreversible, and impossible to bypass

**Status**: Production methodology (extracted from Phase 2 Config Unification)

**Last Updated**: 2025-12-15

---

## Overview

The 5-Tier Verification Pyramid is a comprehensive verification architecture that proves a code migration is complete from multiple perspectives. Each tier builds on the previous, creating defense-in-depth verification.

```
        Tier 5: Quantitative (Metrics)
             "Did the numbers shift?"
                      ↑
        Tier 4: Rollback (Meta-Tests)
          "Can we go back to old way?"
                      ↑
      Tier 3: Active Breaking (Runtime)
          "Does old way still work?"
                      ↑
      Tier 2: Impossibility (Static)
         "Is old code deleted?"
                      ↑
      Tier 1: Anti-Shortcut (Implementation)
        "Is new implementation real?"
```

---

## Tier 1: Anti-Shortcut Detection

**Purpose**: Detect fake implementations that make tests pass without doing real work

**Philosophy**: "How could someone cheat to pass tests without actually completing the migration?"

### Common Shortcuts to Detect

#### Shortcut 1: Facade Pattern
Empty or minimal implementation that compiles but doesn't function:

```rust
// ❌ SHORTCUT: Facade
pub struct ConfigLoader;  // Empty struct!

impl ConfigLoader {
  pub fn load() -> Result<Config> {
    Ok(Config::default())  // Returns hard-coded defaults!
  }
}
```

**Detection**:
```rust
#[test]
fn test_config_loader_not_facade() {
  let source = std::fs::read_to_string("src/config.rs").unwrap();

  // Should have real implementation
  assert!(source.contains("impl ConfigLoader"));
  let impl_lines = source.lines()
    .skip_while(|l| !l.contains("impl ConfigLoader"))
    .take_while(|l| !l.contains("}"))
    .count();

  // Real impl should have >10 lines, not just stub
  assert!(impl_lines > 10);
}
```

#### Shortcut 2: Hard-Coded Values
New interface but returns hard-coded values instead of reading from new source:

```rust
// ❌ SHORTCUT: Hard-coded
pub fn load() -> Result<Config> {
  Ok(Config {
    api_url: "https://api.example.com".to_string(),  // Hard-coded!
    max_retries: 3,  // Hard-coded!
  })
}
```

**Detection**:
```rust
#[test]
fn test_config_not_hard_coded() {
  // Load with different inputs
  std::env::set_var("CONFIG_VAR", "value1");
  let config1 = ConfigLoader::load().unwrap();

  std::env::set_var("CONFIG_VAR", "value2");
  let config2 = ConfigLoader::load().unwrap();

  // Values should differ (not hard-coded)
  assert_ne!(config1.var, config2.var);
}
```

#### Shortcut 3: Hybrid Implementation
New interface that internally calls old implementation:

```rust
// ❌ SHORTCUT: Hybrid
pub fn new_way() -> Result<Data> {
  // New interface, but calls old code internally!
  old_way()
}
```

**Detection**:
```rust
#[test]
fn test_new_impl_doesnt_call_old() {
  let source = std::fs::read_to_string("src/new.rs").unwrap();
  assert!(!source.contains("old_way"));
  assert!(!source.contains("legacy_"));
}
```

#### Shortcut 4: Mocked Implementation
Tests pass because implementation is mocked, not real:

```rust
// ❌ SHORTCUT: Mocked in tests
#[cfg(test)]
impl ConfigLoader {
  pub fn load() -> Result<Config> {
    // Test-only mock!
    Ok(Config::test_defaults())
  }
}
```

**Detection**:
```rust
#[test]
fn test_no_mocked_config_loader() {
  let source = std::fs::read_to_string("src/config.rs").unwrap();
  assert!(!source.contains("#[cfg(test)].*ConfigLoader"));
  assert!(!source.contains("mock_"));
}
```

### Anti-Shortcut Test Strategy

1. **Identify potential shortcuts**: For each migration, ask "how could I fake this?"
2. **Write detection tests**: Create tests that fail if shortcuts are taken
3. **Run before migration**: Verify tests catch the current (shortcut) state
4. **Run after migration**: Verify tests pass with real implementation

---

## Tier 2: Impossibility Testing

**Purpose**: Prove old patterns have been deleted and cannot be used

**Philosophy**: "The old way should be impossible, not just unused"

### What to Test

#### Old Code Deleted
```rust
#[test]
fn old_pattern_deleted() {
  let source = std::fs::read_to_string("src/target.rs").unwrap();

  // Old pattern should not exist
  assert!(!source.contains("old_pattern"));
  assert!(!source.contains("legacy_function"));
}
```

#### Old Dependencies Removed
```rust
#[test]
fn old_dependency_removed() {
  let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
  assert!(!cargo_toml.contains("old_crate"));
}
```

#### New Dependencies Required
```rust
#[test]
fn new_dependency_required() {
  let cargo_toml = std::fs::read_to_string("Cargo.toml").unwrap();
  assert!(cargo_toml.contains("new_crate"));
}
```

#### No Backup Files
```rust
#[test]
fn no_backup_files() {
  let backup_patterns = ["_old", "_backup", "_legacy", "_v1", ".bak"];

  for pattern in &backup_patterns {
    let result = Command::new("find")
      .args(&[".", "-name", &format!("*{pattern}*")])
      .output()
      .unwrap();

    assert!(result.stdout.is_empty(),
      "Found backup files matching *{pattern}*");
  }
}
```

#### No TODO Markers
```rust
#[test]
fn no_migration_todos() {
  let source = std::fs::read_to_string("src/target.rs").unwrap();

  let todo_patterns = [
    "TODO: migrate",
    "FIXME: old",
    "HACK: temporary",
  ];

  for pattern in &todo_patterns {
    assert!(!source.to_lowercase().contains(&pattern.to_lowercase()));
  }
}
```

### Pattern Detection Pitfalls

**Problem**: Exact string matching fails to detect variations

```rust
// Test looks for: env::var("DATABASE_URL")
// Actual code has: std::env::var("DATABASE_URL").ok()
// Result: False negative!
```

**Solution**: Use proximity-based detection

```rust
fn pattern_exists_nearby(source: &str, part1: &str, part2: &str, max_distance: usize) -> bool {
  let lines: Vec<&str> = source.lines().collect();

  for (i, line) in lines.iter().enumerate() {
    if line.contains(part1) {
      // Check nearby lines (±max_distance)
      let start = i.saturating_sub(max_distance);
      let end = (i + max_distance + 1).min(lines.len());

      for nearby_line in &lines[start..end] {
        if nearby_line.contains(part2) {
          return true;
        }
      }
    }
  }

  false
}

#[test]
fn no_manual_env_var() {
  let source = std::fs::read_to_string("src/config.rs").unwrap();

  // Check if env::var and "DATABASE_URL" appear within 2 lines
  assert!(!pattern_exists_nearby(&source, "env::var", "DATABASE_URL", 2));
}
```

---

## Tier 3: Active Breaking

**Purpose**: Prove old patterns don't work at runtime, not just that they're deleted

**Philosophy**: "Static checks prove code is gone; runtime checks prove functionality is gone"

### Active Breaking Tests

#### Old Inputs Ignored
```rust
#[test]
fn old_env_vars_ignored() {
  // Set old environment variables
  std::env::set_var("OLD_VAR_NAME", "should_not_work");

  // Load configuration
  let config = ConfigLoader::load().unwrap();

  // Old var should have no effect
  assert_ne!(config.value, "should_not_work");

  // Cleanup
  std::env::remove_var("OLD_VAR_NAME");
}
```

#### Removing New Dependency Breaks Build
```bash
#!/bin/bash
# Test: Removing new dependency should break compilation

# Remove new dependency
sed -i '/new_crate/d' Cargo.toml

# Try to build (should fail)
if cargo build 2>&1 | grep -q "error\[E0432\]: unresolved import"; then
  echo "✅ Build failed without new dependency (correct)"
  EXIT_CODE=0
else
  echo "❌ Build succeeded without new dependency (wrong!)"
  EXIT_CODE=1
fi

# Restore dependency
git checkout -- Cargo.toml

exit $EXIT_CODE
```

### Difference from Tier 2

- **Tier 2 (Static)**: Code doesn't exist
- **Tier 3 (Runtime)**: Code doesn't work if somehow present

---

## Tier 4: Rollback Impossibility

**Purpose**: Prove migration cannot be reversed (meta-testing)

**Philosophy**: "If you can go back, you haven't truly moved forward"

### Rollback Tests (Meta-Validation)

#### Test 1: Inject Old Pattern
```bash
#!/bin/bash
# Test: Injecting old pattern should be detected by Tier 2 tests

# Inject old code
sed -i '/target_line/a\    let x = old_pattern();' src/target.rs

# Run Tier 2 impossibility tests (should FAIL)
if cargo test impossibility_tests 2>&1 | grep -q "FAILED"; then
  echo "✅ Tier 2 tests detected injection (correct)"
  RESULT="PASS"
else
  echo "❌ Tier 2 tests didn't detect injection (bug in detection!)"
  RESULT="FAIL"
fi

# Restore file
git checkout -- src/target.rs

[ "$RESULT" = "PASS" ]
```

#### Test 2: Remove New Implementation
```bash
# Test: Removing new implementation should break tests

# Remove new code
sed -i '/new_implementation/d' src/new.rs

# Run tests (should FAIL)
if cargo test 2>&1 | grep -q "FAILED"; then
  echo "✅ Tests failed without new implementation (correct)"
else
  echo "❌ Tests passed without new implementation (fake tests!)"
fi

git checkout -- src/new.rs
```

#### Test 3: Restore Old Dependency
```bash
# Test: Re-adding old dependency should not make old code work

# Add old dependency back
echo 'old_crate = "1.0"' >> Cargo.toml

# Try to use old code (should fail - no imports, no code)
if cargo build 2>&1 | grep -q "error"; then
  echo "✅ Can't use old dependency (old code deleted)"
else
  echo "❌ Old dependency usable (migration incomplete!)"
fi

git checkout -- Cargo.toml
```

### Meta-Validation Concept

Rollback tests serve as **meta-tests** that validate other tests:

- **Tier 2 test**: "Old code doesn't exist"
- **Tier 4 test**: "If we inject old code, does Tier 2 detect it?"

This creates a two-level verification system:
1. Primary verification (Tiers 1-3)
2. Meta-verification that primary verification works (Tier 4)

---

## Tier 5: Quantitative Verification

**Purpose**: Measure migration with concrete numbers

**Philosophy**: "If the numbers don't shift, the migration didn't happen"

### Metrics to Track

#### Old Pattern Count
```bash
# Count uses of old pattern before and after
OLD_PATTERN_COUNT=$(grep -r "old_pattern" src/ | wc -l)
echo "Old pattern uses: $OLD_PATTERN_COUNT"
# Target: 0
```

#### New Pattern Count
```bash
# Count uses of new pattern
NEW_PATTERN_COUNT=$(grep -r "new_pattern" src/ | wc -l)
echo "New pattern uses: $NEW_PATTERN_COUNT"
# Target: ≥ expected count
```

#### Replacement Ratio
```bash
# Formula: new_usage / (old_usage + new_usage) × 100
REPLACEMENT_RATIO=$(echo "scale=2; $NEW_PATTERN_COUNT / ($OLD_PATTERN_COUNT + $NEW_PATTERN_COUNT) * 100" | bc)
echo "Replacement ratio: $REPLACEMENT_RATIO%"
# Target: 100%
```

#### Elimination Ratio
```bash
# Formula: current_old_usage / baseline_old_usage × 100
ELIMINATION_RATIO=$(echo "scale=2; $OLD_PATTERN_COUNT / $BASELINE_OLD_COUNT * 100" | bc)
echo "Elimination ratio: $ELIMINATION_RATIO%"
# Target: 0%
```

#### Adoption Ratio
```bash
# Formula: current_new_usage / expected_new_usage × 100
ADOPTION_RATIO=$(echo "scale=2; $NEW_PATTERN_COUNT / $EXPECTED_NEW_COUNT * 100" | bc)
echo "Adoption ratio: $ADOPTION_RATIO%"
# Target: ≥100%
```

### The Addition Trap

**Problem**: Adding new code alongside old instead of replacing

```bash
# Before migration
OLD_COUNT=5
NEW_COUNT=0

# After WRONG migration (addition, not replacement)
OLD_COUNT=5  # Still there!
NEW_COUNT=5  # Added new alongside old
REPLACEMENT_RATIO=50%  # Only 50%! ❌

# After CORRECT migration (replacement)
OLD_COUNT=0  # Deleted old
NEW_COUNT=5  # Added new
REPLACEMENT_RATIO=100%  # Complete replacement ✅
```

**Detection**: Replacement ratio must be 100%, not just >50%

---

## Using the 5-Tier Pyramid

### Step-by-Step Process

1. **Phase 0: Planning**
   - Identify old and new patterns
   - Define expected metrics
   - List potential shortcuts

2. **Phase 1: Tier 1 Tests (Anti-Shortcut)**
   - Write tests for each identified shortcut
   - Run tests (should pass initially)
   - Verify they catch fake implementations

3. **Phase 2: Tier 2 Tests (Impossibility)**
   - Write tests that old code doesn't exist
   - Run tests (should FAIL before migration)
   - These drive the migration (TDD)

4. **Phase 3: Perform Migration**
   - Delete old code completely
   - Implement new code
   - Verify Tier 2 tests now PASS

5. **Phase 4: Tier 3 Tests (Active Breaking)**
   - Verify old patterns don't work at runtime
   - Test with old inputs (should be ignored)
   - Test removing new dependency (should break)

6. **Phase 5: Tier 4 Tests (Rollback)**
   - Attempt to inject old code (Tier 2 should catch)
   - Attempt to remove new code (tests should break)
   - Verify migration is irreversible

7. **Phase 6: Tier 5 Metrics**
   - Measure old/new pattern counts
   - Calculate ratios
   - Verify all targets achieved

### Success Criteria

Migration is complete when ALL of:
- ✅ Tier 1: All anti-shortcut tests pass
- ✅ Tier 2: All impossibility tests pass
- ✅ Tier 3: All active breaking tests pass
- ✅ Tier 4: All rollback attempts fail
- ✅ Tier 5: Replacement ratio = 100%, elimination ratio = 0%, adoption ratio ≥ 100%

---

## Benefits of the Pyramid

### Comprehensive Coverage
- Each tier catches different types of issues
- Defense-in-depth: if one tier misses something, another catches it

### Confidence in Completion
- Quantitative metrics prove migration happened
- Meta-tests prove verification system works
- Multiple perspectives (static, runtime, metrics)

### Prevents Common Failures
- Anti-shortcut tests prevent fake implementations
- Quantitative metrics prevent "addition trap"
- Rollback tests prevent incomplete migrations

### Reusable Pattern
- Same 5-tier structure applies to any migration
- Methodology is language-agnostic
- Scales to migrations of any size

---

## Appendix: Quick Reference

### Tier Summary

| Tier | Question | Method | Pass Criteria |
|------|----------|--------|---------------|
| 1 | Is implementation real? | Anti-shortcut tests | No facades/mocks/hard-coding |
| 2 | Is old code deleted? | Static code analysis | No old patterns found |
| 3 | Does old way work? | Runtime testing | Old inputs ignored |
| 4 | Can we rollback? | Meta-testing | All rollback attempts fail |
| 5 | Did numbers shift? | Quantitative metrics | 100% replacement, 0% elimination |

### Common Pitfalls

1. **Exact string matching in pattern detection** → Use proximity-based detection
2. **Addition instead of replacement** → Track replacement ratio (must be 100%)
3. **Skipping meta-validation** → Use rollback tests to verify detection works
4. **Hard-coded test expectations** → Make tests dynamic based on actual code
5. **Ignoring quantitative evidence** → Always measure old → new shift

---

**Related Documents**:
- `tdd_methodology.md` - How to apply TDD to verification
- `migration_guide.md` - Step-by-step replication template
- `lessons_learned.md` - Insights from real migrations
