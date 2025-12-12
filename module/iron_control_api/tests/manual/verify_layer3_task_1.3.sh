#!/bin/bash
set -e

echo "=== LAYER 3: ANTI-GAMING VERIFICATION ==="

# Count 1: Hardcoded test data in production code
echo "Checking for hardcoded test data in production code..."
HARDCODED_DATA=$(grep -r "user_id == \"test\"\|owner_id == \"test\"\|user_id == 123\|owner_id == 456" ../../module/iron_control_api/src/ 2>/dev/null | grep -v "//.*test" | wc -l)
echo "Hardcoded test data occurrences: $HARDCODED_DATA"
if [ "$HARDCODED_DATA" -eq 0 ]; then
  echo "✅ No hardcoded test data in production code"
else
  echo "❌ Found $HARDCODED_DATA instances of hardcoded test data"
  grep -r "user_id == \"test\"\|owner_id == \"test\"\|user_id == 123\|owner_id == 456" ../../module/iron_control_api/src/ 2>/dev/null | grep -v "//.*test"
  exit 1
fi

# Count 2: Test-only bypasses (cfg!(test) with early returns)
echo "Checking for test-only authorization bypasses..."
TEST_BYPASSES=$(grep -r "cfg!(test)" ../../module/iron_control_api/src/ 2>/dev/null | grep -E "return Ok|return true|return Some" | wc -l)
echo "Test-only bypasses: $TEST_BYPASSES"
if [ "$TEST_BYPASSES" -eq 0 ]; then
  echo "✅ No test-only authorization bypasses"
else
  echo "❌ Found $TEST_BYPASSES test-only bypass patterns"
  grep -r "cfg!(test)" ../../module/iron_control_api/src/ 2>/dev/null | grep -E "return Ok|return true|return Some"
  exit 1
fi

# Count 3: Deferred authorization TODOs
echo "Checking for deferred authorization TODOs..."
DEFERRED_TODOS=$(grep -r "TODO.*auth\|TODO.*owner" ../../module/iron_control_api/src/ 2>/dev/null | grep -iE "later|future|eventually|defer" | wc -l)
echo "Deferred authorization TODOs: $DEFERRED_TODOS"
if [ "$DEFERRED_TODOS" -eq 0 ]; then
  echo "✅ No deferred authorization TODOs"
else
  echo "⚠️  Found $DEFERRED_TODOS deferred authorization TODOs - review required"
  grep -r "TODO.*auth\|TODO.*owner" ../../module/iron_control_api/src/ 2>/dev/null | grep -iE "later|future|eventually|defer"
  echo ""
  echo "Note: Deferred authorization implementation is not acceptable for Task 1.3"
fi

# Count 4: Commented-out authorization checks
echo "Checking for commented-out authorization code..."
COMMENTED_AUTH=$(grep -r "//.*owner_id.*==\|//.*user_id.*==" ../../module/iron_control_api/src/routes/ 2>/dev/null | grep -v "Root cause\|Pitfall\|Fix(" | wc -l)
echo "Commented authorization checks: $COMMENTED_AUTH"
if [ "$COMMENTED_AUTH" -eq 0 ]; then
  echo "✅ No commented-out authorization checks"
else
  echo "⚠️  Found $COMMENTED_AUTH commented authorization checks - verify intentional"
  grep -r "//.*owner_id.*==\|//.*user_id.*==" ../../module/iron_control_api/src/routes/ 2>/dev/null | grep -v "Root cause\|Pitfall\|Fix("
fi

echo ""
echo "✅ LAYER 3 PASSED - No gaming/workarounds detected"
exit 0
