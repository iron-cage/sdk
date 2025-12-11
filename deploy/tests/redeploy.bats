#!/usr/bin/env bats

env_without() {
  local var="$1"
  cp "$FULL_ENV" "$TEST_DIR/etc-environment"
  sed -i "/^$var=/d" "$TEST_DIR/etc-environment"
}

setup() {
  TEST_DIR="$(mktemp -d)"
  SCRIPT_ORIG="$(dirname "$BATS_TEST_FILENAME")/../redeploy.sh"
  SCRIPT_UNDER_TEST="$TEST_DIR/redeploy.sh"
  cp "$SCRIPT_ORIG" "$SCRIPT_UNDER_TEST"
  tr -d '\r' < "$SCRIPT_UNDER_TEST" > "$SCRIPT_UNDER_TEST.tmp"
  mv "$SCRIPT_UNDER_TEST.tmp" "$SCRIPT_UNDER_TEST"
  chmod +x "$SCRIPT_UNDER_TEST"

  FULL_ENV="$TEST_DIR/full-env"
  cat > "$FULL_ENV" <<'EOF'
JWT_SECRET=dummy-jwt
IRON_SECRETS_MASTER_KEY=secret_jwt_master_key
DATABASE_URL=postgres://u:p@db/test_db
DOCKER_IMAGE=example.com/app
EOF

  cp "$FULL_ENV" "$TEST_DIR/etc-environment"

  sed -i "s#/etc/environment#$TEST_DIR/etc-environment#g" "$SCRIPT_UNDER_TEST"

  # stub docker setup
  DOCKER_CALLS="$TEST_DIR/docker-calls.log"
  mkdir -p "$TEST_DIR/bin"
  cat > "$TEST_DIR/bin/docker" <<EOF
#!/bin/bash
echo "docker \$*" >> "$DOCKER_CALLS"
case "\$1" in
  rm|rmi|pull) exit 0 ;;
  compose)
    shift
    if [[ "\$1" == "down" ]] || [[ "\$1" == "up" ]]; then exit 0; fi
    ;;
esac
exit 0
EOF

  tr -d '\r' < "$TEST_DIR/bin/docker" > "$TEST_DIR/bin/docker.tmp"
  mv "$TEST_DIR/bin/docker.tmp" "$TEST_DIR/bin/docker"
  chmod +x "$TEST_DIR/bin/docker"
  PATH="$TEST_DIR/bin:$PATH"
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "succeeds with valid inputs" {
  cp "$FULL_ENV" "$TEST_DIR/etc-environment"
  run env DOCKER_IMAGE="example.com/app" bash -c "set -a && source '$TEST_DIR/etc-environment' && cd '$TEST_DIR' && '$SCRIPT_UNDER_TEST'"
  
  echo "$output"
  echo "Status: $status"

  [ "$status" -eq 0 ]
}

@test "fails when DOCKER_IMAGE is not set" {
  env_without "DOCKER_IMAGE"
  run bash -c "set -a && source '$TEST_DIR/etc-environment' && '$SCRIPT_UNDER_TEST'"
  [ "$status" -ne 0 ]
  [[ "$output" == *"DOCKER_IMAGE is not set in the environment"* ]]
}

@test "cleanup commands are called" {
  cp "$FULL_ENV" "$TEST_DIR/etc-environment"
  run env DOCKER_IMAGE="example.com/app" bash -c "set -a && source '$TEST_DIR/etc-environment' && cd '$TEST_DIR' && '$SCRIPT_UNDER_TEST'"
  [ "$status" -eq 0 ]

  calls="$(cat "$DOCKER_CALLS")"
  [[ "$calls" == *"docker volume rm sqlite_data"* ]]
  [[ "$calls" == *"docker compose down"* ]]
}

@test "missing JWT_SECRET" {
  env_without "JWT_SECRET"
  run env DOCKER_IMAGE="example.com/app" bash -c "set -a && source '$TEST_DIR/etc-environment' && cd '$TEST_DIR' && '$SCRIPT_UNDER_TEST'"
  [ "$status" -ne 0 ]
  [[ "$output" == *"JWT_SECRET is not set in the environment"* ]]
}

@test "missing IRON_SECRETS_MASTER_KEY" {
  env_without "IRON_SECRETS_MASTER_KEY"
  run env DOCKER_IMAGE="example.com/app" bash -c "set -a && source '$TEST_DIR/etc-environment' && cd '$TEST_DIR' && '$SCRIPT_UNDER_TEST'"
  [ "$status" -ne 0 ]
  [[ "$output" == *"IRON_SECRETS_MASTER_KEY is not set in the environment"* ]]
}

@test "missing DATABASE_URL" {
  env_without "DATABASE_URL"
  run env DOCKER_IMAGE="example.com/app" bash -c "set -a && source '$TEST_DIR/etc-environment' && cd '$TEST_DIR' && '$SCRIPT_UNDER_TEST'"
  [ "$status" -ne 0 ]
  [[ "$output" == *"DATABASE_URL is not set in the environment"* ]]
}
