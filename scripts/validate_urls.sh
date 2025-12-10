#!/bin/bash
# URL Validation Script
# Ensures URL consistency across Iron Cage codebase
#
# Purpose: Validate that all URLs follow canonical standards defined in
#          dev/docs/standards/url_standards.md
#
# Usage: bash dev/scripts/validate_urls.sh
#
# Exit codes:
#   0 - All validations passed
#   1 - One or more validations failed

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Change to repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$REPO_ROOT"

echo "=============================================="
echo "   URL Validation for Iron Cage Platform"
echo "=============================================="
echo "Repository: $REPO_ROOT"
echo

# Track overall status
FAILED_TESTS=0
PASSED_TESTS=0

# Test 1: No legacy iron.dev references
echo -n "Test 1: Checking for iron.dev references... "
IRON_DEV_MATCHES=$(grep -r "iron\.dev" . \
  --include="*.md" \
  --include="*.rs" \
  --include="*.toml" \
  --include="*.yaml" \
  --include="*.json" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  --exclude="url_standards.md" \
  --exclude="vocabulary.md" \
  2>/dev/null | wc -l)

if [ "$IRON_DEV_MATCHES" -eq 0 ]; then
  echo -e "${GREEN}✅ PASS${NC}"
  PASSED_TESTS=$((PASSED_TESTS + 1))
else
  echo -e "${RED}❌ FAIL${NC}"
  echo "   Found $IRON_DEV_MATCHES iron.dev references:"
  grep -r "iron\.dev" . \
    --include="*.md" \
    --include="*.rs" \
    --include="*.toml" \
    --include="*.yaml" \
    --include="*.json" \
    --exclude-dir=".git" \
    --exclude="*ultrathink.md" \
    2>/dev/null | head -10
  FAILED_TESTS=$((FAILED_TESTS + 1))
fi
echo

# Test 2: No control-panel subdomain in documentation
echo -n "Test 2: Checking for control-panel subdomain... "
CONTROL_PANEL_MATCHES=$(grep -r "control-panel\.ironcage\.ai" . \
  --include="*.md" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  --exclude="url_standards.md" \
  2>/dev/null | wc -l)

if [ "$CONTROL_PANEL_MATCHES" -eq 0 ]; then
  echo -e "${GREEN}✅ PASS${NC}"
  PASSED_TESTS=$((PASSED_TESTS + 1))
else
  echo -e "${YELLOW}⚠️  WARNING${NC}"
  echo "   Found $CONTROL_PANEL_MATCHES control-panel references:"
  echo "   (Expected 0 - should use api.ironcage.ai instead)"
  grep -r "control-panel\.ironcage\.ai" . \
    --include="*.md" \
    --exclude-dir=".git" \
    --exclude="*ultrathink.md" \
    2>/dev/null | head -10
  # Not counted as failure - may be intentional in some contexts
fi
echo

# Test 3: Production URLs use HTTPS
echo -n "Test 3: Checking production URLs use HTTPS... "
HTTP_PROD_MATCHES=$(grep -rE "http://[^l].*ironcage\.ai" . \
  --include="*.md" \
  --include="*.rs" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  --exclude="url_standards.md" \
  2>/dev/null | grep -v "localhost" | wc -l)

if [ "$HTTP_PROD_MATCHES" -eq 0 ]; then
  echo -e "${GREEN}✅ PASS${NC}"
  PASSED_TESTS=$((PASSED_TESTS + 1))
else
  echo -e "${RED}❌ FAIL${NC}"
  echo "   Found $HTTP_PROD_MATCHES production URLs using HTTP:"
  grep -rE "http://[^l].*ironcage\.ai" . \
    --include="*.md" \
    --include="*.rs" \
    --exclude-dir=".git" \
    --exclude="*ultrathink.md" \
    2>/dev/null | grep -v "localhost" | head -10
  FAILED_TESTS=$((FAILED_TESTS + 1))
fi
echo

# Test 4: Code files use correct API URL
echo -n "Test 4: Checking code uses api.ironcage.ai... "
CODE_API_CHECK=$(grep -r "ironcage\.ai" . \
  --include="*.rs" \
  --exclude-dir=".git" \
  2>/dev/null | grep -v "api\.ironcage\.ai" | grep -v "//" | wc -l)

if [ "$CODE_API_CHECK" -eq 0 ]; then
  echo -e "${GREEN}✅ PASS${NC}"
  PASSED_TESTS=$((PASSED_TESTS + 1))
else
  echo -e "${YELLOW}⚠️  WARNING${NC}"
  echo "   Found $CODE_API_CHECK non-API ironcage.ai references in code:"
  grep -r "ironcage\.ai" . \
    --include="*.rs" \
    --exclude-dir=".git" \
    2>/dev/null | grep -v "api\.ironcage\.ai" | grep -v "//" | head -10
  # Not counted as failure - may be comments or valid references
fi
echo

# Test 5: Subdomain consistency
echo -n "Test 5: Checking subdomain usage patterns... "
API_SUBDOMAIN=$(grep -roh "https://api\.ironcage\.ai" . \
  --include="*.md" \
  --include="*.rs" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  2>/dev/null | wc -l)

DASHBOARD_SUBDOMAIN=$(grep -roh "https://dashboard\.ironcage\.ai" . \
  --include="*.md" \
  --include="*.rs" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  2>/dev/null | wc -l)

GATEWAY_SUBDOMAIN=$(grep -roh "https://gateway\.ironcage\.ai" . \
  --include="*.md" \
  --include="*.rs" \
  --exclude-dir=".git" \
  --exclude="*ultrathink.md" \
  2>/dev/null | wc -l)

echo -e "${GREEN}✅ INFO${NC}"
echo "   api.ironcage.ai:       $API_SUBDOMAIN occurrences"
echo "   dashboard.ironcage.ai: $DASHBOARD_SUBDOMAIN occurrences"
echo "   gateway.ironcage.ai:   $GATEWAY_SUBDOMAIN occurrences"
PASSED_TESTS=$((PASSED_TESTS + 1))
echo

# Summary
echo "=============================================="
echo "                SUMMARY"
echo "=============================================="
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
if [ "$FAILED_TESTS" -gt 0 ]; then
  echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
  echo
  echo -e "${RED}❌ Validation FAILED${NC}"
  echo
  echo "Fix the issues above and re-run validation."
  echo "See dev/docs/standards/url_standards.md for guidelines."
  exit 1
else
  echo -e "Failed: $FAILED_TESTS"
  echo
  echo -e "${GREEN}✅ All validations PASSED${NC}"
  echo
  echo "URL consistency verified successfully."
  exit 0
fi
