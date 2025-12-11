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

: "${CONTAINER_NAME:=cgtools-frontend-app}"

# Required env vars (fail fast if missing):
# - DOCKER_IMAGE: base name used to pull the image.
for var in \
  DOCKER_IMAGE
do
  # Expansion with : "${!var:?...}" exits with an error message if the variable is unset.
  : "${!var:?$var is not set in the environment}"
done
# ----------------------------------------------------------------------------------------------

# Stop and remove previous container if it exists
if docker ps -a --format '{{.Names}}' | grep -Eq "^${CONTAINER_NAME}\$"
then
  __msg_info "Removing old container: ${CONTAINER_NAME}"
  docker rm -f "${CONTAINER_NAME}"
fi

__msg_info "Remove Docker image on the host: ${DOCKER_IMAGE}"
docker rmi "${DOCKER_IMAGE}"  || true

__msg_info "Pulling Docker image: ${DOCKER_IMAGE}"
docker pull "${DOCKER_IMAGE}" || { echo "ERROR: Failed to pull front image"; exit 1; }

# ----------------------------------------------------------------------------------------------

# Run container with port 80 exposed
__msg_info "Running container: ${CONTAINER_NAME}"
docker run -d \
  --name "${CONTAINER_NAME}" \
  -p 80:80 \
  "${DOCKER_IMAGE}"

__msg_success "Deployment successful! App is available at: http://localhost:80"
