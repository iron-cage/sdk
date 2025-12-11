#!/usr/bin/env bash
#
# Development Environment Fresh Start
#
# Resets database, seeds test data, validates everything, and optionally runs tests.
# Use this when you want a clean, validated development environment.
#
# Usage:
#   ./scripts/dev_fresh.sh           # Reset + seed + validate
#   ./scripts/dev_fresh.sh --test    # Also run full test suite
#   ./scripts/dev_fresh.sh --quick   # Skip validation
#
# Or via Make:
#   make dev-fresh         # Full workflow
#   make dev-fresh-test    # With tests

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
RUN_TESTS=0
SKIP_VALIDATION=0

while [[ $# -gt 0 ]]; do
  case $1 in
    --test)
      RUN_TESTS=1
      shift
      ;;
    --quick)
      SKIP_VALIDATION=1
      shift
      ;;
    *)
      echo "Unknown option: $1"
      echo "Usage: $0 [--test] [--quick]"
      exit 1
      ;;
  esac
done

# ============================================================================
# Helper Functions
# ============================================================================

log_step()
{
  echo ""
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${CYAN}STEP: $1${NC}"
  echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo ""
}

log_success()
{
  echo -e "${GREEN}[✓]${NC} $1"
}

log_info()
{
  echo -e "${BLUE}[INFO]${NC} $1"
}

log_error()
{
  echo -e "${RED}[✗]${NC} $1"
}

# ============================================================================
# Main Workflow
# ============================================================================

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}Development Environment Fresh Start${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
log_info "This will reset your development database to a clean state"
log_info "All existing data will be backed up and then deleted"

# Step 1: Reset and seed database
log_step "Reset & Seed Database"

if ! ./scripts/reset_and_seed.sh; then
  log_error "Database reset failed"
  exit 1
fi

log_success "Database reset and seeded successfully"

# Step 2: Run validators (unless --quick)
if [ $SKIP_VALIDATION -eq 0 ]; then
  log_step "Validate Database"

  # Path validation
  log_info "Running path validator..."
  if ! ./scripts/validate_db_paths.sh; then
    log_error "Path validation failed"
    exit 1
  fi
  log_success "Path validation passed"

  # Schema validation
  log_info "Running schema validator..."
  if ! ./scripts/validate_db_schema.sh ./iron.db; then
    log_error "Schema validation failed"
    exit 1
  fi
  log_success "Schema validation passed"

  # Seed data validation
  log_info "Running seed data validator..."
  if ! ./scripts/validate_seed_data.sh ./iron.db; then
    log_error "Seed data validation failed"
    exit 1
  fi
  log_success "Seed data validation passed"
else
  log_info "Skipping validation (--quick mode)"
fi

# Step 3: Run tests (if --test)
if [ $RUN_TESTS -eq 1 ]; then
  log_step "Run Test Suite"

  log_info "Running nextest..."
  if ! RUSTFLAGS="-D warnings" cargo nextest run --all-features; then
    log_error "Tests failed"
    exit 1
  fi

  log_info "Running doctests..."
  if ! RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features; then
    log_error "Doctests failed"
    exit 1
  fi

  log_info "Running clippy..."
  if ! cargo clippy --all-targets --all-features -- -D warnings; then
    log_error "Clippy failed"
    exit 1
  fi

  log_success "All tests passed"
fi

# Final summary
echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✓ DEVELOPMENT ENVIRONMENT READY${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
log_success "Database: ./iron.db (reset and seeded)"
if [ $SKIP_VALIDATION -eq 0 ]; then
  log_success "Validation: All checks passed"
fi
if [ $RUN_TESTS -eq 1 ]; then
  log_success "Tests: All tests passed"
fi
echo ""
log_info "Your development environment is ready for manual testing!"
echo ""
log_info "Test tokens:"
echo "  Admin:      iron_dev_admin_token_001"
echo "  Developer:  iron_dev_pm_token_002"
echo "  Viewer:     iron_dev_viewer_token_003"
echo ""
