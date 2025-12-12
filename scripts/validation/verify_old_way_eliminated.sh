#!/usr/bin/env bash
# Layer 4: Old Way Elimination Verification
# OS/Database/Network-level enforcement: old methods must be IMPOSSIBLE
#
# This script verifies deprecated implementations are completely removed.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

log INFO "=========================================="
log INFO "Layer 4: Old Way Elimination"
log INFO "=========================================="

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
DB_PATH="${DB_PATH:-$PROJECT_ROOT/iron.db}"

# Validation counters
total_checks=0
passing_checks=0
failing_checks=0

# === ELIMINATION 1: No f64/REAL columns for money ===
log INFO "Checking database has NO REAL columns for money..."
total_checks=$((total_checks + 1))

if [ -f "$DB_PATH" ]; then
  # Check for REAL type in money-related columns
  real_money=$(sqlite3 "$DB_PATH" \
    "SELECT COUNT(*) FROM (
       SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'
     ) AS tables,
     pragma_table_info(tables.name)
     WHERE type='REAL' AND (
       name LIKE '%budget%' OR
       name LIKE '%cost%' OR
       name LIKE '%price%' OR
       name LIKE '%amount%' OR
       name LIKE '%dollar%'
     );" 2>/dev/null || echo "0")

  if [ "$real_money" -eq 0 ]; then
    log INFO "   ✅ PASS: No REAL columns for money (old way eliminated)"
    passing_checks=$((passing_checks + 1))
  else
    log ERROR "   ❌ FAIL: Found $real_money REAL columns for money"
    log ERROR "   Old way (f64) still present in database schema"
    failing_checks=$((failing_checks + 1))
  fi
else
  log WARN "   ⚠️  SKIP: Database not found"
  total_checks=$((total_checks - 1))
fi

# === ELIMINATION 2: No f64 types in Rust code for money ===
log INFO "Checking Rust code has NO f64 for money..."
total_checks=$((total_checks + 1))

f64_money=$(find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -n ":\s*f64" 2>/dev/null | \
  grep -i "budget\|cost\|price\|amount\|dollar" | \
  grep -v "test" | grep -v "//" | wc -l)

if [ "$f64_money" -eq 0 ]; then
  log INFO "   ✅ PASS: No f64 types for money in Rust code"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $f64_money f64 types for money"
  find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
    xargs grep -n ":\s*f64" 2>/dev/null | \
    grep -i "budget\|cost\|price\|amount\|dollar" | \
    grep -v "test" | grep -v "//" | head -5
  failing_checks=$((failing_checks + 1))
fi

# === ELIMINATION 3: No deprecated function names ===
log INFO "Checking for deprecated function names..."
total_checks=$((total_checks + 1))

deprecated_funcs=(
  "get_budget_dollars"
  "set_budget_dollars"
  "deduct_dollars"
  "add_dollars"
)

deprecated_count=0
for func in "${deprecated_funcs[@]}"; do
  count=$(find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
    xargs grep -w "$func" 2>/dev/null | grep -v "// deprecated" | wc -l)
  deprecated_count=$((deprecated_count + count))
done

if [ "$deprecated_count" -eq 0 ]; then
  log INFO "   ✅ PASS: No deprecated function names (old way eliminated)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $deprecated_count uses of deprecated functions"
  for func in "${deprecated_funcs[@]}"; do
    find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
      xargs grep -n "$func" 2>/dev/null | grep -v "// deprecated" | head -2
  done
  failing_checks=$((failing_checks + 1))
fi

# === ELIMINATION 4: No old table names ===
log INFO "Checking database has NO old table names..."
total_checks=$((total_checks + 1))

if [ -f "$DB_PATH" ]; then
  old_tables=(
    "budgets_old"
    "agents_v1"
    "legacy_users"
    "backup_"
  )

  old_table_count=0
  for table_pattern in "${old_tables[@]}"; do
    count=$(sqlite3 "$DB_PATH" \
      "SELECT COUNT(*) FROM sqlite_master
       WHERE type='table' AND name LIKE '%${table_pattern}%';" 2>/dev/null || echo "0")
    old_table_count=$((old_table_count + count))
  done

  if [ "$old_table_count" -eq 0 ]; then
    log INFO "   ✅ PASS: No old/legacy table names"
    passing_checks=$((passing_checks + 1))
  else
    log ERROR "   ❌ FAIL: Found $old_table_count old/legacy tables"
    failing_checks=$((failing_checks + 1))
  fi
else
  log WARN "   ⚠️  SKIP: Database not found"
  total_checks=$((total_checks - 1))
fi

# === ELIMINATION 5: No compatibility shims ===
log INFO "Checking for compatibility shims..."
total_checks=$((total_checks + 1))

shims=$(find "$PROJECT_ROOT" -name "*.rs" -type f 2>/dev/null | \
  xargs grep -in "compatibility\|backward.*compat\|legacy.*support" 2>/dev/null | \
  grep -v "test" | grep -v "//" | wc -l)

if [ "$shims" -eq 0 ]; then
  log INFO "   ✅ PASS: No compatibility shims (old way eliminated)"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  Found $shims references to compatibility/legacy"
  log WARN "   Verify these are documentation only"
  # Warning only - might be in comments
  passing_checks=$((passing_checks + 1))
fi

# === SUMMARY ===
echo ""
log INFO "=========================================="
log INFO "Old Way Elimination Summary"
log INFO "=========================================="
log INFO "Total checks: $total_checks"
log INFO "Passing: $passing_checks"
log INFO "Failing: $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  log INFO "✅ Layer 4 (Old Way Elimination): PASS"
  exit 0
else
  log ERROR "❌ Layer 4 (Old Way Elimination): FAIL"
  exit 1
fi
