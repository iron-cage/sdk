#!/usr/bin/env bash
# validate_cross_references.sh - Verify all markdown links resolve correctly
#
# Purpose: Check that all internal markdown links point to existing files
# Usage: ./validate_cross_references.sh <docs_root_directory>

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Counters
TOTAL_LINKS=0
VALID_LINKS=0
BROKEN_LINKS=0

# Extract markdown links from a file
extract_links() {
  local file="$1"
  # Pattern: [text](path) or [text](path#anchor)
  grep -oP '\[.*?\]\(\K[^)]+' "$file" | grep -v '^http' | grep -v '^#' || true
}

# Check if a file exists (resolving relative paths)
check_link() {
  local source_file="$1"
  local link="$2"
  local source_dir=$(dirname "$source_file")

  # Split link into path and anchor
  local path="${link%%#*}"
  local anchor="${link#*#}"
  if [[ "$anchor" == "$link" ]]; then
    anchor=""
  fi

  # Resolve relative path (handle filenames starting with -)
  local resolved_path=$(cd "$source_dir" && realpath -m -- "$path" 2>/dev/null || echo "")

  if [[ -z "$resolved_path" ]]; then
    echo "INVALID_PATH|$link|Could not resolve path"
    return 1
  fi

  # Check if file exists
  if [[ ! -f "$resolved_path" ]]; then
    # Try to find similar files (handle filenames starting with -)
    local filename=$(basename -- "$path")
    local suggestions=$(find "$(dirname "$source_file")/.." -name "$filename" 2>/dev/null | head -n 3)
    echo "NOT_FOUND|$link|File not found|$suggestions"
    return 1
  fi

  # If anchor specified, check if it exists in the target file
  if [[ -n "$anchor" ]]; then
    # Convert anchor to header format (replace - with space, capitalize)
    local header_pattern=$(echo "$anchor" | sed 's/-/ /g' | sed 's/\b\w/\u&/g')

    # Search for matching header in target file
    if ! grep -qiE "^#+\s+$header_pattern" "$resolved_path"; then
      echo "ANCHOR_NOT_FOUND|$link|Anchor '#$anchor' not found in $resolved_path"
      return 1
    fi
  fi

  echo "VALID|$link"
  return 0
}

# Validate all links in a file
validate_file() {
  local file="$1"

  # Skip archived and draft files (handle filenames starting with -)
  if [[ "$(basename -- "$file")" =~ ^-archived_ ]] || [[ "$(basename -- "$file")" =~ ^-draft_ ]]; then
    return 0
  fi

  echo "Checking: $file"

  local links=$(extract_links "$file")
  local file_has_broken=false

  if [[ -z "$links" ]]; then
    echo -e "  ${GREEN}No links to validate${NC}"
    echo ""
    return 0
  fi

  while IFS= read -r link; do
    if [[ -z "$link" ]]; then
      continue
    fi

    TOTAL_LINKS=$((TOTAL_LINKS + 1))

    local result=$(check_link "$file" "$link")
    local status=$(echo "$result" | cut -d'|' -f1)

    if [[ "$status" == "VALID" ]]; then
      VALID_LINKS=$((VALID_LINKS + 1))
    else
      if [[ "$file_has_broken" == false ]]; then
        echo -e "  ${RED}❌ BROKEN LINK(S)${NC}"
        file_has_broken=true
      fi

      BROKEN_LINKS=$((BROKEN_LINKS + 1))

      local link_text=$(echo "$result" | cut -d'|' -f2)
      local error_msg=$(echo "$result" | cut -d'|' -f3)
      local suggestions=$(echo "$result" | cut -d'|' -f4)

      # Find line number
      local line_num=$(grep -n "\[.*\]($link)" "$file" | head -n1 | cut -d':' -f1)

      echo -e "    ${YELLOW}Line $line_num:${NC} [$link_text]($link)"
      echo -e "    ${RED}Error:${NC} $error_msg"

      if [[ -n "$suggestions" ]]; then
        echo -e "    ${GREEN}Suggestions:${NC}"
        while IFS= read -r suggestion; do
          if [[ -n "$suggestion" ]]; then
            # Make path relative to source file
            local rel_path=$(realpath --relative-to="$(dirname "$file")" "$suggestion")
            echo -e "      $rel_path"
          fi
        done <<< "$suggestions"
      fi
      echo ""
    fi
  done <<< "$links"

  if [[ "$file_has_broken" == false ]]; then
    echo -e "  ${GREEN}✅ All links valid${NC}"
  fi
  echo ""
}

# Main execution
main() {
  local docs_root="$1"

  if [[ ! -d "$docs_root" ]]; then
    echo -e "${RED}Error: '$docs_root' is not a valid directory${NC}"
    exit 1
  fi

  echo "===================================="
  echo "Cross-Reference Validator"
  echo "===================================="
  echo ""

  # Find all .md files recursively
  while IFS= read -r file; do
    validate_file "$file"
  done < <(find "$docs_root" -type f -name "*.md" ! -name "-*" | sort)

  # Summary
  echo "===================================="
  echo "Summary:"
  echo "Total Links:   $TOTAL_LINKS"
  echo "Valid Links:   $VALID_LINKS"
  echo "Broken Links:  $BROKEN_LINKS"
  echo "===================================="

  if [[ $BROKEN_LINKS -eq 0 ]]; then
    echo -e "${GREEN}✅ ALL LINKS VALID${NC}"
    exit 0
  else
    echo -e "${RED}❌ SOME LINKS BROKEN${NC}"
    exit 1
  fi
}

# Entry point
if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <docs_root_directory>"
  exit 1
fi

main "$1"
