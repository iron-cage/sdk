#!/usr/bin/env bats
bats_require_minimum_version 1.5.0

setup() {
  export ARTIFACT_REPO_NAME="bats-test-ns"
  export PROJECT_NAME="bats-test"
  export DEPLOYMENT_MODE="dev"
  export PROJECT_DOMAIN="testingdomain"
  export GOOGLE_APPLICATION_REGION="europe-central2"

  BASE="$BATS_TEST_TMPDIR/app"

  mkdir -p "$BASE/secrets"
  mkdir -p "$BASE/k8s"

  # copy real script into sandbox
  cp "$BATS_TEST_DIRNAME/../redeploy.sh" "$BASE/redeploy.sh"
  chmod +x "$BASE/redeploy.sh"

  # fake env
  cat > "$BASE/secrets/env" <<EOF
DUMMY=1
EOF

  # minimal deployment
  # ---------- cluster issuer stub ----------
  cat > "$BASE/k8s/cluster-issuer.yaml" <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: dummy-issuer
EOF

  # ---------- deployment ----------
  cat > "$BASE/k8s/deployment.yaml" <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: app
spec:
  replicas: 1
  selector:
    matchLabels:
      app: app
  template:
    metadata:
      labels:
        app: app
    spec:
      containers:
        - name: app
          image: nginx
EOF

  cat > "$BASE/k8s/kustomization.yaml" <<EOF
resources:
  - deployment.yaml
  - cluster-issuer.yaml
EOF

  export FAKE_SA_JSON='{"type":"service_account"}'
}

teardown() {
  kubectl delete ns "$ARTIFACT_REPO_NAME" --ignore-not-found=true
}

run_deploy() {
  bash -c "printf '%s' '$FAKE_SA_JSON' | bash '$BASE/redeploy.sh'"
}

# -------------------------------------------------
@test "creates namespace" {
  run -0 run_deploy
  run -0 kubectl get ns "$ARTIFACT_REPO_NAME"
}

# -------------------------------------------------
@test "creates docker registry secret" {
  run -0 run_deploy
  run -0 kubectl get secret gar-auth -n "$ARTIFACT_REPO_NAME"
}

# -------------------------------------------------
@test "applies k8s manifests" {
  run -0 run_deploy
  run -0 kubectl get deployment app -n "$ARTIFACT_REPO_NAME"
}

# -------------------------------------------------
@test "rollout completes successfully" {
  run -0 run_deploy
  run -0 kubectl wait --for=condition=available deployment/app -n "$ARTIFACT_REPO_NAME" --timeout=30s
}

# -------------------------------------------------
@test "rollback happens on bad image" {

  cat > "$BASE/k8s/deployment.yaml" <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: certificates.cert-manager.io
spec:
  group: cert-manager.io
  names:
    plural: certificates
    singular: certificate
    kind: Certificate
  scope: Namespaced
  versions:
    - name: v1
      served: true
      storage: true
EOF

  export FAKE_SA_JSON='{"type":"service_account"}'
}
