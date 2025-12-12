#!/usr/bin/env bash
# validate_all.sh - Master validation runner
#
# Purpose: Run all validation scripts with consolidated reporting
# Usage: ./validate_all.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="$(dirname "$SCRIPT_DIR")"

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo "===================================="
echo "Master Documentation Validator"
echo "===================================="
echo ""

EXIT_CODE=0

# Run metadata validation
echo "[1/3] Validating metadata..."
if ! "$SCRIPT_DIR/validate_metadata.sh" "$DOCS_DIR/protocol"; then
  EXIT_CODE=1
fi
echo ""

# Run ID format validation
echo "[2/3] Validating ID formats..."
if ! "$SCRIPT_DIR/validate_id_format.sh" "$DOCS_DIR/protocol"; then
  EXIT_CODE=1
fi
echo ""

# Run cross-reference validation
echo "[3/3] Validating cross-references..."
if ! "$SCRIPT_DIR/validate_cross_references.sh" "$DOCS_DIR"; then
  EXIT_CODE=1
fi
echo ""

# Final summary
echo "===================================="
if [[ $EXIT_CODE -eq 0 ]]; then
  echo -e "${GREEN}✅ ALL VALIDATIONS PASSED${NC}"
else
  echo -e "${RED}❌ SOME VALIDATIONS FAILED${NC}"
fi
echo "===================================="

exit $EXIT_CODE
