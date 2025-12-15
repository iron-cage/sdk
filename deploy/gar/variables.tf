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
