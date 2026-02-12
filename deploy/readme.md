# Deploy Directory

This directory contains the infrastructure-as-code and deployment automation for the project using **Kubernetes (k3s)** for container orchestration with **zero-downtime deployments** and **automatic rollback**.

## Directory Structure

```
deploy/
├── Dockerfile.deploy      # Container image for running deployments
├── Makefile.deploy        # Main deployment orchestration (entry point)
├── cloud-init.yml         # Server initialization script (installs k3s)
├── redeploy.sh            # Kubernetes deployment script (runs on target server)
├── gar/                   # Google Artifact Registry Terraform module
├── hetzner_server_create/ # Hetzner server provisioning Terraform module
├── k8s/                   # Kubernetes manifests (Kustomize)
│   ├── deployment.yaml    # Pod deployment with rolling updates
│   ├── service.yaml       # ClusterIP service
│   ├── ingress.yaml       # Ingress routing rules
│   └── kustomization.yaml # Kustomize configuration
└── service_deploy/        # Service deployment Terraform module
```

## Responsibility Table

| Item | Responsibility |
|------|----------------|
| `Dockerfile.deploy` | Define container image with deployment tools (gcloud, terraform, hcloud) |
| `Makefile.deploy` | Orchestrate deployment targets and manage environment variables |
| `cloud-init.yml` | Configure server initialization script for k3s installation |
| `redeploy.sh` | Execute Kubernetes deployment with rollout and rollback logic |
| `gar/` | Provision Google Artifact Registry repository for Docker images |
| `hetzner_server_create/` | Provision Hetzner Cloud server with firewall and k3s |
| `k8s/` | Define Kubernetes manifests for application deployment |
| `service_deploy/` | Deploy services to server via SSH provisioners |

## How It Works

### Deployment Flow

1. **`make deploy`** - Builds a deployment container and runs the orchestration
2. **Build & Push** - Docker images are built and pushed to Google Artifact Registry
3. **Infrastructure** - Terraform provisions Hetzner server with k3s (if needed)
4. **Service Deploy** - Kubernetes manifests are applied with rolling updates

### Key Features

- **Zero-Downtime Deployments** - Rolling updates with 2 replicas ensure service availability during deployments
- **Automatic Rollback** - Failed deployments automatically rollback to the previous working version
- **Health Checks** - Readiness and liveness probes ensure only healthy pods receive traffic
- **Ingress Routing** - k3s Traefik ingress handles domain-based routing

### Deployment Modes

Supported modes (set via `DEPLOYMENT_MODE`):
- `dev` - Development environment (currently implemented)
- `staging` - Staging environment (not required for this project)
- `production` - Production environment (not required for this project)

> **Note:** This project uses `dev` mode only. Staging and production modes are defined for future extensibility but not implemented as separate environments are not required for this project's scope.

## Environment Variables

Core variables used by deployment scripts. See [.secret/readme.md](../.secret/readme.md) for complete configuration reference.

| Variable | Values | Description |
|----------|--------|-------------|
| `DEPLOYMENT_MODE` | `dev`, `staging`, `production` | Target deployment environment (only `dev` implemented) |
| `PROJECT_NAME` | string | Service identifier, used for folder naming and Kubernetes namespace |
| `PROJECT_DOMAIN` | domain string | Domain for ingress routing and TLS certificate |
| `ARTIFACT_REPO_NAME` | string | Google Artifact Registry repository name, also used as Kubernetes namespace |
| `GOOGLE_APPLICATION_REGION` | GCP region | Region for GAR and cloud resources |
| `HOST_SERVER_IP` | IPv4 address | Target server IP (or `None` to create new) |
| `HOST_SERVER_NAME` | string | Expected hostname for server verification |

## Components

### Makefile.deploy

Main orchestration file that:
- Reads secrets from `.secret/-secret.sh`
- Validates required variables
- Exports Terraform variables (`TF_VAR_*`)
- Runs deployment targets in sequence

Key targets:
- `deploy` - Full deployment from host machine
- `deployment_orchestration` - Runs inside deploy container
- `build_image` / `push_image` - Docker image management
- `gc_setup` - Google Cloud authentication and setup
- `artifact_repo_create` - Creates GAR repository
- `deploy_dev` / `deploy_staging` / `deploy_production` - Mode-specific deployment

### Dockerfile.deploy

