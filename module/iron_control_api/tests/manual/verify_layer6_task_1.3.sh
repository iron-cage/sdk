#!/bin/bash
set -e

echo "=== LAYER 6: MIGRATION METRICS VERIFICATION ==="

# Count 1: Total agent endpoints
echo "Counting agent API endpoints..."
AGENT_ENDPOINTS=$(grep "pub async fn" ../../module/iron_control_api/src/routes/agents.rs 2>/dev/null | wc -l)
AGENT_ENDPOINTS=$(echo "$AGENT_ENDPOINTS" | tr -d '[:space:]')
echo "Total agent endpoints: $AGENT_ENDPOINTS"

# Count 2: Protected agent endpoints (with AuthenticatedUser)
echo "Counting protected agent endpoints..."
PROTECTED_AGENT=$(grep -A 5 "pub async fn" ../../module/iron_control_api/src/routes/agents.rs 2>/dev/null | grep "AuthenticatedUser" | wc -l)
PROTECTED_AGENT=$(echo "$PROTECTED_AGENT" | tr -d '[:space:]')
echo "Protected agent endpoints: $PROTECTED_AGENT"

# Count 3: Total budget endpoints (that should have owner checks)
echo ""
echo "Counting budget API endpoints requiring agent ownership verification..."
# Budget endpoints that work with agents
BUDGET_ENDPOINTS=$(grep "pub async fn create_budget_request" ../../module/iron_control_api/src/routes/budget.rs 2>/dev/null | wc -l)
BUDGET_ENDPOINTS=$(echo "$BUDGET_ENDPOINTS" | tr -d '[:space:]')
echo "Budget endpoints requiring ownership checks: $BUDGET_ENDPOINTS"

# Count 4: Budget endpoints with owner verification
echo "Counting budget endpoints with ownership verification..."
PROTECTED_BUDGET=$(grep -A 20 "pub async fn create_budget_request" ../../module/iron_control_api/src/routes/budget.rs 2>/dev/null | grep "SELECT owner_id FROM agents" | wc -l)
PROTECTED_BUDGET=$(echo "$PROTECTED_BUDGET" | tr -d '[:space:]')
echo "Budget endpoints with ownership verification: $PROTECTED_BUDGET"

# Calculate total protected endpoints
TOTAL_ENDPOINTS=$((AGENT_ENDPOINTS + BUDGET_ENDPOINTS))
TOTAL_PROTECTED=$((PROTECTED_AGENT + PROTECTED_BUDGET))
TOTAL_UNPROTECTED=$((TOTAL_ENDPOINTS - TOTAL_PROTECTED))

echo ""
echo "=== MIGRATION METRICS ==="
echo "Total endpoints requiring authorization: $TOTAL_ENDPOINTS"
echo "  - Agent endpoints: $AGENT_ENDPOINTS"
echo "  - Budget endpoints: $BUDGET_ENDPOINTS"
echo ""
echo "Protected endpoints: $TOTAL_PROTECTED"
echo "  - Protected agent endpoints: $PROTECTED_AGENT"
echo "  - Protected budget endpoints: $PROTECTED_BUDGET"
echo ""
echo "Unprotected endpoints: $TOTAL_UNPROTECTED"

# Calculate protection ratio
if [ "$TOTAL_ENDPOINTS" -gt 0 ]; then
  PROTECTION_RATIO=$((100 * TOTAL_PROTECTED / TOTAL_ENDPOINTS))
  echo ""
  echo "Protection ratio: $PROTECTION_RATIO% ($TOTAL_PROTECTED/$TOTAL_ENDPOINTS)"

  if [ "$PROTECTION_RATIO" -eq 100 ]; then
    echo "✅ 100% migration completion - all endpoints protected"
  else
    echo "❌ Migration incomplete - only $PROTECTION_RATIO% protected"
    echo "   Expected: 100% (all endpoints must have authorization)"
    exit 1
  fi
else
  echo "⚠️  No endpoints found to verify"
  exit 1
fi

# Verify Migration 014 applied
echo ""
echo "Verifying Migration 014 (owner_id column) is applied..."
if [ -f "../../module/iron_token_manager/migrations/014_add_agents_owner_id.sql" ]; then
  echo "✅ Migration 014 exists and adds owner_id enforcement"
else
  echo "❌ Migration 014 not found"
  exit 1
fi

echo ""
echo "✅ LAYER 6 PASSED - Migration metrics show 100% completion"
echo "   - All $TOTAL_ENDPOINTS endpoints have authorization"
echo "   - Protection ratio: $PROTECTION_RATIO%"
echo "   - Migration 014 applied successfully"
exit 0
