# Iron Runtime Server Secrets
#
# SETUP: Copy this file to -iron.sh and fill in your values
#   cp secret/iron.example.sh secret/-iron.sh
#
# Generate secrets:
#   JWT_SECRET: openssl rand -hex 32
#   IRON_SECRETS_MASTER_KEY: openssl rand -base64 32

# =============================================================================
# DATABASE
# =============================================================================

# SQLite connection string
# The ?mode=rwc parameter is REQUIRED for SQLite to create the database file
DATABASE_URL="sqlite://./iron.db?mode=rwc"

# =============================================================================
# AUTHENTICATION
# =============================================================================

# JWT secret key for signing access and refresh tokens
# SECURITY: Generate a unique value for production!
JWT_SECRET=""

# =============================================================================
# AI PROVIDER KEY ENCRYPTION
# =============================================================================

# Master key for AES-256-GCM encryption of AI provider API keys
# REQUIRED for provider key management feature
#
# SECURITY WARNING:
# - Loss of this key = permanent loss of all encrypted API keys
# - Never commit to git
# - Backup in separate secure location
# - For production: use AWS KMS or similar
IRON_SECRETS_MASTER_KEY=""

# =============================================================================
# DEPLOYMENT MODE (optional)
# =============================================================================

# Explicit deployment mode
# Values:
#   pilot       - local development (default)
#   development - explicit dev mode, enables DB wipe on startup
#   production  - confirmed production deployment
IRON_DEPLOYMENT_MODE="pilot"
