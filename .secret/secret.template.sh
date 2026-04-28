# -------------------------------------------------------------------------------------------------------------
#  Required Parameters


# GOOGLE CREDS
## Path to the service account credentials
GOOGLE_APPLICATION_CREDENTIALS=".secret/-service_account.json"
## Project id for deployed resources
GOOGLE_APPLICATION_PROJECT_ID="<your-gcp-project-id>"
GOOGLE_APPLICATION_REGION="europe-central2"
## Encryption key for state/backups
## Generate with: openssl rand -base64 32
GOOGLE_ENCRYPTION_KEY="<generated-encryption-key>"


# SSH KEYS
## Generate with: ssh-keygen -t ed25519 -f .secret/-iron_sdk -C "deploy_key"
SSH_PRIVATE_KEY_PATH=".secret/-iron_sdk"
SSH_PUBLIC_KEY_PATH=".secret/-iron_sdk.pub"

# HOST SERVER
## Set after server is created, or use existing server
HOST_SERVER_NAME="<your-server-hostname>"
HOST_SERVER_IP="<your-server-ip>"


# HETZNER
## Get from https://docs.hetzner.com/cloud/api/getting-started/generating-api-token/
HETZNER_CLOUD_TOKEN="<your-hetzner-api-token>"


# ======== DEPLOYMENT CONFIG ========
## Deployment mode: dev | staging | production
DEPLOYMENT_MODE="dev"


# ======== PROJECT CONFIG ========
## Project name (no hyphens or spaces)
PROJECT_NAME="my_project"
## Project domain for ingress routing
PROJECT_DOMAIN="example.com"
## Project cert email
PROJECT_CERT_EMAIL="example@email.com"


# ======== HETZNER SERVER CONFIG ========
## Server location: fsn1 | nbg1 | hel1 | ash | hil
HOST_SERVER_LOCATION="hel1"
## Server image
HOST_SERVER_IMAGE="ubuntu-24.04"
## Server type
HOST_SERVER_TYPE="cx33"
## Semicolon-separated list of allowed SSH IP/CIDR ranges
ALLOWED_SSH_IPS="0.0.0.0/0;::/0"

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
# Apollo API key for lead data scraping
APOLLO_API_KEY="<Get it from: https://developer.apollo.io/keys>"
