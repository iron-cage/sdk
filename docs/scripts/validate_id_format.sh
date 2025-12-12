#!/usr/bin/env bash
# validate_id_format.sh - Detect hyphenated entity IDs (enforce underscore format)
#
# Purpose: Find instances of hyphenated IDs that should use underscores
# Usage: ./validate_id_format.sh <directory_or_file>

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
TOTAL_FILES=0
CLEAN_FILES=0
FILES_WITH_ISSUES=0
TOTAL_ISSUES=0

# Compound words that should be excluded (legitimate hyphenated terms)
EXCLUDE_PATTERNS=(
  "user-token"
  "api-token"
  "agent-provider"
  "agent-specific"
  "POST-PILOT"
  "MUST-HAVE"
  "NICE-TO-HAVE"
  "rate-limit"
  "real-time"
  "read-only"
  "write-only"
  "time-to-live"
  "cross-reference"
  "cross-site"
  "well-known"
  "best-effort"
  "end-to-end"
  "point-in-time"
  "pre-commit"
  "post-pilot"
  "multi-stage"
  "long-term"
  "short-term"
  "self-sufficient"
  "user-facing"
  "user-level"
  "user-selectable"
  "user-project"
  "project-level"
  "system-wide"
  "admin-only"
  "budget-requests"  # URL path component
  "budget-presets"   # URL path component
  "budget-controlled" # Technical term
  "budget-monitor"   # Technical term
  "budget-lease"     # Technical term
  "audit-logs"       # URL path component
  "audit-logging"    # Technical term
)

# Build exclusion regex
build_exclusion_regex() {
  local regex=""
  for pattern in "${EXCLUDE_PATTERNS[@]}"; do
    if [[ -n "$regex" ]]; then
      regex="$regex|"
    fi
    regex="$regex$pattern"
  done
  echo "$regex"
}

EXCLUSION_REGEX=$(build_exclusion_regex)

# Validate a single file
validate_file() {
  local file="$1"
  local issues=()
  local line_num=0

  # Skip archived and draft files
  if [[ "$(basename "$file")" =~ ^-archived_ ]] || [[ "$(basename "$file")" =~ ^-draft_ ]]; then
    return 0
  fi

  echo "Checking: $file"

  # Search for hyphenated entity IDs
  # Pattern: (entity prefix)-(identifier)
  # Entity prefixes: agent, provider, ip, budget, lease, user, token, project, audit
  local pattern='(agent|provider|ip|budget|lease|user|token|project|audit)-[a-z][a-z0-9]{2,}'

  while IFS= read -r line; do
    line_num=$((line_num + 1))

    # Check if line contains potential hyphenated ID
    if echo "$line" | grep -qE "$pattern"; then
      # Extract all matches
      local matches=$(echo "$line" | grep -oE "$pattern")

      # Filter out excluded compound words
      while IFS= read -r match; do
        if [[ -n "$match" ]] && ! echo "$match" | grep -qE "^($EXCLUSION_REGEX)\$"; then
          # Convert to underscore version for suggestion
          local suggestion=$(echo "$match" | sed 's/-/_/')
          issues+=("$line_num|$match|$suggestion|$line")
        fi
      done <<< "$matches"
    fi
  done < "$file"

  # Report results
  if [[ ${#issues[@]} -eq 0 ]]; then
    echo -e "  ${GREEN}✅ CLEAN${NC}"
    CLEAN_FILES=$((CLEAN_FILES + 1))
  else
    echo -e "  ${YELLOW}⚠️  ISSUES FOUND - ${#issues[@]} hyphenated entity ID(s)${NC}"
    for issue in "${issues[@]}"; do
      IFS='|' read -r line_num match suggestion context <<< "$issue"
      echo -e "    ${YELLOW}Line $line_num:${NC} $context"
      echo -e "    ${YELLOW}Found:${NC} $match"
      echo -e "    ${GREEN}Suggested:${NC} $suggestion"
      echo ""
    done
    FILES_WITH_ISSUES=$((FILES_WITH_ISSUES + 1))
    TOTAL_ISSUES=$((TOTAL_ISSUES + ${#issues[@]}))
  fi
  echo ""

  return ${#issues[@]}
}

# Main execution
main() {
  local target="$1"

  echo "===================================="
  echo "ID Format Validator"
  echo "===================================="
  echo ""

  # Determine if target is file or directory
  if [[ -f "$target" ]]; then
    TOTAL_FILES=1
    validate_file "$target" || true
  elif [[ -d "$target" ]]; then
    # Find all .md files in directory
    while IFS= read -r file; do
      TOTAL_FILES=$((TOTAL_FILES + 1))
      validate_file "$file" || true
    done < <(find "$target" -maxdepth 1 -type f -name "*.md" ! -name "-*" | sort)
  else
    echo -e "${RED}Error: '$target' is not a valid file or directory${NC}"
    exit 1
  fi

  # Summary
  echo "===================================="
  echo "Summary:"
  echo "Total Files:    $TOTAL_FILES"
  echo "Clean Files:    $CLEAN_FILES"
  echo "Files w/Issues: $FILES_WITH_ISSUES"
  echo "Total Issues:   $TOTAL_ISSUES"
  echo "===================================="

  if [[ $FILES_WITH_ISSUES -eq 0 ]]; then
    echo -e "${GREEN}✅ NO HYPHENATED IDs FOUND${NC}"
    exit 0
  else
    echo -e "${YELLOW}⚠️  HYPHENATED IDs FOUND${NC}"
    echo ""
    echo "Replace hyphenated IDs with underscore format per ID Format Standards."
    exit 1
  fi
}

# Entry point
if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <directory_or_file>"
  exit 1
fi

main "$1"
