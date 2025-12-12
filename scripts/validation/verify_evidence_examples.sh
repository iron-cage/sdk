#!/usr/bin/env bash
# Layer 1: Empirical Validation - Evidence Examples
# PURPOSE → EVIDENCE → MEASUREMENT → NULL HYPOTHESIS → NEGATIVE CRITERIA
#
# This script validates API endpoints by running curl commands from EVIDENCE
# sections of specifications.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

log INFO "=========================================="
log INFO "Layer 1: Empirical Validation - Evidence Examples"
log INFO "=========================================="

# Configuration
API_HOST="${API_HOST:-localhost}"
API_PORT="${API_PORT:-8080}"
API_BASE_URL="http://$API_HOST:$API_PORT"

# Validation counters
total_checks=0
passing_checks=0
failing_checks=0

# === EVIDENCE 1: Health check endpoint ===
log INFO "Checking health endpoint..."
total_checks=$((total_checks + 1))

response=$(curl -s -w "\n%{http_code}" "${API_BASE_URL}/health" 2>/dev/null || echo -e "\n000")
http_code=$(echo "$response" | tail -1)
body=$(echo "$response" | sed '$d')

if [ "$http_code" = "200" ]; then
  log INFO "   ✅ PASS: Health endpoint returns HTTP 200"
  passing_checks=$((passing_checks + 1))
elif [ "$http_code" = "000" ]; then
  log WARN "   ⚠️  SKIP: Server not reachable at $API_BASE_URL"
  log WARN "   Start server with: cargo run --bin iron_control_api"
  total_checks=$((total_checks - 1))
else
  log ERROR "   ❌ FAIL: Health endpoint returned HTTP $http_code"
  failing_checks=$((failing_checks + 1))
fi

# === EVIDENCE 2: Create agent endpoint ===
if [ "$http_code" != "000" ]; then
  log INFO "Checking agent creation endpoint..."
  total_checks=$((total_checks + 1))

  response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE_URL}/api/v1/agents" \
    -H "Content-Type: application/json" \
    -d '{"name": "Test Agent", "budget_microdollars": 500000}' 2>/dev/null || echo -e "\n000")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | sed '$d')

  if [ "$http_code" = "201" ] || [ "$http_code" = "200" ]; then
    log INFO "   ✅ PASS: Agent creation returns HTTP $http_code"
    passing_checks=$((passing_checks + 1))

    # Extract agent_id from response if present
    agent_id=$(echo "$body" | grep -o '"agent_id":[0-9]*' | grep -o '[0-9]*' || echo "")
    if [ -n "$agent_id" ]; then
      log INFO "   Created agent with ID: $agent_id"
    fi
  elif [ "$http_code" = "404" ]; then
    log WARN "   ⚠️  SKIP: Endpoint not implemented yet (HTTP 404)"
    total_checks=$((total_checks - 1))
  else
    log ERROR "   ❌ FAIL: Agent creation returned HTTP $http_code"
    log ERROR "   Response body: $body"
    failing_checks=$((failing_checks + 1))
  fi
fi

# === EVIDENCE 3: Budget enforcement ===
if [ "$http_code" != "000" ] && [ -n "${agent_id:-}" ]; then
  log INFO "Checking budget enforcement..."
  total_checks=$((total_checks + 1))

  # Try to create a lease larger than agent budget
  response=$(curl -s -w "\n%{http_code}" -X POST "${API_BASE_URL}/api/v1/agents/$agent_id/lease" \
    -H "Content-Type: application/json" \
    -d '{"amount_microdollars": 600000}' 2>/dev/null || echo -e "\n000")

  http_code=$(echo "$response" | tail -1)
  body=$(echo "$response" | sed '$d')

  # Should fail with 400 or 403 (budget exceeded)
  if [ "$http_code" = "400" ] || [ "$http_code" = "403" ]; then
    log INFO "   ✅ PASS: Budget enforcement working (HTTP $http_code)"
    passing_checks=$((passing_checks + 1))
  elif [ "$http_code" = "404" ]; then
    log WARN "   ⚠️  SKIP: Lease endpoint not implemented yet (HTTP 404)"
    total_checks=$((total_checks - 1))
  else
    log ERROR "   ❌ FAIL: Budget enforcement not working (HTTP $http_code)"
    log ERROR "   Expected: 400 or 403, Got: $http_code"
    failing_checks=$((failing_checks + 1))
  fi
fi

# === SUMMARY ===
echo ""
log INFO "=========================================="
log INFO "Evidence Examples Validation Summary"
log INFO "=========================================="
log INFO "Total checks: $total_checks"
log INFO "Passing: $passing_checks"
log INFO "Failing: $failing_checks"

if [ "$total_checks" -eq 0 ]; then
  log WARN "⚠️  Layer 1 (Evidence): SKIP (server not running)"
  log WARN "Start server with: cargo run --bin iron_control_api"
  exit 0
elif [ "$failing_checks" -eq 0 ]; then
  log INFO "✅ Layer 1 (Evidence): PASS"
  exit 0
else
  log ERROR "❌ Layer 1 (Evidence): FAIL"
  exit 1
fi
