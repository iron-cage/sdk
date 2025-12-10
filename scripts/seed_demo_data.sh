#!/bin/bash
# Demo Data Seeding Script
# Seeds the database with demo data for conference presentation
#
# Prerequisites:
# - iron_control_api_server running at http://localhost:3000
# - Empty or fresh database

API_URL="http://localhost:3000"

echo "=========================================="
echo "Demo Data Seeding"
echo "=========================================="
echo ""

# Step 1: Login as demo user
echo "Step 1: Logging in as demo user..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{"username":"demo","password":"demo"}')

ACCESS_TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.access_token')

if [ -n "$ACCESS_TOKEN" ] && [ "$ACCESS_TOKEN" != "null" ]; then
  echo "  ✅ Login successful"
else
  echo "  ❌ Login failed"
  exit 1
fi
echo ""

# Step 2: Create demo tokens
echo "Step 2: Creating demo tokens..."

# Token 1: Production API token
echo "  Creating token: production-api-token..."
TOKEN1=$(curl -s -X POST "$API_URL/api/tokens" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "user_id": "demo",
    "project_id": "production-app",
    "description": "Production API Token (Main Application)"
  }')
TOKEN1_ID=$(echo "$TOKEN1" | jq -r '.id')
echo "    ✅ Created (ID: $TOKEN1_ID)"

# Token 2: Development API token
echo "  Creating token: development-api-token..."
TOKEN2=$(curl -s -X POST "$API_URL/api/tokens" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "user_id": "demo",
    "project_id": "dev-environment",
    "description": "Development API Token (Testing)"
  }')
TOKEN2_ID=$(echo "$TOKEN2" | jq -r '.id')
echo "    ✅ Created (ID: $TOKEN2_ID)"

# Token 3: Analytics service token
echo "  Creating token: analytics-service-token..."
TOKEN3=$(curl -s -X POST "$API_URL/api/tokens" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "user_id": "demo",
    "project_id": "analytics-service",
    "description": "Analytics Service Token (Data Pipeline)"
  }')
TOKEN3_ID=$(echo "$TOKEN3" | jq -r '.id')
echo "    ✅ Created (ID: $TOKEN3_ID)"

# Token 4: Mobile app token
echo "  Creating token: mobile-app-token..."
TOKEN4=$(curl -s -X POST "$API_URL/api/tokens" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $ACCESS_TOKEN" \
  -d '{
    "user_id": "demo",
    "project_id": "mobile-app",
    "description": "Mobile App Token (iOS/Android)"
  }')
TOKEN4_ID=$(echo "$TOKEN4" | jq -r '.id')
echo "    ✅ Created (ID: $TOKEN4_ID)"
echo ""

# Step 3: List all tokens to verify
echo "Step 3: Verifying demo tokens..."
TOKENS=$(curl -s "$API_URL/api/tokens" \
  -H "Authorization: Bearer $ACCESS_TOKEN")

TOKEN_COUNT=$(echo "$TOKENS" | jq 'length')
echo "  ✅ Total tokens created: $TOKEN_COUNT"
echo ""

# Summary
echo "=========================================="
echo "Demo Data Seeding Complete"
echo "=========================================="
echo ""
echo "Created tokens:"
echo "  1. production-api-token (ID: $TOKEN1_ID)"
echo "  2. development-api-token (ID: $TOKEN2_ID)"
echo "  3. analytics-service-token (ID: $TOKEN3_ID)"
echo "  4. mobile-app-token (ID: $TOKEN4_ID)"
echo ""
echo "Demo environment ready!"
echo "  Frontend: http://localhost:5173"
echo "  Login credentials: demo / demo"
echo ""
