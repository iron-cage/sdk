#!/usr/bin/env bash
# Reset and seed development database in one command
#
# DESTRUCTIVE: Deletes existing database, creates fresh schema, and populates with test data.
# Creates timestamped backup before deletion for safety.
#
# Usage:
#   ./scripts/reset_and_seed.sh [database_path]
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

log_section() {
  echo ""
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo -e "${GREEN}$1${NC}"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
}

# Main execution
log_section "STEP 1: Reset Database"
"$SCRIPT_DIR/reset_dev_db.sh" "$DB_PATH"

log_section "STEP 2: Seed Test Data"
"$SCRIPT_DIR/seed_dev_data.sh" "$DB_PATH"

log_section "✓ COMPLETE"
log_success "Database reset and seeded successfully"
echo ""
log_info "Your development database is ready for manual testing!"
echo ""
log_info "Quick reference:"
echo "  Database: $DB_PATH"
echo "  Test tokens saved in output above"
echo "  Run 'sqlite3 $DB_PATH' to inspect database"
echo ""
