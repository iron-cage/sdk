#!/usr/bin/env bash
# Layer 3: Anti-Workaround Verification
# Detect test-passing shortcuts and circumventions
#
# This script detects patterns that make tests pass without proper implementation.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

log INFO "=========================================="
log INFO "Layer 3: Anti-Workaround Verification"
log INFO "=========================================="

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

# Validation counters
total_checks=0
passing_checks=0
failing_checks=0

# === WORKAROUND 1: Early returns in tests ===
log INFO "Checking for early returns in tests..."
total_checks=$((total_checks + 1))

early_returns=$(find "$PROJECT_ROOT/tests" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -l "return;" 2>/dev/null | wc -l)

if [ "$early_returns" -eq 0 ]; then
  log INFO "   ✅ PASS: No early returns in tests"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $early_returns test files with early returns"
  find "$PROJECT_ROOT/tests" -name "*.rs" -type f 2>/dev/null | \
    xargs grep -l "return;" 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

# === WORKAROUND 2: Empty test functions ===
log INFO "Checking for empty test functions..."
total_checks=$((total_checks + 1))

# Pattern: #[test] followed by fn name() {} with only whitespace/comments inside
empty_tests=$(find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -Pzo '#\[test\][^{]*\{[\s\n]*\}' 2>/dev/null | grep -c "#\[test\]" || echo "0")

if [ "$empty_tests" -eq 0 ]; then
  log INFO "   ✅ PASS: No empty test functions"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $empty_tests empty test functions"
  failing_checks=$((failing_checks + 1))
fi

# === WORKAROUND 3: TODO/FIXME in test files ===
log INFO "Checking for TODO/FIXME markers in tests..."
total_checks=$((total_checks + 1))

todo_markers=$(find "$PROJECT_ROOT/tests" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -i "TODO\|FIXME\|XXX\|HACK" 2>/dev/null | wc -l)

if [ "$todo_markers" -eq 0 ]; then
  log INFO "   ✅ PASS: No TODO/FIXME markers in tests"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $todo_markers TODO/FIXME markers in tests"
  log WARN "   These may indicate incomplete test implementations"
  find "$PROJECT_ROOT/tests" -name "*.rs" -type f 2>/dev/null | \
    xargs grep -in "TODO\|FIXME\|XXX\|HACK" 2>/dev/null | head -5
  # Warning only - not a hard failure
  passing_checks=$((passing_checks + 1))
fi

# === WORKAROUND 4: Panic/unwrap in production code ===
log INFO "Checking for panic/unwrap in production code..."
total_checks=$((total_checks + 1))

panic_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f 2>/dev/null | \
  xargs grep -n "panic!\|unwrap()\|expect(" 2>/dev/null | \
  grep -v "test" | grep -v "debug_assert" | wc -l)

if [ "$panic_count" -eq 0 ]; then
  log INFO "   ✅ PASS: No panic/unwrap in production code"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $panic_count panic/unwrap in production code"
  find "$PROJECT_ROOT" -path "*/src/*.rs" -type f 2>/dev/null | \
    xargs grep -n "panic!\|unwrap()\|expect(" 2>/dev/null | \
    grep -v "test" | grep -v "debug_assert" | head -10
  failing_checks=$((failing_checks + 1))
fi

# === WORKAROUND 5: Hardcoded success responses ===
log INFO "Checking for hardcoded success responses..."
total_checks=$((total_checks + 1))

hardcoded=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f 2>/dev/null | \
  xargs grep -n 'Ok(\s*()' 2>/dev/null | \
  grep -v "test" | grep -v "// " | wc -l)

if [ "$hardcoded" -le 5 ]; then
  log INFO "   ✅ PASS: Minimal hardcoded Ok(()) responses ($hardcoded found)"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $hardcoded Ok(()) responses (review for workarounds)"
  # Warning only - Ok(()) is sometimes legitimate
  passing_checks=$((passing_checks + 1))
fi

# === WORKAROUND 6: Commented code blocks ===
log INFO "Checking for commented code blocks..."
total_checks=$((total_checks + 1))

commented_blocks=$(find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -c "^\\s*//.*{\\|^\\s*/\\*" 2>/dev/null | \
  awk -F: '$2 > 10 {print $1}' | wc -l)

if [ "$commented_blocks" -eq 0 ]; then
  log INFO "   ✅ PASS: No large commented code blocks"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $commented_blocks files with large commented blocks"
  # Warning only
  passing_checks=$((passing_checks + 1))
fi

# === WORKAROUND 7: Test-only feature flags ===
log INFO "Checking for test-only feature flags in production..."
total_checks=$((total_checks + 1))

test_features=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f 2>/dev/null | \
  xargs grep -n "#\[cfg(test)\]" 2>/dev/null | \
  grep -v "mod tests" | wc -l)

if [ "$test_features" -eq 0 ]; then
  log INFO "   ✅ PASS: No test-only features in production code"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $test_features cfg(test) in production code"
  log WARN "   Verify these are legitimate (module declarations OK)"
  # Warning only
  passing_checks=$((passing_checks + 1))
fi

# === WORKAROUND 8: Unreachable code ===
log INFO "Checking for unreachable code..."
total_checks=$((total_checks + 1))

unreachable=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f 2>/dev/null | \
  xargs grep -n "unreachable!" 2>/dev/null | wc -l)

if [ "$unreachable" -eq 0 ]; then
  log INFO "   ✅ PASS: No unreachable!() macros"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $unreachable unreachable!() macros"
  # Warning only - sometimes legitimate
  passing_checks=$((passing_checks + 1))
fi

# === SUMMARY ===
echo ""
log INFO "=========================================="
log INFO "Anti-Workaround Summary"
log INFO "=========================================="
log INFO "Total checks: $total_checks"
log INFO "Passing: $passing_checks"
log INFO "Failing: $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  log INFO "✅ Layer 3 (Anti-Workaround): PASS"
  exit 0
else
  log ERROR "❌ Layer 3 (Anti-Workaround): FAIL"
  exit 1
fi
