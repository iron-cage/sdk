# Hetzner API token
variable "HETZNER_CLOUD_TOKEN" {
  sensitive = true
}

# Specifies region location that will be used for all recources
variable "REGION" {
  description = "region of the resources"
}

# Project id where all resources will be created 
variable "PROJECT_ID" {
  description = "project id for the resources"
}

# Project name value 
variable "PROJECT_NAME" {
  description = "Project name value"
}

# Artifact Registry repository name  
variable "REPO_NAME" {
  description = "artifact registry name"
}

# Name of the docker image to pull
variable "IMAGE_NAME" {
  description = "name of the webapp image"
}

# Name of the bucket
variable "BUCKET_NAME" {
  description = "Bucket name"
}

# JWT secret key for signing access and refresh tokens
variable "JWT_SECRET" {
  description = "JWT secret key for signing access and refresh tokens"
}

# Master key for AES-256-GCM encryption of AI provider API keys
variable "IRON_SECRETS_MASTER_KEY" {
  description = "Master key for AES-256-GCM encryption of AI provider API keys"
}

# SQLite connection string for pilot mode
variable "DATABASE_URL" {
  description = "SQLite connection string for pilot mode"
}

# Docker image tag
variable "TAG" {
  description = "Docker image tag"
}

# Google Cloud Platform credentials
data "local_sensitive_file" "service_account_creds" {
  filename = var.GOOGLE_SE_CREDS_PATH
}

# Private key for SSH connection
data "local_sensitive_file" "ssh_private_key" {
  filename = var.SECRET_RSA_PRIVATE_KEY_PATH
}

# Public key for SSH connection
data "local_sensitive_file" "ssh_public_key" {
  filename = var.SECRET_RSA_PUBLIC_KEY_PATH
}

# Path to the GCP service account JSON key file
variable "GOOGLE_SE_CREDS_PATH" {
  description = "Path to the GCP service account JSON key file"
  type        = string
}

# Path to the rsa privat key file
variable "SECRET_RSA_PRIVATE_KEY_PATH" {
  description = "Path to the rsa private key file"
  type        = string
}

# Path to the rsa publiv key path file
variable "SECRET_RSA_PUBLIC_KEY_PATH" {
  description = "Path to the rsa public key path file"
  type        = string
}
