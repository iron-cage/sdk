
# =================================================================================================
# Project

variable "PROJECT_NAME" {
  description = "Project name value"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9_]{3,40}$", var.PROJECT_NAME))
    error_message = "PROJECT_NAME must be lowercase snake-case (3-40 chars)"
  }
}

variable "DEPLOYMENT_MODE" {
  description = "Deployment mode"
  type        = string

  validation {
    condition     = contains(["dev", "staging", "production"], var.DEPLOYMENT_MODE)
    error_message = "DEPLOYMENT_MODE must be dev | staging | production"
  }
}

# =================================================================================================
# Network

variable "HOST_SERVER_IP" {
  description = "Server public IPv4"
  type        = string

  validation {
    condition     = can(regex("^(?:[0-9]{1,3}\\.){3}[0-9]{1,3}$", var.HOST_SERVER_IP))
    error_message = "HOST_SERVER_IP must be a valid IPv4 address"
  }
}


variable "PROJECT_DOMAIN" {
  description = "Project domain"
  type        = string
  validation {
    condition = can(regex(
      "^[a-z0-9_][a-z0-9_-]{0,61}[a-z0-9_](\\.[a-z0-9-]{1,63})+$",
      var.PROJECT_DOMAIN
    ))
    error_message = "PROJECT_DOMAIN must be a valid domain like dashboard_demo.obox.systems"
  }
}

variable "PROJECT_CERT_EMAIL" {
  description = "Email for Let's Encrypt notifications"
  type        = string

  validation {
    condition = can(regex(
      "^[^@\\s]+@[^@\\s]+\\.[^@\\s]+$",
      var.PROJECT_CERT_EMAIL
    ))
    error_message = "PROJECT_CERT_EMAIL must be a valid email (example@example.com)"
  }
}

variable "SERVER_PORT" {
  description = "Port for backend container"
  type        = number

  validation {
    condition     = var.SERVER_PORT >= 3000 && var.SERVER_PORT <= 65535
    error_message = "SERVER_PORT must be between 3000 and 65535."
  }
}

# =================================================================================================
# Host

variable "HOST_SERVER_NAME" {
  description = "Host server name"
  type        = string

  validation {
    condition     = can(regex("^[a-z0-9-]{3,40}$", var.HOST_SERVER_NAME))
    error_message = "HOST_SERVER_NAME must be lowercase kebab-case"
  }
}

# =================================================================================================
# Docker tag

variable "TAG" {
  description = "Docker image base tag (registry/repo/name)"
  type        = string

  validation {
    condition     = length(var.TAG) > 5
    error_message = "TAG must not be empty"
  }
}

variable "VERSION" {
  description = "Docker version/tag"
  type        = string

  validation {
    # allow: timestamp OR git sha
    condition = can(regex("^[a-zA-Z0-9._-]{3,50}$", var.VERSION))
    error_message = "VERSION must be a valid docker tag"
  }
}

# =================================================================================================
# env map

variable "PROJECT_MAP_VARIABLES" {
  description = "Environment variables for project backend application"
  type        = map(string)
  sensitive   = true
  validation {
    condition     = length(var.PROJECT_MAP_VARIABLES) > 0
    error_message = "PROJECT_MAP_VARIABLES cannot be empty"
  }
}

# =================================================================================================
# Secrets

variable "GOOGLE_APPLICATION_CREDENTIALS" {
  description = "Path to the google application credentials file"
  type        = string

  validation {
    condition     = fileexists(var.GOOGLE_APPLICATION_CREDENTIALS)
    error_message = "Service account credentials file not found"
  }
}


variable "SSH_PRIVATE_KEY_PATH" {
  description = "Path to the ssh private key file"
  type        = string

  validation {
    condition     = fileexists(var.SSH_PRIVATE_KEY_PATH)
    error_message = "SSH private key file not found"
  }
}

# =================================================================================================
# GOOGLE CREDS
data "local_sensitive_file" "service_account_creds" {
  filename = var.GOOGLE_APPLICATION_CREDENTIALS
}

data "local_sensitive_file" "ssh_private_key" {
  filename = var.SSH_PRIVATE_KEY_PATH
}

variable "GOOGLE_APPLICATION_REGION" {
  description = "GCP region"
  type        = string

  validation {
    condition     = can(regex("^[a-z]+-[a-z]+[0-9]+$", var.GOOGLE_APPLICATION_REGION))
    error_message = "Region must match GCP format (e.g., europe-central2)"
  }
}
