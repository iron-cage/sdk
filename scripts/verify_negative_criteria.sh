#!/bin/bash
# Layer 2: NEGATIVE CRITERIA - Verify Anti-Patterns Count = 0
# This script checks that prohibited patterns do not exist in the codebase

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "════════════════════════════════════════════════════════"
echo "Layer 2: NEGATIVE CRITERIA - Anti-Pattern Detection"
echo "════════════════════════════════════════════════════════"
echo ""

# Track statistics
total_checks=0
passing_checks=0
failing_checks=0

# Check 1: No backup files
echo "Check 1: No Backup Files (*_backup, *_old, *_v1, *.bak)"
total_checks=$((total_checks + 1))

backup_count=$(find "$PROJECT_ROOT" -type f \
  \( -name "*_backup*" -o -name "*_old*" -o -name "*_v[0-9]*" -o -name "*.bak" -o -name "*.orig" \) \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/node_modules/*" \
  | wc -l)

if [ "$backup_count" -eq 0 ]; then
  echo "   ✅ PASS: No backup files found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $backup_count backup files (expected 0)"
  find "$PROJECT_ROOT" -type f \
    \( -name "*_backup*" -o -name "*_old*" -o -name "*_v[0-9]*" -o -name "*.bak" -o -name "*.orig" \) \
    ! -path "*/target/*" \
    ! -path "*/.git/*" \
    ! -path "*/node_modules/*" \
    | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 2: No prohibited filenames (utils.rs, helpers.rs, common.rs)
echo "Check 2: No Prohibited Filenames (utils.rs, helpers.rs, common.rs)"
total_checks=$((total_checks + 1))

prohibited_count=$(find "$PROJECT_ROOT" -type f \
  \( -name "utils.rs" -o -name "helpers.rs" -o -name "common.rs" -o -name "misc.rs" \) \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  | wc -l)

if [ "$prohibited_count" -eq 0 ]; then
  echo "   ✅ PASS: No prohibited filenames found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $prohibited_count prohibited filenames (expected 0)"
  find "$PROJECT_ROOT" -type f \
    \( -name "utils.rs" -o -name "helpers.rs" -o -name "common.rs" -o -name "misc.rs" \) \
    ! -path "*/target/*" \
    ! -path "*/.git/*" \
    | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 3: No REAL columns for money (should be INTEGER microdollars)
echo "Check 3: No REAL Columns for Money Fields"
total_checks=$((total_checks + 1))

DB_PATH="${DB_PATH:-$PROJECT_ROOT/iron.db}"

if [ -f "$DB_PATH" ]; then
  real_money_count=$(sqlite3 "$DB_PATH" \
    "SELECT COUNT(*) FROM (
       SELECT m.name as table_name, p.name as column_name, p.type
       FROM sqlite_master m
       JOIN pragma_table_info(m.name) p
       WHERE m.type='table'
         AND (p.name LIKE '%budget%' OR p.name LIKE '%cost%' OR p.name LIKE '%price%' OR p.name LIKE '%amount%')
         AND p.type LIKE '%REAL%'
     )" 2>/dev/null || echo "0")

  if [ "$real_money_count" -eq 0 ]; then
    echo "   ✅ PASS: No REAL columns for money (count = 0)"
    passing_checks=$((passing_checks + 1))
  else
    echo "   ❌ FAIL: Found $real_money_count REAL columns for money (expected 0)"
    failing_checks=$((failing_checks + 1))
  fi
else
  echo "   ⚠️  WARN: Database not found, skipping check"
  passing_checks=$((passing_checks + 1))
fi

# Check 4: No non-hyphenated temporary files in project root
echo "Check 4: No Non-Hyphenated Temporary Files"
total_checks=$((total_checks + 1))

temp_pattern_count=$(find "$PROJECT_ROOT" -maxdepth 2 -type f \
  \( -name "*temp*" -o -name "*tmp*" -o -name "*scratch*" -o -name "*notes*" -o -name "*draft*" \) \
  ! -name "-*" \
  ! -path "*/target/*" \
  ! -path "*/.git/*" \
  ! -path "*/node_modules/*" \
  | wc -l)

if [ "$temp_pattern_count" -eq 0 ]; then
  echo "   ✅ PASS: No non-hyphenated temp files (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $temp_pattern_count non-hyphenated temp files (expected 0)"
  find "$PROJECT_ROOT" -maxdepth 2 -type f \
    \( -name "*temp*" -o -name "*tmp*" -o -name "*scratch*" -o -name "*notes*" -o -name "*draft*" \) \
    ! -name "-*" \
    ! -path "*/target/*" \
    ! -path "*/.git/*" \
    ! -path "*/node_modules/*" \
    | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 5: No mocking frameworks in test files
echo "Check 5: No Mocking Frameworks in Tests"
total_checks=$((total_checks + 1))

mock_count=$(find "$PROJECT_ROOT" -path "*/tests/*.rs" -type f -exec grep -l "mockall\|mockito\|mock::" {} \; 2>/dev/null | wc -l)

if [ "$mock_count" -eq 0 ]; then
  echo "   ✅ PASS: No mocking frameworks found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $mock_count test files using mocks (expected 0)"
  find "$PROJECT_ROOT" -path "*/tests/*.rs" -type f -exec grep -l "mockall\|mockito\|mock::" {} \; 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 6: No disabled/ignored tests
echo "Check 6: No Disabled or Ignored Tests"
total_checks=$((total_checks + 1))

ignored_test_count=$(find "$PROJECT_ROOT" -path "*/tests/*.rs" -o -path "*/src/*.rs" -type f -exec grep -c "#\[ignore\]" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$ignored_test_count" -eq 0 ]; then
  echo "   ✅ PASS: No ignored tests found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $ignored_test_count ignored tests (expected 0)"
  find "$PROJECT_ROOT" -path "*/tests/*.rs" -o -path "*/src/*.rs" -type f -exec grep -l "#\[ignore\]" {} \; 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 7: No cargo fmt usage (should use custom codestyle)
echo "Check 7: No cargo fmt in CI/CD Scripts"
total_checks=$((total_checks + 1))

cargo_fmt_count=$(find "$PROJECT_ROOT" -name "*.sh" -o -name "*.yml" -o -name "*.yaml" -type f -exec grep -c "cargo fmt" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$cargo_fmt_count" -eq 0 ]; then
  echo "   ✅ PASS: No cargo fmt found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $cargo_fmt_count cargo fmt usages (expected 0)"
  find "$PROJECT_ROOT" -name "*.sh" -o -name "*.yml" -o -name "*.yaml" -type f -exec grep -l "cargo fmt" {} \; 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

echo ""

# Summary
echo "════════════════════════════════════════════════════════"
echo "NEGATIVE CRITERIA Verification Summary"
echo "════════════════════════════════════════════════════════"
echo "Total Checks:  $total_checks"
echo "Passing:       $passing_checks"
echo "Failing:       $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  echo "Status:        ✅ ALL ANTI-PATTERNS ELIMINATED"
  echo "════════════════════════════════════════════════════════"
  exit 0
else
  echo "Status:        ❌ ANTI-PATTERNS DETECTED"
  echo "════════════════════════════════════════════════════════"
  exit 1
fi
