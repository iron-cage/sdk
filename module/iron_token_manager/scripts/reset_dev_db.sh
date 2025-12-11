#!/usr/bin/env bash
# Reset development database to clean state
#
# DESTRUCTIVE: Deletes existing database and creates fresh schema.
# Creates timestamped backup before deletion for safety.
#
# Usage:
#   ./scripts/reset_dev_db.sh [database_path]
#
# Default database: ./iron.db (canonical path)

set -euo pipefail

# Configuration
DB_PATH="${1:-./iron.db}"
BACKUP_DIR="./backups"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
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

log_error() {
  echo -e "${RED}[ERROR]${NC} $1"
}

# Check if this is production database (safety check)
if [[ "$DB_PATH" == *"production"* ]] || [[ "$DB_PATH" == *"prod"* ]]; then
  log_error "Refusing to reset production database: $DB_PATH"
  log_error "This script is for DEVELOPMENT ONLY"
  exit 1
fi

# Create backup directory if it doesnt exist
mkdir -p "$BACKUP_DIR"

# Backup existing database
if [ -f "$DB_PATH" ]; then
  TIMESTAMP=$(date +%Y%m%d_%H%M%S)
  BACKUP_PATH="$BACKUP_DIR/iron_backup_$TIMESTAMP.db"

  log_info "Creating backup: $BACKUP_PATH"
  cp "$DB_PATH" "$BACKUP_PATH"
  log_success "Backup created"

  # Delete old database
  log_warning "Deleting database: $DB_PATH"
  rm -f "$DB_PATH"
else
  log_info "No existing database found at $DB_PATH"
fi

# Create new empty database
log_info "Creating new database: $DB_PATH"
touch "$DB_PATH"

# Apply migrations using SQL files
log_info "Applying migrations..."

# Enable foreign keys first
sqlite3 "$DB_PATH" "PRAGMA foreign_keys = ON;"

# Apply migrations in order
MIGRATIONS_DIR="$PROJECT_ROOT/migrations"
for migration_num in 001 002 003 004 005 006 008 009 010; do
  migration_file="$MIGRATIONS_DIR/${migration_num}_*.sql"
  if ls $migration_file 1> /dev/null 2>&1; then
    for file in $migration_file; do
      log_info "Applying $(basename "$file")..."
      sqlite3 "$DB_PATH" < "$file"
    done
  else
    log_warning "Migration file not found: ${migration_num}_*.sql"
  fi
done

# Note: Migration 007 is intentionally skipped (reserved)

# Verify database structure
TABLE_COUNT=$(sqlite3 "$DB_PATH" "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%';")

log_success "Database reset complete"
echo ""
log_info "Summary:"
echo "  Database: $DB_PATH"
echo "  Tables created: $TABLE_COUNT"
echo "  Backup: ${BACKUP_PATH:-none}"
echo ""
log_info "Next steps:"
echo "  1. Run ./scripts/seed_dev_data.sh to populate with test data"
echo "  2. Or use ./scripts/reset_and_seed.sh for one-command reset+seed"
