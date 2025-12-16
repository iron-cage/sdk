terraform {
  # Specifies terraform API provider to use for `hcloud`
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "1.45.0"
    }
  }

  backend "gcs" {}
}

# Configures hcloud provider for deploy
provider "hcloud" {
  # Hetzner API token 
  token = var.HETZNER_CLOUD_TOKEN
}

# Creates an SSH key used for redeploy
resource "hcloud_ssh_key" "master_key" {
  name       = "${var.PROJECT_NAME}-ssh-key"
  public_key = data.local_sensitive_file.ssh_public_key.content
}

# Static IP for the instance
resource "hcloud_primary_ip" "primary_ip" {
  name          = "${var.PROJECT_NAME}-ip"
  datacenter    = "hel1-dc2"
  type          = "ipv4"
  assignee_type = "server"
  auto_delete   = false
}

# Hetzner instance itself
resource "hcloud_server" "main_resource" {
  name        = "${var.REPO_NAME}"
  image       = "ubuntu-22.04"
  server_type = "cx23"
  datacenter  = "hel1-dc2"

  public_net {
    ipv4_enabled = true
    ipv4         = hcloud_primary_ip.primary_ip.id
    ipv6_enabled = false
  }

  ssh_keys = [hcloud_ssh_key.master_key.name]

  # Startup script for the instance
  # Installs docker, gcloud CLI, downloads docker images and starts the container
  user_data = templatefile("${path.module}/../cloud-init.tpl", {
    MASTER_SSH_KEY            = "${hcloud_ssh_key.master_key.name}"
    location                  = "${var.REGION}"
    project_id                = "${var.PROJECT_ID}"
    repo_name                 = "${var.REPO_NAME}"
    image_name                = "${var.IMAGE_NAME}"
    jwt_secret                = "${var.JWT_SECRET}"
    iron_secrets_master_key   = "${var.IRON_SECRETS_MASTER_KEY}"
    database_url              = "${var.DATABASE_URL}"
    tag                       = "${var.TAG}"
    ip_token_key              = "${var.IP_TOKEN_KEY}"
    ic_token_key              = "${var.IC_TOKEN_SECRET}"
    iron_deployment_mode      = "${var.IRON_DEPLOYMENT_MODE}"
    allowed_origins           = "${var.ALLOWED_ORIGINS}"
    server_port               = "${var.SERVER_PORT}"
    enable_demo_seed          = "${var.ENABLE_DEMO_SEED}"
    service_account_creds_b64 = "${base64encode(data.local_sensitive_file.service_account_creds.content)}"
  })
}

resource "terraform_data" "redeploy" {
  triggers_replace = timestamp()
  depends_on       = [hcloud_server.main_resource]

  connection {
    type        = "ssh"
    user        = "root"
    private_key = data.local_sensitive_file.ssh_private_key.content
    host        = hcloud_primary_ip.primary_ip.ip_address
  }

  provisioner "file" {
    source      = "${path.module}/../redeploy.sh"
    destination = "/deploy/redeploy.sh"
  }

  provisioner "remote-exec" {
    inline = [
      # Wait cloud-init to finish
      "bash -lc 'command -v cloud-init >/dev/null 2>&1 && timeout 30m cloud-init status --wait || true'",
      # Run /tmp/redeploy.sh script
      "bash -lc 'set -a; source /deploy/.secret; set +a; chmod +x /deploy/redeploy.sh; /deploy/redeploy.sh' ",
    ]
  }
}
