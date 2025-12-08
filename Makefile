# Iron Runtime Development Makefile
# Provides convenient commands for iron_runtime repository
#
# NOTE: This repository is part of a two-repository architecture:
#   - iron_runtime (this repo): Control Panel, Agent Runtime, runtime services
#   - iron_cage (separate repo): Sandboxing, CLI tools, foundation modules
#
# This Makefile manages iron_runtime components (Control Panel, Agent Runtime).
# For sandboxing and foundation modules, see the iron_cage repository.

.PHONY: help
.DEFAULT_GOAL := help

#==============================================================================
# Configuration
#==============================================================================

BACKEND_DIR := .
DASHBOARD_DIR := module/iron_dashboard
API_DIR := module/iron_api
RUNTIME_DIR := module/iron_runtime
DB_FILE := module/iron_api/iron_api.db

# Python package directories
PYTHON_SDK_DIR := python/iron_sdk
PYTHON_EXAMPLES_DIR := python/iron_examples

#==============================================================================
# Help
#==============================================================================

help: ## Show this help message
	@echo "Iron Runtime Development Commands"
	@echo "=================================="
	@echo ""
	@echo "This is the iron_runtime repository (Control Panel, Agent Runtime)"
	@echo "For sandboxing/foundation modules, see iron_cage repository"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-25s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Quick Start:"
	@echo "  make dev               # Run Control Panel (API + Dashboard)"
	@echo "  make api-run           # Run API server only (port 3000)"
	@echo "  make dashboard-dev     # Run dashboard only (port 5173)"
	@echo "  make test              # Run all Rust tests"
	@echo "  make validate          # Full validation for production"

#==============================================================================
# Full Stack Commands (API + Dashboard)
#==============================================================================

dev: ## Run Control Panel (API server + Dashboard)
	@echo "🚀 Starting Control Panel (Ctrl+C to stop both)..."
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "📦 Dashboard dependencies not found, installing..."; \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	@trap 'kill 0' EXIT; \
		(cd $(BACKEND_DIR) && cargo run --release --bin iron_api_server) & \
		sleep 3 && \
		(cd $(DASHBOARD_DIR) && npm run dev)

dev-tmux: ## Run Control Panel in tmux (API + Dashboard side-by-side)
	@echo "🚀 Starting Control Panel in tmux..."
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "📦 Dashboard dependencies not found, installing..."; \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	@tmux new-session -d -s iron_runtime \
		"cd $(BACKEND_DIR) && cargo run --release --bin iron_api_server" \; \
		split-window -h "cd $(DASHBOARD_DIR) && npm run dev" \; \
		attach

dev-debug: ## Run Control Panel in debug mode (faster compilation)
	@echo "🚀 Starting Control Panel (debug mode)..."
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "📦 Dashboard dependencies not found, installing..."; \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	@trap 'kill 0' EXIT; \
		(cd $(BACKEND_DIR) && cargo run --bin iron_api_server) & \
		sleep 3 && \
		(cd $(DASHBOARD_DIR) && npm run dev)

build-all: api-build dashboard-build ## Build API + Dashboard for production

test-all: test dashboard-test ## Run all tests (Rust + Dashboard)

clean-all: api-clean dashboard-clean ## Clean all build artifacts

#==============================================================================
# API Server Commands (iron_api)
#==============================================================================

api-build: ## Build API server for production
	@echo "🔨 Building API server..."
	cd $(BACKEND_DIR) && cargo build --release --bin iron_api_server

api-build-debug: ## Build API server (debug mode, faster)
	@echo "🔨 Building API server (debug)..."
	cd $(BACKEND_DIR) && cargo build --bin iron_api_server

api-run: ## Run API server (release mode, port 3000)
	@echo "🚀 Starting API server (port 3000)..."
	cd $(BACKEND_DIR) && cargo run --release --bin iron_api_server

api-run-debug: ## Run API server (debug mode)
	@echo "🚀 Starting API server (debug)..."
	cd $(BACKEND_DIR) && cargo run --bin iron_api_server

api-clean: ## Clean Rust build artifacts
	@echo "🧹 Cleaning Rust build..."
	cd $(BACKEND_DIR) && cargo clean

#==============================================================================
# Dashboard Commands (iron_dashboard)
#==============================================================================

dashboard-install: ## Install dashboard dependencies
	@echo "📦 Installing dashboard dependencies..."
	cd $(DASHBOARD_DIR) && npm install

