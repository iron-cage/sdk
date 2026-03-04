# Hetzner Server Create

Terraform configuration for provisioning a Hetzner Cloud server with firewall and static IP, pre-configured with **k3s** (lightweight Kubernetes) for container orchestration.

## Architecture

This Terraform setup provisions a single server running k3s with Traefik ingress controller for routing traffic to Kubernetes deployments.

```
                      Internet
                          │
                          ▼
                   Cloudflare (DNS)
                          │
                          ▼
                   Public Server (1 IP)
                          │
                          ▼
                k3s Cluster (single node)
                          │
                          ▼
                Traefik Ingress Controller
                 ┌────────┴────────┐
                 ▼                 ▼
            iron-site           sdk-app
            (Ingress)           (Ingress)
                 │                 │
                 ▼                 ▼
           Deployment         Deployment
           (2 replicas)       (2 replicas)
```

Each service runs as a Kubernetes Deployment with:
- **Zero-downtime rolling updates**
- **Automatic rollback** on failed deployments
- **Health checks** via readiness/liveness probes
- **Ingress routing** based on domain/path

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `main.tf` | Define Hetzner provider, firewall rules, SSH key, static IP, and server instance |
| `variables.tf` | Declare input variables for server configuration and SSH keys |
| `outputs.tf` | Export server public IPv4 address for use by other modules |

## Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HETZNER_CLOUD_TOKEN` | Hetzner API token (sensitive) | - |
| `HOST_SERVER_NAME` | Name of the server | - |
| `HOST_SERVER_TYPE` | Server type/size | `cx23` |
| `HOST_SERVER_IMAGE` | OS image | `ubuntu-24.04` |
| `HOST_SERVER_LOCATION` | Server location | `hel1` |
| `SSH_PUBLIC_KEY_PATH` | Path to SSH public key file | - |
| `GOOGLE_APPLICATION_REGION` | GCP region for cloud-init configuration | - |
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to GCP service account credentials file | - |

## Terraform State

This module **does not use a remote backend** (`backend "gcs" {}` is intentionally omitted). The tfstate is not persisted because:

1. **One-time execution** — the module runs only when a new server needs to be created (when `HOST_SERVER_IP` is set to `None`)
2. **IP saved externally** — after creation, the server's public IP is stored in the `HOST_SERVER_IP` variable, which is the only output needed by downstream modules
3. **No re-apply needed** — once the server exists and its IP is known, this module is never called again, making stored state unnecessary

This is by design: the server lifecycle is managed through the Hetzner Cloud API/console, not through repeated `terraform apply` runs.

## Resources Created

- **Firewall** - Opens ports 22 (SSH), 80 (HTTP), 443 (HTTPS), 6443 (k8s API)
- **SSH Key** - Uploads public key for deployment access
- **Primary IP** - Static IPv4 address for the server
- **Server** - Hetzner Cloud instance with cloud-init provisioning (k3s installation)

## Server Initialization

The cloud-init script (`cloud-init.yml`) automatically:
1. Installs k3s (lightweight Kubernetes distribution)
2. Configures kubectl access via KUBECONFIG
3. Sets up registry authentication for Google Artifact Registry
4. Configures Traefik as the default ingress controller

## Limitations

| Limitation | Impact | Mitigation |
|------------|--------|------------|
| Single-node cluster | Server failure = complete outage (no HA) | Suitable for dev/staging; consider multi-node for production |
| No etcd backup | Data loss on disk failure | Implement periodic etcd snapshots for critical workloads |
| Single region | Regional outage affects service | Acceptable for non-critical services |

For production high-availability, consider:
- Multi-master k3s with external etcd
- Load balancer in front of multiple nodes
- Cross-region failover

## Server Access

### SSH to Server

```bash
ssh root@${HOST_SERVER_IP}
```

### Access Pods

```bash
# List all namespaces
kubectl get namespaces

# List all pods in namespace
kubectl get pods -n ${ARTIFACT_REPO_NAME}

# Execute shell in a pod
kubectl exec -it <pod-name> -n ${ARTIFACT_REPO_NAME} -- /bin/sh

# View pod logs
kubectl logs <pod-name> -n ${ARTIFACT_REPO_NAME}

# Follow logs in real-time
kubectl logs -f <pod-name> -n ${ARTIFACT_REPO_NAME}

# Port-forward to access pod locally
kubectl port-forward <pod-name> 8080:80 -n ${ARTIFACT_REPO_NAME}
```
