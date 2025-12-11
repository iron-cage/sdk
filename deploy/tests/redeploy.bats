#!/usr/bin/env bats

# NOTE:
# This test uses a mocked version of the `docker` binary to simulate specific behaviors
# during redeployment without launching actual containers. This is done for the following reasons:
#
# 1. **Speed & Simplicity**: Spinning up real Docker containers for each test case would introduce
#    significant overhead and complexity, especially when running in CI environments.
#
# 2. **Controlled Behavior**: By mocking Docker, we can simulate specific scenarios (e.g., build failures,
#    container exits, or unexpected outputs) that would be difficult or slow to reproduce reliably using real containers.
#
# 3. **Environment Isolation**: Not all CI environments have Docker fully configured or enabled. This mock allows
#    tests to run even in constrained or sandboxed environments without requiring privileged access.
#
# 4. **Purpose of Test**: This script is meant to test **deployment logic and script behavior**, not Docker's internal workings.
#    The correctness of actual container builds and execution is covered elsewhere in integration and e2e tests.
#
# ⚠️ If you're modifying this script or running it locally and wish to validate real Docker behavior,
# you can disable the mock and use the real binary.

setup() {
  TEST_DIR="$(mktemp -d)"

  # Directory where this .bats file lives
  TEST_FILE_DIR="$(dirname "$BATS_TEST_FILENAME")"

  # Path to redeploy.sh (one level above deploy/tests => deploy/)
  SCRIPT_ORIG="$TEST_FILE_DIR/../redeploy.sh"
  SCRIPT_UNDER_TEST="$TEST_DIR/redeploy.sh"

  if [[ ! -f "$SCRIPT_ORIG" ]]
  then
    echo "ERROR: redeploy.sh not found at $SCRIPT_ORIG" >&2
    exit 1
  fi

  cp "$SCRIPT_ORIG" "$SCRIPT_UNDER_TEST"

  tr -d '\r' < "$SCRIPT_UNDER_TEST" > "$SCRIPT_UNDER_TEST.tmp"
  mv "$SCRIPT_UNDER_TEST.tmp" "$SCRIPT_UNDER_TEST"
  chmod +x "$SCRIPT_UNDER_TEST"

  # fake /etc/environment (minimal)
  cat > "$TEST_DIR/etc-environment" <<'EOF'
DOCKER_IMAGE=dummy/image
EOF

  # patch script to use test env file
  sed -i "s#/etc/environment#$TEST_DIR/etc-environment#g" "$SCRIPT_UNDER_TEST"

  # mock docker
  DOCKER_CALLS="$TEST_DIR/docker-calls.log"
  mkdir -p "$TEST_DIR/bin"

  cat > "$TEST_DIR/bin/docker" <<EOF
#!/bin/bash
echo "docker \$*" >> "$DOCKER_CALLS"
exit 0
EOF

  chmod +x "$TEST_DIR/bin/docker"
  PATH="$TEST_DIR/bin:$PATH"
}

teardown() {
  rm -rf "$TEST_DIR"
}

@test "fails when DOCKER_IMAGE is not set" {
  # empty env file (no DOCKER_IMAGE)
  echo "" > "$TEST_DIR/etc-environment"
  run bash "$SCRIPT_UNDER_TEST"
  [ "$status" -ne 0 ]
  [[ "$output" == *"DOCKER_IMAGE is not set in the environment"* ]]
}

@test "succeeds when DOCKER_IMAGE is set" {
  run bash "$SCRIPT_UNDER_TEST"
  [ "$status" -eq 0 ]
  [[ "$output" == *"Deployment successful!"* ]]
}

@test "docker commands are called in correct order" {
  # Simulate existing container
  echo "cgtools-frontend-app" > "$TEST_DIR/fake-container"
  cat > "$TEST_DIR/bin/docker" <<EOF
#!/bin/bash
if [[ "\$*" == *"ps -a"* ]]; then
  echo "cgtools-frontend-app"
  exit 0
fi
echo "docker \$*" >> "$DOCKER_CALLS"
exit 0
EOF
  chmod +x "$TEST_DIR/bin/docker"

  run bash "$SCRIPT_UNDER_TEST"
  [ "$status" -eq 0 ]
  calls="$(cat "$DOCKER_CALLS")"
  [[ "$calls" == *"docker rm -f cgtools-frontend-app"* ]]
  [[ "$calls" == *"docker rmi dummy/image"* ]]
  [[ "$calls" == *"docker pull dummy/image"* ]]
  [[ "$calls" == *"docker run -d --name cgtools-frontend-app -p 80:80 dummy/image"* ]]
}
