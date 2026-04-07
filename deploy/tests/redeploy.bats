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

  # ---------- cluster issuer stub ----------
  cat > "$BASE/k8s/cluster-issuer.yaml" <<EOF
apiVersion: v1
kind: ConfigMap
metadata:
  name: dummy-issuer
EOF

  # ---------- backend deployment ----------
  cat > "$BASE/k8s/backend.yaml" <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
        - name: backend
          image: nginx
          ports:
            - containerPort: 8080
EOF

  # ---------- frontend deployment ----------
  cat > "$BASE/k8s/frontend.yaml" <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: frontend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: frontend
  template:
    metadata:
      labels:
        app: frontend
    spec:
      containers:
        - name: frontend
          image: nginx
          ports:
            - containerPort: 80
EOF

  # ---------- kustomization ----------
  cat > "$BASE/k8s/kustomization.yaml" <<EOF
resources:
  - backend.yaml
  - frontend.yaml
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
@test "applies backend and frontend deployments" {
  run -0 run_deploy
  run -0 kubectl get deployment backend -n "$ARTIFACT_REPO_NAME"
  run -0 kubectl get deployment frontend -n "$ARTIFACT_REPO_NAME"
}

# -------------------------------------------------
@test "backend and frontend rollouts complete" {
  run -0 run_deploy
  run -0 kubectl rollout status deployment/backend -n "$ARTIFACT_REPO_NAME" --timeout=30s
  run -0 kubectl rollout status deployment/frontend -n "$ARTIFACT_REPO_NAME" --timeout=30s
}

# -------------------------------------------------
@test "rollback happens on bad backend image" {
  run -0 run_deploy

  run kubectl get deployment backend -n "$ARTIFACT_REPO_NAME" \
    -o jsonpath='{.spec.template.spec.containers[0].image}'
  previous_image="$output"

  # break backend image
  cat > "$BASE/k8s/backend.yaml" <<EOF
apiVersion: apps/v1
kind: Deployment
metadata:
  name: backend
spec:
  replicas: 1
  selector:
    matchLabels:
      app: backend
  template:
    metadata:
      labels:
        app: backend
    spec:
      containers:
        - name: backend
          image: definitely-not-existing-image:fail
EOF

  run run_deploy
  [ "$status" -ne 0 ]

  run -0 kubectl rollout status deployment/backend -n "$ARTIFACT_REPO_NAME"

  run kubectl get deployment backend -n "$ARTIFACT_REPO_NAME" \
    -o jsonpath='{.spec.template.spec.containers[0].image}'
  [ "$output" = "$previous_image" ]
}
