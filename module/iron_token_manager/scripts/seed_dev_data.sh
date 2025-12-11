#!/usr/bin/env bash
# Seed development database with test data
#
# Creates test users, tokens, projects, and usage records for manual testing.
# Safe to run multiple times (uses INSERT OR IGNORE).
#
# Usage:
#   ./scripts/seed_dev_data.sh [database_path]
#
# Default database: ./iron.db (canonical path)

set -euo pipefail

# Configuration
DB_PATH="${1:-./iron.db}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Helper functions
log_info() {
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
  echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
  echo -e "${YELLOW}[WARNING]${NC} $1"
}

# Compute SHA-256 hash for token
hash_token() {
  echo -n "$1" | sha256sum | awk '{print $1}'
}

# Get current time in milliseconds
current_time_ms() {
  date +%s%3N
}

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
  log_warning "Database not found at $DB_PATH"
  log_info "Creating new database..."
  touch "$DB_PATH"
fi

log_info "Seeding database: $DB_PATH"

# Test token values (memorizable for manual testing)
TOKEN_ADMIN="iron_dev_admin_token_001"
TOKEN_PM="iron_dev_pm_token_002"
TOKEN_VIEWER="iron_dev_viewer_token_003"

# Compute hashes
HASH_ADMIN=$(hash_token "$TOKEN_ADMIN")
HASH_PM=$(hash_token "$TOKEN_PM")
HASH_VIEWER=$(hash_token "$TOKEN_VIEWER")

# Current timestamp
NOW_MS=$(current_time_ms)

# Execute SQL
sqlite3 "$DB_PATH" <<EOF
-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- USERS (from migrations 003 and 005)
-- ============================================================================

-- Insert test users (INSERT OR IGNORE for idempotency)
-- Note: id is INTEGER AUTOINCREMENT, so we dont specify it
INSERT OR IGNORE INTO users (id, username, email, role, password_hash, is_active, created_at)
VALUES
  (
    'admin',
    'admin@admin.com',
    'admin',
    'admin',
    '\$2b\$12\$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2',
    1,
    $NOW_MS
  ),
  (
    'developer',
    'developer',
    'developer@developer.com',
    'developer',
    '\$2b\$12\$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2',
    1,
    $NOW_MS
  ),
  (
    'viewer',
    'viewer',
    'viewer@viewer.com',
    'viewer',
    '\$2b\$12\$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2',
    0,
    $NOW_MS
  );

-- ============================================================================
-- API TOKENS
-- ============================================================================

-- Insert test tokens (INSERT OR IGNORE for idempotency)
-- Note: user_id is TEXT (not FK to users.id), project_id is TEXT
INSERT OR IGNORE INTO api_tokens (
  token_hash,
  user_id,
  project_id,
  name,
  is_active,
  created_at
)
VALUES
  (
    '$HASH_ADMIN',
    'admin',
    'project_alpha',
    'Admin Development Token',
    1,
    $NOW_MS
  ),
  (
    '$HASH_PM',
    'developer',
    'project_beta',
    'Developer Development Token',
    1,
    $NOW_MS
  ),
  (
    '$HASH_VIEWER',
    'viewer',
    'project_beta',
    'Viewer Development Token',
    1,
    $NOW_MS
  );

-- ============================================================================
-- USAGE LIMITS
-- ============================================================================

-- Set generous limits for development (INSERT OR IGNORE for idempotency)
-- Schema: max_tokens_per_day, max_requests_per_minute, max_cost_cents_per_month
INSERT OR IGNORE INTO usage_limits (
  user_id,
  project_id,
  max_tokens_per_day,
  max_requests_per_minute,
  max_cost_cents_per_month,
  created_at,
  updated_at
)
VALUES
  (
    'admin',
    NULL,
    1000000,
    1000,
    1000000,
    $NOW_MS,
    $NOW_MS
  ),
  (
    'developer',
    'project_beta',
    500000,
    500,
    500000,
    $NOW_MS,
    $NOW_MS
  ),
  (
    'viewer',
    'project_beta',
    100000,
    100,
    100000,
    $NOW_MS,
    $NOW_MS
  );

-- ============================================================================
-- TOKEN USAGE (sample data)
-- ============================================================================

-- Insert sample usage records for admin token over the last 7 days
-- Schema: token_id, provider, model, input_tokens, output_tokens, total_tokens, requests_count, cost_cents, recorded_at
INSERT OR IGNORE INTO token_usage (
  token_id,
  provider,
  model,
  input_tokens,
  output_tokens,
  total_tokens,
  requests_count,
  cost_cents,
  recorded_at
)
SELECT
  (SELECT id FROM api_tokens WHERE token_hash = '$HASH_ADMIN' LIMIT 1),
  'openai',
  'gpt-4',
  1500,
  500,
  2000,
  1,
  250,
  $NOW_MS - (86400000 * day_offset)
FROM (
  SELECT 0 AS day_offset UNION SELECT 1 UNION SELECT 2 UNION
  SELECT 3 UNION SELECT 4 UNION SELECT 5 UNION SELECT 6
) AS days
LIMIT 7;

-- ============================================================================
-- AUDIT LOG (sample entries)
-- ============================================================================

-- Schema: entity_type, entity_id, action, actor_user_id, changes, logged_at
INSERT OR IGNORE INTO audit_log (
  entity_type,
  entity_id,
  action,
  actor_user_id,
  changes,
  logged_at
)
SELECT
  'token',
  (SELECT id FROM api_tokens WHERE token_hash = '$HASH_ADMIN' LIMIT 1),
  'created',
  'admin',
  '{"method":"api","reason":"development"}',
  $NOW_MS - 86400000
UNION ALL
SELECT
  'token',
  (SELECT id FROM api_tokens WHERE token_hash = '$HASH_PM' LIMIT 1),
  'created',
  'developer',
  '{"method":"api","reason":"development"}',
  $NOW_MS - 86400000
UNION ALL
SELECT
  'token',
  (SELECT id FROM api_tokens WHERE token_hash = '$HASH_VIEWER' LIMIT 1),
  'created',
  'viewer',
  '{"method":"api","reason":"development"}',
  $NOW_MS - 86400000;

EOF

# Verify data was inserted
USER_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM users WHERE username IN ('admin', 'developer', 'viewer');")
TOKEN_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM api_tokens WHERE token_hash IN ('$HASH_ADMIN', '$HASH_PM', '$HASH_VIEWER');")
USAGE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM token_usage WHERE token_id IN (SELECT id FROM api_tokens WHERE token_hash IN ('$HASH_ADMIN', '$HASH_PM', '$HASH_VIEWER'));")

log_success "Database seeded successfully"
echo ""
log_info "Summary:"
echo "  Users created:   $USER_COUNT"
echo "  Tokens created:  $TOKEN_COUNT"
echo "  Usage records:   $USAGE_COUNT"
echo ""
log_info "Test tokens (save these for API testing):"
echo "  Admin:      $TOKEN_ADMIN"
echo "  Developer:  $TOKEN_PM"
echo "  Viewer:     $TOKEN_VIEWER"
echo ""
log_info "Test credentials:"
echo "  Admin:      username=admin, role=admin, is_active=1"
echo "  Developer:  username=developer, role=user, is_active=1"
echo "  Viewer:     username=viewer, role=user, is_active=0 (INACTIVE)"
echo ""
log_info "Projects:"
echo "  project_alpha (admin)"
echo "  project_beta (developer + viewer)"
