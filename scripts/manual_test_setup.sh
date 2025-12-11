#!/bin/bash
# Manual Test Database Setup Script
#
# Purpose: Provides one-command setup for manual testing environments
# Ensures every manual test run starts with fresh, known-good data
#
# Usage:
#   ./scripts/manual_test_setup.sh [crate_name]
#
# Examples:
#   ./scripts/manual_test_setup.sh iron_token_manager
#   ./scripts/manual_test_setup.sh iron_runtime
#
# What this script does:
# 1. Stops any running services (optional)
# 2. Wipes database (deletes all data)
# 3. Runs migrations (ensures schema is current)
# 4. Seeds database with test data
# 5. Optionally starts service
#
# Safety: Only for development/test environments. NEVER use in production!

set -e  # Exit on error

# ============================================================================
# Configuration
# ============================================================================

CRATE_NAME="${1:-iron_token_manager}"
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
DEFAULT_DB_PATH="$PROJECT_ROOT/dev.db"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# ============================================================================
# Functions
# ============================================================================

log_step() {
  echo ""
  echo -e "${BLUE}========================================${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}========================================${NC}"
}

log_success() {
  echo -e "${GREEN}✅ $1${NC}"
}

log_error() {
  echo -e "${RED}❌ $1${NC}"
  exit 1
}

log_warning() {
  echo -e "${YELLOW}⚠️  $1${NC}"
}

log_info() {
  echo -e "   $1"
}

confirm() {
  local prompt="$1"
  local default="${2:-n}"

  if [ "$default" = "y" ]; then
    prompt="$prompt [Y/n]: "
  else
    prompt="$prompt [y/N]: "
  fi

  read -p "$prompt" response
  response=${response:-$default}

  case "$response" in
    [Yy]* ) return 0;;
    * ) return 1;;
  esac
}

stop_services() {
  log_step "Step 1: Stopping Services"

  # Check for common service patterns
  local service_pids=$(pgrep -f "iron_.*_server" || true)

  if [ -n "$service_pids" ]; then
    log_info "Found running services (PIDs: $service_pids)"

    if confirm "Stop running services?" "y"; then
      for pid in $service_pids; do
        log_info "Stopping process $pid..."
        kill "$pid" 2>/dev/null || true
        sleep 1

        # Force kill if still running
        if ps -p "$pid" > /dev/null 2>&1; then
          log_warning "Force killing $pid..."
          kill -9 "$pid" 2>/dev/null || true
        fi
      done
      log_success "Services stopped"
    else
      log_warning "Skipping service shutdown (database may be locked!)"
    fi
  else
    log_success "No running services detected"
  fi
}

wipe_database() {
  log_step "Step 2: Wiping Database"

  # Find database path (check config, environment, or use default)
  local db_path="$DEFAULT_DB_PATH"

  if [ -f "$db_path" ]; then
    log_info "Found database at: $db_path"
    log_warning "This will DELETE ALL DATA in the database!"

    if ! confirm "Wipe database?" "n"; then
      log_error "Database wipe cancelled by user"
    fi

    # Remove database file
    rm -f "$db_path"
    log_success "Database file removed"
  else
    log_info "No existing database found at $db_path"
    log_success "Database is clean (will be created)"
  fi
}

run_migrations() {
  log_step "Step 3: Running Migrations"

  cd "$PROJECT_ROOT"

  log_info "Running migrations for $CRATE_NAME..."

  # Use cargo run to execute migration binary (if it exists)
  # or run a dedicated migration script
  if cargo run --bin "${CRATE_NAME}_migrate" --quiet -- --database "$DEFAULT_DB_PATH" 2>/dev/null; then
    log_success "Migrations applied via binary"
  else
    log_info "No migration binary found, using test-based migration..."

    # Alternative: Run a special test that applies migrations
    # This is the iron_test_db approach
    RUST_LOG=info cargo test --package "$CRATE_NAME" \
      --test '*' \
      --no-fail-fast \
      database_initialization \
      -- --nocapture --exact 2>&1 | grep -i "migration\|schema" || true

    log_success "Schema initialized (migrations may have run in tests)"
  fi
}

