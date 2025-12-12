#!/usr/bin/env bash
# Layer 2: NEGATIVE CRITERIA Verification
# Anti-pattern prevention: count=0 checks
#
# This script ensures prohibited patterns have ZERO occurrences in the codebase.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

log INFO "=========================================="
log INFO "Layer 2: NEGATIVE CRITERIA Verification"
log INFO "=========================================="

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

# Validation counters
total_checks=0
passing_checks=0
failing_checks=0

# === NEGATIVE CRITERIA 1: No backup files ===
log INFO "Checking for backup files..."
total_checks=$((total_checks + 1))

backup_files=$(find "$PROJECT_ROOT" -type f \( \
  -name "*_backup*" -o \
  -name "*_old*" -o \
  -name "*_v1*" -o \
  -name "*_v2*" -o \
  -name "*_legacy*" -o \
  -name "*.bak" -o \
  -name "*.orig" \
  \) 2>/dev/null | grep -v ".git" | wc -l)

if [ "$backup_files" -eq 0 ]; then
  log INFO "   ✅ PASS: No backup files found (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $backup_files backup files"
  find "$PROJECT_ROOT" -type f \( \
    -name "*_backup*" -o \
    -name "*_old*" -o \
    -name "*_v1*" -o \
    -name "*_v2*" -o \
    -name "*_legacy*" -o \
    -name "*.bak" -o \
    -name "*.orig" \
    \) 2>/dev/null | grep -v ".git" | head -10
  failing_checks=$((failing_checks + 1))
fi

# === NEGATIVE CRITERIA 2: No prohibited filenames ===
log INFO "Checking for prohibited filenames..."
total_checks=$((total_checks + 1))

prohibited_files=$(find "$PROJECT_ROOT" -type f \( \
  -name "utils.rs" -o \
  -name "helpers.rs" -o \
  -name "common.rs" -o \
  -name "misc.rs" \
  \) 2>/dev/null | grep -v ".git" | grep -v "target/" | wc -l)

if [ "$prohibited_files" -eq 0 ]; then
  log INFO "   ✅ PASS: No prohibited filenames (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $prohibited_files prohibited filenames"
  find "$PROJECT_ROOT" -type f \( \
    -name "utils.rs" -o \
    -name "helpers.rs" -o \
    -name "common.rs" -o \
    -name "misc.rs" \
    \) 2>/dev/null | grep -v ".git" | grep -v "target/"
  failing_checks=$((failing_checks + 1))
fi

# === NEGATIVE CRITERIA 3: No REAL columns for money ===
log INFO "Checking for REAL columns (money must be INTEGER)..."
total_checks=$((total_checks + 1))

DB_PATH="${DB_PATH:-$PROJECT_ROOT/iron.db}"

if [ -f "$DB_PATH" ]; then
  real_money_columns=$(sqlite3 "$DB_PATH" \
    "SELECT tbl_name || '.' || name FROM pragma_table_info(
       SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'
     ) WHERE type='REAL' AND (
       name LIKE '%budget%' OR
       name LIKE '%cost%' OR
       name LIKE '%price%' OR
       name LIKE '%amount%' OR
       name LIKE '%dollar%'
     );" 2>/dev/null | wc -l)

  if [ "$real_money_columns" -eq 0 ]; then
    log INFO "   ✅ PASS: No REAL columns for money (count=0)"
    passing_checks=$((passing_checks + 1))
  else
    log ERROR "   ❌ FAIL: Found $real_money_columns REAL columns for money"
    failing_checks=$((failing_checks + 1))
  fi
else
  log WARN "   ⚠️  SKIP: Database not found"
  total_checks=$((total_checks - 1))
fi

# === NEGATIVE CRITERIA 4: No non-hyphenated temp files ===
log INFO "Checking for non-hyphenated temp files..."
total_checks=$((total_checks + 1))

temp_keywords=(
  "temp"
  "tmp"
  "scratch"
  "test_plan"
  "notes"
  "draft"
  "experiment"
)

temp_files_count=0
for keyword in "${temp_keywords[@]}"; do
  # Find files containing keyword but NOT starting with hyphen
  count=$(find "$PROJECT_ROOT" -maxdepth 2 -type f -name "*${keyword}*" \
    ! -name "-*" 2>/dev/null | grep -v ".git" | grep -v "target/" | wc -l)
  temp_files_count=$((temp_files_count + count))
done

if [ "$temp_files_count" -eq 0 ]; then
  log INFO "   ✅ PASS: No non-hyphenated temp files (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  FAIL: Found $temp_files_count non-hyphenated temp files"
  log WARN "   Temp files must start with '-' for git exclusion"
  failing_checks=$((failing_checks + 1))
fi

# === NEGATIVE CRITERIA 5: No mocking frameworks ===
log INFO "Checking for mocking frameworks..."
total_checks=$((total_checks + 1))

mock_usage=$(grep -r "mockall\|mockito\|mock_derive" "$PROJECT_ROOT" \
  --include="*.rs" --include="*.toml" 2>/dev/null | grep -v "target/" | wc -l)

if [ "$mock_usage" -eq 0 ]; then
  log INFO "   ✅ PASS: No mocking frameworks (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $mock_usage references to mocking frameworks"
  grep -r "mockall\|mockito\|mock_derive" "$PROJECT_ROOT" \
    --include="*.rs" --include="*.toml" 2>/dev/null | grep -v "target/" | head -5
  failing_checks=$((failing_checks + 1))
fi

# === NEGATIVE CRITERIA 6: No ignored/disabled tests ===
log INFO "Checking for ignored tests..."
total_checks=$((total_checks + 1))

ignored_tests=$(grep -r "#\[ignore\]\|#\[cfg(.*ignore.*)\]" "$PROJECT_ROOT" \
  --include="*.rs" 2>/dev/null | grep -v "target/" | wc -l)

if [ "$ignored_tests" -eq 0 ]; then
  log INFO "   ✅ PASS: No ignored tests (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $ignored_tests ignored tests"
  grep -r "#\[ignore\]\|#\[cfg(.*ignore.*)\]" "$PROJECT_ROOT" \
    --include="*.rs" 2>/dev/null | grep -v "target/" | head -5
  failing_checks=$((failing_checks + 1))
fi

# === NEGATIVE CRITERIA 7: No cargo fmt usage ===
log INFO "Checking for cargo fmt usage..."
total_checks=$((total_checks + 1))

cargo_fmt_refs=$(grep -r "cargo fmt\|rustfmt" "$PROJECT_ROOT" \
  --include="*.sh" --include="*.md" --include="*.yml" --include="*.yaml" \
  2>/dev/null | grep -v ".git" | grep -v "Forbidden" | wc -l)

if [ "$cargo_fmt_refs" -eq 0 ]; then
  log INFO "   ✅ PASS: No cargo fmt usage (count=0)"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $cargo_fmt_refs cargo fmt references (verify they're documentation only)"
  passing_checks=$((passing_checks + 1))  # Warning only
fi

# === SUMMARY ===
echo ""
log INFO "=========================================="
log INFO "NEGATIVE CRITERIA Summary"
log INFO "=========================================="
log INFO "Total checks: $total_checks"
log INFO "Passing: $passing_checks"
log INFO "Failing: $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  log INFO "✅ Layer 2 (NEGATIVE CRITERIA): PASS"
  exit 0
else
  log ERROR "❌ Layer 2 (NEGATIVE CRITERIA): FAIL"
  exit 1
fi
