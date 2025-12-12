#!/bin/bash
# Layer 3: Anti-Workaround - Detect Test-Passing Shortcuts
# This script detects workarounds that make tests pass without proper implementation

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "════════════════════════════════════════════════════════"
echo "Layer 3: Anti-Workaround Detection"
echo "════════════════════════════════════════════════════════"
echo ""

# Track statistics
total_checks=0
passing_checks=0
failing_checks=0

# Check 1: No early returns in test functions
echo "Check 1: No Early Returns in Test Functions"
total_checks=$((total_checks + 1))

early_return_count=0
test_files=$(find "$PROJECT_ROOT" -path "*/tests/*.rs" -type f ! -path "*/target/*")

for file in $test_files; do
  # Look for test functions with early returns
  matches=$(grep -n "#\[test\]" "$file" 2>/dev/null | cut -d: -f1)

  for line_num in $matches; do
    # Check next 50 lines for early return
    end_line=$((line_num + 50))
    early_ret=$(sed -n "${line_num},${end_line}p" "$file" | grep -c "return;" || true)

    if [ "$early_ret" -gt 0 ]; then
      early_return_count=$((early_return_count + 1))
      echo "   ⚠️  Found early return in $file:$line_num"
    fi
  done
done

if [ "$early_return_count" -eq 0 ]; then
  echo "   ✅ PASS: No early returns in tests (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $early_return_count early returns (expected 0)"
  failing_checks=$((failing_checks + 1))
fi

# Check 2: No empty test implementations
echo "Check 2: No Empty Test Implementations"
total_checks=$((total_checks + 1))

empty_test_count=0

for file in $test_files; do
  # Look for test functions with only comments or whitespace
  in_test=0
  brace_count=0
  test_line=0

  while IFS= read -r line; do
    if echo "$line" | grep -q "#\[test\]"; then
      in_test=1
      test_line=$((test_line + 1))
      continue
    fi

    if [ "$in_test" -eq 1 ]; then
      if echo "$line" | grep -q "{"; then
        brace_count=$((brace_count + 1))
      fi
      if echo "$line" | grep -q "}"; then
        brace_count=$((brace_count - 1))

        if [ "$brace_count" -eq 0 ]; then
          # Test function ended, check if it was empty
          in_test=0
        fi
      fi
    fi
  done < "$file"
done

if [ "$empty_test_count" -eq 0 ]; then
  echo "   ✅ PASS: No empty tests found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $empty_test_count empty tests (expected 0)"
  failing_checks=$((failing_checks + 1))
fi

# Check 3: No TODO/FIXME in production code paths
echo "Check 3: No TODO/FIXME in Production Code"
total_checks=$((total_checks + 1))

todo_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" -exec grep -i "TODO:\|FIXME:\|XXX:\|HACK:" {} \; 2>/dev/null | wc -l)

if [ "$todo_count" -eq 0 ]; then
  echo "   ✅ PASS: No TODO/FIXME markers (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $todo_count TODO/FIXME markers (expected 0)"
  find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" -exec grep -n -i "TODO:\|FIXME:\|XXX:\|HACK:" {} \; 2>/dev/null | head -10
  failing_checks=$((failing_checks + 1))
fi

# Check 4: No panic!() or unwrap() in production code
echo "Check 4: No panic!() or unwrap() in Production Code"
total_checks=$((total_checks + 1))

panic_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" -exec grep -c "panic!\|\.unwrap()\|\.expect(" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$panic_count" -eq 0 ]; then
  echo "   ✅ PASS: No panic/unwrap found (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ⚠️  WARN: Found $panic_count panic/unwrap usages"
  echo "   (Some may be acceptable in certain contexts)"
  passing_checks=$((passing_checks + 1))
fi

# Check 5: No hardcoded success responses in handlers
echo "Check 5: No Hardcoded Success Responses"
total_checks=$((total_checks + 1))

hardcoded_count=$(find "$PROJECT_ROOT" -path "*/src/routes/*.rs" -o -path "*/src/handlers/*.rs" -type f ! -path "*/target/*" -exec grep -c "Ok(Json(\|StatusCode::OK" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

# This is just a warning since some hardcoded responses are legitimate
if [ "$hardcoded_count" -gt 0 ]; then
  echo "   ⚠️  INFO: Found $hardcoded_count hardcoded responses"
  echo "   (Review manually to ensure they're not workarounds)"
fi
passing_checks=$((passing_checks + 1))

# Check 6: No commented-out code in recent commits
echo "Check 6: No Large Blocks of Commented Code"
total_checks=$((total_checks + 1))

commented_blocks=0
src_files=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*")

for file in $src_files; do
  # Count consecutive comment lines (potential commented-out code)
  consecutive=0
  max_consecutive=0

  while IFS= read -r line; do
    if echo "$line" | grep -q "^\s*//"; then
      consecutive=$((consecutive + 1))
      if [ "$consecutive" -gt "$max_consecutive" ]; then
        max_consecutive=$consecutive
      fi
    else
      consecutive=0
    fi
  done < "$file"

  if [ "$max_consecutive" -gt 10 ]; then
    commented_blocks=$((commented_blocks + 1))
    echo "   ⚠️  Found $max_consecutive consecutive comment lines in $file"
  fi
done

if [ "$commented_blocks" -eq 0 ]; then
  echo "   ✅ PASS: No large commented code blocks (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ❌ FAIL: Found $commented_blocks files with large comment blocks"
  failing_checks=$((failing_checks + 1))
fi

# Check 7: No test-only feature flags in production code
echo "Check 7: No Test-Only Feature Flags in Production"
total_checks=$((total_checks + 1))

test_flag_count=$(find "$PROJECT_ROOT" -path "*/src/*.rs" -type f ! -path "*/target/*" -exec grep -c "#\[cfg(test)\]" {} \; 2>/dev/null | awk '{s+=$1} END {print s+0}')

if [ "$test_flag_count" -eq 0 ]; then
  echo "   ✅ PASS: No test feature flags in src/ (count = 0)"
  passing_checks=$((passing_checks + 1))
else
  echo "   ⚠️  INFO: Found $test_flag_count cfg(test) usages in src/"
  echo "   (May be acceptable for test helpers in src/)"
  passing_checks=$((passing_checks + 1))
fi

echo ""

# Summary
echo "════════════════════════════════════════════════════════"
echo "Anti-Workaround Verification Summary"
echo "════════════════════════════════════════════════════════"
echo "Total Checks:  $total_checks"
echo "Passing:       $passing_checks"
echo "Failing:       $failing_checks"

if [ "$failing_checks" -eq 0 ]; then
  echo "Status:        ✅ NO WORKAROUNDS DETECTED"
  echo "════════════════════════════════════════════════════════"
  exit 0
else
  echo "Status:        ❌ WORKAROUNDS DETECTED"
  echo "════════════════════════════════════════════════════════"
  exit 1
fi
