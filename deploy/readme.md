# Terraform Deployment

This directory contains Terraform configurations and tooling for deploying the web application to multiple cloud providers **through a single entry point**: the project’s root `Makefile`.

Supported providers:

- Google Cloud Platform (GCP) – GCS, Artifact Registry
- Hetzner Cloud

You **do not** run `terraform` manually. Instead:

- Terraform is executed **inside a Docker container** built from `Dockerfile`.
- Terraform state is stored **automatically in Google Cloud Storage (GCS)** via the `gcs` backend.
- All modules (`gcs`, `gar`, `hetzner`) are orchestrated by **`make -f Makefile.deploy deploy`**.

---

## Content:

- [Quick Start: One Command Deploy](#quick-start-one-command-deploy)
- [Terraform State in GCS](#terraform-state-in-gcs)
- [Useful Make Targets](#useful-make-targets)
- [Terraform Module Layout](#terraform-module-layout)
- [Testing the Redeploy Script](#testing-the-redeploy-script)
- [CI/CD Deployment with GitHub Actions](#cicd-deployment-with-github-actions)
- [Summary](#summary)

---

## Directory Contents

| File | Responsibility |
|---|---|
| `Dockerfile` | Defines the Docker image containing Terraform and cloud CLIs (`gcloud`, `hcloud`) for executing deployments. |
| `Makefile` | Orchestrates the entire deployment process, providing targets like `deploy`, `clean`, and `z_destroy_all`. |
| `redeploy.sh` | Contains the script run on the Hetzner instance to pull and restart the application Docker container. |
| `gcs/` | Contains Terraform configuration for the GCS bucket used for remote state storage. |
| `gar/` | Contains Terraform configuration for the Google Artifact Registry repository. |
| `hetzner/` | Contains Terraform configuration for provisioning the Hetzner cloud server and related resources. |
| `tests/` | Contains Bats tests for the `redeploy.sh` script. |

---

## Quick Start: One Command Deploy

1. **Create secrets file**

   In the project root, create:

   ```bash
   secret/-secret.sh
   ```

   This file must contain `KEY=value` lines (bash syntax).  
   It is **not committed** to git; only documentation templates like `readme.md` or `env.example` should be tracked.

2. **Fill in required variables**

   Minimum example for deployment to **HETZNER**:

   ```bash
   # Cloud provider selection: hetzner
   CSP="hetzner"
   SECRET_HETZNER_CLOUD_TOKEN="YOUR_HETZNER_API_TOKEN"

   # Project name (used for tagging & defaults)
   PROJECT_NAME="project_name"

   # Required secrets
   SECRET_STATE_ARCHIVE_KEY="change-me-random-string"
   GOOGLE_SE_CREDS_PATH="secret/gcp-service-account.json"
   SECRET_RSA_PRIVATE_KEY_PATH="secret/id_rsa"
   SECRET_RSA_PUBLIC_KEY_PATH="secret/id_rsa.pub"

   # Optional overrides (can be miss)
   TF_VAR_PROJECT_ID="my-gcp-project-id"
   TF_VAR_REGION="europe-central2"
   TF_VAR_ZONE="europe-central2-a"
   TF_VAR_BUCKET_NAME="my-terraform-state-bucket"  # must be globally unique
   ```
   For more detailed description of variables, see [`../secret/readme.md`](../secret/readme.md).

3. **Run deploy**

   From the project root:

   ```bash
   make -f Makefile.deploy deploy
   ```

   This will:

   1. Load `secret/-secret.sh` and validate required variables and secret files.
   2. Build the **application Docker image** from the root `Dockerfile` and tag it for Artifact Registry.
   3. Build a helper image `deploy-<image-name>` from `deploy/Dockerfile` (with `terraform`, `gcloud`, and `docker`).
   4. Run that image and execute `make -f Makefile.deploy deploy_in_container` **inside the container**.

   Inside the container, `deploy_in_container` performs:

   1. `lock_check` – checks for active Terraform locks in the GCS bucket.
   2. `gcp_service` – authenticates `gcloud` using your service account JSON.
   3. `state_storage_init` – creates the **GCS bucket** for Terraform state if it doesn’t exist yet.
   4. `check_keys_<CSP>` – validates provider-specific secrets (Hetzner).
   5. `gcp_docker` – configures Docker to authenticate to Artifact Registry.
   6. `push_image` – runs `tf_init`, creates the Artifact Registry repo, and pushes the app image.
   7. `create_<CSP>` – runs `terraform apply` for the chosen provider module (`hetzner`).
   8. `show_state_info` – prints where the Terraform state files live in GCS.

> For a regular user / developer the workflow is: **configure `secret/-secret.sh` → run `make -f Makefile.deploy deploy` → done.**

---

## Terraform State in GCS

Each Terraform module (`deploy/gcs`, `deploy/gar`, `deploy/hetzner`) contains:

```hcl
terraform {
  backend "gcs" {}
}
```

The Makefile passes backend settings during `terraform init`:

```make
terraform -chdir=$(TF_DIR)/$$dir init   -backend-config="bucket=$(TF_VAR_BUCKET_NAME)"   -backend-config="prefix=$$dir"
```

As a result, after a successful deploy your states are stored in:

- `gs://<bucket-name>/gar/default.tfstate` – Artifact Registry
- `gs://<bucket-name>/hetzner/default.tfstate` – Hetzner infrastructure

The bucket itself is managed by the `gcs` module and created automatically by `state_storage_init` (using `TF_VAR_BUCKET_NAME`, which defaults to `bucket-<repo-name>` if not set).

> ✅ You never manually manage `terraform.tfstate` files — they live in GCS and are shared-safe.

---

## Useful Make Targets

Although **`make -f Makefile.deploy deploy`** is the main entry point, a few other targets are helpful.

| Target              | What it does                                                                                  |
|---------------------|------------------------------------------------------------------------------------------------|
| `make -f Makefile.deploy deploy`       | Builds app image, builds Terraform Docker image, runs full deployment inside the container.   |
| `make -f Makefile.deploy clean`        | Cleans local Docker artifacts used for build (`name:<image>` and buildx cache).              |
| `make -f Makefile.deploy tf_init`      | Runs `terraform init` for all modules (`gar`, `hetzner`) with the GCS backend.  |
| `make -f Makefile.deploy tf_plan`      | Runs `terraform plan` for all modules using your local Terraform CLI (if installed).         |
| `make -f Makefile.deploy lock_check`   | Checks for active `.tflock` files in the state bucket for all modules.                        |
| `make -f Makefile.deploy lock_unlock`  | Force-unlocks Terraform state for all modules by reading lock IDs from GCS.                   |
| `make -f Makefile.deploy z_destroy_all`| Destroys infrastructure in all modules **from inside the Docker container** (asks for confirm). |


> ⚠️ `z_destroy_all` is destructive: it removes resources in `hetzner`, `gar` and delete the state bucket. 

Most users will only need:

```bash
make -f Makefile.deploy deploy         # create/update infrastructure
make -f Makefile.deploy z_destroy_all  # tear everything down (if really needed)
make -f Makefile.deploy clean     # cleans local Docker artifacts used for build (if really needed)
```

---

## Terraform Module Layout

Each cloud provider (and shared resources) lives under `deploy/`:

- `deploy/gcs/` – GCS bucket for Terraform remote state.
- `deploy/gar/` – Google Artifact Registry repository for Docker images.
- `deploy/hetzner/` – Hetzner instance + deploy logic.

You normally **do not run Terraform directly** inside these directories — the Makefile handles:

- backend initialization,
- passing variables,
- authentication,
- and running `apply` / `destroy` in the correct order inside the Docker container.

---

## Testing the Redeploy Script

The instance-side deployment logic (the `redeploy.sh` script) can be tested with [Bats](https://github.com/bats-core/bats-core).

1. Install Bats:

   ```bash
   sudo apt-get update && sudo apt-get install -y bats
   ```

2. Run tests:

   ```bash
   cd deploy
   bats tests/redeploy.bats
   ```

---

## CI/CD Deployment with GitHub Actions

The same `make -f Makefile.deploy deploy` flow can be executed automatically in CI using **GitHub Actions**. The repository contains a `Deploy CI` workflow that recreates `secret/-secret.sh` from repository secrets and then runs `make -f Makefile.deploy deploy`.

### 1. Configure repository secrets

In the GitHub repository:

1. Open **Settings → Secrets and variables → Actions**.
2. Create the following **repository secrets** (names must match exactly):

| Secret name                    | Purpose / example value                                                                                           |
|--------------------------------|--------------------------------------------------------------------------------------------------------------------|
| `CSP`                          | Cloud provider selector, `hetzner` available |
| `PROJECT_NAME`                 | Logical project name used for tagging and resource naming (for example `my-project`).                             |
| `SECRET_GCP_CREDENTIALS`       | Contents of the Google Cloud service account key in JSON format; used by `google-github-actions/auth`. Must be decoded to base64 format `base64 -w 0 service-account.json`  |
| `GOOGLE_SE_CREDS_PATH`         | Path where the service account JSON will be written during the workflow (for example `secret/gcp-service-account.json`). |
| `SECRET_STATE_ARCHIVE_KEY`     | Encryption key / passphrase used by the deploy scripts for the state/archive.                                     |
| `SECRET_RSA_PRIVATE_KEY`       | RSA private key used by Terraform / deploy scripts (for example SSH key for VMs).                                 |
| `SECRET_RSA_PRIVATE_KEY_PATH`  | Path where the private key file will be created (for example `secret/id_rsa`).                                   |
| `SECRET_RSA_PUBLIC_KEY`        | Matching RSA public key.                                                                                           |
| `SECRET_RSA_PUBLIC_KEY_PATH`   | Path where the public key file will be created (for example `secret/id_rsa.pub`).                                |
| `SECRET_HETZNER_CLOUD_TOKEN`   | API token for Hetzner Cloud (required when `CSP=hetzner`).                                                        |
| `TF_VAR_PROJECT_ID`            | Project id for deployed resources |
| `DATABASE_URL`                 | SQLite connection string for pilot mode   |
| `JWT_SECRET`                   | JWT secret key for signing access and refresh tokens   |
| `IRON_SECRETS_MASTER_KEY`      | Master key for AES-256-GCM encryption of AI provider API keys   |
| `OPENAI_API_KEY`               | OPENAI API key for accessing GPT, DALL·E, etc.   |
| `APOLLO_API_KEY`               | Apollo Studio API key for GraphQL schema publishing and analytics   |

These secrets are written into `secret/-secret.sh` and into key files by the workflow so that `make -f Makefile.deploy deploy` sees the same configuration as in local runs.

### 2. Deploy from CI

The deploy workflow:

- Triggers on pushes to the `master` and `feat/hosting` branches, and can also be started manually via **Actions → Deploy CI → Run workflow**.
- Authenticates to GCP using `SECRET_GCP_CREDENTIALS`.
- Creates `secret/-secret.sh` and the key files from the repository secrets.
- Runs `make -f Makefile.deploy deploy`, which builds the image, pushes it, and applies Terraform.

Once the secrets above are configured, no additional manual steps are required: pushing to the configured branches will automatically run the same deployment pipeline that you can run locally.

---

## Summary

- Configure everything in **`secret/-secret.sh`**.
- Use a **single command**: `make -f Makefile.deploy deploy`.
- Terraform always runs in a **Docker container** built from `deploy/Dockerfile`.
- Terraform state is stored **automatically in GCS** via the `gcs` backend.
- The same flow works locally and in CI, and supports **Hetzner** from one place.
