#!/bin/bash
# Layer 1 Verification: TDD Workflow (RED Phase Complete)
# Task 1.3: Add Authorization Checks
#
# This script verifies that RED phase tests have been created and pass.

set -e

TASK="1.3"
MODULE_ROOT="/home/user1/pro/lib/wip_iron/iron_runtime/dev/module/iron_control_api"

echo "=== LAYER 1: TDD WORKFLOW (RED PHASE) ==="
echo ""

# 1. Test file exists
echo "[ 1/4 ] Checking authorization test file exists..."
TEST_FILE="$MODULE_ROOT/tests/authorization_checks.rs"
if [ ! -f "$TEST_FILE" ]; then
  echo "❌ Test file not found: $TEST_FILE"
  exit 1
fi
echo "✅ Authorization test file exists"
echo ""

# 2. Test file contains required documentation
echo "[ 2/4 ] Verifying test documentation..."
REQUIRED_SECTIONS=(
  "Root Cause"
  "Why Not Caught"
  "Fix Applied"
  "Prevention"
  "Pitfall"
)

MISSING_SECTIONS=()
for section in "${REQUIRED_SECTIONS[@]}"; do
  if ! grep -q "# $section" "$TEST_FILE"; then
    MISSING_SECTIONS+=("$section")
  fi
done

if [ ${#MISSING_SECTIONS[@]} -gt 0 ]; then
  echo "❌ Missing documentation sections:"
  for section in "${MISSING_SECTIONS[@]}"; do
    echo "   - $section"
  done
  exit 1
fi
echo "✅ Test documentation complete"
echo ""

# 3. Tests compile
echo "[ 3/4 ] Verifying tests compile..."
cd "$MODULE_ROOT" || exit 1
if ! cargo test --test authorization_checks --no-run &>/dev/null; then
  echo "❌ Tests failed to compile"
  exit 1
fi
echo "✅ Tests compile successfully"
echo ""

# 4. RED phase tests pass (verifying broken state)
echo "[ 4/4 ] Running RED phase tests..."
TEST_OUTPUT=$(cargo nextest run --test authorization_checks 2>&1)

# Extract test results
if echo "$TEST_OUTPUT" | grep -q "PASS.*test_user_cannot_access_other_users_agents"; then
  echo "✅ test_user_cannot_access_other_users_agents - PASS"
else
  echo "❌ test_user_cannot_access_other_users_agents - FAIL"
  exit 1
fi

if echo "$TEST_OUTPUT" | grep -q "PASS.*test_database_filters_agents_by_owner"; then
  echo "✅ test_database_filters_agents_by_owner - PASS"
else
  echo "❌ test_database_filters_agents_by_owner - FAIL"
  exit 1
fi

if echo "$TEST_OUTPUT" | grep -q "PASS.*test_handshake_rejects_unauthorized_agent_access"; then
  echo "✅ test_handshake_rejects_unauthorized_agent_access - PASS"
else
  echo "❌ test_handshake_rejects_unauthorized_agent_access - FAIL"
  exit 1
fi

if echo "$TEST_OUTPUT" | grep -q "PASS.*test_budget_request_rejects_unauthorized_agent"; then
  echo "✅ test_budget_request_rejects_unauthorized_agent - PASS"
else
  echo "❌ test_budget_request_rejects_unauthorized_agent - FAIL"
  exit 1
fi

if echo "$TEST_OUTPUT" | grep -q "PASS.*test_list_agents_filters_by_owner"; then
  echo "✅ test_list_agents_filters_by_owner - PASS"
else
  echo "❌ test_list_agents_filters_by_owner - FAIL"
  exit 1
fi

echo ""

# Summary
echo "================================================"
echo "✅ LAYER 1 (RED PHASE) VERIFICATION PASSED"
echo "================================================"
echo ""
echo "RED phase complete. All tests verify the broken state:"
echo "  - Agents table has NO owner_id column"
echo "  - Queries with owner_id fail as expected"
echo "  - Authorization checks dont exist yet"
echo ""
echo "Next step: Phase 2 - GREEN Phase (minimal implementation)"
echo ""

exit 0
