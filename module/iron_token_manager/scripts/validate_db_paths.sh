#!/usr/bin/env bash
# Database Path Validator
#
# Validates that all database paths in the codebase follow the canonical standard.
#
# Rules:
# 1. Scripts must default to ./iron.db (not dev_tokens.db)
# 2. Config files must use sqlite:///./iron.db?mode=rwc
# 3. Documentation must reference iron.db (not dev_tokens.db)
# 4. Test code must use in-memory databases (sqlite::memory:)
#
# Usage:
#   ./scripts/validate_db_paths.sh
#
# Exit codes:
#   0 - All validations passed
#   1 - Validation failures found

set -euo pipefail

# Configuration
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
echo -e "${GREEN}Database Path Validator${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# ============================================================================
# Rule 1: Check for forbidden dev_tokens.db references
# ============================================================================

log_info "Rule 1: Checking for forbidden 'dev_tokens.db' references..."

FORBIDDEN_FILES=$(find . -type f \( -name "*.sh" -o -name "*.toml" -o -name "*.md" -o -name "*.rs" \) \
  -not -path "*/target/*" \
  -not -path "*/backups/*" \
  -not -path "*/.git/*" \
  -not -path "*/scripts/validate_db_paths.sh" \
  -not -path "*/docs/database_path_standards.md" \
  -exec grep -l "dev_tokens\.db" {} \; 2>/dev/null || true)

if [ -n "$FORBIDDEN_FILES" ]; then
  log_error "Found forbidden 'dev_tokens.db' references:"
  echo ""
  while IFS= read -r file; do
    echo -e "  ${RED}✗${NC} $file"
    grep -n "dev_tokens\.db" "$file" | while IFS= read -r line; do
      echo -e "      ${YELLOW}→${NC} $line"
    done
    echo ""
  done <<< "$FORBIDDEN_FILES"
  ((VIOLATIONS++))
else
  log_success "No forbidden 'dev_tokens.db' references found"
fi

# ============================================================================
# Rule 2: Validate script default paths
# ============================================================================

log_info "Rule 2: Validating script default paths (must use './iron.db')..."

SCRIPT_VIOLATIONS=0

# Check reset_dev_db.sh
if grep -q 'DB_PATH="\${1:-\.\/iron\.db}"' scripts/reset_dev_db.sh 2>/dev/null; then
  log_success "scripts/reset_dev_db.sh uses canonical path"
else
  log_error "scripts/reset_dev_db.sh does not use canonical default path"
  ((SCRIPT_VIOLATIONS++))
fi

# Check seed_dev_data.sh
if grep -q 'DB_PATH="\${1:-\.\/iron\.db}"' scripts/seed_dev_data.sh 2>/dev/null; then
  log_success "scripts/seed_dev_data.sh uses canonical path"
else
  log_error "scripts/seed_dev_data.sh does not use canonical default path"
  ((SCRIPT_VIOLATIONS++))
fi

# Check reset_and_seed.sh
if grep -q 'DB_PATH="\${1:-\.\/iron\.db}"' scripts/reset_and_seed.sh 2>/dev/null; then
  log_success "scripts/reset_and_seed.sh uses canonical path"
else
  log_error "scripts/reset_and_seed.sh does not use canonical default path"
  ((SCRIPT_VIOLATIONS++))
fi

# Check backup path naming convention
if grep -q 'BACKUP_PATH="\$BACKUP_DIR/iron_backup_\$TIMESTAMP.db"' scripts/reset_dev_db.sh 2>/dev/null; then
  log_success "scripts/reset_dev_db.sh uses canonical backup naming"
else
  log_error "scripts/reset_dev_db.sh does not use canonical backup naming (should be 'iron_backup_*.db')"
  ((SCRIPT_VIOLATIONS++))
fi

if [ $SCRIPT_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 3: Validate config files
# ============================================================================

log_info "Rule 3: Validating config files (must use 'sqlite:///./iron.db?mode=rwc')..."

CONFIG_VIOLATIONS=0

# Check config.dev.toml
if grep -q 'url = "sqlite:///./iron.db?mode=rwc"' config.dev.toml 2>/dev/null; then
  log_success "config.dev.toml uses canonical database URL"
else
  log_error "config.dev.toml does not use canonical database URL"
  ((CONFIG_VIOLATIONS++))
fi

if [ $CONFIG_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Rule 4: Validate Rust code defaults
# ============================================================================

log_info "Rule 4: Validating Rust code defaults (Config::default_dev())..."

CODE_VIOLATIONS=0

# Check src/config.rs default_dev() function
if grep -A 10 'pub fn default_dev()' src/config.rs 2>/dev/null | grep -q 'url: "sqlite:///./iron.db?mode=rwc"'; then
  log_success "src/config.rs Config::default_dev() uses canonical path"
else
  log_error "src/config.rs Config::default_dev() does not use canonical path"
  ((CODE_VIOLATIONS++))
fi

if [ $CODE_VIOLATIONS -gt 0 ]; then
  ((VIOLATIONS++))
fi

# ============================================================================
# Summary
# ============================================================================

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $VIOLATIONS -eq 0 ]; then
  log_success "All database path validations passed!"
  echo ""
  log_info "Canonical paths:"
  echo "  - Scripts default:     ./iron.db"
  echo "  - Config development:  sqlite:///./iron.db?mode=rwc"
  echo "  - Backup naming:       iron_backup_YYYYMMDD_HHMMSS.db"
  echo ""
  exit 0
else
  log_error "Found $VIOLATIONS validation failure(s)"
  echo ""
  log_info "Required fixes:"
  echo "  1. Replace all 'dev_tokens.db' → 'iron.db'"
  echo "  2. Update script defaults to './iron.db'"
  echo "  3. Update config files to 'sqlite:///./iron.db?mode=rwc'"
  echo "  4. Update backup naming to 'iron_backup_*.db'"
  echo ""
  exit 1
fi
