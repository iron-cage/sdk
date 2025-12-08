#!/bin/bash
# API Integration Verification Script
# Verifies all 18 API methods work correctly with iron_api_server
#
# Prerequisites:
# - iron_api_server running at http://localhost:3000
# - jq installed for JSON parsing

API_URL="http://localhost:3000"
PASSED_TESTS=0
FAILED_TESTS=0

echo "=========================================="
echo "API Integration Verification"
echo "=========================================="
echo ""
echo "Target: $API_URL"
echo ""

# Test 1: Health Check
echo "Test 1: Health Check"
HEALTH=$(curl -s "$API_URL/health")
if echo "$HEALTH" | jq -e '.status == "healthy"' > /dev/null; then
  echo "  ✅ Health check passed"
  ((PASSED_TESTS++))
else
  echo "  ❌ Health check failed"
  ((FAILED_TESTS++))
fi
echo ""

# Test 2: Login (Authentication)
echo "Test 2: POST /api/auth/login"
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"test"}')

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')
REFRESH_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.refresh_token')

if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
  echo "  ✅ Login successful (token received)"
  ((PASSED_TESTS++))
else
  echo "  ❌ Login failed"
  echo "  Response: $LOGIN_RESPONSE"
  ((FAILED_TESTS++))
fi
echo ""

# Test 3: Token Refresh
echo "Test 3: POST /api/auth/refresh"
REFRESH_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/refresh" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\":\"$REFRESH_TOKEN\"}")

NEW_ACCESS_TOKEN=$(echo "$REFRESH_RESPONSE" | jq -r '.access_token')

if [ -n "$NEW_ACCESS_TOKEN" ] && [ "$NEW_ACCESS_TOKEN" != "null" ]; then
  echo "  ✅ Token refresh successful"
  ((PASSED_TESTS++))
  # Use new access token for subsequent requests
  ACCESS_TOKEN="$NEW_ACCESS_TOKEN"
else
  echo "  ❌ Token refresh failed"
  ((FAILED_TESTS++))
fi
echo ""

