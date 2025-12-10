#!/bin/bash
# Documentation Consistency Validation Script
# Ensures all protocol and architecture documentation follows standards

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

DOCS_DIR="$(cd "$(dirname "$0")/../docs" && pwd)"
PROTOCOL_DIR="$DOCS_DIR/protocol"
ARCHITECTURE_DIR="$DOCS_DIR/architecture"

echo "=================================="
echo "Documentation Consistency Validation"
echo "=================================="
echo ""

TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Helper function to run a test
run_test() {
  local test_name="$1"
  local test_result="$2"

  TOTAL_TESTS=$((TOTAL_TESTS + 1))

  if [ "$test_result" -eq 0 ]; then
    echo -e "${GREEN}✅ PASS${NC} - $test_name"
    PASSED_TESTS=$((PASSED_TESTS + 1))
  else
    echo -e "${RED}❌ FAIL${NC} - $test_name"
    FAILED_TESTS=$((FAILED_TESTS + 1))
  fi
}

# Test 1: Protocol title format validation
echo "Test 1: Protocol title format (# Protocol NNN: Name)"
echo "------------------------------------------------------"

PROTOCOL_TITLE_FAILURES=0
for file in "$PROTOCOL_DIR"/[0-9][0-9][0-9]_*.md; do
  if [ -f "$file" ]; then
    filename=$(basename "$file")
    number=$(echo "$filename" | grep -o '^[0-9][0-9][0-9]')

    # Check if first line matches pattern: # Protocol NNN: Name
    first_line=$(head -n 1 "$file")
    if ! echo "$first_line" | grep -q "^# Protocol $number:"; then
      echo "  ❌ $filename: Title doesn't match pattern '# Protocol $number: Name'"
      echo "     Found: $first_line"
      PROTOCOL_TITLE_FAILURES=$((PROTOCOL_TITLE_FAILURES + 1))
    fi
  fi
done

if [ "$PROTOCOL_TITLE_FAILURES" -eq 0 ]; then
  run_test "Protocol titles follow standard format" 0
else
  echo "  Found $PROTOCOL_TITLE_FAILURES files with incorrect title format"
  run_test "Protocol titles follow standard format" 1
fi
echo ""

# Test 2: Architecture title format validation (no "Protocol" prefix, no number)
echo "Test 2: Architecture title format (# Name, no prefix)"
echo "------------------------------------------------------"

ARCH_TITLE_FAILURES=0
for file in "$ARCHITECTURE_DIR"/[0-9][0-9][0-9]_*.md; do
  if [ -f "$file" ]; then
    filename=$(basename "$file")

    # Check if first line is just a title (no "Protocol" or number prefix)
    first_line=$(head -n 1 "$file")
    if echo "$first_line" | grep -q "^# Protocol"; then
      echo "  ❌ $filename: Title shouldn't have 'Protocol' prefix"
      echo "     Found: $first_line"
      ARCH_TITLE_FAILURES=$((ARCH_TITLE_FAILURES + 1))
    elif echo "$first_line" | grep -q "^# [0-9]"; then
      echo "  ❌ $filename: Title shouldn't have number prefix"
      echo "     Found: $first_line"
      ARCH_TITLE_FAILURES=$((ARCH_TITLE_FAILURES + 1))
    fi
  fi
done

if [ "$ARCH_TITLE_FAILURES" -eq 0 ]; then
  run_test "Architecture titles follow standard format" 0
else
  echo "  Found $ARCH_TITLE_FAILURES files with incorrect title format"
  run_test "Architecture titles follow standard format" 1
fi
echo ""

# Test 3: Metadata block presence (Status, Version, Last Updated)
echo "Test 3: Metadata blocks present in all docs"
echo "------------------------------------------------------"

