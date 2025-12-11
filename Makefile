# Iron Runtime Development Makefile
# Minimal commands for daily development workflow

.PHONY: help dev api dashboard test clean setup status ports validate build lint-docs lint-python
.PHONY: db-reset db-reset-seed db-seed db-inspect debug-setup
.PHONY: py-build py-dev py-test py-test-e2e py-test-manual py-sync py-clean
.DEFAULT_GOAL := help

# Configuration
DASHBOARD_DIR := module/iron_dashboard
RUNTIME_DIR := module/iron_runtime
CONFIG_DEV := config.dev.toml

#===============================================================================
# Help
#===============================================================================

help: ## Show this help
	@echo "Iron Runtime - Essential Commands"
	@echo "=================================="
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick Start:  make setup && make dev"

#===============================================================================
# Development (Daily Use)
#===============================================================================

dev: ## Run full stack (API:3000 + Dashboard:5173)
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "Installing dashboard dependencies..."; \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	@trap 'kill 0' EXIT; \
		RUST_LOG="trace" cargo run --release --bin iron_control_api_server & \
		sleep 2 && cd $(DASHBOARD_DIR) && npm run dev

api: ## Run API server only (port 3000)
	RUST_LOG="trace" cargo run --release --bin iron_control_api_server

dashboard: ## Run dashboard only (port 5173)
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	cd $(DASHBOARD_DIR) && npm run dev

#===============================================================================
# Testing
#===============================================================================

test: ## Run all tests (nextest + clippy + doc tests)
	w3 .test l::3

test-quick: ## Run tests fast (nextest only)
	cargo nextest run --all-features

#===============================================================================
# Build & Validation
#===============================================================================

build: ## Build API + Dashboard for production
	cargo build --release --bin iron_control_api_server
	cd $(DASHBOARD_DIR) && npm run build

validate: ## Full production validation
	@echo "=== Rust Tests ===" && w3 .test l::3
	@echo "=== Dashboard ===" && cd $(DASHBOARD_DIR) && npm run type-check && npm run build
	@echo "=== Build ===" && cargo build --release --bin iron_control_api_server
	@echo "✅ Validation complete"

lint-docs: ## Check documentation ID format compliance
	@scripts/lint_id_formats.sh

lint-python: ## Check Python tooling compliance
	@scripts/lint_python_tooling.sh

#===============================================================================
# Setup & Maintenance
#===============================================================================

setup: ## Initial setup (install dependencies)
	cd $(DASHBOARD_DIR) && npm install
	@echo "✅ Setup complete. Run: make dev"

clean: ## Clean all build artifacts
	cargo clean
	rm -rf $(DASHBOARD_DIR)/node_modules $(DASHBOARD_DIR)/dist

status: ## Show installation status
	@echo "=== Iron Runtime Status ==="
	@cargo --version
	@[ -d "$(DASHBOARD_DIR)/node_modules" ] && echo "Dashboard: ✅ installed" || echo "Dashboard: ❌ run make setup"
	@[ -f dev_tokens.db ] && echo "Database: ✅ exists (dev_tokens.db)" || echo "Database: ⚠️  run make db-reset-seed"

#===============================================================================
# Database Management
#===============================================================================
# All database targets follow dev_*.db naming convention
# See test_organization.rulebook.md for complete standards

db-reset-seed: ## Fresh database with seed data (recommended)
	@echo "Resetting databases and populating seed data..."
	@module/iron_token_manager/scripts/reset_and_seed.sh dev_tokens.db
	@echo "✅ Database reset and seeded: dev_tokens.db"

db-reset: ## Delete all development databases
	@rm -f dev_*.db
	@echo "✅ Development databases deleted (dev_*.db)"
	@echo "   Run 'make db-reset-seed' to recreate with seed data"
	@echo "   Or start runtime to create fresh databases"

db-seed: ## Populate seed data (assumes database exists)
	@echo "Populating seed data..."
	@module/iron_token_manager/scripts/seed_dev_data.sh dev_tokens.db
	@echo "✅ Seed data populated: dev_tokens.db"

db-seed-admin: ## Create admin user
	@sqlite3 dev_tokens.db "INSERT OR IGNORE INTO users (id, email, username, password_hash, role, is_active, created_at) VALUES ('admin', 'admin@admin.com', 'admin', '\$$2b\$$12\$$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2', 'admin', 1, strftime('%s', 'now'));"
	@echo "✅ Admin user created (admin/testpass)"

