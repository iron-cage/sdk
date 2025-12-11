#!/usr/bin/env bash
# Seed Data Completeness Validator
#
# Validates that the development database has been properly seeded with test data.
#
# Rules:
# 1. Database must have 3-5 test users (bash seed: 3, Rust seed: 5)
# 2. Database must have 3-8 test tokens (bash seed: 3, Rust seed: 8)
# 3. Database must have usage records (at least 7 total)
# 4. Database must have exactly 3 usage limits
# 5. Core test users must exist (admin, developer, viewer; optionally tester, guest)
#
# Usage:
#   ./scripts/validate_seed_data.sh [database_path]
#
# Default database: ./iron.db
#
# Exit codes:
#   0 - All validations passed
#   1 - Validation failures found

set -euo pipefail

# Configuration
DB_PATH="${1:-./iron.db}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Helper functions
log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[✓]${NC} $1"
}

log_error() {
  echo -e "${RED}[✗]${NC} $1"
}

log_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Validation state
VIOLATIONS=0

# Change to project root
cd "$PROJECT_ROOT"

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}Seed Data Completeness Validator${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
  log_error "Database not found: $DB_PATH"
  log_info "Run './scripts/reset_and_seed.sh' to create and seed database"
  exit 1
fi

log_info "Validating seed data in: $DB_PATH"
echo ""

# ============================================================================
# Rule 1: Check user count
# ============================================================================

log_info "Rule 1: Checking test user count (expect 3-5)..."

USER_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users;")

if [ "$USER_COUNT" -ge 3 ] && [ "$USER_COUNT" -le 5 ]; then
  log_success "User count correct: $USER_COUNT (bash seed: 3, Rust seed: 5)"
else
  log_error "User count mismatch: expected 3-5, found $USER_COUNT"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 2: Check specific test users exist
# ============================================================================

log_info "Rule 2: Checking specific test users..."

USER_VIOLATIONS=0

# Check for admin user
ADMIN_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username='admin' AND role='admin' AND is_active=1;")
if [ "$ADMIN_EXISTS" -eq 1 ]; then
  log_success "Test user exists: admin (role=admin, active=1)"
else
  log_error "Test user missing or incorrect: admin"
  ((USER_VIOLATIONS++))
fi

# Check for developer user
PM_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username='developer' AND role='user' AND is_active=1;")
if [ "$PM_EXISTS" -eq 1 ]; then
  log_success "Test user exists: developer (role=user, active=1)"
else
  log_error "Test user missing or incorrect: developer"
  ((USER_VIOLATIONS++))
fi

# Check for viewer user
VIEWER_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username='viewer' AND role='user' AND is_active=0;")
if [ "$VIEWER_EXISTS" -eq 1 ]; then
  log_success "Test user exists: viewer (role=user, active=0)"
else
  log_error "Test user missing or incorrect: viewer"
  ((USER_VIOLATIONS++))
fi

# Check for tester user (optional - only in Rust seed)
TESTER_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username='tester' AND role='user' AND is_active=1;")
if [ "$TESTER_EXISTS" -eq 1 ]; then
  log_success "Test user exists: tester (role=user, active=1) [Rust seed]"
fi

# Check for guest user (optional - only in Rust seed)
GUEST_EXISTS=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username='guest' AND role='user' AND is_active=1;")
if [ "$GUEST_EXISTS" -eq 1 ]; then
  log_success "Test user exists: guest (role=user, active=1) [Rust seed]"
fi

if [ $USER_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 3: Check token count
# ============================================================================

log_info "Rule 3: Checking test token count (expect 3-8)..."

TOKEN_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens;")

if [ "$TOKEN_COUNT" -ge 3 ] && [ "$TOKEN_COUNT" -le 8 ]; then
  log_success "Token count correct: $TOKEN_COUNT (bash seed: 3, Rust seed: 8)"
else
  log_error "Token count mismatch: expected 3-8, found $TOKEN_COUNT"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 4: Check token assignments
# ============================================================================

log_info "Rule 4: Checking token user assignments..."

TOKEN_VIOLATIONS=0

# Check admin token (at least 1 active)
ADMIN_TOKEN=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE user_id='admin' AND is_active=1;")
if [ "$ADMIN_TOKEN" -ge 1 ]; then
  log_success "Admin has at least 1 active token ($ADMIN_TOKEN)"
else
  log_error "Admin token missing or inactive"
  ((TOKEN_VIOLATIONS++))
fi

# Check developer tokens (at least 1 active)
PM_TOKEN=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE user_id='developer' AND is_active=1;")
if [ "$PM_TOKEN" -ge 1 ]; then
  log_success "Developer has at least 1 active token ($PM_TOKEN)"
else
  log_error "Developer token missing or inactive"
  ((TOKEN_VIOLATIONS++))
fi

# Check viewer tokens (at least 1, any state)
VIEWER_TOKEN=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE user_id='viewer';")
if [ "$VIEWER_TOKEN" -ge 1 ]; then
  log_success "Viewer has at least 1 token ($VIEWER_TOKEN)"
else
  log_error "Viewer token missing"
  ((TOKEN_VIOLATIONS++))
fi

# Check tester tokens (optional - only in Rust seed)
if [ "$TESTER_EXISTS" -eq 1 ]; then
  TESTER_TOKEN=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE user_id='tester' AND is_active=1;")
  if [ "$TESTER_TOKEN" -ge 1 ]; then
    log_success "Tester has at least 1 active token ($TESTER_TOKEN) [Rust seed]"
  else
    log_error "Tester exists but has no active tokens"
    ((TOKEN_VIOLATIONS++))
  fi
fi

# Check guest has NO tokens (optional - only in Rust seed)
if [ "$GUEST_EXISTS" -eq 1 ]; then
  GUEST_TOKEN=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE user_id='guest';")
  if [ "$GUEST_TOKEN" -eq 0 ]; then
    log_success "Guest has no tokens (edge case verified) [Rust seed]"
  else
    log_error "Guest should have 0 tokens, found $GUEST_TOKEN"
    ((TOKEN_VIOLATIONS++))
  fi
fi

if [ $TOKEN_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 5: Check usage records
# ============================================================================

log_info "Rule 5: Checking usage records (expect at least 7)..."

USAGE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM token_usage;")

if [ "$USAGE_COUNT" -ge 7 ]; then
  log_success "Usage records present: $USAGE_COUNT (bash seed: 7, Rust seed: 10+)"
else
  log_error "Usage records insufficient: found $USAGE_COUNT (expected ≥7)"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 6: Check usage limits
# ============================================================================

log_info "Rule 6: Checking usage limits (expect 3)..."

LIMIT_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM usage_limits;")

if [ "$LIMIT_COUNT" -eq 3 ]; then
  log_success "Usage limit count correct: $LIMIT_COUNT"
else
  log_error "Usage limit count mismatch: expected 3, found $LIMIT_COUNT"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 7: Check usage limits assignments
# ============================================================================

log_info "Rule 7: Checking usage limit assignments..."

LIMIT_VIOLATIONS=0

# Check admin limit
ADMIN_LIMIT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM usage_limits WHERE user_id='admin';")
if [ "$ADMIN_LIMIT" -eq 1 ]; then
  log_success "Admin usage limit exists"
else
  log_error "Admin usage limit missing"
  ((LIMIT_VIOLATIONS++))
fi

# Check developer limit
PM_LIMIT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM usage_limits WHERE user_id='developer';")
if [ "$PM_LIMIT" -eq 1 ]; then
  log_success "Developer usage limit exists"
else
  log_error "Developer usage limit missing"
  ((LIMIT_VIOLATIONS++))
fi

# Check viewer limit
VIEWER_LIMIT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM usage_limits WHERE user_id='viewer';")
if [ "$VIEWER_LIMIT" -eq 1 ]; then
  log_success "Viewer usage limit exists"
else
  log_error "Viewer usage limit missing"
  ((LIMIT_VIOLATIONS++))
fi

if [ $LIMIT_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $VIOLATIONS -eq 0 ]; then
  log_success "All seed data validations passed!"
  echo ""
  log_info "Seed data summary:"
  echo "  - Users:        $USER_COUNT (core: admin, developer, viewer)"
  echo "  - Tokens:       $TOKEN_COUNT"
  echo "  - Usage:        $USAGE_COUNT records"
  echo "  - Limits:       $LIMIT_COUNT (admin, developer, viewer)"
  echo ""
  log_info "Core test tokens for manual testing:"
  echo "  - Admin:      iron_dev_admin_token_001"
  echo "  - Developer:  iron_dev_pm_token_002"
  echo "  - Viewer:     iron_dev_viewer_token_003"
  if [ "$TOKEN_COUNT" -ge 4 ]; then
    echo ""
    log_info "Extended tokens (Rust seed):"
    echo "  - Expired:       iron_dev_expired_token_004"
    echo "  - Project:       iron_dev_project_token_005"
    echo "  - Expiring Soon: iron_dev_expiring_token_006"
    echo "  - Tester:        iron_dev_tester_token_007"
    echo "  - Tester 2:      iron_dev_tester_token_008"
  fi
  echo ""
  exit 0
else
  log_error "Found $VIOLATIONS validation failure(s)"
  echo ""
  log_info "To fix seed data issues:"
  echo "  1. Run: ./scripts/reset_and_seed.sh"
  echo "  2. Or: ./scripts/seed_dev_data.sh (if schema exists)"
  echo "  3. Verify seed script completed successfully"
  echo ""
  exit 1
fi
