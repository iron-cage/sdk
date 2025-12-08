# Iron Runtime Development Makefile
# Minimal commands for daily development workflow

.PHONY: help dev api dashboard test clean setup status db-reset db-seed ports validate build
.DEFAULT_GOAL := help

# Configuration
DASHBOARD_DIR := module/iron_dashboard
DB_FILE := module/iron_api/iron_api.db

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
		cargo run --release --bin iron_api_server & \
		sleep 2 && cd $(DASHBOARD_DIR) && npm run dev

api: ## Run API server only (port 3000)
	cargo run --release --bin iron_api_server

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
	cargo build --release --bin iron_api_server
	cd $(DASHBOARD_DIR) && npm run build

validate: ## Full production validation
	@echo "=== Rust Tests ===" && w3 .test l::3
	@echo "=== Dashboard ===" && cd $(DASHBOARD_DIR) && npm run type-check && npm run build
	@echo "=== Build ===" && cargo build --release --bin iron_api_server
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
