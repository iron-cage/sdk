# Configuration Reference for `.secret/`

This document explains how to configure the `.secret/` directory used by the **Makefile** to deploy the application to Hetzner Cloud with **Kubernetes (k3s)**.

```bash
make deploy
```

The `.secret/-secret.sh` file contains **all secrets and environment-specific settings**.
It is sourced by the Makefile before starting the Docker-based Terraform environment, and its variables are passed into all Terraform modules, helper scripts, and converted to **Kubernetes secrets** on the target server.

---

## Content

- [Directory Layout](#directory-layout)
- [Required Variables](#required-variables)
- [Optional Variables](#optional-variables)
- [Retrieving Keys](#retrieving-keys)

---

## Directory Layout

```text
.secret/
  |- -secret.sh                 # main secrets file (NOT committed, copy from template)
  |- secret.template.sh         # template file with all variables
  |- env.dev.tfvars             # Terraform variables for dev deployment
  |- readme.md                  # this documentation file
  |- -service_account.json      # GCP service account key
  |- -id_ed25519                # SSH private key (ed25519)
  |- -id_ed25519.pub            # SSH public key (ed25519)
  |- .gitignore                 # ignores sensitive files
```

Files prefixed with `-` are ignored by git (e.g., `-secret.sh`, `-id_ed25519`).

The `-secret.sh` file is a **regular bash script**, composed of lines like:

```bash
KEY="value"
ANOTHER_KEY=value
```

The Makefile essentially does:

```bash
source .secret/-secret.sh
```

and then uses the exported variables.

---

## Required Variables

### Google Credentials

| Variable | Description | How to generate |
|----------|-------------|-----------------|
| `GOOGLE_APPLICATION_CREDENTIALS` | Path to GCP service account JSON | - |
| `GOOGLE_APPLICATION_PROJECT_ID` | GCP project ID | From GCP Console |
| `GOOGLE_APPLICATION_REGION` | GCP region | Default: `europe-central2` |
| `GOOGLE_ENCRYPTION_KEY` | Encryption key for state/backups | `openssl rand -base64 32` |

### SSH Keys

| Variable | Description |
|----------|-------------|
| `SSH_PRIVATE_KEY_PATH` | Path to SSH private key |
| `SSH_PUBLIC_KEY_PATH` | Path to SSH public key |

> Generate SSH keys with: `ssh-keygen -t ed25519 -f .secret/-id_ed25519 -C "deploy_key"`

### Host Server

| Variable | Description |
|----------|-------------|
| `HOST_SERVER_NAME` | Server hostname |
| `HOST_SERVER_IP` | Server IP address (set after creation or use existing) |

### Hetzner Credentials

| Variable | Description | How to get |
|----------|-------------|------------|
| `HETZNER_CLOUD_TOKEN` | Hetzner Cloud API token | [Hetzner Docs](https://docs.hetzner.com/cloud/api/getting-started/generating-api-token/) |

### Deployment Variables

| Variable | Description |
|----------|-------------|
| `DEPLOYMENT_MODE` | Deployment mode: `dev`, `staging`, `production` |

### Project Variables

| Variable | Description |
|----------|-------------|
| `PROJECT_NAME` | Project name (no `-` or spaces), used as Kubernetes namespace |
| `PROJECT_DOMAIN` | Domain for Kubernetes ingress routing |
| `PROJECT_CERT_EMAIL` | Email for TLS sertificate |

### Frontend Variables

| Variable | Description |
|----------|-------------|
| `FRONTEND_BASE_URL` | Backend API base URL (`/api` or full URL) |

---

## Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST_SERVER_LOCATION` | Hetzner datacenter location | `hel1` |
| `HOST_SERVER_IMAGE` | Server OS image | `ubuntu-24.04` |
| `HOST_SERVER_TYPE` | Server type/size | `cx33` |
| `ALLOWED_SSH_IPS` | Semicolon-separated list of allowed SSH IP/CIDR ranges | `0.0.0.0/0;::/0` |

---

## Retrieving Keys

### How to get `service_account.json` (GCP service account key)

Place your GCP service account key file under `.secret/` and point `GOOGLE_APPLICATION_CREDENTIALS` to it.

Steps to create a key in the GCP Console:

1. Open: <https://console.cloud.google.com/iam-admin/serviceaccounts>
2. Choose or create a Service Account.
3. Go to **Keys** → **Add Key** → **Create new key**.
4. Select **JSON** and click **Create**.

Save the downloaded JSON file as `.secret/-service_account.json`.

### How to get `GOOGLE_ENCRYPTION_KEY`

Generate with OpenSSL:

```bash
openssl rand -base64 32
```

### How to get `HETZNER_CLOUD_TOKEN`

This key is retrieved from your **Hetzner Cloud Console**:

1. Open the Hetzner Cloud Console.
2. Go to **Security** → **API Tokens**.
3. Click **Generate API Token**.
4. Fill in a description.
5. Select **Read & Write** access.
6. Create the token and copy it.

---

## How Secrets Are Used in Kubernetes

During deployment, the variables from `-secret.sh` are:

1. **Exported as Terraform variables** (`TF_VAR_*`) for infrastructure provisioning
2. **Copied to the server** as an environment file (`secrets/env`)
3. **Applied as Kubernetes secrets** via:

```bash
kubectl create secret generic app-env \
  --from-env-file=secrets/env \
  --dry-run=client -o yaml | kubectl apply -n "${NAMESPACE}" -f -
```

This allows pods to access secrets as environment variables without storing them in version control or Kubernetes manifests.