dashboard-dev: ## Run dashboard dev server (port 5173)
	@echo "🎨 Starting dashboard dev server..."
	@if [ ! -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "📦 Dependencies not found, installing..."; \
		cd $(DASHBOARD_DIR) && npm install; \
	fi
	cd $(DASHBOARD_DIR) && npm run dev

dashboard-build: ## Build dashboard for production
	@echo "🔨 Building dashboard..."
	cd $(DASHBOARD_DIR) && npm run build

dashboard-preview: ## Preview dashboard production build
	@echo "👀 Previewing dashboard production build..."
	cd $(DASHBOARD_DIR) && npm run preview

dashboard-test: ## Run dashboard tests
	@echo "🧪 Running dashboard tests..."
	cd $(DASHBOARD_DIR) && npm test

dashboard-lint: ## Lint dashboard code
	@echo "🔍 Linting dashboard..."
	cd $(DASHBOARD_DIR) && npm run lint

dashboard-type-check: ## Type check dashboard
	@echo "🔍 Type checking dashboard..."
	cd $(DASHBOARD_DIR) && npm run type-check

dashboard-clean: ## Clean dashboard build artifacts
	@echo "🧹 Cleaning dashboard..."
	cd $(DASHBOARD_DIR) && rm -rf node_modules dist

dashboard-validate: ## Validate dashboard for production
	@echo "✅ Validating dashboard for production..."
	@echo "  1. Type checking..."
	@cd $(DASHBOARD_DIR) && npm run type-check
	@echo "  2. Running tests..."
	@cd $(DASHBOARD_DIR) && npm test
	@echo "  3. Linting..."
	@cd $(DASHBOARD_DIR) && npm run lint
	@echo "  4. Building..."
	@cd $(DASHBOARD_DIR) && npm run build
	@echo ""
	@echo "✅ Dashboard validation complete!"

#==============================================================================
# Database Commands
#==============================================================================

db-reset: ## Reset database (deletes all data)
	@echo "🗄️  Resetting database..."
	@rm -f $(DB_FILE)
	@echo "Database reset complete. Restart server to recreate."

db-inspect: ## Open database in SQLite shell
	@echo "🔍 Opening database..."
	sqlite3 $(DB_FILE)

db-schema: ## Show database schema
	@echo "📋 Database schema:"
	@sqlite3 $(DB_FILE) ".schema" 2>/dev/null || echo "Database not found. Start server first."

db-users: ## List users in database
	@echo "👥 Users:"
	@sqlite3 $(DB_FILE) "SELECT id, username, created_at FROM users;" 2>/dev/null || echo "No users table found"

db-tokens: ## List tokens in database
	@echo "🎟️  Tokens:"
	@sqlite3 $(DB_FILE) "SELECT id, name, user_id, created_at FROM tokens;" 2>/dev/null || echo "No tokens table found"

#==============================================================================
# Testing Commands
#==============================================================================

test: ## Run all Rust tests (level 3)
	@echo "🧪 Running iron_runtime tests..."
	cd $(BACKEND_DIR) && w3 .test l::3

test-l1: ## Run tests level 1 (nextest only)
	@echo "🧪 Running tests (level 1)..."
	cd $(BACKEND_DIR) && w3 .test level::1

test-l2: ## Run tests level 2 (nextest + doc tests)
	@echo "🧪 Running tests (level 2)..."
	cd $(BACKEND_DIR) && w3 .test level::2

test-l3: ## Run tests level 3 (nextest + doc tests + clippy)
	@echo "🧪 Running tests (level 3)..."
	cd $(BACKEND_DIR) && w3 .test level::3

test-l4: ## Run tests level 4 (level 3 + udeps + audit)
	@echo "🧪 Running tests (level 4)..."
	cd $(BACKEND_DIR) && w3 .test level::4

test-quick: ## Quick test (nextest only, fast)
	@echo "🧪 Quick test..."
	cd $(BACKEND_DIR) && cargo nextest run --all-features

test-watch: ## Run tests in watch mode
	@echo "🧪 Watch mode..."
	cd $(BACKEND_DIR) && cargo watch -x "nextest run --all-features"

test-api: ## Run iron_api tests only
	@echo "🧪 Running iron_api tests..."
	cd $(API_DIR) && cargo nextest run --all-features

test-runtime: ## Run iron_runtime tests only
	@echo "🧪 Running iron_runtime tests..."
	cd $(RUNTIME_DIR) && cargo nextest run --all-features

#==============================================================================
# Python SDK Commands
#==============================================================================

sdk-install: ## Install Python SDK in development mode
	@echo "🐍 Installing iron_sdk..."
	cd $(PYTHON_SDK_DIR) && pip install -e .

sdk-test: ## Run Python SDK tests
	@echo "🧪 Running iron_sdk tests..."
	cd $(PYTHON_SDK_DIR) && pytest

examples-install: ## Install Python examples dependencies
	@echo "🐍 Installing iron_examples..."
	cd $(PYTHON_EXAMPLES_DIR) && pip install -e .

examples-run: ## Run example agent (requires API server running)
	@echo "🤖 Running example agent..."
	cd $(PYTHON_EXAMPLES_DIR) && python -m iron_examples.basic_agent

#==============================================================================
# Utility Commands
#==============================================================================

check: ## Run cargo check (fast compilation check)
	cd $(BACKEND_DIR) && cargo check --all-targets --all-features

fmt: ## Format all Rust code (DO NOT USE - custom formatting required)
	@echo "⚠️  WARNING: cargo fmt is forbidden per rulebook"
	@echo "All code must follow custom codestyle from rulebooks"
	@echo "Use manual formatting or editor configuration instead"

clippy: ## Run clippy linter
	cd $(BACKEND_DIR) && cargo clippy --all-targets --all-features -- -D warnings

logs: ## Show API server logs (if running in background)
	@tail -f logs/iron_api.log 2>/dev/null || echo "No logs found"

ports-check: ## Check if ports 3000 (API), 5173 (dashboard) are in use
	@echo "🔍 Checking ports..."
	@lsof -i :3000 | grep LISTEN && echo "Port 3000 (API) is in use" || echo "Port 3000 (API) is free"
	@lsof -i :5173 | grep LISTEN && echo "Port 5173 (dashboard) is in use" || echo "Port 5173 (dashboard) is free"

ports-kill: ## Kill processes on ports 3000 and 5173
	@echo "🛑 Killing processes on ports 3000 and 5173..."
	@lsof -i :3000 -t | xargs -r kill -9
	@lsof -i :5173 -t | xargs -r kill -9
	@echo "Ports cleared"

status: ## Show iron_runtime repository status
	@echo "📊 Iron Runtime Status"
	@echo "======================"
	@echo ""
	@echo "Rust (iron_runtime modules):"
	@cd $(BACKEND_DIR) && cargo --version
	@echo ""
	@echo "Dashboard:"
	@if [ -d "$(DASHBOARD_DIR)/node_modules" ]; then \
		echo "  ✅ Dependencies installed"; \
	else \
		echo "  ❌ Dependencies not installed (run: make dashboard-install)"; \
	fi
	@echo ""
	@echo "Database:"
	@if [ -f $(DB_FILE) ]; then \
		echo "  ✅ Database exists ($(DB_FILE))"; \
		echo "  Size: $$(du -h $(DB_FILE) | cut -f1)"; \
	else \
		echo "  ⚠️  Database not found (start API server to create)"; \
	fi
	@echo ""
	@echo "Python SDK:"
	@if python -c "import iron_sdk" 2>/dev/null; then \
		echo "  ✅ iron_sdk installed"; \
	else \
		echo "  ⚠️  iron_sdk not installed (run: make sdk-install)"; \
	fi
	@echo ""

#==============================================================================
# CI/CD Commands
#==============================================================================

ci: ## Run CI pipeline (tests + dashboard validation)
	@echo "🤖 Running CI pipeline..."
	make test-l3
	make dashboard-validate

ci-full: ## Run full CI pipeline (level 4 tests + dashboard)
	@echo "🤖 Running full CI pipeline..."
	make test-l4
	make dashboard-validate

validate: ## Full validation for production deployment
	@echo "🚀 Production Deployment Validation"
	@echo "===================================="
	@echo ""
	@echo "1️⃣  Rust Tests (Level 3)"
	@make test-l3 || (echo "❌ Rust tests failed" && exit 1)
	@echo "   ✅ Rust tests passed"
	@echo ""
	@echo "2️⃣  Dashboard Validation"
	@make dashboard-validate || (echo "❌ Dashboard validation failed" && exit 1)
	@echo "   ✅ Dashboard validated"
	@echo ""
	@echo "3️⃣  Build Verification"
	@make api-build || (echo "❌ API build failed" && exit 1)
	@echo "   ✅ API built successfully"
	@echo ""
	@echo "✅ Production validation complete!"
	@echo ""
	@echo "Next steps for deployment:"
	@echo "  • Deploy API server (Docker or binary)"
	@echo "  • Deploy dashboard (static files)"
	@echo "  • Configure database (SQLite/PostgreSQL)"

#==============================================================================
# Development Workflow Commands
#==============================================================================

setup: dashboard-install ## Initial setup for iron_runtime
	@echo "✅ iron_runtime setup complete!"
	@echo ""
	@echo "Next steps:"
	@echo "  make dev           # Start Control Panel (API + Dashboard)"
	@echo "  make test          # Run all tests"
	@echo "  make sdk-install   # Install Python SDK"

quick-start: setup dev ## Complete setup + start Control Panel

verify: ## Verify installation and run quick test
	@echo "✅ Verifying iron_runtime installation..."
	@make check
	@make test-quick
	@echo ""
	@echo "✅ All checks passed!"
