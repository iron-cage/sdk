#!/usr/bin/env bash
# Database Schema Validator
#
# Validates that the database schema matches expected structure.
#
# Rules:
# 1. Database must have exactly 17 tables (11 application + 6 migration guards)
# 2. Database must have exactly 32 indexes (idx_* pattern)
# 3. All migration guard tables must exist
# 4. Foreign keys must be enabled
# 5. Core application tables must exist
#
# Usage:
#   ./scripts/validate_db_schema.sh [database_path]
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
echo -e "${GREEN}Database Schema Validator${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if database exists
if [ ! -f "$DB_PATH" ]; then
  log_error "Database not found: $DB_PATH"
  log_info "Run './scripts/reset_and_seed.sh' to create database"
  exit 1
fi

log_info "Validating database: $DB_PATH"
echo ""

# ============================================================================
# Rule 1: Check table count
# ============================================================================

log_info "Rule 1: Checking table count (expect 17 total)..."

# Count application tables (excluding migration guards and sqlite_ tables)
APP_TABLE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' AND name NOT LIKE '_migration_%';")

# Count migration guard tables
GUARD_TABLE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name LIKE '_migration_%';")

TOTAL_TABLE_COUNT=$((APP_TABLE_COUNT + GUARD_TABLE_COUNT))

if [ "$TOTAL_TABLE_COUNT" -eq 17 ]; then
  log_success "Table count correct: $TOTAL_TABLE_COUNT (${APP_TABLE_COUNT} application + ${GUARD_TABLE_COUNT} guards)"
else
  log_error "Table count mismatch: expected 17, found $TOTAL_TABLE_COUNT (${APP_TABLE_COUNT} application + ${GUARD_TABLE_COUNT} guards)"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 2: Check index count
# ============================================================================

log_info "Rule 2: Checking index count (expect 32)..."

INDEX_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND name LIKE 'idx_%';")

if [ "$INDEX_COUNT" -eq 32 ]; then
  log_success "Index count correct: $INDEX_COUNT"
else
  log_error "Index count mismatch: expected 32, found $INDEX_COUNT"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 3: Check migration guard tables
# ============================================================================

log_info "Rule 3: Checking migration guard tables..."

GUARD_VIOLATIONS=0

# Expected migration guards (migrations 002-006, 008 - 007 is reserved/skipped)
EXPECTED_GUARDS=("_migration_002_completed" "_migration_003_completed" "_migration_004_completed" "_migration_005_completed" "_migration_006_completed" "_migration_008_completed")

for guard in "${EXPECTED_GUARDS[@]}"; do
  COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='$guard';")
  if [ "$COUNT" -eq 1 ]; then
    log_success "Guard table exists: $guard"
  else
    log_error "Guard table missing: $guard"
    ((GUARD_VIOLATIONS++))
  fi
done

if [ $GUARD_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 4: Check foreign keys can be enabled
# ============================================================================

log_info "Rule 4: Checking foreign keys can be enabled..."

# Note: PRAGMA foreign_keys is session-based in SQLite, so we enable it and verify
FK_ENABLED=$(sqlite3 "$DB_PATH" "PRAGMA foreign_keys = ON; PRAGMA foreign_keys;")

if [ "$FK_ENABLED" -eq 1 ]; then
  log_success "Foreign keys enabled"
else
  log_error "Foreign keys NOT enabled (PRAGMA foreign_keys = $FK_ENABLED)"
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 5: Check core application tables exist
# ============================================================================

log_info "Rule 5: Checking core application tables..."

CORE_TABLES=("agents" "ai_provider_keys" "api_call_traces" "api_tokens" "audit_log" "project_provider_key_assignments" "token_blacklist" "token_usage" "usage_limits" "user_audit_log" "users")

TABLE_VIOLATIONS=0

for table in "${CORE_TABLES[@]}"; do
  COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='$table';")
  if [ "$COUNT" -eq 1 ]; then
    log_success "Core table exists: $table"
  else
    log_error "Core table missing: $table"
    ((TABLE_VIOLATIONS++))
  fi
done

if [ $TABLE_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $VIOLATIONS -eq 0 ]; then
  log_success "All database schema validations passed!"
  echo ""
  log_info "Schema summary:"
  echo "  - Total tables:       $TOTAL_TABLE_COUNT"
  echo "  - Application tables: $APP_TABLE_COUNT"
  echo "  - Migration guards:   $GUARD_TABLE_COUNT"
  echo "  - Indexes:            $INDEX_COUNT"
  echo "  - Foreign keys:       enabled"
  echo ""
  exit 0
else
  log_error "Found $VIOLATIONS validation failure(s)"
  echo ""
  log_info "To fix schema issues:"
  echo "  1. Run: ./scripts/reset_dev_db.sh"
  echo "  2. Or: ./scripts/reset_and_seed.sh"
  echo "  3. Verify migrations applied correctly"
  echo ""
  exit 1
fi