METADATA_FAILURES=0
for dir in "$PROTOCOL_DIR" "$ARCHITECTURE_DIR"; do
  for file in "$dir"/[0-9][0-9][0-9]_*.md; do
    if [ -f "$file" ]; then
      filename=$(basename "$file")

      # Check for required metadata fields in first 20 lines
      head -n 20 "$file" > /tmp/doc_header_$$

      if ! grep -q "^\*\*Status:\*\*" /tmp/doc_header_$$; then
        echo "  ❌ $filename: Missing **Status:** field"
        METADATA_FAILURES=$((METADATA_FAILURES + 1))
      fi

      if ! grep -q "^\*\*Version:\*\*" /tmp/doc_header_$$; then
        echo "  ❌ $filename: Missing **Version:** field"
        METADATA_FAILURES=$((METADATA_FAILURES + 1))
      fi

      if ! grep -q "^\*\*Last Updated:\*\*" /tmp/doc_header_$$; then
        echo "  ❌ $filename: Missing **Last Updated:** field"
        METADATA_FAILURES=$((METADATA_FAILURES + 1))
      fi

      rm /tmp/doc_header_$$
    fi
  done
done

if [ "$METADATA_FAILURES" -eq 0 ]; then
  run_test "All docs have required metadata blocks" 0
else
  echo "  Found $METADATA_FAILURES missing metadata fields"
  run_test "All docs have required metadata blocks" 1
fi
echo ""

# Test 4: Version format validation (X.Y.Z or N/A)
echo "Test 4: Version format validation"
echo "------------------------------------------------------"

VERSION_FAILURES=0
for dir in "$PROTOCOL_DIR" "$ARCHITECTURE_DIR"; do
  for file in "$dir"/[0-9][0-9][0-9]_*.md; do
    if [ -f "$file" ]; then
      filename=$(basename "$file")

      # Extract version value
      version_line=$(grep "^\*\*Version:\*\*" "$file" || true)
      if [ -n "$version_line" ]; then
        version=$(echo "$version_line" | sed 's/\*\*Version:\*\* *//')

        # Check if version matches X.Y.Z pattern or is N/A
        if ! echo "$version" | grep -qE '^[0-9]+\.[0-9]+\.[0-9]+$|^N/A$'; then
          echo "  ❌ $filename: Invalid version format '$version' (should be X.Y.Z or N/A)"
          VERSION_FAILURES=$((VERSION_FAILURES + 1))
        fi
      fi
    fi
  done
done

if [ "$VERSION_FAILURES" -eq 0 ]; then
  run_test "All versions follow X.Y.Z or N/A format" 0
else
  echo "  Found $VERSION_FAILURES files with invalid version format"
  run_test "All versions follow X.Y.Z or N/A format" 1
fi
echo ""

# Test 5: Date format validation (YYYY-MM-DD)
echo "Test 5: Date format validation"
echo "------------------------------------------------------"

DATE_FAILURES=0
for dir in "$PROTOCOL_DIR" "$ARCHITECTURE_DIR"; do
  for file in "$dir"/[0-9][0-9][0-9]_*.md; do
    if [ -f "$file" ]; then
      filename=$(basename "$file")

      # Extract date value
      date_line=$(grep "^\*\*Last Updated:\*\*" "$file" || true)
      if [ -n "$date_line" ]; then
        date_value=$(echo "$date_line" | sed 's/\*\*Last Updated:\*\* *//')

        # Check if date matches YYYY-MM-DD pattern
        if ! echo "$date_value" | grep -qE '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
          echo "  ❌ $filename: Invalid date format '$date_value' (should be YYYY-MM-DD)"
          DATE_FAILURES=$((DATE_FAILURES + 1))
        fi
      fi
    fi
  done
done

if [ "$DATE_FAILURES" -eq 0 ]; then
  run_test "All dates follow YYYY-MM-DD format" 0
else
  echo "  Found $DATE_FAILURES files with invalid date format"
  run_test "All dates follow YYYY-MM-DD format" 1
fi
echo ""

# Test 6: Protocol numbering continuity
echo "Test 6: Protocol numbering continuity"
echo "------------------------------------------------------"

EXPECTED_PROTOCOLS=(
  "002_rest_api_protocol.md"
  "003_websocket_protocol.md"
  "005_budget_control_protocol.md"
  "006_token_management_api.md"
  "007_authentication_api.md"
  "008_user_management_api.md"
  "009_reserved.md"
  "010_agents_api.md"
  "011_providers_api.md"
  "012_analytics_api.md"
  "013_budget_limits_api.md"
  "014_api_tokens_api.md"
  "015_projects_api.md"
  "016_settings_api.md"
  "017_budget_requests_api.md"
)

