# Provider for resource creation
provider "google" {
  project     = var.GOOGLE_APPLICATION_PROJECT_ID
  region      = var.GOOGLE_APPLICATION_REGION
}

terraform {
  required_version = ">= 1.0"
  backend "gcs" {}
  # State locking enabled automatically via GCS object metadata
  # Do not run concurrent terraform apply operations
}

# Artifact Registry block
resource "google_artifact_registry_repository" "container-images-repo" {
  # Location for the repository
  location      = var.GOOGLE_APPLICATION_REGION
  project       = var.GOOGLE_APPLICATION_PROJECT_ID
  repository_id = var.ARTIFACT_REPO_NAME
  description   = "Docker image registry for the deployments"
  # Format of the repository. Using Docker.
  format = "DOCKER"
}