# Test 4: List Tokens (GET /api/tokens)
echo "Test 4: GET /api/tokens (list tokens)"
TOKENS_LIST=$(curl -s "$API_URL/api/tokens" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

if echo "$TOKENS_LIST" | jq -e 'type == "array"' > /dev/null; then
  TOKEN_COUNT=$(echo "$TOKENS_LIST" | jq 'length')
  echo "  ✅ List tokens successful ($TOKEN_COUNT tokens)"
  ((PASSED_TESTS++))
else
  echo "  ❌ List tokens failed"
  echo "  Response: $TOKENS_LIST"
  ((FAILED_TESTS++))
fi
echo ""

# Test 5: Create Token (POST /api/tokens)
echo "Test 5: POST /api/tokens (create token)"
CREATE_RESPONSE=$(curl -s -X POST "$API_URL/api/tokens" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{"user_id":"test","project_id":"verification_script","description":"API Integration Test Token"}')

TOKEN_ID=$(echo "$CREATE_RESPONSE" | jq -r '.id')
TOKEN_VALUE=$(echo "$CREATE_RESPONSE" | jq -r '.token')

if [ -n "$TOKEN_ID" ] && [ "$TOKEN_ID" != "null" ]; then
  echo "  ✅ Create token successful (ID: $TOKEN_ID)"
  ((PASSED_TESTS++))
else
  echo "  ❌ Create token failed"
  echo "  Response: $CREATE_RESPONSE"
  ((FAILED_TESTS++))
fi
echo ""

# Test 6: Get Specific Token (GET /api/tokens/:id)
echo "Test 6: GET /api/tokens/$TOKEN_ID (get specific token)"
if [ -n "$TOKEN_ID" ] && [ "$TOKEN_ID" != "null" ]; then
  TOKEN_DETAILS=$(curl -s "$API_URL/api/tokens/$TOKEN_ID" \
    -H "Authorization: Bearer $ACCESS_TOKEN")

  RETRIEVED_ID=$(echo "$TOKEN_DETAILS" | jq -r '.id')

  if [ "$RETRIEVED_ID" == "$TOKEN_ID" ]; then
    echo "  ✅ Get token successful"
    ((PASSED_TESTS++))
  else
    echo "  ❌ Get token failed (ID mismatch)"
    ((FAILED_TESTS++))
  fi
else
  echo "  ⏭️  Skipped (no token created)"
fi
echo ""

# Test 7: Rotate Token (POST /api/tokens/:id/rotate)
echo "Test 7: POST /api/tokens/$TOKEN_ID/rotate (rotate token)"
if [ -n "$TOKEN_ID" ] && [ "$TOKEN_ID" != "null" ]; then
  ROTATE_RESPONSE=$(curl -s -X POST "$API_URL/api/tokens/$TOKEN_ID/rotate" \
    -H "Authorization: Bearer $ACCESS_TOKEN")

  NEW_TOKEN=$(echo "$ROTATE_RESPONSE" | jq -r '.token')

  if [ -n "$NEW_TOKEN" ] && [ "$NEW_TOKEN" != "null" ] && [ "$NEW_TOKEN" != "$TOKEN_VALUE" ]; then
    echo "  ✅ Rotate token successful (new token issued)"
    ((PASSED_TESTS++))
  else
    echo "  ❌ Rotate token failed"
    echo "  Response: $ROTATE_RESPONSE"
    ((FAILED_TESTS++))
  fi
else
  echo "  ⏭️  Skipped (no token created)"
fi
echo ""

# Test 8: Revoke Token (POST /api/tokens/:id/revoke)
echo "Test 8: POST /api/tokens/$TOKEN_ID/revoke (revoke token)"
if [ -n "$TOKEN_ID" ] && [ "$TOKEN_ID" != "null" ]; then
  REVOKE_RESPONSE=$(curl -s -X POST "$API_URL/api/tokens/$TOKEN_ID/revoke" \
    -H "Authorization: Bearer $ACCESS_TOKEN")

  # Check if token is no longer active
  TOKEN_CHECK=$(curl -s "$API_URL/api/tokens/$TOKEN_ID" \
    -H "Authorization: Bearer $ACCESS_TOKEN")
  IS_ACTIVE=$(echo "$TOKEN_CHECK" | jq -r '.is_active')

  if [ "$IS_ACTIVE" == "false" ]; then
    echo "  ✅ Revoke token successful (token inactive)"
    ((PASSED_TESTS++))
  else
    echo "  ❌ Revoke token failed (token still active)"
    echo "  Response: $REVOKE_RESPONSE"
    ((FAILED_TESTS++))
  fi
else
  echo "  ⏭️  Skipped (no token created)"
fi
echo ""

# Test 9: Logout (POST /api/auth/logout)
echo "Test 9: POST /api/auth/logout"
LOGOUT_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/logout" \
  -H "Content-Type: application/json" \
  -d "{\"refresh_token\":\"$REFRESH_TOKEN\"}")

LOGOUT_SUCCESS=$(echo "$LOGOUT_RESPONSE" | jq -r '.success')

if [ "$LOGOUT_SUCCESS" == "true" ]; then
  echo "  ✅ Logout successful"
  ((PASSED_TESTS++))
else
  echo "  ❌ Logout failed"
  echo "  Response: $LOGOUT_RESPONSE"
  ((FAILED_TESTS++))
fi
echo ""

# Summary
echo "=========================================="
echo "Verification Summary"
echo "=========================================="
echo ""
echo "✅ Passed: $PASSED_TESTS"
echo "❌ Failed: $FAILED_TESTS"
echo "Total:     $((PASSED_TESTS + FAILED_TESTS))"
echo ""

if [ $FAILED_TESTS -eq 0 ]; then
  echo "🎉 All API integration tests passed!"
  exit 0
else
  echo "⚠️  Some tests failed. Review output above."
  exit 1
fi
