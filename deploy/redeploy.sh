#!/bin/bash

# ================== INIT ======================================================================

# set -x # for debug
set -o errexit
set -o nounset
set -o pipefail


# Color codes
RED="\e[31m"
YELLOW="\e[33m"
GREEN="\e[32m"
BLUE="\e[34m"
RESET="\e[0m"

# Default log flags
ERROR=${ERROR:-1}
DEBUG=${DEBUG:-0}
INFO=${INFO:-1}
SUCCESS=${SUCCESS:-1}

function __msg_error() {
    [[ "${ERROR}" == "1" ]] && echo -e "${RED}[ERROR]: $*${RESET}"
}

function __msg_debug() {
    [[ "${DEBUG}" == "1" ]] && echo -e "${BLUE}[DEBUG]: $*${RESET}"
}

function __msg_info() {
    [[ "${INFO}" == "1" ]] && echo -e "${YELLOW}[INFO]: $*${RESET}"
}

function __msg_success() {
    [[ "${SUCCESS}" == "1" ]] && echo -e "${GREEN}[SUCCESS]: $*${RESET}"
}

# ==============================================================================================
# ================== Set up main file ==========================================================
# Load environment variables exported in /etc/environment into this shell.
set -a
. /etc/environment
set +a


# Required env vars (fail fast if missing):
# - DOCKER_IMAGE: base name used to pull the image.
for var in \
  DOCKER_IMAGE \
  JWT_SECRET \
  IRON_SECRETS_MASTER_KEY \
  DATABASE_URL
do
  # Expansion with : "${!var:?...}" exits with an error message if the variable is unset.
  : "${!var:?$var is not set in the environment}"
done
# ----------------------------------------------------------------------------------------------

# Stop and remove previous container if it exists
__msg_info "Removing old docker compose"
docker rm -f "hypeproxies-db-test" 2>/dev/null || true
docker compose down -v || echo "Nothing to remove"


__msg_info "Remove Docker image on the host: ${DOCKER_IMAGE}"
docker rmi "${DOCKER_IMAGE}_front"  || true
docker rmi "${DOCKER_IMAGE}_back"   || true

__msg_info "Pulling Docker image: ${DOCKER_IMAGE}"
docker pull "${DOCKER_IMAGE}_front" || { echo "ERROR: Failed to pull front image"; exit 1; }
docker pull "${DOCKER_IMAGE}_back"  || { echo "ERROR: Failed to pull backend image"; exit 1; }

# ----------------------------------------------------------------------------------------------
cat <<EOF > compose.yml
services:
  backend:
    image: ${DOCKER_IMAGE}_back
    container_name: iron_backend
    environment:
      DATABASE_URL: sqlite:///app/data/iron.db?mode=rwc
      JWT_SECRET: ${JWT_SECRET}
      IRON_SECRETS_MASTER_KEY: ${IRON_SECRETS_MASTER_KEY}
      IRON_DEPLOYMENT_MODE: production
      RUST_LOG: info
    volumes:
      - sqlite_data:/app/data
    networks:
      - iron_network
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:3000/api/health"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    restart: unless-stopped

  frontend:
    image: ${DOCKER_IMAGE}_front
    container_name: iron_frontend
    ports:
      - "80:80"
    depends_on:
      backend:
        condition: service_healthy
    networks:
      - iron_network
    healthcheck:
      test: ["CMD", "wget", "--quiet", "--tries=1", "--spider", "http://localhost:80"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

networks:
  iron_network:
    driver: bridge

volumes:
  sqlite_data:
    driver: local
EOF

docker compose up -d

__msg_success "Deployment successful! App is available at: http://localhost:80"
