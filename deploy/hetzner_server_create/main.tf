terraform {
  required_version = ">= 1.0"

  # No backend "gcs" {} — tfstate is intentionally not persisted.
  # This module runs only once to create the server. After creation,
  # the server IP is saved externally (HOST_SERVER_IP variable),
  # so subsequent deployments skip this module entirely.
  # See readme.md "Terraform State" section for details.

  # Specifies terraform API provider to use for `hcloud`
  required_providers {
    hcloud = {
      source  = "hetznercloud/hcloud"
      version = "1.60.0"
    }
  }
}

# Configures hcloud provider for deploy
provider "hcloud" {
  # Hetzner API token 
  token = var.HETZNER_CLOUD_TOKEN
}

# =================================================================================================
# Add firewall

resource "hcloud_firewall" "web" {
  name = "${var.HOST_SERVER_NAME}-web"

  # SSH
  rule {
    direction  = "in"
    protocol   = "tcp"
    port       = "22"
    source_ips = var.allowed_ssh_ips
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    port      = "80"
    source_ips = [
      "0.0.0.0/0",
      "::/0"
    ]
  }

  rule {
    direction = "in"
    protocol  = "tcp"
    port      = "443"
    source_ips = [
      "0.0.0.0/0",
      "::/0"
    ]
  }
}

resource "hcloud_firewall_attachment" "web" {
  firewall_id = hcloud_firewall.web.id
  server_ids   = [hcloud_server.host_server.id]

  depends_on = [
    hcloud_server.host_server
  ]
}

# =================================================================================================
# Creates an SSH key used for redeploy
resource "hcloud_ssh_key" "redeploy" {
  name       = "${var.HOST_SERVER_NAME}-redeploy-key"
  public_key = data.local_sensitive_file.ssh_public_key.content
}

# =================================================================================================
# Static IP for the instance
resource "hcloud_primary_ip" "primary_ip" {
  name          = "${var.HOST_SERVER_NAME}-primary-ip"
  location      = var.HOST_SERVER_LOCATION
  type          = "ipv4"
  assignee_type = "server"
  auto_delete   = false
}

# =================================================================================================
# Hetzner instance itselfs
resource "hcloud_server" "host_server" {
  name        = var.HOST_SERVER_NAME
  server_type = var.HOST_SERVER_TYPE
  image       = var.HOST_SERVER_IMAGE
  location    = var.HOST_SERVER_LOCATION

  public_net {
    ipv4_enabled = true
    ipv4         = hcloud_primary_ip.primary_ip.id
    ipv6_enabled = true
  }

  ssh_keys = [hcloud_ssh_key.redeploy.name]

  # Startup cloud-init script for the instance
  user_data = templatefile("${path.module}/../cloud-init.yml", {
    ssh_public_key=data.local_sensitive_file.ssh_public_key.content,
    google_application_region="${var.GOOGLE_APPLICATION_REGION}",
  })
}
