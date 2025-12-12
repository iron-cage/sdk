#!/usr/bin/env bash
# validate_metadata.sh - Enforce consistent metadata across protocol files
#
# Purpose: Validate that all protocol files have standardized metadata format
# Usage: ./validate_metadata.sh <directory_or_file>

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL_FILES=0
PASSED_FILES=0
FAILED_FILES=0

# Valid values for each field
VALID_STATUS=("Specification" "Pilot" "POST-PILOT" "Archived")
VALID_PRIORITY=("MUST-HAVE" "NICE-TO-HAVE" "POST-PILOT" "TBD")

# Validation function for a single file
validate_file() {
  local file="$1"
  local errors=()

  # Skip archived, draft, and readme files
  local basename=$(basename "$file")
  if [[ "$basename" =~ ^-archived_ ]] || [[ "$basename" =~ ^-draft_ ]] || [[ "$basename" == "readme.md" ]]; then
    return 0
  fi

  echo "Validating: $file"

  # Read lines 3-8 (metadata section)
  local line3=$(sed -n '3p' "$file")
  local line4=$(sed -n '4p' "$file")
  local line5=$(sed -n '5p' "$file")
  local line6=$(sed -n '6p' "$file")
  local line8=$(sed -n '8p' "$file")

  # Validate Line 3: **Status:**
  if [[ ! "$line3" =~ ^\*\*Status:\*\*[[:space:]]+(Specification|Pilot|POST-PILOT|Archived)$ ]]; then
    errors+=("Line 3: Missing or malformed **Status:** field (expected: Specification|Pilot|POST-PILOT|Archived)")
  fi

  # Validate Line 4: **Version:**
  if [[ ! "$line4" =~ ^\*\*Version:\*\*[[:space:]]+[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    errors+=("Line 4: Missing or malformed **Version:** field (expected: X.Y.Z semantic versioning)")
  fi

  # Validate Line 5: **Last Updated:**
  if [[ ! "$line5" =~ ^\*\*Last[[:space:]]Updated:\*\*[[:space:]]+[0-9]{4}-[0-9]{2}-[0-9]{2}$ ]]; then
    errors+=("Line 5: Missing or malformed **Last Updated:** field (expected: YYYY-MM-DD)")
  else
    # Validate date format more strictly
    local date_str=$(echo "$line5" | grep -oP '\d{4}-\d{2}-\d{2}')
    if ! date -d "$date_str" &>/dev/null; then
      errors+=("Line 5: Invalid date in **Last Updated:** field (not a valid ISO 8601 date)")
    fi
  fi

  # Validate Line 6: **Priority:**
  if [[ ! "$line6" =~ ^\*\*Priority:\*\*[[:space:]]+(MUST-HAVE|NICE-TO-HAVE|POST-PILOT|TBD)$ ]]; then
    errors+=("Line 6: Missing or malformed **Priority:** field (expected: MUST-HAVE|NICE-TO-HAVE|POST-PILOT|TBD)")
  fi

  # Validate Line 8: Separator
  if [[ "$line8" != "---" ]]; then
    errors+=("Line 8: Missing or incorrect separator line (expected: ---)")
  fi

  # Check for duplicate metadata at end of file
  # Look for multiple consecutive metadata fields (indicates a metadata block, not just a field mention)
  local last_20_lines=$(tail -n 20 "$file")
  if echo "$last_20_lines" | grep -Pzo '^\*\*Status:\*\*.*\n.*^\*\*Version:\*\*' &>/dev/null; then
    errors+=("Duplicate metadata block found at end of file (should only be at top)")
  fi

  # Report results
  if [[ ${#errors[@]} -eq 0 ]]; then
    echo -e "  ${GREEN}✅ PASS${NC}"
    PASSED_FILES=$((PASSED_FILES + 1))
  else
    echo -e "  ${RED}❌ FAIL${NC}"
    for error in "${errors[@]}"; do
      echo -e "    ${YELLOW}▸${NC} $error"
    done
    FAILED_FILES=$((FAILED_FILES + 1))
  fi
  echo ""

  return ${#errors[@]}
}

# Main execution
main() {
  local target="$1"

  echo "===================================="
  echo "Metadata Validator"
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
  echo "Total Files:  $TOTAL_FILES"
  echo "Passed:       $PASSED_FILES"
  echo "Failed:       $FAILED_FILES"
  echo "===================================="

  if [[ $FAILED_FILES -eq 0 ]]; then
    echo -e "${GREEN}✅ ALL VALIDATIONS PASSED${NC}"
    exit 0
  else
    echo -e "${RED}❌ SOME VALIDATIONS FAILED${NC}"
    exit 1
  fi
}

# Entry point
if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <directory_or_file>"
  exit 1
fi

main "$1"
