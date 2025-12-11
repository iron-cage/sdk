# Google Artifact Registry (GAR) Terraform Infrastructure

This repository contains Terraform configuration for provisioning a **Google Artifact Registry (GAR) repository** to store **Docker container images** (for example, images used by your GCE deployments).

> ⚠️ This project assumes you already have a Google Cloud project, valid credentials, and (optionally) an existing GCS bucket for the Terraform backend.

---

## Prerequisites

Before you start, make sure you have:

- [Terraform](https://developer.hashicorp.com/terraform/downloads) installed  
  (see `versions.tf` in your repo if you add a specific version constraint).
- A **Google Cloud project** where the Artifact Registry repository will be created.
- A **service account key file** with permissions to manage Artifact Registry and (optionally) GCS, for example:
  - `roles/artifactregistry.admin` (or a narrower custom role),
  - `roles/storage.admin` or similar, if you manage GCS buckets with the same account.

---

## Authentication / Configuration

The Terraform **Google provider** in `main.tf` looks like this:

```hcl
provider "google" {
  project     = var.PROJECT_ID
  region      = var.REGION
  credentials = file(var.GOOGLE_SE_CREDS_PATH)
}
```

This means:

- The **project ID** and **region** are taken from Terraform variables `PROJECT_ID` and `REGION`.
- Credentials are read from a **service account JSON key file** whose path is provided via `GOOGLE_SE_CREDS_PATH`.

A convenient way to configure your environment is a shell script at `./.secret/-secret.sh`, for example:

```sh
# Example: ./.secret/-secret.sh

# Path to your service account JSON key
TF_VAR_GOOGLE_SE_CREDS_PATH="$HOME/.config/gcloud/tf-sa.json"

# Terraform variables
TF_VAR_PROJECT_ID="your-gcp-project-id" # Can be load from google json keys
TF_VAR_REGION="europe-central2"            # example region
TF_VAR_REPO_NAME="demo-images"       # Artifact Registry repo name
TF_VAR_BUCKET_NAME="my-tfstate-bucket"     # GCS bucket for Terraform state (backend)
```

Typical usage:

```sh
# 1. Create and edit the secret file
nano .secret/-secret.sh   # or use your preferred editor

# 2. Load the environment
source .secret/-secret.sh
```

Make sure that:

- The `.secret` directory (and `-secret.sh`) is **ignored by git** so keys and IDs are never committed.
- The script exports all variables your Terraform configuration expects:
  - `TF_VAR_PROJECT_ID`
  - `TF_VAR_REGION`
  - `TF_VAR_REPO_NAME`
  - `TF_VAR_BUCKET_NAME` (if you use a GCS backend)
  - `TF_VAR_GOOGLE_SE_CREDS_PATH`

You **do not need** to set `GOOGLE_APPLICATION_CREDENTIALS` for this module, because the provider reads the key from `GOOGLE_SE_CREDS_PATH` explicitly, but it won’t hurt if you also set it.

---

## Terraform Backend

The configuration defines a Terraform backend:

```hcl
terraform {
  backend "gcs" {}
}
```

Backend details (like the bucket name and prefix) are provided when you run **`terraform init`**, for example:

```sh
terraform init   -backend-config="bucket=${TF_VAR_BUCKET_NAME}"   -backend-config="prefix=gar/state"
```

> 💡 You can reuse the same **GCS backend bucket** that you created in your GCS or GCE Terraform projects.

If you prefer **local state**, you can remove or adjust the backend block in `main.tf`.

---

## What This Configuration Creates

From `main.tf`:

```hcl
resource "google_artifact_registry_repository" "container-images-repo" {
  # Location for the repository
  location      = var.REGION
  project       = var.PROJECT_ID
  repository_id = var.REPO_NAME
  description   = "Docker image registry for the deployments"
  # Format of the repository. We are using Docker.
  format        = "DOCKER"
}
```

This resource:

- Creates an **Artifact Registry repository** in the specified **region** and **project**.
- Names the repository using `REPO_NAME` (e.g. `demo-images`).
- Sets the repository format to **DOCKER**, so it can store Docker container images.

You can then **push images** to this repository and pull them from other modules (like your GCE Terraform project) using standard Artifact Registry URLs.

---

## Getting Started

1. **Clone the repository**

   ```sh
   git clone <YOUR_REPO_URL>.git
   cd <YOUR_REPO_DIR>
   ```

2. **Configure credentials and variables**

   Create `./.secret/-secret.sh` as shown above and load it:

   ```sh
   source .secret/-secret.sh
   ```

3. **Initialize Terraform**

   Initialize the working directory, providers, and backend:

   ```sh
   terraform init -backend-config="bucket=${TF_VAR_BUCKET_NAME}" -backend-config="prefix=gar/state"
   ```

4. **Review the execution plan**

   See what Terraform intends to create or change:

   ```sh
   terraform plan
   ```

5. **Apply the changes**

   Create the Artifact Registry repository:

   ```sh
   terraform apply
   ```

   Terraform will show the plan again and ask for confirmation before applying.

6. **Destroy the infrastructure (optional)**

   To remove the repository created by this configuration:

   ```sh
   terraform destroy
   ```

   ⚠️ Make sure you understand the impact of deleting an Artifact Registry repository (images stored there will be removed). Terraform will prompt for confirmation.

---

## Configuration & Variables

From `variables.tf` the key variables are:

```hcl
# Specifies region location that will be used for all recources
variable "REGION" {
  description = "Region of the resources"
}

# Project id where all resources will be created 
variable "PROJECT_ID" {
  description = "Project id for the resources"
}

# Artifact Registry repository name  
variable "REPO_NAME" {
  description = "Artifact registry name"
}

# Name of the bucket that will be created 
variable "BUCKET_NAME" {
  description = "name for the bucket to be created"
}

# Path to the service account key file
variable "GOOGLE_SE_CREDS_PATH" {
  description = "Path to the service account key file"
  type        = string
}
```

### Variable Reference

| Variable              | Description                                                                 | Default       |
|---------------------- |-----------------------------------------------------------------------------|--------------|
| `PROJECT_ID`          | Google Cloud project ID for all resources                                   | _no default_ |
| `REGION`              | Region where the Artifact Registry repository will be created               | _no default_ |
| `REPO_NAME`           | Artifact Registry repository ID (e.g. `demo-images`)                  | _no default_ |
| `BUCKET_NAME`         | Name of the GCS bucket used for Terraform state or other related resources  | _no default_ |
| `GOOGLE_SE_CREDS_PATH`| Local filesystem path to the service account JSON key file                  | _no default_ |

You can set these:

- via `TF_VAR_...` environment variables (as in `.secret/-secret.sh`), or  
- via `terraform.tfvars`, or  
- via `-var` flags on the command line.

---

## Outputs

The following output is defined in `outputs.tf`:

```hcl
output "repo_name" {
  description = "Name of the Artifact Registry"
  value       = google_artifact_registry_repository.container-images-repo.name
}
```

### Output Reference

| Name        | Description                                       |
|------------ |---------------------------------------------------|
| `repo_name` | The name/ID of the created Artifact Registry repo |

You can view the output value with:

```sh
terraform output
# or:
terraform output repo_name
```

This is useful for verifying the created repository or passing it to other tools/modules.

---

## Security Notes

- **Never commit secrets** (service account keys, JSON files, etc.) to the repository.
- Keep `.secret/` and any other sensitive files in your `.gitignore`.
- Use minimally-privileged service accounts (only the permissions your Artifact Registry / backend actually need).
- Rotate service account keys / credentials if you suspect they have been exposed.
- Be careful when granting access to the Artifact Registry repository, especially if it hosts production images.
