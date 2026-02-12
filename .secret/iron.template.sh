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

# IC Token secret for agent authentication (Protocol 005)
# Used to sign/verify IC tokens for budget handshake
IC_TOKEN_SECRET=""

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
# CORS CONFIGURATION
# =============================================================================

# Allowed origins for CORS (comma-separated URLs)
# Controls which web domains can make API requests from a browser
# Example: "http://localhost:5173,http://localhost:3001"
ALLOWED_ORIGINS=""

# =============================================================================
# SERVER CONFIGURATION
# =============================================================================

# Server port (1-65535)
SERVER_PORT=""

# =============================================================================
# DEPLOYMENT MODE (optional)
# =============================================================================

# Explicit deployment mode
# Values:
#   pilot       - local development (default)
#   development - explicit dev mode, enables DB wipe on startup
#   production  - confirmed production deployment
IRON_DEPLOYMENT_MODE="pilot"

# =============================================================================
# DATABASE SEEDING (optional)
# =============================================================================

# Enable demo seed mode for production demo deployments
# When database is empty:
#
# false (default): No auto-seeding - empty database, create users via API
# true:            Seed demo accounts (admin@ironcage.ai / IronDemo2025!)
#
# See docs/demo_credentials.md for full list of demo accounts
ENABLE_DEMO_SEED="false"
