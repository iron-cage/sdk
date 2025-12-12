#!/usr/bin/env bash
# Layer 3: Anti-Workaround Verification (Quick version for pre-commit)
# Fast checks for test-passing shortcuts

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$SCRIPT_DIR/../core/lib.sh"

PROJECT_ROOT="${PROJECT_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"

# Get staged files only
staged_files=$(git diff --cached --name-only --diff-filter=ACM | grep "\.rs$" || true)

if [ -z "$staged_files" ]; then
  log INFO "No Rust files staged, skipping workaround checks"
  exit 0
fi

log INFO "Layer 3 (Quick): Checking staged files for workarounds..."

failures=0

# Check for early returns in test files
if echo "$staged_files" | grep -q "^tests/"; then
  if echo "$staged_files" | grep "^tests/" | xargs grep -l "return;" 2>/dev/null; then
    log ERROR "❌ Early returns found in test files"
    failures=$((failures + 1))
  fi
fi

# Check for panic/unwrap in src files
if echo "$staged_files" | grep -q "^src/\|^module/.*src/"; then
  if echo "$staged_files" | grep "^src/\|^module/.*src/" | \
     xargs grep -n "panic!\|unwrap()" 2>/dev/null | grep -v "test" | grep -v "debug_assert"; then
    log ERROR "❌ panic/unwrap found in production code"
    failures=$((failures + 1))
  fi
fi

if [ "$failures" -eq 0 ]; then
  log INFO "✅ Layer 3 (Quick): PASS"
  exit 0
else
  log ERROR "❌ Layer 3 (Quick): FAIL"
  exit 1
fi
