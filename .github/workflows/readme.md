# GitHub Actions Workflows

CI/CD pipeline for Iron Cage SDK. All workflows run on self-hosted runners configured via repository variables.

## Directory Structure

### Responsibility Table

| File | Responsibility |
|------|----------------|
| `deploy.yaml` | Deploy to Hetzner on push to master or manual trigger |
| `deploy-check.yml` | Validate deploy infrastructure on PRs to master |
| `iron_token_manager_validation.yml` | Run iron_token_manager tests and schema checks |
| `readme.md` | Document workflow directory structure and usage |

## Workflows

### `deploy.yaml` — Deploy CI

Builds, pushes and deploys the application to a Hetzner server via GCP infrastructure.

**Triggers:**
- Push to `master` → deploys to `dev` environment
- Manual (`workflow_dispatch`) → deploys to chosen environment (`dev` / `staging` / `production`)

**Concurrency:** one deploy per environment at a time; new runs wait, never cancel in-progress.

**Steps:**
1. Decode GCP service account credentials from base64 secret into `.secret/-service_account.json`
2. Write SSH key pair to `.secret/-iron_sdk` / `.secret/-iron_sdk.pub`
3. Assemble `.secret/-secret.sh` with all runtime variables using `printf` (safe for special characters)
4. `make deploy` — build image, push to GAR, deploy to server
5. Notify Google Chat on failure via `jq` + `curl --fail` (skipped silently if `GCHAT_WEBHOOK_URL` is not set); `continue-on-error: true`
6. Shred all secret files with `shred -vfz -n 3` (runs even on failure)

**Required secrets:** `SECRET_GCP_CREDENTIALS`, `SECRET_RSA_PRIVATE_KEY`, `SECRET_RSA_PUBLIC_KEY`, `TF_VAR_PROJECT_ID`, `GOOGLE_APPLICATION_REGION`, `SECRET_STATE_ARCHIVE_KEY`, `HOST_SERVER_NAME`, `HOST_SERVER_IP`, `SECRET_HETZNER_CLOUD_TOKEN`, `DATABASE_URL`, `JWT_SECRET`, `IC_TOKEN_SECRET`, `IP_TOKEN_KEY`, `IRON_SECRETS_MASTER_KEY`, `ALLOWED_ORIGINS`, `SERVER_PORT`, `IRON_DEPLOYMENT_MODE`, `ENABLE_DEMO_SEED`, `GCHAT_WEBHOOK_URL` (optional)

**Required vars:** `GH_RUNNER_DEPLOY`, `DEPLOYMENT_MODE`, `PROJECT_NAME`, `PROJECT_DOMAIN`, `PROJECT_CERT_EMAIL`, `HOST_SERVER_IMAGE`, `HOST_SERVER_LOCATION`, `HOST_SERVER_TYPE`, `ALLOWED_SSH_IPS`, `RUST_LOG`

---

### `deploy-check.yml` — Deployment CI

Validates deploy infrastructure on every PR targeting `master`. Does not deploy anything.

**Triggers:** Pull request to `master`

**Jobs:**
- **`deploy-tests`** — runs `deploy/tests/redeploy.bats` via Bats against a local k3s cluster
- **`terraform-validate`** — runs `terraform init -backend=false` + `terraform validate` for each Terraform module (`gar`, `hetzner_server_create`, `service_deploy`) in parallel

**Required vars:** `GH_RUNNER_DEPLOY_CHECK`

---

### `iron_token_manager_validation.yml` — Iron Token Manager Validation

Validates the `module/iron_token_manager` crate on changes to that module.

**Triggers:**
- PR touching `module/iron_token_manager/**`
- Push to `master` / `main` touching `module/iron_token_manager/**`

**Jobs (sequential):**
1. **`path-validation`** — runs `scripts/validate_db_paths.sh`
2. **`test-and-validate`** — full Rust test suite via `cargo nextest`, doctests, Clippy, DB schema validation, seed data validation
3. **`validation-summary`** — reports combined result; fails the check if either prior job failed

**Runner:** `ubuntu-latest` (GitHub-hosted)
