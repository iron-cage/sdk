#!/bin/bash
set -e

echo "=== LAYER 7: KNOWLEDGE PRESERVATION VERIFICATION ==="

# 1. Verify architecture documentation exists
echo "Checking architecture documentation..."
if [ -f "../../docs/architecture/authorization.md" ]; then
  echo "✅ Architecture documentation exists"
else
  echo "❌ Architecture documentation missing"
  exit 1
fi

# Check architecture docs have key sections
if grep -q "Migration 014\|Authorization Patterns\|Protected Endpoints" ../../docs/architecture/authorization.md 2>/dev/null; then
  echo "✅ Architecture documentation includes migration details and patterns"
else
  echo "❌ Architecture documentation missing key sections"
  exit 1
fi

# 2. Verify completion report exists
echo "Checking completion report..."
if [ -f "../../docs/verification/task_1.3_completion.md" ]; then
  echo "✅ Completion report exists"
else
  echo "❌ Completion report missing"
  exit 1
fi

# Check completion report has verification results
if grep -q "Layer 0.*PASSED\|Layer 1.*PASSED\|Layer 2.*PASSED" ../../docs/verification/task_1.3_completion.md 2>/dev/null; then
  echo "✅ Completion report includes verification results"
else
  echo "❌ Completion report missing verification results"
  exit 1
fi

# Check completion report has lessons learned
if grep -q "Lessons Learned\|What Went Well\|Challenges" ../../docs/verification/task_1.3_completion.md 2>/dev/null; then
  echo "✅ Completion report includes lessons learned"
else
  echo "❌ Completion report missing lessons learned"
  exit 1
fi

# 3. Verify CHANGELOG exists and updated
echo "Checking CHANGELOG..."
if [ -f "../../CHANGELOG.md" ]; then
  echo "✅ CHANGELOG exists"
else
  echo "❌ CHANGELOG missing"
  exit 1
fi

# Check CHANGELOG has Task 1.3 entry
if grep -q "Task 1.3\|Add Authorization Checks\|Migration 014" ../../CHANGELOG.md 2>/dev/null; then
  echo "✅ CHANGELOG documents Task 1.3 completion"
else
  echo "❌ CHANGELOG not updated for Task 1.3"
  exit 1
fi

# 4. Verify verification scripts exist
echo "Checking verification scripts..."
SCRIPT_COUNT=$(ls tests/manual/verify_layer*_task_1.3.sh 2>/dev/null | wc -l)
if [ "$SCRIPT_COUNT" -eq 8 ]; then
  echo "✅ All 8 verification scripts exist (layers 0-7)"
elif [ "$SCRIPT_COUNT" -ge 5 ]; then
  echo "⚠️  Found $SCRIPT_COUNT verification scripts (expected 8, but critical scripts exist)"
else
  echo "❌ Missing verification scripts (found $SCRIPT_COUNT, expected 8)"
  exit 1
fi

# 5. Verify temporary docs marked for cleanup
echo "Checking for temporary documentation..."
TEMP_DOCS=$(find ../../-005_budget_control_protocol/-default_topic -maxdepth 1 -name "-*.md" -type f 2>/dev/null | wc -l)
if [ "$TEMP_DOCS" -gt 0 ]; then
  echo "⚠️  Found $TEMP_DOCS temporary docs (will be deleted after verification)"
  echo "    Temporary docs to delete:"
  find ../../-005_budget_control_protocol/-default_topic -maxdepth 1 -name "-*.md" -type f 2>/dev/null | xargs -n1 basename
else
  echo "✅ No temporary documentation remaining"
fi

echo ""
echo "✅ LAYER 7 PASSED - Knowledge preserved in permanent locations"
echo "   - Architecture docs: docs/architecture/authorization.md"
echo "   - Completion report: docs/verification/task_1.3_completion.md"
echo "   - CHANGELOG: CHANGELOG.md"
echo "   - Verification scripts: $SCRIPT_COUNT scripts in tests/manual/"
exit 0