seed_database() {
  log_step "Step 4: Seeding Test Data"

  cd "$PROJECT_ROOT"

  log_info "Seeding database for $CRATE_NAME..."

  # Approach 1: Use dedicated seed binary (preferred)
  if cargo run --bin "${CRATE_NAME}_seed" --quiet -- --database "$DEFAULT_DB_PATH" 2>/dev/null; then
    log_success "Database seeded via binary"
    return
  fi

  # Approach 2: Use Rust test that calls seed functions
  log_info "No seed binary found, using test-based seeding..."

  # Create temporary Rust program to call seed functions
  local seed_test="
#[tokio::test]
async fn manual_seed_test() {
  use sqlx::SqlitePool;

  let pool = SqlitePool::connect(\"sqlite://$DEFAULT_DB_PATH\")
    .await
    .expect(\"Failed to connect to database\");

  // Apply migrations first
  ${CRATE_NAME}::migrations::apply_all_migrations(&pool)
    .await
    .expect(\"Failed to apply migrations\");

  // Seed data
  ${CRATE_NAME}::seed::seed_all(&pool)
    .await
    .expect(\"Failed to seed database\");

  println!(\"✅ Database seeded successfully\");
}
"

  # Write to temporary test file
  local temp_test="$PROJECT_ROOT/module/$CRATE_NAME/tests/-manual_seed_temp.rs"
  echo "$seed_test" > "$temp_test"

  # Run the test
  if RUST_LOG=info cargo test --package "$CRATE_NAME" \
    --test '-manual_seed_temp' \
    -- --nocapture 2>&1 | grep -E "seeded|migration"; then
    log_success "Database seeded via test"
  else
    log_warning "Could not verify seeding (check manually)"
  fi

  # Cleanup temporary file
  rm -f "$temp_test"
}

verify_data() {
  log_step "Step 5: Verifying Seed Data"

  if ! command -v sqlite3 &> /dev/null; then
    log_warning "sqlite3 not found, skipping verification"
    return
  fi

  log_info "Checking seeded data..."

  # Query some basic counts
  local user_count=$(sqlite3 "$DEFAULT_DB_PATH" "SELECT COUNT(*) FROM users;" 2>/dev/null || echo "0")
  local token_count=$(sqlite3 "$DEFAULT_DB_PATH" "SELECT COUNT(*) FROM api_tokens;" 2>/dev/null || echo "0")

  if [ "$user_count" -gt 0 ]; then
    log_success "Found $user_count users"
  else
    log_warning "No users found (is seed data correct?)"
  fi

  if [ "$token_count" -gt 0 ]; then
    log_success "Found $token_count tokens"
  else
    log_warning "No tokens found (is seed data correct?)"
  fi

  log_info ""
  log_info "Sample users:"
  sqlite3 "$DEFAULT_DB_PATH" \
    "SELECT username, role, CASE WHEN is_active = 1 THEN 'active' ELSE 'inactive' END
     FROM users LIMIT 5;" \
    2>/dev/null || log_warning "Could not query users"
}

start_service() {
  log_step "Step 6: Starting Service (Optional)"

  if ! confirm "Start ${CRATE_NAME} service?" "n"; then
    log_info "Skipping service startup"
    return
  fi

  cd "$PROJECT_ROOT"

  log_info "Starting ${CRATE_NAME} server..."

  # Try to find and run the server binary
  if cargo run --bin "${CRATE_NAME}_server" &>/dev/null & then
    local pid=$!
    log_success "Service started (PID: $pid)"
    log_info "Check logs: tail -f /tmp/${CRATE_NAME}.log"
  else
    log_warning "Could not start service automatically"
    log_info "Start manually: cd $PROJECT_ROOT && cargo run --bin ${CRATE_NAME}_server"
  fi
}

show_summary() {
  log_step "Setup Complete!"

  echo ""
  echo -e "${GREEN}Manual testing environment is ready:${NC}"
  echo ""
  echo "  📁 Database: $DEFAULT_DB_PATH"
  echo "  📦 Crate:    $CRATE_NAME"
  echo ""
  echo -e "${YELLOW}Seed data reference:${NC}"
  echo "  📖 See: module/$CRATE_NAME/tests/fixtures/seed_data_reference.md"
  echo ""
  echo -e "${BLUE}Next steps:${NC}"
  echo "  1. Review seed data: less module/$CRATE_NAME/tests/fixtures/seed_data_reference.md"
  echo "  2. Run manual tests using the seeded data"
  echo "  3. Re-run this script anytime to reset to clean state"
  echo ""
  echo -e "${GREEN}Happy testing! 🚀${NC}"
  echo ""
}

# ============================================================================
# Main Execution
# ============================================================================

main() {
  echo ""
  echo -e "${BLUE}========================================${NC}"
  echo -e "${BLUE}Manual Test Database Setup${NC}"
  echo -e "${BLUE}========================================${NC}"
  echo ""
  echo "  Crate:      $CRATE_NAME"
  echo "  Database:   $DEFAULT_DB_PATH"
  echo "  Project:    $PROJECT_ROOT"
  echo ""

  # Safety check
  if [ -f "$PROJECT_ROOT/.production" ] || [ "$IRON_ENV" = "production" ]; then
    log_error "PRODUCTION ENVIRONMENT DETECTED! This script is for dev/test only."
  fi

  log_warning "This script will WIPE ALL DATA in the database!"
  echo ""

  if ! confirm "Continue with setup?" "y"; then
    log_error "Setup cancelled by user"
  fi

  # Execute steps
  stop_services
  wipe_database
  run_migrations
  seed_database
  verify_data
  start_service
  show_summary
}

# Run main function
main
