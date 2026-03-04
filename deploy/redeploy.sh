#!/usr/bin/env bash

# ================== INIT ======================================================================

# set -x # for debug
set -euo pipefail

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
# NOTE: KUBECONFIG is NOT set here intentionally.
# In production, cloud-init configures KUBECONFIG globally (/etc/rancher/k3s/k3s.yaml)
# via /etc/environment, so kubectl works without explicit export.
# Adding `export KUBECONFIG=...` here breaks the test suite (bats),
# which provides its own KUBECONFIG pointing to a test cluster.
# ==============================================================================================
# ================== Set up main file ==========================================================

cd "$(dirname "$0")"

set -a
. ./secrets/env
set +a

# SA_JSON key reading
if [ -t 0 ]; then
  __msg_error "Service Account JSON must be provided via stdin"
  echo "Usage:"
  echo "  deploy.sh < sa.json"
  exit 1
fi

SA_JSON="$(cat || true)"

if [[ -z "${SA_JSON}" ]]; then
  __msg_error "Empty Service Account JSON"
  exit 1
fi

if ! grep -q '"type":' <<< "$SA_JSON"; then
  __msg_error "Invalid GCP service account JSON"
  exit 1
fi

# Cert-manager check
__msg_info "Check cert-manager"
if ! kubectl get crd certificates.cert-manager.io >/dev/null 2>&1; then
  __msg_info "cert-manager not found → installing"

  kubectl apply -f https://github.com/cert-manager/cert-manager/releases/download/v1.13.3/cert-manager.yaml

  __msg_info "Waiting cert-manager to be ready..."

  kubectl wait --for=condition=available deployment/cert-manager -n cert-manager --timeout=180s
  kubectl wait --for=condition=available deployment/cert-manager-webhook -n cert-manager --timeout=180s
  kubectl wait --for=condition=available deployment/cert-manager-cainjector -n cert-manager --timeout=180s

  __msg_success "cert-manager installed"
else
  __msg_info "cert-manager already installed, skip"
fi

# Clusterissue apply
__msg_info "Apply ClusterIssuer"
kubectl apply -f ./k8s/cluster-issuer.yaml

# Namespace create
__msg_info "Create namespace"
kubectl create namespace ${ARTIFACT_REPO_NAME} --dry-run=client -o yaml | kubectl apply -f -

__msg_info "Apply app env secret"
kubectl create secret generic app-env \
  --from-env-file=./secrets/env \
  --namespace ${ARTIFACT_REPO_NAME} \
  --dry-run=client -o yaml | kubectl apply -f -

__msg_info "Apply GAR docker-registry secret"
printf "%s" "$SA_JSON" | kubectl create secret docker-registry gar-auth \
  --docker-server=${GOOGLE_APPLICATION_REGION}-docker.pkg.dev \
  --docker-username=_json_key \
  --docker-password="$(cat -)" \
  --namespace ${ARTIFACT_REPO_NAME} \
  --dry-run=client -o yaml | kubectl apply -f -

__msg_info "Apply k8s"
kubectl apply -k ./k8s -n ${ARTIFACT_REPO_NAME}

__msg_info "Wait for deployment"

__msg_info "Wait for backend deployment"
if ! kubectl rollout status deployment/backend -n ${ARTIFACT_REPO_NAME} --timeout=120s; then
  __msg_error "Backend rollout failed -> rolling back both deployments"

  kubectl rollout undo deployment/backend -n ${ARTIFACT_REPO_NAME} || true
  kubectl rollout undo deployment/frontend -n ${ARTIFACT_REPO_NAME} || true

  exit 1
fi
__msg_success "Backend deployment ready"

__msg_info "Wait for frontend deployment"
if ! kubectl rollout status deployment/frontend -n ${ARTIFACT_REPO_NAME} --timeout=120s; then
  __msg_error "Frontend rollout failed -> rolling back both deployments"

  kubectl rollout undo deployment/frontend -n ${ARTIFACT_REPO_NAME} || true
  kubectl rollout undo deployment/backend -n ${ARTIFACT_REPO_NAME} || true

  exit 1
fi
__msg_success "Frontend deployment ready"

__msg_success "Deployment complete"
