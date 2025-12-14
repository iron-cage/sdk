# Hetzner Cloud Infrastructure

This repository contains Terraform configuration for provisioning and managing infrastructure on [Hetzner Cloud](https://www.hetzner.com/cloud).

> ⚠️ This project assumes you already have a Hetzner Cloud account and an API token.

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

- [Terraform](https://learn.hashicorp.com/tutorials/terraform/install-cli) installed  
  (see `versions.tf` for the exact required version).
- A Hetzner Cloud account.
- A Hetzner Cloud **API Token** with sufficient permissions.

## Authentication / Configuration

The Hetzner Terraform provider requires an API token for authentication.

This repository expects the token to be set via a shell script at `./secret/-secret.sh`:

```sh
SECRET_HETZNER_CLOUD_TOKEN="YOUR_HETZNER_API_TOKEN"
```

Typical usage:

```sh
# 1. Create and edit the secret file
nano secret/-secret.sh   # or use your preferred editor
```

Make sure that:

- The `secret` directory (and `-secret.sh`) is **ignored by git** so your token is never committed.
- The script exports or sets whatever variable your Terraform configuration expects  
  (for example, you might export `HCLOUD_TOKEN` or pass the value into a Terraform variable).

You can also provide the token via environment variables or Terraform variables directly, depending on how your provider block is configured.

## Getting Started

1. **Clone the repository**

   ```sh
   git clone <YOUR_REPO_URL>.git
   cd <YOUR_REPO_DIR>
   ```

2. **Configure your API token**

   Set up `./secret/-secret.sh` as described above and `source` it, or configure your token/variables in another secure way.

3. **Initialize Terraform**

   Initialize the working directory and download the required providers and modules:

   ```sh
   terraform init
   ```

4. **Review the execution plan**

   See what Terraform intends to create, change, or destroy:

   ```sh
   terraform plan
   ```

5. **Apply the changes**

   Create or update the infrastructure:

   ```sh
   terraform apply
   ```

   Terraform will show you the plan again and ask for confirmation before applying.

6. **Destroy the infrastructure (optional)**

   To remove all resources created by this configuration:

   ```sh
   terraform destroy
   ```

   Terraform will prompt for confirmation.

## Configuration & Variables

Key configuration options (defined in `variables.tf`, if present) typically include:

- Hetzner API token
- Server type (e.g. `cx11`, `cx21`, …)
- Location / datacenter
- Image (e.g. Ubuntu version)
- SSH keys and other infrastructure parameters

Example variable documentation (adjust to match your actual variables):

| Variable           | Description                             | Default         |
|------------------- |-----------------------------------------|-----------------|
| `hcloud_token`     | Hetzner Cloud API token                 | _no default_    |
| `server_type`      | Hetzner server type                     | `cx23`          |
| `datacenter`       | Hetzner location (e.g. `hel1-dc2`)      | `hel1-dc2`      |
| `image`            | Image of the OS (e.g. `ubuntu-22.04`)   | `ubuntu-22.04`  |


## Outputs

The following outputs are defined in `outputs.tf`:

| Name         | Description                          |
|--------------|--------------------------------------|
| `ipv4`       | The public IP address of the server. |

You can view the output values with:

```sh
terraform output
# or:
terraform output ipv4
```

You can then use `ipv4` to connect to the server (for example via SSH), or integrate it into other tools.

## Security Notes

- **Never commit secrets** (API tokens, private keys, etc.) to the repository.
- Keep `secret/` and any other secret files in your `.gitignore`.
- Rotate your Hetzner API token if you suspect it has been exposed.
