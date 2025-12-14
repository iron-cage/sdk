# -------------------------------------------------------------------------------------------------------------
#  Required Parameters

# KEYS
# Google Cloud
SECRET_STATE_ARCHIVE_KEY="<To generate use: openssl rand -hex 32>"
# Path to the service account credentials
GOOGLE_SE_CREDS_PATH="secret/service_account.json"

# SSH keys
SECRET_RSA_PRIVATE_KEY_PATH="secret/<ssh_key_name>"
SECRET_RSA_PUBLIC_KEY_PATH="secret/<ssh_key_name>.pub"

# Specifies where to deploy the project. Possible values: `hetzner`
CSP="hetzner"
# Hetzner API Token
SECRET_HETZNER_CLOUD_TOKEN="<Get it from: https://console.hetzner.cloud → Security → API Tokens>"

# Project variables
# Default project name (Should not consists "-" or spaces) / `iron_site`
PROJECT_NAME="iron_cage_sdk"

# test | production
ENVIRONMENT="test"

# SQLite connection string for pilot mode
DATABASE_URL="sqlite://./iron.db?mode=rwc"
# JWT secret key for signing access and refresh tokens
JWT_SECRET="<To generate use: openssl rand -hex 32>"
# Master key for AES-256-GCM encryption of AI provider API keys
IRON_SECRETS_MASTER_KEY="To generate use: openssl rand -base64 32"


# OPENAI API key for accessing GPT, DALL·E, etc.
OPENAI_API_KEY="<Get it from: https://platform.openai.com/account/api-keys>"
# Apollo Studio API key for GraphQL schema publishing and analytics
APOLLO_API_KEY="<Get it from: https://studio.apollographql.com → Settings → API Keys>"

## -------------------------------------------------------------------------------------------------------------
##  Optional Parameters

# Google cloud region
TF_VAR_REGION="us-central1"
# Project id for deployed resources | Can be set in secret/-secret.sh
TF_VAR_PROJECT_ID="The project id of the google cloud .json: exists in service_account.json -> project_id "
# Artifact Repository name for pushing the Docker images | Should not have "_"
TF_VAR_REPO_NAME=
# Pushed image name | Can have "_"
TF_VAR_IMAGE_NAME=
# Helper var for tagging local image
TAG=
# Zone location for the resource
TF_VAR_ZONE=
# Cloud Storage bucket name | Should not have "_"
TF_VAR_BUCKET_NAME=
# Base terraform directory
TF_DIR=
