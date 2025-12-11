#!/usr/bin/env bash
# Lint script to check Python tooling compliance
# Enforces uv + pyproject.toml standards (no pip, no .python-version, no requirements.txt)
#
# Exit codes:
#   0 - No critical violations (warnings OK)
#   1 - Critical violations found
#   2 - Script error

set -euo pipefail

# Configuration
readonly PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
readonly MODULE_DIR="module"
readonly DOCS_DIR="docs"
readonly EXIT_SUCCESS=0
readonly EXIT_VIOLATIONS=1
readonly EXIT_ERROR=2

# Color codes
readonly RED='\033[0;31m'
readonly YELLOW='\033[1;33m'
readonly GREEN='\033[0;32m'
readonly NC='\033[0m'

# Counters
critical_count=0
warning_count=0

print_header() {
  echo "========================================"
  echo "  Python Tooling Compliance Check"
  echo "========================================"
  echo ""
}

print_summary() {
  echo ""
  echo "========================================"
  if [ "$critical_count" -eq 0 ]; then
    if [ "$warning_count" -eq 0 ]; then
      echo -e "${GREEN}✓ Python Tooling Compliance: PASS${NC}"
      echo "  No violations found"
    else
      echo -e "${GREEN}✓ Python Tooling Compliance: PASS${NC}"
      echo -e "${YELLOW}  (${warning_count} warnings - should fix)${NC}"
    fi
    echo "========================================"
    return $EXIT_SUCCESS
  else
    echo -e "${RED}✗ Python Tooling Compliance: FAIL${NC}"
    echo "  ${critical_count} critical violation(s)"
    if [ "$warning_count" -gt 0 ]; then
      echo "  ${warning_count} warning(s)"
    fi
    echo "========================================"
    echo ""
    echo "Fix Guide:"
    echo "  1. Remove .python-version files (use pyproject.toml requires-python)"
    echo "  2. Migrate requirements.txt to pyproject.toml dependencies"
    echo "  3. Replace 'pip install' with 'uv add' in development docs"
    echo ""
    echo "See docs/standards/python_tooling_standards.md for details"
    return $EXIT_VIOLATIONS
  fi
}