Container with deployment tools:
- Google Cloud SDK
- [Terraform 1.14.4](https://releases.hashicorp.com/terraform/1.14.4/)
- Hetzner CLI (hcloud)
- Docker CLI
- Make, jq, curl

### cloud-init.yml

Server initialization script that runs on first boot:
- Configures SSH (key-only auth)
- Installs **k3s** (lightweight Kubernetes)
- Configures registry authentication for Google Artifact Registry
- Sets up KUBECONFIG for kubectl access

### Terraform Modules

#### gar/
Creates Google Artifact Registry repository for Docker images.

#### hetzner_server_create/
Provisions Hetzner Cloud server with:
- Firewall rules (SSH, HTTP, HTTPS)
- Static IPv4 address
- SSH key for access
- Cloud-init for k3s installation

#### service_deploy/
Deploys services to the server:
- Copies `redeploy.sh` and Kubernetes manifests
- Generates environment secrets from variables
- Runs `redeploy.sh` to apply Kubernetes resources

### k8s/

Kubernetes manifests managed via Kustomize:
- **deployment.yaml** - Defines pod replicas, container specs, health probes
- **service.yaml** - ClusterIP service for internal routing
- **ingress.yaml** - Domain-based routing via Traefik ingress
- **kustomization.yaml** - Kustomize configuration for resource aggregation

### redeploy.sh

Runs on the target server to:
- Create Kubernetes namespace (if needed)
- Apply secrets from environment file
- Deploy Kubernetes manifests via `kubectl apply -k`
- Wait for rollout completion with timeout
- **Automatic rollback** on deployment failure

## Rollback Procedures

### Automatic Rollback

Triggered automatically if deployment fails health checks within 120s. The `redeploy.sh` script detects rollout failure and executes `kubectl rollout undo` to restore the previous working version.

### Manual Rollback

```bash
# SSH to server
ssh root@${HOST_SERVER_IP}

# Check current revision history
kubectl rollout history deployment/app -n ${ARTIFACT_REPO_NAME}

# Rollback to previous version
kubectl rollout undo deployment/app -n ${ARTIFACT_REPO_NAME}

# Verify rollback completed
kubectl rollout status deployment/app -n ${ARTIFACT_REPO_NAME} --timeout=120s

# Check pod health
kubectl get pods -n ${ARTIFACT_REPO_NAME} -l app=app
```

## Required Secrets

Create `.secret/-secret.sh` with:

```bash
# Required
GOOGLE_APPLICATION_CREDENTIALS="path/to/service-account.json"
GOOGLE_APPLICATION_PROJECT_ID="your-gcp-project"
SSH_PRIVATE_KEY_PATH="path/to/id_rsa"
SSH_PUBLIC_KEY_PATH="path/to/id_rsa.pub"
HOST_SERVER_NAME="server-hostname"
PROJECT_NAME="project_name"
PROJECT_DOMAIN="example.com"
DEPLOYMENT_MODE="dev"
GOOGLE_ENCRYPTION_KEY="terraform-state-encryption-key"

# Optional
GOOGLE_APPLICATION_REGION="europe-central2"
HOST_SERVER_IP="None"  # Set to IP after first deploy, or "None" to create new server
HOST_SERVER_LOCATION="hel1"
HOST_SERVER_IMAGE="ubuntu-24.04"
HOST_SERVER_TYPE="cx22"
HETZNER_CLOUD_TOKEN="your-hetzner-token"
```

## Terraform State

Remote state (Google Cloud Storage) is used **only by `gar/`** module:
- Bucket: `bucket-{project-name}`
- States are encrypted with `GOOGLE_ENCRYPTION_KEY`
- Prefix: `{mode}/gar/`

The other two modules **intentionally do not persist tfstate**:

| Module | Reason |
|--------|--------|
| `hetzner_server_create/` | Runs once to create the server. The server IP is saved in `HOST_SERVER_IP`, so re-apply is never needed |
| `service_deploy/` | Acts as an orchestration script, not traditional Terraform. Resources are recreated on every run (`triggers_replace = timestamp()`), so previous state is irrelevant |

See individual module readmes for details.

## Usage

```bash
# Full deployment
make -f deploy/Makefile.deploy deploy

# View configured variables
make -f deploy/Makefile.deploy all

# Show state storage info
make -f deploy/Makefile.deploy show_state_info
```
