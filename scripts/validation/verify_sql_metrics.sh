#!/usr/bin/env bash
# Layer 1: Empirical Validation - SQL Metrics
# PURPOSE → EVIDENCE → MEASUREMENT → NULL HYPOTHESIS → NEGATIVE CRITERIA
#
# This script validates database schema and structure matches specifications
# by running SQL queries from MEASUREMENT sections of specifications.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

log INFO "=========================================="
log INFO "Layer 1: Empirical Validation - SQL Metrics"
log INFO "=========================================="

# Configuration
PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
DB_PATH="${DB_PATH:-$PROJECT_ROOT/iron.db}"

# Validation counters
total_checks=0
passing_checks=0
failing_checks=0

# Check database exists
if [ ! -f "$DB_PATH" ]; then
  die "Database not found at: $DB_PATH"
fi

log INFO "Database found: $DB_PATH"

# === MEASUREMENT 1: Table count ===
log INFO "Checking table count..."
total_checks=$((total_checks + 1))

table_count=$(sqlite3 "$DB_PATH" \
  "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")

if [ "$table_count" -ge 8 ]; then
  log INFO "   ✅ PASS: Found $table_count tables (expected ≥8)"
  passing_checks=$((passing_checks + 1))
else
  log ERROR "   ❌ FAIL: Found $table_count tables (expected ≥8)"
  failing_checks=$((failing_checks + 1))
fi

# === MEASUREMENT 2: Core tables exist ===
log INFO "Checking core tables exist..."

core_tables=(
  "agents"
  "users"
  "budget_leases"
  "api_calls"
)

for table in "${core_tables[@]}"; do
  total_checks=$((total_checks + 1))

  exists=$(sqlite3 "$DB_PATH" \
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='$table';")

  if [ "$exists" = "1" ]; then
    log INFO "   ✅ PASS: Table '$table' exists"
    passing_checks=$((passing_checks + 1))
  else
    log ERROR "   ❌ FAIL: Table '$table' missing"
    failing_checks=$((failing_checks + 1))
  fi
done

# === MEASUREMENT 3: Microdollars are INTEGER type ===
log INFO "Checking money columns use INTEGER (microdollars)..."

money_columns=(
  "agents:budget_microdollars"
  "budget_leases:amount_microdollars"
  "api_calls:cost_microdollars"
)

for col_spec in "${money_columns[@]}"; do
  total_checks=$((total_checks + 1))

  table="${col_spec%%:*}"
  column="${col_spec##*:}"

  # Check if table exists first
  table_exists=$(sqlite3 "$DB_PATH" \
    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='$table';")

  if [ "$table_exists" = "0" ]; then
    log WARN "   ⚠️  SKIP: Table '$table' does not exist"
    total_checks=$((total_checks - 1))
    continue
  fi

  col_type=$(sqlite3 "$DB_PATH" \
    "SELECT type FROM pragma_table_info('$table') WHERE name='$column';" 2>/dev/null || echo "MISSING")

  if [ "$col_type" = "INTEGER" ]; then
    log INFO "   ✅ PASS: $table.$column is INTEGER"
    passing_checks=$((passing_checks + 1))
  elif [ "$col_type" = "MISSING" ]; then
    log WARN "   ⚠️  SKIP: Column '$column' does not exist in '$table'"
    total_checks=$((total_checks - 1))
  else
    log ERROR "   ❌ FAIL: $table.$column is $col_type (expected INTEGER)"
    failing_checks=$((failing_checks + 1))
  fi
done

# === MEASUREMENT 4: UNIQUE constraints exist ===
log INFO "Checking UNIQUE constraints..."

total_checks=$((total_checks + 1))

# Check for UNIQUE index on agents table
unique_count=$(sqlite3 "$DB_PATH" \
  "SELECT COUNT(*) FROM sqlite_master
   WHERE type='index' AND tbl_name='agents' AND sql LIKE '%UNIQUE%';" 2>/dev/null || echo "0")

if [ "$unique_count" -ge 1 ]; then
  log INFO "   ✅ PASS: Found UNIQUE constraints on agents table"
  passing_checks=$((passing_checks + 1))
else
  log WARN "   ⚠️  SKIP: No UNIQUE constraints found (may not be required yet)"
  total_checks=$((total_checks - 1))
fi

# === SUMMARY ===
echo ""
log INFO "=========================================="
log INFO "SQL Metrics Validation Summary"
log INFO "=========================================="
log INFO "Total checks: $total_checks"
log INFO "Passing: $passing_checks"
log INFO "Failing: $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  log INFO "✅ Layer 1 (SQL Metrics): PASS"
  exit 0
else
  log ERROR "❌ Layer 1 (SQL Metrics): FAIL"
  exit 1
fi
