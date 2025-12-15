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

# =============================================================================
# Iron Runtime Server Secrets

# SQLite connection string
DATABASE_URL="sqlite://./iron.db?mode=rwc"
# JWT secret key for signing access and refresh tokens
JWT_SECRET="<To generate use: openssl rand -hex 32>"
# IC Token secret for agent authentication (Protocol 005)
IC_TOKEN_SECRET="<To generate use: openssl rand -hex 32>"
# Secret key used to sign and validate IP-based access tokens
IP_TOKEN_KEY="<To generate use: openssl rand -hex 32>"
# Master key for AES-256-GCM encryption of AI provider API keys
IRON_SECRETS_MASTER_KEY="<To generate use: openssl rand -base64 32>"
# Allowed origins for CORS (comma-separated URLs)
ALLOWED_ORIGINS="http://localhost:5173,http://localhost:3001"
# TCP port on which the backend HTTP API listens for incoming requests
SERVER_PORT="3001"
# Explicit deployment mode
# Values:
#   pilot       - local development (default)
#   development - explicit dev mode, enables DB wipe on startup
#   production  - confirmed production deployment
IRON_DEPLOYMENT_MODE="pilot"
ENABLE_DEMO_SEED="true"

# =============================================================================
# AI keys

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
