#!/usr/bin/env bash
# Lint script to check ID format violations in documentation
# Enforces underscore format (prefix_identifier) for all entity IDs
#
# Exit codes:
#   0 - No violations found
#   1 - Violations detected
#   2 - Script error

set -euo pipefail

# Configuration
readonly DOCS_DIR="docs"
readonly EXCLUDE_PATTERNS="-default_topic|^-|/\-"
readonly EXIT_SUCCESS=0
readonly EXIT_VIOLATIONS=1
readonly EXIT_ERROR=2

# Color codes for output
readonly RED='\033[0;31m'
readonly GREEN='\033[0;32m'
readonly YELLOW='\033[1;33m'
readonly NC='\033[0m' # No Color

# Violation patterns to check
declare -A VIOLATIONS=(
  ["Provider ID violations"]="ip-[a-z0-9]+-[0-9]+"
  ["User ID violations"]="user-[a-z0-9]+"
  ["Project ID violations"]="proj-[a-z]+"
  ["Agent ID violations"]="agent-[a-z0-9]+"
  ["Token ID violations"]="(ic|ip)-[a-z0-9]+-[0-9]+"
)

# Terms that should NOT be flagged (context-dependent)
readonly EDGE_CASE_EXCLUDE="user-token|user-facing|user-level|user-specific|user-defined|sub-agent"

# Track violation count
violation_count=0

print_header() {
  echo "========================================"
  echo "  ID Format Lint Check"
  echo "========================================"
  echo ""
}

print_summary() {
  echo ""
  echo "========================================"
  if [ "$violation_count" -eq 0 ]; then
    echo -e "${GREEN}✓ No ID format violations found${NC}"
    echo "========================================"
    return $EXIT_SUCCESS
  else
    echo -e "${RED}✗ Found $violation_count violation(s)${NC}"
    echo "========================================"
    echo ""
    echo "ID Format Standards:"
    echo "  Provider IDs:  ip_openai_001   (NOT ip-openai-001)"
    echo "  User IDs:      user_xyz789     (NOT user-xyz789)"
    echo "  Project IDs:   proj_master     (NOT proj-master)"
    echo "  Agent IDs:     agent_abc123    (NOT agent-abc123)"
    echo "  Token IDs:     ic_def456       (NOT ic-def456)"
    echo ""
    echo "See docs/standards/id_format_standards.md for details"
    return $EXIT_VIOLATIONS
  fi
}

check_pattern() {
  local category="$1"
  local pattern="$2"

  echo "Checking: $category..."

  # Search for violations, excluding edge cases, temp files, and research docs
  local results
  results=$(find "$DOCS_DIR" -type f -name "*.md" \
    ! -path "*/\-*" \
    ! -name "\-*" \
    ! -path "*/research/*" \
    -exec grep -Hn -E "$pattern" {} \; 2>/dev/null | \
    grep -v -E "$EDGE_CASE_EXCLUDE" | \
    grep -v "NOT " | \
    grep -v -- "--user-id" | \
    grep -v -- "--user-name" | \
    grep -v -- "--agent-id" | \
    grep -v "agent-provider" | \
    grep -v "agent-facing" | \
    grep -v "agent-hour" | \
    grep -v "agent-level" | \
    grep -v "agent-specific" | \
    grep -v '"name".*-agent-' | \
    grep -v "'agent-" | \
    grep -v "Pay-per-" || true)

  if [ -n "$results" ]; then
    local count
    count=$(echo "$results" | wc -l)
    violation_count=$((violation_count + count))
    echo -e "${RED}✗ $category ($count violations)${NC}"
    echo "$results" | head -20 | while IFS= read -r line; do
      echo "    $line"
    done
    if [ "$count" -gt 20 ]; then
      echo "    ... and $((count - 20)) more"
    fi
    echo ""
    return 1
  else
    echo -e "${GREEN}✓ $category: OK${NC}"
    return 0
  fi
}

verify_underscore_format() {
  echo "Verifying underscore format compliance..."

  # Count proper underscore IDs
  local underscore_count
  underscore_count=$(find "$DOCS_DIR" -type f -name "*.md" ! -path "*/\-*" ! -name "\-*" -exec grep -oh -E "(ip|user|proj|agent|ic)_[a-z0-9_]+" {} \; 2>/dev/null | \
    sort -u | \
    wc -l)

  echo -e "${GREEN}✓ Found $underscore_count properly formatted IDs${NC}"
  echo ""
}

main() {
  # Change to script directory's parent (project root)
  cd "$(dirname "$0")/.."

  # Verify docs directory exists
  if [ ! -d "$DOCS_DIR" ]; then
    echo -e "${RED}Error: $DOCS_DIR directory not found${NC}" >&2
    exit $EXIT_ERROR
  fi

  print_header

  # Check each violation pattern
  for category in "${!VIOLATIONS[@]}"; do
    check_pattern "$category" "${VIOLATIONS[$category]}" || true
  done

  # Verify positive cases
  verify_underscore_format

  # Print summary and exit
  print_summary
  exit $?
}

main "$@"
