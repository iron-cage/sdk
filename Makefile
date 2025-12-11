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

# ============================================================================
# Deploy
# ============================================================================

## --------------------------------------------------------------------------------------
## READ variables from .secret/-secret.sh

## Configuration variables for deployment. Can be edited for desired behavior.
SHELL := /usr/bin/env bash
## Local secrets file for development (NOT committed; .secret/ is ignored).
LOCAL_SECRETS_FILE ?= .secret/-secret.sh

ifneq ("$(wildcard $(LOCAL_SECRETS_FILE))","")
	include $(LOCAL_SECRETS_FILE)
else
	$(error "Secrets file $(LOCAL_SECRETS_FILE) not found")
endif

strip_quotes = $(subst ",,$(1))

## --------------------------------------------------------------------------------------
## Required Parameters
## GOOGLE_SE_CREDS_PATH
REQUIRED_SECRET_VARIABLES:= \
	SECRET_STATE_ARCHIVE_KEY \
	GOOGLE_SE_CREDS_PATH \
	SECRET_RSA_PRIVATE_KEY_PATH \
	SECRET_RSA_PUBLIC_KEY_PATH \
	CSP \
	PROJECT_NAME \
	ENVIRONMENT

$(foreach v,$(REQUIRED_SECRET_VARIABLES),\
  $(if $($v),,$(error Required secret variable '$(v)' is missing or empty after including $(LOCAL_SECRETS_FILE))))

SECRET_STATE_ARCHIVE_KEY 	:= $(call strip_quotes,$(SECRET_STATE_ARCHIVE_KEY))
GOOGLE_SE_CREDS_PATH 		:= $(call strip_quotes,$(GOOGLE_SE_CREDS_PATH))
SECRET_RSA_PRIVATE_KEY_PATH := $(call strip_quotes,$(SECRET_RSA_PRIVATE_KEY_PATH))
SECRET_RSA_PUBLIC_KEY_PATH 	:= $(call strip_quotes,$(SECRET_RSA_PUBLIC_KEY_PATH))
CSP 						:= $(call strip_quotes,$(CSP))
PROJECT_NAME 				:= $(call strip_quotes,$(PROJECT_NAME))

## --------------------------------------------------------------------------------------
## Check that secret files exist
REQUIRED_SECRET_FILES := \
	$(GOOGLE_SE_CREDS_PATH) \
	$(SECRET_RSA_PRIVATE_KEY_PATH) \
	$(SECRET_RSA_PUBLIC_KEY_PATH)

define check_file_exists
  $(if $(wildcard $(call strip_quotes,$1)),, \
    $(error Required secret file '$(call strip_quotes,$1)' does not exist. Please check path.))
endef

$(foreach file,$(REQUIRED_SECRET_FILES),$(eval $(call check_file_exists,$(file))))

## --------------------------------------------------------------------------------------
## Optional Parameters
OPTIONAL_SECRET_VARIABLES:= \
	TF_VAR_PROJECT_ID \
	TF_VAR_REGION \
	SECRET_HETZNER_CLOUD_TOKEN \
	TF_VAR_REPO_NAME \
	TF_VAR_IMAGE_NAME \
	TAG \
	TF_VAR_ZONE \
	TF_VAR_BUCKET_NAME \
	TF_DIR
	
## Project id for deployed resources | Can be set in .secret/-secret.sh
TF_VAR_PROJECT_ID := $(or $(call strip_quotes,$(TF_VAR_PROJECT_ID)),$(shell jq -r '.project_id' "$(GOOGLE_SE_CREDS_PATH)"))
## Location for deployed resources | Can be set in .secret/-secret.sh
TF_VAR_REGION := $(or $(call strip_quotes,$(TF_VAR_REGION)),"europe-central2")
## Artifact Repository name for pushing the Docker images | Should not have "_"
TF_VAR_REPO_NAME := $(or $(call strip_quotes,$(TF_VAR_REPO_NAME)),$(subst _,-,$(PROJECT_NAME)))
## Pushed image name | Can be set in .secret/-secret.sh
TF_VAR_IMAGE_NAME := $(or $(call strip_quotes,$(TF_VAR_IMAGE_NAME)),$(PROJECT_NAME))
## Helper var for tagging local image
TAG := $(or $(call strip_quotes,$(TAG)),$(TF_VAR_REGION)-docker.pkg.dev/$(TF_VAR_PROJECT_ID)/$(TF_VAR_REPO_NAME)/$(TF_VAR_IMAGE_NAME))/$(ENVIRONMENT)
## Zone location for the resource
TF_VAR_ZONE := $(or $(call strip_quotes,$(TF_VAR_ZONE)),$(TF_VAR_REGION)-a)
## Cloud Storage bucket name
TF_VAR_BUCKET_NAME := $(or $(call strip_quotes,$(TF_VAR_BUCKET_NAME)),bucket-$(TF_VAR_REPO_NAME)-$(ENVIRONMENT))
## Base terraform directory
TF_DIR := $(or $(call strip_quotes,$(TF_DIR)),deploy)
## IMAGE BUILD
export GOOGLE_APPLICATION_CREDENTIALS=$(TF_VAR_GOOGLE_SE_CREDS_PATH)

build-image: ## Builds uarust_conf_site image
	docker build . -f Dockerfile.frontend -t name:$(TF_VAR_IMAGE_NAME)_front -t $(TAG)_front
	docker build . -f Dockerfile.backend -t name:$(TF_VAR_IMAGE_NAME)_back -t $(TAG)_back

push-image: gcp-docker create-artifact-repo ## Builds and pushes local docker image to the private repository
	docker push $(TAG)_front
	docker push $(TAG)_back

## Deploys using tools from the container
deploy: check_gcp_keys build_image
	docker build . \
	  -t deploy-$(TF_VAR_IMAGE_NAME) \
	  -f ./$(TF_DIR)/Dockerfile \
	  --build-arg google_se_creds="$(GOOGLE_SE_CREDS_PATH)"
	@docker run --rm \
	--user $(shell id -u):$(shell id -g) \
	-v /var/run/docker.sock:/var/run/docker.sock \
	-v $(CURDIR):/app \
	-e SECRET_STATE_ARCHIVE_KEY="$(SECRET_STATE_ARCHIVE_KEY)" \
	-e TF_VAR_HETZNER_CLOUD_TOKEN="$(SECRET_HETZNER_CLOUD_TOKEN)" \
	-e TF_VAR_BUCKET_NAME="$(TF_VAR_BUCKET_NAME)" \
	-e TF_VAR_PROJECT_ID="$(TF_VAR_PROJECT_ID)" \
	-e TF_VAR_PROJECT_NAME="$(PROJECT_NAME)" \
	-e TF_VAR_REGION="$(TF_VAR_REGION)" \
	-e TF_VAR_REPO_NAME="$(TF_VAR_REPO_NAME)" \
	-e TF_VAR_IMAGE_NAME="$(TF_VAR_IMAGE_NAME)" \
	-e TF_VAR_ENVIRONMENT="$(ENVIRONMENT)" \
	-e CSP="$(CSP)" \
	-e TF_VAR_GOOGLE_SE_CREDS_PATH="/app/$(GOOGLE_SE_CREDS_PATH)" \
	-e GOOGLE_APPLICATION_CREDENTIALS="/app/$(GOOGLE_SE_CREDS_PATH)" \
	-e TF_VAR_SECRET_RSA_PRIVATE_KEY_PATH="/app/$(SECRET_RSA_PRIVATE_KEY_PATH)" \
	-e TF_VAR_SECRET_RSA_PUBLIC_KEY_PATH="/app/$(SECRET_RSA_PUBLIC_KEY_PATH)" \
	-t deploy-$(TF_VAR_IMAGE_NAME)

## Check if required GCP keys are present
check_gcp_keys:
	@[ -f "$(GOOGLE_SE_CREDS_PATH)" ] \
		|| echo "ERROR: Key file $(GOOGLE_SE_CREDS_PATH) does not exist"
	@[ ! -z "${SECRET_STATE_ARCHIVE_KEY}" ] \
		|| echo "ERROR: Key SECRET_STATE_ARCHIVE_KEY does not exist"
	@[ -f "$(GOOGLE_SE_CREDS_PATH)" ] || exit 1
	@[ ! -z "${SECRET_STATE_ARCHIVE_KEY}" ] || exit 1

## --------------------------------------------------------------------
## DEPLOY IN DOCKER CONTAINER

## Deploys everything and updates terraform states (called inside deploy container)
deploy_in_container: lock_check gcp_service state_storage_init check_keys_$(CSP) gcp_docker push_image
	$(MAKE) create_$(CSP) || { echo "Deployment failed"; exit 1; }
	$(MAKE) show_state_info || { echo "Showing state info failed"; exit 1; }
## Authorize to GCP with service account
gcp_service:
	gcloud auth activate-service-account --key-file=$(GOOGLE_APPLICATION_CREDENTIALS)
## Creates GCS Bucket for terraform states
state_storage_init:
	@if gsutil ls -b "gs://$(TF_VAR_BUCKET_NAME)" > /dev/null 2>&1; then \
		echo "GCS bucket $(TF_VAR_BUCKET_NAME) already exists. Skipping creation."; \
	else \
		echo "Creating GCS bucket $(TF_VAR_BUCKET_NAME)..."; \
		gcloud storage buckets create gs://$(TF_VAR_BUCKET_NAME) \
			--project=$(TF_VAR_PROJECT_ID) \
			--location=$(TF_VAR_REGION) \
			--uniform-bucket-level-access; \
	fi
## Builds and pushes local docker image to the private repository
push_image: tf_init create_artifact_repo
	docker push $(TAG)
## Initializes all terraform projects. Downloads required modules and validates .tf files
tf_init:
	@for dir in gar hetzner; do \
	  terraform -chdir=$(TF_DIR)/$$dir init \
	    -backend-config="bucket=$(TF_VAR_BUCKET_NAME)" \
	    -backend-config="prefix=$$dir"; \
	done
## Creates Artifact Registry repository on GCP in specified location
create_artifact_repo:
	terraform -chdir=$(TF_DIR)/gar apply -auto-approve
## Add docker repo auth helper
gcp_docker:
	gcloud auth configure-docker $(TF_VAR_REGION)-docker.pkg.dev --quiet
## Creates Hetzner instance with the website configured on boot
create_hetzner: 
	terraform -chdir=$(TF_DIR)/hetzner apply -auto-approve

## --------------------------------------------------------------------
## KEY / SECRETS CHECKS
## Check Hetzner and deployment related keys
check_keys_hetzner:
	@[ ! -z "${SECRET_HETZNER_CLOUD_TOKEN}" ] \
		|| { echo "ERROR: Key SECRET_HETZNER_CLOUD_TOKEN does not exist"; exit 1; }

## --------------------------------------------------------------------
## TERRAFORM STATE STORAGE
show_state_info:
	@echo "Terraform states are now stored in:"
	@echo "gs://$(TF_VAR_BUCKET_NAME)/gar/default.tfstate"
	@echo "gs://$(TF_VAR_BUCKET_NAME)/hetzner/default.tfstate"

## --------------------------------------------------------------------
## TERRAFORM PLAN / DESTROY
## Review changes that terraform will do on apply
tf_plan: tf_init
	terraform -chdir=$(TF_DIR)/gar plan
	terraform -chdir=$(TF_DIR)/hetzner plan

ABS_GOOGLE_SE_CREDS_PATH := $(abspath $(GOOGLE_SE_CREDS_PATH))
## Destroy created infrastructure on all supported providers and finally GCS bucket
z_destroy_all:
	@echo "⚠️  WARNING: This will destroy ALL cloud resources and terraform state!"
	@read -p "❓ Are you sure you want to continue? Type 'destroy' to proceed: " confirm; \
	if [ "$$confirm" != "destroy" ]; then echo "❌ Aborted by user."; exit 1; fi
	@echo "🧨 Destroying all infrastructure across modules..."
	@for dir in hetzner gar; do \
		echo "🔻 Destroying: $$dir..."; \
		docker run --rm \
			-v /var/run/docker.sock:/var/run/docker.sock \
			-v $(CURDIR):/app \
			-w /app/deploy/$$dir \
			-e GOOGLE_APPLICATION_CREDENTIALS="/app/$(GOOGLE_SE_CREDS_PATH)" \
			-e TF_VAR_GOOGLE_SE_CREDS_PATH="/app/$(GOOGLE_SE_CREDS_PATH)" \
			-e TF_VAR_REGION="$(TF_VAR_REGION)" \
			-e TF_VAR_ZONE="$(TF_VAR_ZONE)" \
			-e TF_VAR_PROJECT_ID="$(TF_VAR_PROJECT_ID)" \
			-e TF_VAR_REPO_NAME="$(TF_VAR_REPO_NAME)" \
			-e TF_VAR_IMAGE_NAME="$(TF_VAR_IMAGE_NAME)" \
			-e TF_VAR_BUCKET_NAME="$(TF_VAR_BUCKET_NAME)" \
			-e SECRET_STATE_ARCHIVE_KEY="$(SECRET_STATE_ARCHIVE_KEY)" \
			-e TF_VAR_HETZNER_CLOUD_TOKEN="$(SECRET_HETZNER_CLOUD_TOKEN)" \
			-e TF_VAR_PROJECT_NAME="$(PROJECT_NAME)" \
			-e CSP="$(CSP)" \
			-e TF_VAR_SECRET_RSA_PRIVATE_KEY_PATH="/app/$(SECRET_RSA_PRIVATE_KEY_PATH)" \
			-e TF_VAR_SECRET_RSA_PUBLIC_KEY_PATH="/app/$(SECRET_RSA_PUBLIC_KEY_PATH)" \
			-v $(CURDIR)/$(GOOGLE_SE_CREDS_PATH):/app/$(GOOGLE_SE_CREDS_PATH) \
			-it hashicorp/terraform:1.7.4 destroy -auto-approve \
		|| echo "⚠️  Warning: Failed to destroy $$dir. Skipping."; \
	done
	@echo "🗑️  Destroying GCS backend bucket (if exists)..."
	gsutil -m rm -r gs://$(TF_VAR_BUCKET_NAME) || echo "⚠️ GCS destroy skipped or failed"
	@echo "✅ All resources destroyed successfully."

## --------------------------------------------------------------------
## CHECK | DESTROY LOCK

## Check if any Terraform state lock files exist
lock_check:
	@echo "🔍 Checking for active Terraform state locks in bucket: $(TF_VAR_BUCKET_NAME)..."
	@FOUND_LOCKS=0; \
	for dir in gar hetzner; do \
	  LOCK_PATH=gs://$(TF_VAR_BUCKET_NAME)/$$dir/default.tflock; \
	  if gsutil stat $$LOCK_PATH > /dev/null 2>&1; then \
	    echo "⚠️  Lock found in module: $$dir"; \
	    echo "    ➤ To unlock: make lock_unlock"; \
	    echo "    ➤ Or manually: terraform -chdir=$(TF_DIR)/$$dir force-unlock <LOCK_ID>"; \
	    echo "    ➤ To get LOCK_ID: gsutil cat $$LOCK_PATH | grep '\"ID\"'"; \
	    FOUND_LOCKS=1; \
	  else \
	    echo "✅ No lock in module: $$dir"; \
	  fi \
	done; \
	if [ $$FOUND_LOCKS -eq 1 ]; then \
	  echo "💡 Some locks are active. Please, run <make lock_unlock> to unlock before retrying apply."; \
	  exit 1; \
	else \
	  echo "🎉 No active locks detected."; \
	fi

## Force unlock Terraform state (tries to find lock for all modules)
lock_unlock:
	@for dir in gar hetzner; do \
	  LOCK_FILE=gs://$(TF_VAR_BUCKET_NAME)/$$dir/default.tflock; \
	  if gsutil stat $$LOCK_FILE > /dev/null 2>&1; then \
	    echo "⚠️ Found lock file in: $$LOCK_FILE"; \
	    LOCK_ID=$$(gsutil cat $$LOCK_FILE | grep '"ID"' | sed -E 's/.*"ID": ?"([^"]+)".*/\1/'); \
	    echo "🛠️ Forcing unlock: $$LOCK_ID in $$dir"; \
	    terraform -chdir=$(TF_DIR)/$$dir force-unlock $$LOCK_ID || true; \
	  else \
	    echo "✅ No lock in $$dir"; \
	  fi \
	done