check_python_version_files() {
  echo "Checking for .python-version files..."

  local results
  results=$(find "$PROJECT_ROOT" -name ".python-version" -type f \
    ! -path "*/\-*" \
    ! -path "*/.venv/*" \
    ! -path "*/venv/*" \
    2>/dev/null || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    critical_count=$((critical_count + count))
    echo -e "${RED}✗ Found .python-version files (${count})${NC}"
    echo "$results" | while IFS= read -r line; do
      echo "    $line"
    done
    echo ""
  else
    echo -e "${GREEN}✓ No .python-version files found${NC}"
  fi
}

check_requirements_txt() {
  echo "Checking for requirements.txt files..."

  local results
  results=$(find "$PROJECT_ROOT/$MODULE_DIR" -name "requirements*.txt" -type f \
    ! -path "*/-archived*" \
    ! -path "*/\-*" \
    ! -path "*/.venv/*" \
    ! -path "*/venv/*" \
    2>/dev/null || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    critical_count=$((critical_count + count))
    echo -e "${RED}✗ Found requirements.txt files (${count})${NC}"
    echo "$results" | while IFS= read -r line; do
      echo "    $line"
    done
    echo ""
  else
    echo -e "${GREEN}✓ No legacy requirements files found${NC}"
  fi
}

check_setup_py() {
  echo "Checking for setup.py files..."

  local results
  results=$(find "$PROJECT_ROOT/$MODULE_DIR" -name "setup.py" -type f \
    ! -path "*/-archived*" \
    ! -path "*/\-*" \
    2>/dev/null || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    critical_count=$((critical_count + count))
    echo -e "${RED}✗ Found setup.py files (${count})${NC}"
    echo "$results" | while IFS= read -r line; do
      echo "    $line"
    done
    echo ""
  else
    echo -e "${GREEN}✓ No setup.py files found${NC}"
  fi
}

check_pip_install_in_docs() {
  echo "Checking for 'pip install' in development docs..."

  local results
  results=$(grep -rn "pip install" \
    "$PROJECT_ROOT/$DOCS_DIR" \
    "$PROJECT_ROOT/contributing.md" \
    "$PROJECT_ROOT/Makefile" \
    --include="*.md" \
    --include="Makefile" \
    2>/dev/null | \
    grep -v "# User-facing" | \
    grep -v "pip install iron-sdk" | \
    grep -v "pip install iron-" | \
    grep -v "END USERS" || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    warning_count=$((warning_count + count))
    echo -e "${YELLOW}⚠ Found 'pip install' in development docs (${count})${NC}"
    echo "$results" | head -10 | while IFS= read -r line; do
      echo "    $line"
    done
    if [ "$count" -gt 10 ]; then
      echo "    ... and $((count - 10)) more"
    fi
    echo ""
  else
    echo -e "${GREEN}✓ No 'pip install' in development docs${NC}"
  fi
}

check_uv_pip_install() {
  echo "Checking for 'uv pip install' (old syntax)..."

  local results
  results=$(grep -rn "uv pip install" \
    "$PROJECT_ROOT/$DOCS_DIR" \
    "$PROJECT_ROOT/contributing.md" \
    "$PROJECT_ROOT/$MODULE_DIR" \
    --include="*.md" \
    2>/dev/null || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    warning_count=$((warning_count + count))
    echo -e "${YELLOW}⚠ Found 'uv pip install' old syntax (${count})${NC}"
    echo "$results" | head -10 | while IFS= read -r line; do
      echo "    $line"
    done
    if [ "$count" -gt 10 ]; then
      echo "    ... and $((count - 10)) more"
    fi
    echo ""
  else
    echo -e "${GREEN}✓ No 'uv pip install' old syntax found${NC}"
  fi
}

check_virtualenv_in_docs() {
  echo "Checking for virtualenv/venv commands..."

  local results
  results=$(grep -rn "virtualenv\|python -m venv" \
    "$PROJECT_ROOT/$DOCS_DIR" \
    "$PROJECT_ROOT/contributing.md" \
    --include="*.md" \
    2>/dev/null || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    warning_count=$((warning_count + count))
    echo -e "${YELLOW}⚠ Found virtualenv/venv commands (${count})${NC}"
    echo "$results" | while IFS= read -r line; do
      echo "    $line"
    done
    echo ""
  else
    echo -e "${GREEN}✓ No virtualenv/venv commands found${NC}"
  fi
}

verify_pyproject_toml() {
  echo "Verifying pyproject.toml files..."

  local module_count=0
  local missing_python=0

  for dir in "$PROJECT_ROOT/$MODULE_DIR"/*/; do
    # Skip archived modules and temp directories
    if [[ "$dir" == *"-archived"* ]] || [[ "$dir" == *"-"* ]]; then
      continue
    fi

    # Check if it's a Python module (has .py files)
    if find "$dir" -name "*.py" -type f | grep -q .; then
      module_count=$((module_count + 1))

      if [ ! -f "$dir/pyproject.toml" ]; then
        critical_count=$((critical_count + 1))
        echo -e "${RED}✗ Missing pyproject.toml in $(basename "$dir")${NC}"
      else
        # Check for requires-python
        if ! grep -q "requires-python" "$dir/pyproject.toml"; then
          missing_python=$((missing_python + 1))
        fi
      fi
    fi
  done

  if [ "$missing_python" -gt 0 ]; then
    warning_count=$((warning_count + missing_python))
    echo -e "${YELLOW}⚠ ${missing_python} module(s) missing requires-python in pyproject.toml${NC}"
  fi

  if [ "$critical_count" -eq 0 ] && [ "$missing_python" -eq 0 ]; then
    echo -e "${GREEN}✓ Found ${module_count} Python modules with pyproject.toml${NC}"
  fi
}

verify_uv_lock_files() {
  echo "Verifying uv.lock files..."

  local missing_count=0

  for dir in "$PROJECT_ROOT/$MODULE_DIR"/*/; do
    # Skip archived modules and temp directories
    if [[ "$dir" == *"-archived"* ]] || [[ "$dir" == *"-"* ]]; then
      continue
    fi

    # Check if it has pyproject.toml
    if [ -f "$dir/pyproject.toml" ]; then
      if [ ! -f "$dir/uv.lock" ]; then
        missing_count=$((missing_count + 1))
        warning_count=$((warning_count + 1))
        echo -e "${YELLOW}⚠ Missing uv.lock in $(basename "$dir")${NC}"
      fi
    fi
  done

  if [ "$missing_count" -eq 0 ]; then
    echo -e "${GREEN}✓ All modules have uv.lock files${NC}"
  else
    echo ""
  fi
}

main() {
  cd "$PROJECT_ROOT"

  print_header

  # Critical checks
  check_python_version_files
  check_requirements_txt
  check_setup_py
  verify_pyproject_toml

  # Warning checks
  check_pip_install_in_docs
  check_uv_pip_install
  check_virtualenv_in_docs
  verify_uv_lock_files

  # Print summary and exit
  print_summary
  exit $?
}

main "$@"
