# Google Artifact Registry

Terraform configuration for provisioning a Google Artifact Registry repository to store Docker images for deployments.

## Responsibility Table

| File | Responsibility |
|------|----------------|
| `main.tf` | Define Google provider and Artifact Registry repository resource |
| `variables.tf` | Declare input variables for GCP region, project ID, and repository name |
| `outputs.tf` | Export created repository name for use by other modules |

## Variables

| Variable | Description |
|----------|-------------|
| `GOOGLE_APPLICATION_REGION` | GCP region for the repository |
| `GOOGLE_APPLICATION_PROJECT_ID` | GCP project ID |
| `ARTIFACT_REPO_NAME` | Name of the Artifact Registry repository |
