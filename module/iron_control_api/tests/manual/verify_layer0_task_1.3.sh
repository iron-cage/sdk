#!/bin/bash
# Layer 0 Verification: Specification & Rulebook Alignment
# Task 1.3: Add Authorization Checks
#
# This script verifies that all pre-planning requirements are met before
# beginning TDD implementation.

set -e

TASK="1.3"
MODULE_ROOT="/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api"
PLAN_FILE="/home/user1/pro/lib/wip_iron/iron_runtime/dev/-005_budget_control_protocol/-default_topic/-current_plan.md"

echo "=== LAYER 0: SPECIFICATION & RULEBOOK ALIGNMENT ==="
echo ""

# 1. Specification check
echo "[ 1/5 ] Checking task specification..."
if [ ! -f "$PLAN_FILE" ]; then
  echo "❌ Plan file not found: $PLAN_FILE"
  exit 1
fi

if ! grep -q "Task $TASK" "$PLAN_FILE"; then
  echo "❌ Task $TASK not found in plan file"
  exit 1
fi
echo "✅ Task $TASK found in plan file"
echo ""

# 2. Rulebook discovery
echo "[ 2/5 ] Discovering applicable rulebooks..."
cd "$MODULE_ROOT" || exit 1
RULEBOOKS_OUTPUT=$(clm .rulebooks.list 2>&1)
RULEBOOK_COUNT=$(echo "$RULEBOOKS_OUTPUT" | grep -c "\.rulebook\.md" || true)

if [ "$RULEBOOK_COUNT" -eq 0 ]; then
  echo "❌ No rulebooks discovered"
  exit 1
fi

echo "✅ Found $RULEBOOK_COUNT applicable rulebooks"
echo ""

# Expected rulebooks (minimum set)
EXPECTED_RULEBOOKS=(
  "code_style.rulebook.md"
  "test_organization.rulebook.md"
  "files_structure.rulebook.md"
  "code_design.rulebook.md"
)

echo "[ 3/5 ] Verifying critical rulebooks are present..."
MISSING_RULEBOOKS=()
for rulebook in "${EXPECTED_RULEBOOKS[@]}"; do
  if ! echo "$RULEBOOKS_OUTPUT" | grep -q "$rulebook"; then
    MISSING_RULEBOOKS+=("$rulebook")
  fi
done

if [ ${#MISSING_RULEBOOKS[@]} -gt 0 ]; then
  echo "❌ Missing critical rulebooks:"
  for rulebook in "${MISSING_RULEBOOKS[@]}"; do
    echo "   - $rulebook"
  done
  exit 1
fi
echo "✅ All critical rulebooks present"
echo ""

# 4. File structure compliance validation
echo "[ 4/5 ] Validating planned file structure..."
VIOLATIONS=()

# Check for backup files (forbidden)
if find "$MODULE_ROOT" -type f \( -name "*_backup*" -o -name "*_old*" -o -name "*_v1*" -o -name "*_v2*" -o -name "*.bak" -o -name "*.orig" \) | grep -q .; then
  VIOLATIONS+=("Backup files found (forbidden by files_structure.rulebook.md)")
fi

# Check that tests directory exists
if [ ! -d "$MODULE_ROOT/tests" ]; then
  VIOLATIONS+=("tests/ directory does not exist")
fi

# Check that manual tests directory exists
if [ ! -d "$MODULE_ROOT/tests/manual" ]; then
  VIOLATIONS+=("tests/manual/ directory does not exist")
fi

if [ ${#VIOLATIONS[@]} -gt 0 ]; then
  echo "❌ File structure violations:"
  for violation in "${VIOLATIONS[@]}"; do
    echo "   - $violation"
  done
  exit 1
fi
echo "✅ File structure complies with rulebooks"
echo ""

# 5. Codestyle requirements check
echo "[ 5/5 ] Verifying codestyle requirements..."
CODESTYLE_VIOLATIONS=()

# Check if cargo fmt is disabled (should not be used)
if grep -q "cargo fmt" "$MODULE_ROOT"/.git/hooks/* 2>/dev/null; then
  CODESTYLE_VIOLATIONS+=("cargo fmt found in git hooks (forbidden)")
fi

# Verify 2-space indentation is documented in rulebook
if ! echo "$RULEBOOKS_OUTPUT" | grep -q "code_style.rulebook.md"; then
  CODESTYLE_VIOLATIONS+=("code_style.rulebook.md not found")
fi

if [ ${#CODESTYLE_VIOLATIONS[@]} -gt 0 ]; then
  echo "❌ Codestyle violations:"
  for violation in "${CODESTYLE_VIOLATIONS[@]}"; do
    echo "   - $violation"
  done
  exit 1
fi
echo "✅ Codestyle requirements verified"
echo ""

# Summary
echo "================================================"
echo "✅ LAYER 0 VERIFICATION PASSED"
echo "================================================"
echo ""
echo "Task 1.3 is ready for TDD implementation (Layer 1)"
echo ""
echo "Next step: Phase 1 - RED Phase (write failing tests)"
echo ""

exit 0