db-inspect: ## Open interactive SQLite shell (dev_tokens.db)
	@if [ ! -f dev_tokens.db ]; then \
		echo "❌ dev_tokens.db not found"; \
		echo "   Run 'make db-reset-seed' first"; \
		exit 1; \
	fi
	@echo "Opening dev_tokens.db (press Ctrl+D or .exit to quit)"
	@echo "Useful commands:"
	@echo "  .tables                    -- List all tables"
	@echo "  .schema users             -- Show table structure"
	@echo "  SELECT * FROM users;      -- View data"
	@sqlite3 dev_tokens.db

debug-setup: db-reset-seed ## Complete debug environment setup
	@echo "Building workspace..."
	@cargo build --workspace
	@echo "✅ Debug environment ready"
	@echo ""
	@echo "Next steps:"
	@echo "  1. Start API server: make api"
	@echo "  2. Inspect database: make db-inspect"
	@echo "  3. Check test tokens: See output from db-reset-seed above"

ports: ## Kill processes on ports 3000/5173
	@lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
	@lsof -ti:5173 | xargs -r kill -9 2>/dev/null || true
	@echo "Ports 3000 and 5173 cleared"

#===============================================================================
# Python Bindings (iron_runtime / LlmRouter)
#===============================================================================

py-build: ## Build iron_runtime Python wheel (release)
	cd $(RUNTIME_DIR) && uv run maturin build --release

py-dev: ## Build and install iron_runtime for development
	cd $(RUNTIME_DIR) && uv run maturin develop

py-test: ## Run iron_runtime Python tests (unit)
	cd $(RUNTIME_DIR) && uv run pytest python/tests/ -v --ignore=python/tests/test_llm_router_e2e.py

py-test-e2e: ## Run E2E tests (requires IC_TOKEN, IC_SERVER)
	@if [ -z "$$IC_TOKEN" ] || [ -z "$$IC_SERVER" ]; then \
		echo "ERROR: Set IC_TOKEN and IC_SERVER environment variables"; \
		echo "  export IC_TOKEN=iron_xxx"; \
		echo "  export IC_SERVER=http://localhost:3000"; \
		exit 1; \
	fi
	cd $(RUNTIME_DIR) && uv run pytest python/tests/test_llm_router_e2e.py -v

py-test-manual: ## Run manual LlmRouter test (requires IC_TOKEN, IC_SERVER)
	@if [ -z "$$IC_TOKEN" ] || [ -z "$$IC_SERVER" ]; then \
		echo "ERROR: Set IC_TOKEN and IC_SERVER environment variables"; \
		echo "  export IC_TOKEN=iron_xxx"; \
		echo "  export IC_SERVER=http://localhost:3000"; \
		exit 1; \
	fi
	cd $(RUNTIME_DIR) && uv run python python/examples/test_manual.py

py-sync: ## Sync Python dependencies for all modules
	@echo "Syncing Python dependencies..."
	@cd module/iron_runtime && uv sync
	@cd module/iron_sdk && uv sync
	@cd module/iron_cli_py && uv sync
	@echo "✅ Dependencies synced"

py-clean: ## Clean Python build artifacts
	cd $(RUNTIME_DIR) && rm -rf target/wheels dist *.egg-info
	find $(RUNTIME_DIR) -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find $(RUNTIME_DIR) -type f -name "*.so" -delete 2>/dev/null || true

# ============================================================================
# Docker Compose Targets
# ============================================================================

docker-build: ## Build Docker images for Control Panel
	@echo "Building Docker images..."
	docker compose build

docker-up: ## Start Control Panel services
	@echo "Starting Control Panel services..."
	docker compose up -d
	@echo "✅ Control Panel available at http://localhost:8080"

docker-down: ## Stop Control Panel services (keeps volumes)
	@echo "Stopping Control Panel services..."
	docker compose down

docker-down-volumes: ## Stop Control Panel and delete volumes (DESTRUCTIVE)
	@echo "WARNING: This will delete all database data!"
	@read -p "Are you sure? [y/N] " -n 1 -r; \
	echo; \
	if [[ $$REPLY =~ ^[Yy]$$ ]]; then \
		docker compose down -v; \
		echo "✅ Volumes deleted"; \
	else \
		echo "Cancelled"; \
	fi

docker-logs: ## View logs from all services
	docker compose logs -f

docker-logs-backend: ## View backend API logs only
	docker compose logs -f backend

docker-logs-frontend: ## View frontend nginx logs only
	docker compose logs -f frontend

docker-ps: ## Show status of Control Panel services
	docker compose ps
