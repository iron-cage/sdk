# service_deploy Module

## Purpose

Deploys containerized services to existing Hetzner Cloud servers using Terraform provisioners. This module handles the complete deployment lifecycle: waiting for server readiness, copying Kubernetes manifests, and executing the deployment script to apply resources with **zero-downtime rolling updates** and **automatic rollback** on failure.

## Inputs

| Name | Description | Type | Default | Required |
|------|-------------|------|---------|----------|
| `PROJECT_NAME` | Project name used for folder naming and Kubernetes namespace | `string` | - | yes |
| `PROJECT_DOMAIN` | Domain name for ingress configuration (validated format) | `string` | - | yes |
| `HOST_SERVER_IP` | Target server IPv4 address (validated format) | `string` | - | yes |
| `HOST_SERVER_NAME` | Expected hostname for server verification | `string` | - | yes |
| `DEPLOYMENT_MODE` | Deployment environment (`dev`, `staging`, `production`) | `string` | - | yes |
| `PROJECT_FOLDER_NAME` | Custom project folder path override | `string` | `null` | no |
| `PROJECT_MAP_VARIABLES` | Environment variables map for the application (sensitive) | `map(string)` | - | yes |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to GCP service account credentials file | `string` | - | yes |
| `SSH_PRIVATE_KEY_PATH` | Path to SSH private key file for server connection | `string` | - | yes |

## Usage Example

```terraform
module "deploy" {
  source = "./service_deploy"

  PROJECT_NAME       = "myapp"
  PROJECT_DOMAIN     = "myapp.example.com"
  HOST_SERVER_IP     = "1.2.3.4"
  HOST_SERVER_NAME   = "prod-server-1"
  DEPLOYMENT_MODE    = "production"

  PROJECT_MAP_VARIABLES = {
    TAG             = "europe-west1-docker.pkg.dev/myproject/repo/myapp:v1.0.0"
    PROJECT_NAME    = "myapp"
    DEPLOYMENT_MODE = "production"
    DATABASE_URL    = var.database_url
  }

  GOOGLE_APPLICATION_CREDENTIALS = "./credentials/service-account.json"
  SSH_PRIVATE_KEY_PATH           = "~/.ssh/deploy_key"
}
```

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           Terraform Execution                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  1. Wait for cloud-init completion (k3s installation, timeout: 600s)        │
│  2. Verify hostname matches HOST_SERVER_NAME                                │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  3. Create project folder: /opt/services/{PROJECT_NAME}_{DEPLOYMENT_MODE}   │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  4. Copy deployment files via SSH:                                          │
│     ├── redeploy.sh              → Project folder                           │
│     ├── k8s/deployment.yaml      → Project folder/k8s/                      │
│     ├── k8s/service.yaml         → Project folder/k8s/                      │
│     ├── k8s/ingress.yaml         → Project folder/k8s/                      │
│     ├── k8s/kustomization.yaml   → Project folder/k8s/                      │
│     └── secrets/env              → Project folder/secrets/ (from map)       │
└─────────────────────────────────────────────────────────────────────────────┘
                                      │
                                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│  5. Execute redeploy.sh (Kubernetes deployment):                            │
│     ├── Create namespace (if needed)                                        │
│     ├── Apply secrets from env file                                         │
│     ├── Apply k8s manifests via kubectl apply -k                            │
│     ├── Wait for rollout completion (timeout: 120s)                         │
│     └── AUTO ROLLBACK on failure via kubectl rollout undo                   │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Terraform State

This module **does not use a remote backend** (`backend "gcs" {}` is intentionally omitted). The tfstate is not persisted because:

1. **Orchestration script, not infrastructure** — the module uses `terraform_data` with provisioners to copy files and execute scripts over SSH, it does not manage traditional cloud resources
2. **Every run is a fresh deploy** — `triggers_replace = timestamp()` forces recreation on every apply, so previous state is always irrelevant
3. **Idempotent by design** — the underlying Kubernetes deployment (`kubectl apply -k`) and `redeploy.sh` are idempotent, making state tracking unnecessary

This module uses Terraform purely as a deployment orchestration tool, not as an infrastructure state manager.

## Zero-Downtime & Rollback

The deployment uses Kubernetes rolling updates to ensure zero-downtime:

1. **Rolling Update Strategy** - New pods are created before old ones are terminated
2. **Health Checks** - Readiness probes ensure traffic only goes to healthy pods
3. **Multiple Replicas** - 2 replicas ensure at least one pod is always available
4. **Automatic Rollback** - If `kubectl rollout status` fails within 120s, the deployment automatically rolls back to the previous version

## Deployment Triggers

The module redeploys when any of these files change (tracked via MD5 hash):
- `k8s/deployment.yaml`
- `k8s/service.yaml`
- `k8s/ingress.yaml`
- `redeploy.sh`
- `TAG` value in `PROJECT_MAP_VARIABLES`

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `main.tf` | Implement deployment logic using terraform_data resource with SSH provisioners |
| `variables.tf` | Declare and validate input variables for deployment configuration |

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Cloud-init timeout | k3s installation taking too long | Check server logs: `journalctl -u k3s` |
| Hostname mismatch | Server name doesn't match expected | Verify `HOST_SERVER_NAME` matches actual hostname |
| SSH connection failed | Invalid key or IP | Check `SSH_PRIVATE_KEY_PATH` and `HOST_SERVER_IP` |
| Rollout timeout | Pods not becoming ready | Check pod logs: `kubectl logs -n <namespace> <pod>` |
| Rollback triggered | Deployment failed health checks | Check events: `kubectl describe deployment -n <namespace> app` |
| Image pull error | Registry auth issues | Check k3s registry config: `/etc/rancher/k3s/registries.yaml` |