NUMBERING_FAILURES=0
for protocol in "${EXPECTED_PROTOCOLS[@]}"; do
  if [ ! -f "$PROTOCOL_DIR/$protocol" ]; then
    echo "  ❌ Missing expected protocol: $protocol"
    NUMBERING_FAILURES=$((NUMBERING_FAILURES + 1))
  fi
done

if [ "$NUMBERING_FAILURES" -eq 0 ]; then
  run_test "Protocol numbering is continuous" 0
else
  echo "  Found $NUMBERING_FAILURES missing protocols"
  run_test "Protocol numbering is continuous" 1
fi
echo ""

# Test 7: Architecture numbering continuity (001-009)
echo "Test 7: Architecture numbering continuity"
echo "------------------------------------------------------"

EXPECTED_ARCHITECTURE=(
  "001_execution_models.md"
  "002_layer_model.md"
  "003_service_boundaries.md"
  "004_data_flow.md"
  "005_service_integration.md"
  "006_roles_and_permissions.md"
  "007_entity_model.md"
  "008_runtime_modes.md"
  "009_resource_catalog.md"
)

ARCH_NUMBERING_FAILURES=0
for arch in "${EXPECTED_ARCHITECTURE[@]}"; do
  if [ ! -f "$ARCHITECTURE_DIR/$arch" ]; then
    echo "  ❌ Missing expected architecture doc: $arch"
    ARCH_NUMBERING_FAILURES=$((ARCH_NUMBERING_FAILURES + 1))
  fi
done

if [ "$ARCH_NUMBERING_FAILURES" -eq 0 ]; then
  run_test "Architecture numbering is continuous (001-009)" 0
else
  echo "  Found $ARCH_NUMBERING_FAILURES missing architecture docs"
  run_test "Architecture numbering is continuous (001-009)" 1
fi
echo ""

# Test 8: No broken internal links (basic check)
echo "Test 8: Internal link validation"
echo "------------------------------------------------------"

BROKEN_LINKS=0
for dir in "$PROTOCOL_DIR" "$ARCHITECTURE_DIR"; do
  for file in "$dir"/*.md; do
    if [ -f "$file" ] && [ "$(basename "$file")" != "readme.md" ]; then
      # Find all markdown links to local files
      while IFS= read -r link; do
        # Extract the file path from markdown link [text](path)
        filepath=$(echo "$link" | sed -n 's/.*](\([^)]*\)).*/\1/p' | head -1)

        # Skip external URLs, anchors, and empty links
        if [[ "$filepath" =~ ^https?:// ]] || [[ "$filepath" =~ ^# ]] || [ -z "$filepath" ]; then
          continue
        fi

        # Resolve relative path
        link_dir=$(dirname "$file")
        target="$link_dir/$filepath"

        # Check if target exists
        if [ ! -f "$target" ] && [ ! -d "$target" ]; then
          echo "  ❌ $(basename "$file"): Broken link to $filepath"
          BROKEN_LINKS=$((BROKEN_LINKS + 1))
        fi
      done < <(grep -o '\[.\{1,\}\](.\{1,\})' "$file" || true)
    fi
  done
done

if [ "$BROKEN_LINKS" -eq 0 ]; then
  run_test "No broken internal links detected" 0
else
  echo "  Found $BROKEN_LINKS broken links"
  run_test "No broken internal links detected" 1
fi
echo ""

# Summary
echo "=================================="
echo "Summary"
echo "=================================="
echo "Total Tests: $TOTAL_TESTS"
echo -e "${GREEN}Passed: $PASSED_TESTS${NC}"
if [ "$FAILED_TESTS" -gt 0 ]; then
  echo -e "${RED}Failed: $FAILED_TESTS${NC}"
else
  echo "Failed: $FAILED_TESTS"
fi
echo ""

if [ "$FAILED_TESTS" -eq 0 ]; then
  echo -e "${GREEN}✅ All documentation consistency tests passed!${NC}"
  exit 0
else
  echo -e "${RED}❌ Some tests failed. Please fix the issues above.${NC}"
  exit 1
fi
