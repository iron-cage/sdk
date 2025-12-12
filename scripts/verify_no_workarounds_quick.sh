#!/bin/bash
# Layer 3: Anti-Workaround - Quick Pre-Commit Check
# This is a fast version for pre-commit hooks that only checks staged files

set -e

echo "Quick Anti-Workaround Check (staged files only)..."

# Get staged files
staged_files=$(git diff --cached --name-only --diff-filter=ACM | grep "\.rs$" || true)

if [ -z "$staged_files" ]; then
  echo "✅ No Rust files staged"
  exit 0
fi

failed=0

# Check 1: No TODO/FIXME in staged production code
for file in $staged_files; do
  if echo "$file" | grep -q "^src/"; then
    if git diff --cached "$file" | grep -i "^+.*TODO:\|^+.*FIXME:\|^+.*XXX:\|^+.*HACK:" > /dev/null; then
      echo "❌ TODO/FIXME found in $file"
      git diff --cached "$file" | grep -n -i "^+.*TODO:\|^+.*FIXME:\|^+.*XXX:\|^+.*HACK:"
      failed=1
    fi
  fi
done

# Check 2: No large blocks of commented code
for file in $staged_files; do
  if git diff --cached "$file" | grep -E "^\+\s*//" | wc -l | awk '$1 > 10 {exit 1}'; then
    :
  else
    echo "❌ Large block of commented code in $file"
    failed=1
  fi
done

# Check 3: No panic/unwrap in staged production code
for file in $staged_files; do
  if echo "$file" | grep -q "^src/"; then
    if git diff --cached "$file" | grep "^+.*panic!\|^+.*\.unwrap()" > /dev/null; then
      echo "⚠️  WARNING: panic/unwrap found in $file"
      git diff --cached "$file" | grep -n "^+.*panic!\|^+.*\.unwrap()"
    fi
  fi
done

if [ "$failed" -eq 0 ]; then
  echo "✅ Quick workaround check passed"
  exit 0
else
  echo "❌ Workarounds detected in staged files"
  exit 1
fi
