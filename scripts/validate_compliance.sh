#!/usr/bin/env bash
# Comprehensive compliance validation
# Part of comprehensive remediation plan automation framework

set -e

cd "$(dirname "$0")/.."

echo "=== Crate Distribution Compliance Validation ==="
echo ""

VIOLATIONS=0
CHECKS=0

# Check 1: Module responsibility tables
echo "Checking module responsibility tables..."
CHECKS=$((CHECKS + 20))
for module in module/*/readme.md; do
  if [ -f "$module" ]; then
    if ! grep -q "| File" "$module"; then
      echo "❌ MISSING: $module responsibility table"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  fi
done

# Check 2: License files
echo "Checking license files..."
CHECKS=$((CHECKS + 14))
for crate in module/iron_*; do
  if [ -d "$crate" ]; then
    if [ ! -f "$crate/license" ]; then
      echo "❌ MISSING: $crate/license"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  fi
done

# Check 3: Distribution metadata
echo "Checking distribution metadata..."
CHECKS=$((CHECKS + 42))  # 14 crates × 3 fields
for crate in module/iron_*/Cargo.toml; do
  if [ -f "$crate" ]; then
    if ! grep -q "^description" "$crate"; then
      echo "❌ MISSING: $crate description"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
    if ! grep -q "^keywords" "$crate"; then
      echo "❌ MISSING: $crate keywords"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
    if ! grep -q "^categories" "$crate"; then
      echo "❌ MISSING: $crate categories"
      VIOLATIONS=$((VIOLATIONS + 1))
    fi
  fi
done

# Check 4: Feature sections (optional check - not all crates need features)
echo "Checking [features] sections..."
CHECKS=$((CHECKS + 14))
feature_count=0
for crate in module/iron_*/Cargo.toml; do
  if [ -f "$crate" ]; then
    if grep -q "^\[features\]" "$crate"; then
      feature_count=$((feature_count + 1))
    fi
  fi
done
echo "  ℹ️  Found [features] sections in $feature_count crates"

# Calculate compliance
COMPLIANT=$((CHECKS - VIOLATIONS))
COMPLIANCE=$(awk "BEGIN {printf \"%.1f\", ($COMPLIANT / $CHECKS) * 100}")

echo ""
echo "=== Compliance Report ==="
echo "Checks Passed: $COMPLIANT / $CHECKS"
echo "Violations: $VIOLATIONS"
echo "Compliance: $COMPLIANCE%"
echo ""

if [ "$VIOLATIONS" -eq 0 ]; then
  echo "✅ All compliance checks passed!"
  exit 0
else
  echo "❌ $VIOLATIONS violations remaining"
  exit 1
fi
