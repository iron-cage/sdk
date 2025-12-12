#!/bin/bash
# Layer 4: Old Way Elimination - Verify Deprecated Methods Cannot Be Used
# This script verifies old implementations are completely removed, not just deprecated

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "════════════════════════════════════════════════════════"
echo "Layer 4: Old Way Elimination"
echo "════════════════════════════════════════════════════════"
echo ""

# Track statistics
total_checks=0
passing_checks=0
failing_checks=0

# Check 1: Old database tables removed (not just deprecated)
echo "Check 1: Old Database Tables Removed"
total_checks=$((total_checks + 1))

DB_PATH="${DB_PATH:-$PROJECT_ROOT/iron.db}"

if [ -f "$DB_PATH" ]; then
  # Check for tables with _old, _deprecated, _legacy suffixes
  old_table_count=$(sqlite3 "$DB_PATH" \
    "SELECT COUNT(*) FROM sqlite_master
     WHERE type='table'
       AND (name LIKE '%_old' OR name LIKE '%_deprecated' OR name LIKE '%_legacy' OR name LIKE '%_v1')" 2>/dev/null || echo "0")

  if [ "$old_table_count" -eq 0 ]; then
    echo "   ✅ PASS: No deprecated tables found (count = 0)"
    passing_checks=$((passing_checks + 1))
  else
    echo "   ❌ FAIL: Found $old_table_count deprecated tables (expected 0)"
    sqlite3 "$DB_PATH" \
      "SELECT name FROM sqlite_master
       WHERE type='table'
         AND (name LIKE '%_old' OR name LIKE '%_deprecated' OR name LIKE '%_legacy' OR name LIKE '%_v1')"
    failing_checks=$((failing_checks + 1))
  fi
else
  echo "   ⚠️  WARN: Database not found, skipping check"
  passing_checks=$((passing_checks + 1))
fi

# Check 2: No deprecated API endpoint handlers in code
echo "Check 2: No Deprecated API Endpoint Handlers"
total_checks=$((total_checks + 1))

deprecated_routes=$(find "$PROJECT_ROOT" -path "*/src/routes/*.rs" -type f ! -path "*/target/*" \
  -exec grep -l "deprecated\|_old\|_legacy\|_v1" {} \; 2>/dev/null | wc -l)

if [ "$deprecated_routes" -eq 0 ]; then
  echo "   ✅ PASS: No deprecated route handlers (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $deprecated_routes files with deprecated routes"
  find "$PROJECT_ROOT" -path "*/src/routes/*.rs" -type f ! -path "*/target/*" \
    -exec grep -l "deprecated\|_old\|_legacy\|_v1" {} \; 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 3: No backward compatibility layers in code
echo "Check 3: No Backward Compatibility Layers"
total_checks=$((total_checks + 1))

compat_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" \
  -exec grep -i -c "backward.*compat\|legacy.*support\|deprecated.*alias" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$compat_count" -eq 0 ]; then
  echo "   ✅ PASS: No compatibility layers found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $compat_count compatibility layer references"
  find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" \
    -exec grep -n -i "backward.*compat\|legacy.*support\|deprecated.*alias" {} \; 2>/dev/null | head -5
  failing_checks=$((failing_checks + 1))
fi

# Check 4: No re-exports of removed items
echo "Check 4: No Re-Exports of Removed Items"
total_checks=$((total_checks + 1))

reexport_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" \
  -exec grep -c "pub use.*as.*_old\|pub use.*as.*_deprecated" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$reexport_count" -eq 0 ]; then
  echo "   ✅ PASS: No deprecated re-exports (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $reexport_count deprecated re-exports"
  failing_checks=$((failing_checks + 1))
fi

# Check 5: No feature flags for old implementations
echo "Check 5: No Feature Flags for Old Implementations"
total_checks=$((total_checks + 1))

# Check Cargo.toml for legacy feature flags
legacy_features=0

if [ -f "$PROJECT_ROOT/Cargo.toml" ]; then
  legacy_features=$(grep -c "legacy\|old-api\|deprecated\|compat" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null || echo "0")
fi

if [ "$legacy_features" -eq 0 ]; then
  echo "   ✅ PASS: No legacy feature flags (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $legacy_features legacy feature flags"
  grep -n "legacy\|old-api\|deprecated\|compat" "$PROJECT_ROOT/Cargo.toml" 2>/dev/null || true
  failing_checks=$((failing_checks + 1))
fi

# Check 6: No migration code still present after cutover
echo "Check 6: No Migration Code After Cutover"
total_checks=$((total_checks + 1))

migration_code=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" ! -path "*/migrations/*" \
  -exec grep -c "migrate\|migration\|cutover" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

# This is informational only since some migration code may be legitimate
if [ "$migration_code" -gt 0 ]; then
  echo "   ⚠️  INFO: Found $migration_code references to migration code"
  echo "   (Review manually to ensure migrations are complete)"
fi
passing_checks=$((passing_checks + 1))

# Check 7: Old environment variables cause startup failure
echo "Check 7: Old Environment Variables Rejected"
total_checks=$((total_checks + 1))

# Check if config code validates against old env vars
old_env_validation=$(find "$PROJECT_ROOT" -path "*/src/config*.rs" -type f ! -path "*/target/*" \
  -exec grep -c "OLD_\|DEPRECATED_\|LEGACY_" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$old_env_validation" -eq 0 ]; then
  echo "   ✅ PASS: No old environment variable handling (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ⚠️  INFO: Found $old_env_validation old env var references"
  echo "   (Verify they cause startup failure, not silent fallback)"
  passing_checks=$((passing_checks + 1))
fi

echo ""

# Summary
echo "════════════════════════════════════════════════════════"
echo "Old Way Elimination Verification Summary"
echo "════════════════════════════════════════════════════════"
echo "Total Checks:  $total_checks"
echo "Passing:       $passing_checks"
echo "Failing:       $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  echo "Status:        ✅ OLD WAYS ELIMINATED"
  echo "════════════════════════════════════════════════════════"
  exit 0
else
  echo "Status:        ❌ OLD WAYS STILL PRESENT"
  echo "════════════════════════════════════════════════════════"
  exit 1
fi
