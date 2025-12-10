# Iron Runtime Development Makefile
# Minimal commands for daily development workflow

.PHONY: help dev api dashboard test clean setup status db-reset db-seed ports validate build
.PHONY: py-build py-dev py-test py-test-e2e py-test-manual py-clean
.DEFAULT_GOAL := help

# Configuration
DASHBOARD_DIR := module/iron_dashboard
RUNTIME_DIR := module/iron_runtime
DB_FILE := module/iron_control_api/iron_control_api.db

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
		cargo run --release --bin iron_control_api_server & \
		sleep 2 && cd $(DASHBOARD_DIR) && npm run dev

api: ## Run API server only (port 3000)
	cargo run --release --bin iron_control_api_server

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
	@[ -f $(DB_FILE) ] && echo "Database: ✅ exists" || echo "Database: ⚠️  start API to create"

#===============================================================================
# Database & Ports
#===============================================================================

db-seed: ## Create demo user (demo/testpass)
	@sqlite3 iron.db "INSERT OR IGNORE INTO users (username, password_hash, role, is_active, created_at) VALUES ('demo', '\$$2b\$$12\$$zZOfQakwkynHa0mBVlSvQ.rmzFZxkkN6OelZE/bLDCY1whIW.IWf2', 'user', 1, strftime('%s', 'now'));"
	@echo "✅ Demo user created (demo/testpass)"

db-reset: ## Reset database (deletes all data)
	rm -f iron.db $(DB_FILE)
	@echo "Database reset. Restart API to recreate."

ports: ## Kill processes on ports 3000/5173
	@lsof -ti:3000 | xargs -r kill -9 2>/dev/null || true
	@lsof -ti:5173 | xargs -r kill -9 2>/dev/null || true
	@echo "Ports 3000 and 5173 cleared"

#===============================================================================
# Python Bindings (iron_runtime / LlmRouter)
#===============================================================================

py-build: ## Build iron_runtime Python wheel (release)
	cd $(RUNTIME_DIR) && maturin build --release

py-dev: ## Build and install iron_runtime for development
	cd $(RUNTIME_DIR) && maturin develop

py-test: ## Run iron_runtime Python tests (unit)
	cd $(RUNTIME_DIR) && python -m pytest python/tests/ -v --ignore=python/tests/test_llm_router_e2e.py

py-test-e2e: ## Run E2E tests (requires IC_TOKEN, IC_SERVER)
	@if [ -z "$$IC_TOKEN" ] || [ -z "$$IC_SERVER" ]; then \
		echo "ERROR: Set IC_TOKEN and IC_SERVER environment variables"; \
		echo "  export IC_TOKEN=iron_xxx"; \
		echo "  export IC_SERVER=http://localhost:3000"; \
		exit 1; \
	fi
	cd $(RUNTIME_DIR) && python -m pytest python/tests/test_llm_router_e2e.py -v

py-test-manual: ## Run manual LlmRouter test (requires IC_TOKEN, IC_SERVER)
	@if [ -z "$$IC_TOKEN" ] || [ -z "$$IC_SERVER" ]; then \
		echo "ERROR: Set IC_TOKEN and IC_SERVER environment variables"; \
		echo "  export IC_TOKEN=iron_xxx"; \
		echo "  export IC_SERVER=http://localhost:3000"; \
		exit 1; \
	fi
	cd $(RUNTIME_DIR) && python python/examples/test_manual.py

py-clean: ## Clean Python build artifacts
	cd $(RUNTIME_DIR) && rm -rf target/wheels dist *.egg-info
	find $(RUNTIME_DIR) -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find $(RUNTIME_DIR) -type f -name "*.so" -delete 2>/dev/null || true
