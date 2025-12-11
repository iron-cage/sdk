# iron_token_manager - Development Makefile
#
# Common development tasks for database management and testing.

.PHONY: help
help: ## Show this help message
	@echo "iron_token_manager - Development Commands"
	@echo ""
	@echo "Database Management:"
	@grep -E '^db-[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Validation:"
	@grep -E '^validate[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Testing:"
	@grep -E '^test[a-zA-Z_-]*:.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Development:"
	@grep -E '^(build|clean|check|fmt|clippy):.*?## .*$$' $(MAKEFILE_LIST) | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'

# ============================================================================
# Database Management
# ============================================================================

DB_PATH ?= ./iron.db

.PHONY: db-reset
db-reset: ## Reset database to clean state (creates backup)
	@./scripts/reset_dev_db.sh $(DB_PATH)

.PHONY: db-seed
db-seed: ## Populate database with test data
	@./scripts/seed_dev_data.sh $(DB_PATH)

.PHONY: db-reset-seed
db-reset-seed: ## Reset database and populate with test data (one command)
	@./scripts/reset_and_seed.sh $(DB_PATH)

.PHONY: db-inspect
db-inspect: ## Open database in sqlite3 CLI
	@sqlite3 $(DB_PATH)

.PHONY: db-schema
db-schema: ## Show database schema
	@sqlite3 $(DB_PATH) ".schema"

.PHONY: db-tables
db-tables: ## List all tables
	@sqlite3 $(DB_PATH) ".tables"

.PHONY: db-users
db-users: ## Show all users
	@sqlite3 $(DB_PATH) "SELECT id, username, role, super_user FROM users;"

.PHONY: db-tokens
db-tokens: ## Show all tokens (with metadata, not plaintext)
	@sqlite3 $(DB_PATH) "SELECT id, user_id, name, is_active, created_at FROM api_tokens;"

.PHONY: db-usage
db-usage: ## Show usage summary
	@sqlite3 $(DB_PATH) "SELECT COUNT(*) as total_requests, SUM(input_tokens) as total_input, SUM(output_tokens) as total_output, SUM(cost_cents)/100.0 as total_cost_usd FROM token_usage;"

.PHONY: db-backup
db-backup: ## Create manual backup
	@mkdir -p ./backups
	@cp $(DB_PATH) ./backups/manual_backup_$$(date +%Y%m%d_%H%M%S).db
	@echo "Backup created in ./backups/"

.PHONY: db-clean
db-clean: ## Delete database and all backups (DESTRUCTIVE)
	@read -p "Delete $(DB_PATH) and all backups? (y/N) " confirm && [ "$$confirm" = "y" ] || exit 1
	@rm -f $(DB_PATH)
	@rm -rf ./backups
	@echo "Database and backups deleted"

# ============================================================================
# Validation
# ============================================================================

.PHONY: validate
validate: validate-paths validate-schema validate-seed ## Run all validators

.PHONY: validate-paths
validate-paths: ## Validate database path consistency
	@./scripts/validate_db_paths.sh

.PHONY: validate-schema
validate-schema: ## Validate database schema correctness
	@./scripts/validate_db_schema.sh $(DB_PATH)

.PHONY: validate-seed
validate-seed: ## Validate seed data completeness
	@./scripts/validate_seed_data.sh $(DB_PATH)

# ============================================================================
# Git Hooks
# ============================================================================

.PHONY: install-hooks
install-hooks: ## Install git pre-commit hook for validation
	@echo "Installing pre-commit hook..."
	@mkdir -p ../../.git/hooks
	@cp scripts/pre-commit.hook ../../.git/hooks/pre-commit
	@chmod +x ../../.git/hooks/pre-commit
	@echo "✓ Pre-commit hook installed"
	@echo "  The hook will validate database paths before each commit"

.PHONY: uninstall-hooks
uninstall-hooks: ## Uninstall git pre-commit hook
	@if [ -f ../../.git/hooks/pre-commit ]; then \
		rm ../../.git/hooks/pre-commit; \
		echo "✓ Pre-commit hook uninstalled"; \
	else \
		echo "No pre-commit hook found"; \
	fi

# ============================================================================
# Development Workflows
# ============================================================================

.PHONY: dev-fresh
dev-fresh: ## Fresh development environment (reset + seed + validate)
	@./scripts/dev_fresh.sh

.PHONY: dev-fresh-test
dev-fresh-test: ## Fresh environment with full test suite
	@./scripts/dev_fresh.sh --test

.PHONY: dev-fresh-quick
dev-fresh-quick: ## Quick fresh environment (no validation)
	@./scripts/dev_fresh.sh --quick

# ============================================================================
# Testing
# ============================================================================

.PHONY: test
test: ## Run all tests (nextest + doctests + clippy)
	@clear
	@RUSTFLAGS="-D warnings" cargo nextest run --all-features
	@RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features
	@cargo clippy --all-targets --all-features -- -D warnings

.PHONY: test-quick
test-quick: ## Run tests only (no clippy)
	@clear
	@RUSTFLAGS="-D warnings" cargo nextest run --all-features

.PHONY: test-watch
test-watch: ## Run tests in watch mode
	@cargo watch -x "nextest run --all-features"

.PHONY: test-coverage
test-coverage: ## Generate test coverage report
	@cargo llvm-cov nextest --all-features --html
	@echo "Coverage report: target/llvm-cov/html/index.html"

# ============================================================================
# Development
# ============================================================================

.PHONY: build
build: ## Build project
	@cargo build --all-features

.PHONY: build-release
build-release: ## Build project in release mode
	@cargo build --all-features --release

.PHONY: check
check: ## Check code without building
	@cargo check --all-features

.PHONY: clippy
clippy: ## Run clippy linter
	@cargo clippy --all-targets --all-features -- -D warnings

.PHONY: clean
clean: ## Clean build artifacts
	@cargo clean
	@rm -f $(DB_PATH)

.PHONY: doc
doc: ## Build and open documentation
	@cargo doc --all-features --open

.PHONY: deps
deps: ## Show dependency tree
	@cargo tree

.PHONY: outdated
outdated: ## Check for outdated dependencies
	@cargo outdated

# ============================================================================
# CI/CD
# ============================================================================

.PHONY: ci
ci: clean test ## Run full CI pipeline (clean + test)

.PHONY: pre-commit
pre-commit: test ## Run pre-commit checks
