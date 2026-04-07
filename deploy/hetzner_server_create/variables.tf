# =================================================================================================
# Hetzner API token

variable "HETZNER_CLOUD_TOKEN" {
  description = "Hetzner API token"
  type        = string
  sensitive   = true

  validation {
    condition     = length(var.HETZNER_CLOUD_TOKEN) > 20
    error_message = "HETZNER_CLOUD_TOKEN must be a valid non-empty token"
  }
}

variable "allowed_ssh_ips" {
  type        = list(string)
  sensitive   = false
  description = "Allowed IPs for server access"
}

variable "HOST_SERVER_NAME" {
  description = "Name of the new server"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9-]{3,30}$", var.HOST_SERVER_NAME))
    error_message = "HOST_SERVER_NAME must be lowercase kebab-case (3-30 chars)"
  }
}

variable "HOST_SERVER_TYPE" {
  description = "Hetzner server type"
  type        = string
  default     = "cx23"

  validation {
    condition = contains([
      "cx23","cx33","cx43","cx53","cpx22","cpx32","cpx42","cpx52"
    ], var.HOST_SERVER_TYPE)

    error_message = "Invalid Hetzner server type"
  }
}


variable "HOST_SERVER_IMAGE" {
  description = "Server OS image"
  type        = string
  default     = "ubuntu-24.04"

  validation {
    condition = contains([
      "ubuntu-22.04",
      "ubuntu-24.04",
      "debian-12"
    ], var.HOST_SERVER_IMAGE)

    error_message = "Unsupported OS image"
  }
}

variable "HOST_SERVER_LOCATION" {
  description = "Hetzner location"
  type        = string
  default     = "hel1"

  validation {
    condition = contains([
      "hel1",
      "fsn1",
      "nbg1"
    ], var.HOST_SERVER_LOCATION)

    error_message = "Invalid Hetzner location"
  }
}

# =================================================================================================
# Public key for SSH connection

variable "SSH_PUBLIC_KEY_PATH" {
  description = "Path to the ssh public key file"
  type        = string

  validation {
    condition     = fileexists(var.SSH_PUBLIC_KEY_PATH)
    error_message = "SSH public key file does not exist"
  }
}

data "local_sensitive_file" "ssh_public_key" {
  filename = "${var.SSH_PUBLIC_KEY_PATH}"
}

# =================================================================================================
# GOOGLE CREDS

variable "GOOGLE_APPLICATION_REGION" {
  description = "GCP region"
  type        = string

  validation {
    condition     = can(regex("^[a-z]+-[a-z]+[0-9]+$", var.GOOGLE_APPLICATION_REGION))
    error_message = "Region must match GCP format (e.g., europe-central2)"
  }
}

variable "GOOGLE_APPLICATION_CREDENTIALS" {
  description = "Path to service account credentials"
  type        = string

  validation {
    condition     = fileexists(var.GOOGLE_APPLICATION_CREDENTIALS)
    error_message = "Service account credentials file not found"
  }
}

# Google Cloud Platform credentials
data "local_sensitive_file" "service_account_creds" {
  filename = "${var.GOOGLE_APPLICATION_CREDENTIALS}"
}
