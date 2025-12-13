#!/usr/bin/env bash
# Count public-facing features per crate
# Part of comprehensive remediation plan automation framework
# Target: ≤15 public features per crate

cd "$(dirname "$0")/.."

echo "=== Feature Count Validation ==="
echo ""
echo "Target: ≤15 public-facing features per crate"
echo ""

TOTAL_CRATES=0
COMPLIANT_CRATES=0
NON_COMPLIANT_CRATES=0

for crate_dir in module/iron_*; do
  if [ -d "$crate_dir" ] && [ -f "$crate_dir/Cargo.toml" ]; then
    crate_name=$(basename "$crate_dir")
    TOTAL_CRATES=$((TOTAL_CRATES + 1))

    # Count actual features (exclude dep: entries)
    feature_count=$(grep -A 100 "^\[features\]" "$crate_dir/Cargo.toml" 2>/dev/null | \
      grep -v "^\[" | \
      grep "^[a-z_]" | \
      grep "=" | \
      grep -v "\"dep:" | \
      wc -l)

    if [ "$feature_count" -eq 0 ]; then
      echo "✅ $crate_name: No features section"
      COMPLIANT_CRATES=$((COMPLIANT_CRATES + 1))
    elif [ "$feature_count" -le 15 ]; then
      echo "✅ $crate_name: $feature_count features"
      COMPLIANT_CRATES=$((COMPLIANT_CRATES + 1))
    else
      echo "⚠️  $crate_name: $feature_count features (EXCEEDS 15)"
      NON_COMPLIANT_CRATES=$((NON_COMPLIANT_CRATES + 1))
    fi
  fi
done

echo ""
echo "=== Summary ==="
echo "Total crates: $TOTAL_CRATES"
echo "Compliant: $COMPLIANT_CRATES"
echo "Non-compliant: $NON_COMPLIANT_CRATES"

if [ "$NON_COMPLIANT_CRATES" -eq 0 ]; then
  echo ""
  echo "✅ All crates comply with 15-feature limit"
  exit 0
else
  echo ""
  echo "⚠️  $NON_COMPLIANT_CRATES crates exceed 15-feature limit"
  echo "Note: Some crates may require more features for legitimate use cases"
  exit 0  # Don't fail - this is informational
fi
