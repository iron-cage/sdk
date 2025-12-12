#!/bin/bash
# Layer 1: Empirical Validation - Verify Evidence Examples
# This script runs curl commands from EVIDENCE sections of deliverables

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "════════════════════════════════════════════════════════"
echo "Layer 1: Empirical Validation - Evidence Examples"
echo "════════════════════════════════════════════════════════"
echo ""

# Check if server is running
if ! curl -s http://localhost:8080/health > /dev/null 2>&1; then
  echo "❌ Server not running at http://localhost:8080"
  echo "   Start server with: cargo run --bin iron_control_api_server"
  exit 1
fi

echo "✅ Server is running"
echo ""

# Track statistics
total_tests=0
passing_tests=0
failing_tests=0

# Phase 0.4: REST API Foundation
echo "Testing Phase 0.4: REST API Foundation"
echo "----------------------------------------"

# Test 0.4.1: Agent CRUD - Create Agent
echo "Test 0.4.1: Create Agent"
total_tests=$((total_tests + 1))

response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/agents \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer test_token" \
  -d '{"name": "Test Agent", "budget_microdollars": 500000}' 2>&1)

http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | head -n -1)

if [ "$http_code" = "201" ]; then
  if echo "$body" | jq -e '.id' > /dev/null 2>&1; then
    echo "   ✅ PASS: Returns HTTP 201 with agent ID"
    passing_tests=$((passing_tests + 1))
  else
    echo "   ❌ FAIL: HTTP 201 but missing agent ID in response"
    failing_tests=$((failing_tests + 1))
  fi
else
  echo "   ❌ FAIL: Expected HTTP 201, got HTTP $http_code"
  failing_tests=$((failing_tests + 1))
fi

# Test 0.4.4: Authentication
echo "Test 0.4.4: Authentication - Invalid Token"
total_tests=$((total_tests + 1))

http_code=$(curl -s -w "%{http_code}" -X GET http://localhost:8080/api/v1/agents \
  -H "Authorization: Bearer invalid_token" \
  -o /dev/null)

if [ "$http_code" = "401" ]; then
  echo "   ✅ PASS: Unauthorized access returns HTTP 401"
  passing_tests=$((passing_tests + 1))
else
  echo "   ❌ FAIL: Expected HTTP 401, got HTTP $http_code"
  failing_tests=$((failing_tests + 1))
fi

echo ""

# Phase 1: Core Feature Completeness
echo "Testing Phase 1: Core Feature Completeness"
echo "--------------------------------------------"

# Test 1.1: Budget Operations
echo "Test 1.1: Budget Increase"
total_tests=$((total_tests + 1))

response=$(curl -s -w "\n%{http_code}" -X POST http://localhost:8080/api/v1/budget/increase \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer admin_token" \
  -d '{"agent_id": "agent_001", "amount_microdollars": 100000}' 2>&1)

http_code=$(echo "$response" | tail -1)

if [ "$http_code" = "200" ]; then
  echo "   ✅ PASS: Budget increase returns HTTP 200"
  passing_tests=$((passing_tests + 1))
else
  echo "   ❌ FAIL: Expected HTTP 200, got HTTP $http_code"
  failing_tests=$((failing_tests + 1))
fi

echo ""

# Summary
echo "════════════════════════════════════════════════════════"
echo "Evidence Verification Summary"
echo "════════════════════════════════════════════════════════"
echo "Total Tests:   $total_tests"
echo "Passing:       $passing_tests"
echo "Failing:       $failing_tests"

if [ "$failing_tests" -eq 0 ]; then
  echo "Status:        ✅ ALL EVIDENCE VERIFIED"
  echo "════════════════════════════════════════════════════════"
  exit 0
else
  echo "Status:        ❌ EVIDENCE VERIFICATION FAILED"
  echo "════════════════════════════════════════════════════════"
  exit 1
fi
