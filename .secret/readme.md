# Configuration Reference for `.secret/-secret.sh`

This document explains how to configure the `.secret/-secret.sh` file used by the root **Makefile** to deploy the application to cloud providers (Hetzner) with a single command:

```bash
make deploy
```

The `.secret/-secret.sh` file contains **all secrets and environment-specific settings**.  
It is sourced by the Makefile before starting the Docker-based Terraform environment, and its variables are passed into all Terraform modules and helper scripts.

---

## Content

- [Recommended `.secret/` Layout](#recommended-secret-layout)
- [Required Variables](#required-variables)
    - [1. Project & cloud provider](#1-project--cloud-provider)
    - [2. Archive / state key](#2-archive--state-key)
    - [3. GCP service account](#3-gcp-service-account)
    - [4. SSH keys](#4-ssh-keys)
    - [5. Provider-specific secrets](#5-provider-specific-secrets)
- [Optional Variables & Defaults](#optional-variables--defaults)
    - [1. GCP / Terraform parameters](#1-gcp--terraform-parameters)
    - [2. Docker / Artifact Registry](#2-docker--artifact-registry)
    - [3. Terraform modules root](#3-terraform-modules-root)
- [Minimal Configuration Examples](#minimal-configuration-examples)
- [Retrieving Keys](#retrieving-keys)
    - [How to get `service_account.json` (GCP service account key)](#how-to-get-service_accountjson-gcp-service-account-key)
    - [How to get `SECRET_STATE_ARCHIVE_KEY`](#how-to-get-secret_state_archive_key)
    - [How to get `SECRET_HETZNER_CLOUD_TOKEN` (Hetzner API token)](#how-to-get-secret_hetzner_cloud_token-hetzner-api-token)

---

## Recommended `.secret/` Layout

Example structure:

```text
.secret/
  |- -secret.sh                 # main secrets file (NOT committed)
  |- readme.md                  # this documentation file
  |- service_account.json       # GCP service account key (name is configurable)
  |- id_rsa                     # SSH private key
  |- id_rsa.pub                 # SSH public key
```

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

These **must** be set, otherwise `make deploy` will fail during the environment check.

### 1. Project & cloud provider

| Variable       | Required | Description                                                                             |
|----------------|----------|-----------------------------------------------------------------------------------------|
| `PROJECT_NAME` | Yes      | Logical project name (no spaces). Used for tags, defaults, repo names, etc.           |
| `CSP`          | Yes      | Selected cloud provider: `hetzner`                                    |

### 2. Archive / state key

| Variable                   | Required | Description                                                                                   |
|----------------------------|----------|-----------------------------------------------------------------------------------------------|
| `SECRET_STATE_ARCHIVE_KEY` | Yes      | Secret key used for encrypting archives / backups. Use a strong, random value                |

### 3. GCP service account

| Variable               | Required | Description                                                                                  |
|------------------------|----------|----------------------------------------------------------------------------------------------|
| `GOOGLE_SE_CREDS_PATH` | Yes      | Path to the GCP service account JSON key. Used for GCS backend, Artifact Registry, etc.     |

> Even if you deploy to Hetzner, this variable is required because Terraform state is stored in **GCS**.

### 4. SSH keys

| Variable                        | Required | Description                                                                                   |
|---------------------------------|----------|-----------------------------------------------------------------------------------------------|
| `SECRET_RSA_PRIVATE_KEY_PATH`   | Yes      | Path to the **private** SSH key, used by Terraform to connect to VMs                          |
| `SECRET_RSA_PUBLIC_KEY_PATH`    | Yes      | Path to the **public** SSH key, injected into the VM / cloud provider key pair configuration  |

### 5. Provider-specific secrets

Depending on the chosen `CSP`, some additional variables are required.

#### For Hetzner (`CSP="hetzner"`)

| Variable                 | Required | Description               |
|--------------------------|----------|---------------------------|
| `SECRET_HETZNER_CLOUD_TOKEN`     | Yes      | Hetzner Cloud API Token   |

---

## Optional Variables & Defaults

These variables are optional. If omitted, the Makefile / scripts will derive **sensible defaults** where possible.

### 1. GCP / Terraform parameters

| Variable             | Type     | Default / Behaviour                                                                         |
|----------------------|----------|---------------------------------------------------------------------------------------------|
| `TF_VAR_PROJECT_ID`  | Optional | If not set, taken from `.project_id` inside `GOOGLE_SE_CREDS_PATH` (service account JSON)  |
| `TF_VAR_REGION`      | Optional | Defaults to `europe-central2`                                                              |
| `TF_VAR_ZONE`        | Optional | Defaults to `<region>-a` (e.g. `europe-central2-a`)                                       |
| `TF_VAR_BUCKET_NAME` | Optional | Defaults to `bucket-<repo-name>`; must be globally unique in GCS and should not contain `_` |

### 2. Docker / Artifact Registry

| Variable             | Type     | Default / Behaviour                                                                         |
|----------------------|----------|---------------------------------------------------------------------------------------------|
| `TF_VAR_REPO_NAME`   | Optional | Defaults to `PROJECT_NAME` with `_` replaced by `-`                                        |
| `TF_VAR_IMAGE_NAME`  | Optional | Defaults to `PROJECT_NAME`                                                                 |
| `TAG`                | Optional | If not set, Makefile constructs a tag from `REGION`, `PROJECT_ID`, `REPO_NAME`, `IMAGE_NAME` |

### 3. Terraform modules root

| Variable  | Type     | Default / Behaviour                      |
|---------- |----------|-------------------------------------------|
| `TF_DIR`  | Optional | Base directory for Terraform modules; default is `deploy` |

---

## Minimal Configuration Examples

- **Minimal `.secret/-secret.sh`**

```bash
# Cloud provider:  hetzner
CSP="hetzner"
SECRET_HETZNER_CLOUD_TOKEN="YOUR_HETZNER_API_TOKEN"
# Project name (for tags and defaults)
PROJECT_NAME="project_name"
# Required secrets
SECRET_STATE_ARCHIVE_KEY="change-me-random-string"
GOOGLE_SE_CREDS_PATH=".secret/service_account.json"
SECRET_RSA_PRIVATE_KEY_PATH=".secret/id_rsa"
SECRET_RSA_PUBLIC_KEY_PATH=".secret/id_rsa.pub"

# Optional (can be miss) overrides
# Google cloud region
TF_VAR_REGION="europe-central2"
# Project id for deployed resources | Can be set in .secret/-secret.sh
TF_VAR_PROJECT_ID=
# Artifact Repository name for pushing the Docker images | Should not have "_"
TF_VAR_REPO_NAME=
# Pushed image name | Can have "_"
TF_VAR_IMAGE_NAME=
# Helper var for tagging local image
TAG=
# Zone location for the resource
TF_VAR_ZONE=
# Cloud Storage bucket name | must be unique | Should not have "_"
TF_VAR_BUCKET_NAME=
# Base terraform directory
TF_DIR=
```

---

## Retrieving Keys

This section explains **how to obtain all required keys** used in `.secret/-secret.sh`.

### How to get `service_account.json` (GCP service account key)

You can place your GCP service account key file under `.secret/` and point `GOOGLE_SE_CREDS_PATH` to it (for example: `.secret/service_account.json`).

Steps to create a key in the GCP Console:

1. Open: <https://console.cloud.google.com/iam-admin/serviceaccounts>
2. Choose or create a Service Account.
3. Go to **Keys** → **Add Key** → **Create new key**.
4. Select **JSON** and click **Create**.

Save the downloaded JSON file into `.secret/` and update `GOOGLE_SE_CREDS_PATH` accordingly.

The actual filename does not matter, as long as `GOOGLE_SE_CREDS_PATH` matches it.

### How to get `SECRET_STATE_ARCHIVE_KEY`

You can generate this key in many ways. It should be a sufficiently long, random string.

Some inspiration and background on keys used with GCP can be found here:  
<https://cloud.google.com/storage/docs/encryption/using-customer-supplied-keys>

Examples of simple generation approaches:

```bash
# Using OpenSSL (hex)
openssl rand -hex 32

# Using /dev/urandom (base64)
head -c 32 /dev/urandom | base64
```

Take the generated value and set it as `SECRET_STATE_ARCHIVE_KEY` in `.secret/-secret.sh`.

### How to get `SECRET_HETZNER_CLOUD_TOKEN` (Hetzner API token)

This key is retrieved from your **Hetzner Cloud Console**:

1. Open the Hetzner Cloud Console.
2. Go to **Security** → **API Tokens**.
3. Click **Generate API Token**.
4. Fill in a description.
5. Select **Read & Write** access (needed to create and manage instances).
6. Create the token and copy it.

Paste the token value into your `.secret/-secret.sh` as `SECRET_HETZNER_CLOUD_TOKEN` (or the exact variable name used in your Makefile).
