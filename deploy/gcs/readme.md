# Google Cloud Storage (GCS) Terraform Backend

This repository contains Terraform configuration for provisioning a **Google Cloud Storage (GCS) bucket** that can be used as a **remote backend for Terraform state**.

> ⚠️ This project assumes you already have a Google Cloud project and valid Google Cloud credentials.

---

## Directory Contents

| File | Responsibility |
|---|---|
| `main.tf` | Defines the Hetzner cloud server (`hcloud_server`), SSH key, and other related resources. |
| `variables.tf` | Declares input variables for the module, such as server type, image, and SSH keys. |
| `outputs.tf` | Defines module outputs, primarily the public IPv4 address of the created server. |
| `versions.tf` | Specifies the required version of Terraform and the Hetzner Provider. |

---

## Prerequisites

Before you start, make sure you have:

- [Terraform](https://developer.hashicorp.com/terraform/downloads) installed  
  (see `versions.tf` if you add one with a specific version constraint).
- A **Google Cloud project** where the bucket will be created.
- A way to authenticate to Google Cloud, for example:
  - `gcloud` CLI with Application Default Credentials, or
  - A **service account key** with permissions to manage GCS buckets (e.g. `roles/storage.admin`).

---

## Authentication / Configuration

The Terraform **Google provider** in `main.tf` looks like this:

```hcl
provider "google" {
  project = var.PROJECT_ID
}
```

This means:

- The **project ID** is passed via the `PROJECT_ID` variable.
- Credentials are picked up from your environment (Application Default Credentials, `GOOGLE_APPLICATION_CREDENTIALS`, etc.).

This repository assumes you configure your environment via a shell script at `./secret/-secret.sh`:

```sh
# Example: ./secret/-secret.sh

# Path to your service account JSON key (or rely on gcloud ADC instead)
GOOGLE_APPLICATION_CREDENTIALS="$HOME/.config/gcloud/tf-sa.json"

# Terraform variables
TF_VAR_PROJECT_ID="your-gcp-project-id"
TF_VAR_REGION="europe-central2"        # example region
TF_VAR_BUCKET_NAME="my-tfstate-bucket" # must be globally unique
```

Typical usage:

```sh
# 1. Create and edit the secret file
nano secret/-secret.sh   # or use your preferred editor
```

Make sure that:

- The `secret` directory (and `-secret.sh`) is **ignored by git** so keys and IDs are never committed.
- The script exports whatever variables your Terraform configuration expects:
  - `TF_VAR_PROJECT_ID`
  - `TF_VAR_REGION`
  - `TF_VAR_BUCKET_NAME`
  - and optionally `GOOGLE_APPLICATION_CREDENTIALS` if you use a service account file.

---

## Terraform Backend

The configuration also defines a Terraform backend:

```hcl
terraform {
  backend "gcs" {}
}
```

Backend details (like the bucket name and prefix) are typically provided via **`terraform init`** flags or a backend configuration file, for example:

```sh
terraform init   -backend-config="bucket=${TF_VAR_BUCKET_NAME}"   -backend-config="prefix=terraform/state"
```

> 💡 You can use this bucket as a shared remote state backend for other Terraform projects as well.

---

## Getting Started

1. **Clone the repository**

   ```sh
   git clone <YOUR_REPO_URL>.git
   cd <YOUR_REPO_DIR>
   ```

2. **Configure credentials and variables**

   Create `./secret/-secret.sh` as shown above and load it:

   ```sh
   source secret/-secret.sh
   ```

3. **Initialize Terraform**

   Initialize the working directory, providers, and backend:

   ```sh
   terraform init      -backend-config="bucket=${TF_VAR_BUCKET_NAME}"      -backend-config="prefix=terraform/state"
   ```

4. **Review the execution plan**

   See what Terraform intends to create or change:

   ```sh
   terraform plan
   ```

5. **Apply the changes**

   Create the GCS bucket:

   ```sh
   terraform apply
   ```

   Terraform will show the plan again and ask for confirmation before applying.

6. **Destroy the infrastructure (optional)**

   To remove the GCS bucket:

   ```sh
   terraform destroy
   ```

   Terraform will prompt for confirmation before deleting the bucket (and its contents, because `force_destroy = true`).

---

## Configuration & Variables

The key configuration options defined in `main.tf` are:

```hcl
variable "BUCKET_NAME" {
  description = "name for the bucket to be created"
}

variable "REGION" {
  description = "region of the resources"
}

variable "PROJECT_ID" {
  description = "project id for the resources"
}
```

### Variable Reference

| Variable      | Description                                       | Default       |
|-------------- |---------------------------------------------------|--------------|
| `PROJECT_ID`  | Google Cloud project ID for all resources         | _no default_ |
| `REGION`      | Region where the GCS bucket will be created       | _no default_ |
| `BUCKET_NAME` | Name of the GCS bucket (must be globally unique)  | _no default_ |

You can set these:

- via `TF_VAR_...` environment variables (as in `secret/-secret.sh`), or  
- via `terraform.tfvars`, or  
- via `-var` flags on the command line.

---

## Outputs

This minimal configuration does **not** define any outputs yet.

If you want to expose useful information from this module, you can add an `outputs.tf` file with values like:

- Bucket name
- Bucket URL
- Backend configuration hints

Example:

```hcl
output "bucket_name" {
  value       = google_storage_bucket.tfstate-storage.name
  description = "Name of the Terraform state bucket"
}
```

You would then see the value with:

```sh
terraform output
# or
terraform output bucket_name
```

---

## Security Notes

- **Never commit secrets** (service account keys, `GOOGLE_APPLICATION_CREDENTIALS` paths, etc.) to the repository.
- Keep `secret/` and any other sensitive files in your `.gitignore`.
- Prefer using minimally-privileged service accounts (just enough permissions to manage the bucket).
- Rotate service account keys / credentials if you suspect they have been exposed.
