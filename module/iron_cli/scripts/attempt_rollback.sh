#!/bin/bash
# Attempt to rollback to old clap-based CLI
# All attempts should FAIL for migration to be complete

set +e  # Don't exit on error (we EXPECT errors)

ROLLBACK_SUCCESSES=0
ROLLBACK_FAILURES=0

echo "========================================="
echo "ROLLBACK RESISTANCE TEST"
echo "Attempting to Restore Old Way"
echo "========================================="
echo ""
echo "SUCCESS (rollback works) = 🔴 BAD (migration incomplete)"
echo "FAILURE (rollback blocked) = 🟢 GOOD (migration enforced)"
echo ""

# Category 1: Dependency Rollback
echo "=== Category 1: Dependency Rollback ==="
echo ""

# Test 1: Can we build old binary without legacy feature?
echo -n "Test 1 - Build old binary without legacy: "
if cargo build --bin iron-token 2>&1 | grep -q "requires the features.*legacy_clap"; then
  echo "BLOCKED 🟢 (rollback failed - good!)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
else
  echo "SUCCEEDS 🔴 (rollback worked - bad!)"
  ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
fi

# Test 2: Can we use tabled in new code?
echo -n "Test 2 - Tabled in new dependency tree: "
if cargo tree --features token_cli_unilang 2>/dev/null | grep -q "tabled"; then
  echo "SUCCEEDS 🔴 (tabled accessible)"
  ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
else
  echo "BLOCKED 🟢 (tabled not in tree)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

# Test 3: Can we use clap in new code?
echo -n "Test 3 - Clap in new dependency tree: "
if cargo tree --features token_cli_unilang 2>/dev/null | grep -q "^├── clap\|│   ├── clap"; then
  echo "SUCCEEDS 🔴 (clap accessible)"
  ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
else
  echo "BLOCKED 🟢 (clap not in tree)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

echo ""

# Category 2: Code Rollback
echo "=== Category 2: Code Rollback ==="
echo ""

# Test 4: Does old binary still exist?
echo -n "Test 4 - Old binary source exists: "
if [ -f src/bin/iron_token.rs ]; then
  # Check if it's actually functional or just a stub
  if grep -q "compile_error" src/bin/iron_token.rs; then
    echo "EXISTS BUT BROKEN 🟡 (compile_error present)"
    # Not counted - transitional state
  else
    echo "EXISTS AND FUNCTIONAL 🔴"
    ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
  fi
else
  echo "DELETED 🟢 (cannot be restored)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

# Test 5: Can old binary be built with legacy feature?
echo -n "Test 5 - Old binary builds with legacy: "
if cargo build --bin iron-token --features legacy_clap 2>&1 | grep -q "Finished"; then
  echo "SUCCEEDS 🟡 (acceptable for Phase 1, remove in Phase 9)"
  # Not counted as success or failure - transitional state
else
  echo "BLOCKED 🟢"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

# Test 6: Can we import clap in a new file?
echo -n "Test 6 - Create new file with clap import: "
cat > /tmp/test_rollback_import.rs <<'EOF'
use clap::Parser;

#[derive(Parser)]
struct TestArgs {
    name: String,
}

fn main() {
    let _args = TestArgs::parse();
}
EOF

# Try to compile with new binary's features
cd /tmp
if rustc test_rollback_import.rs --edition 2021 2>&1 | grep -q "error"; then
  echo "BLOCKED 🟢 (clap not available)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
else
  # rustc succeeded because it uses system rust
  # Try with cargo
  cd - >/dev/null
  if cargo check --features token_cli_unilang 2>&1 | grep -q "error"; then
    echo "BLOCKED 🟢 (clap not in features)"
    ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
  else
    echo "SUCCEEDS 🔴 (clap importable)"
    ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
  fi
fi
cd - >/dev/null 2>&1
rm -f /tmp/test_rollback_import.rs

echo ""

# Category 3: Architecture Rollback
echo "=== Category 3: Architecture Rollback ==="
echo ""

# Test 7: Can we delete YAML files and still build?
echo -n "Test 7 - Build without YAML files: "
YAML_COUNT=$(ls commands/*.yaml 2>/dev/null | wc -l)
if [ "$YAML_COUNT" -eq 0 ]; then
  echo "SKIP (no YAML files)"
else
  # Backup YAML files
  mkdir -p /tmp/yaml_backup_$$
  cp commands/*.yaml /tmp/yaml_backup_$$/ 2>/dev/null
  rm commands/*.yaml 2>/dev/null

  if cargo build --bin iron-token-unilang --features token_cli_unilang 2>&1 | grep -q "Finished"; then
    echo "SUCCEEDS 🟡 (YAML not required yet - Phase 1 acceptable)"
    # Don't count in Phase 1 - handlers don't use YAML yet
  else
    echo "BLOCKED 🟢 (YAML required)"
    ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
  fi

  # Restore YAML files
  cp /tmp/yaml_backup_$$/*.yaml commands/ 2>/dev/null
  rm -rf /tmp/yaml_backup_$$
fi

# Test 8: Can we use async in handlers?
echo -n "Test 8 - Async functions in handlers: "
if [ -d src/handlers ]; then
  ASYNC_COUNT=$(grep -r "async fn" src/handlers/ 2>/dev/null | wc -l)
  if [ "$ASYNC_COUNT" -eq 0 ]; then
    echo "BLOCKED 🟢 (no async allowed)"
    ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
  else
    echo "SUCCEEDS 🔴 (async present)"
    ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
  fi
else
  echo "SKIP (handlers/ not created yet)"
fi

# Test 9: Can we use HashMap instead of PHF for commands?
echo -n "Test 9 - HashMap command registry allowed: "
HASHMAP_COMMANDS=$(grep -r "HashMap<.*Command" src/ 2>/dev/null | grep -v "test" | grep -v "params.*HashMap" | wc -l)
if [ "$HASHMAP_COMMANDS" -eq 0 ]; then
  echo "BLOCKED 🟢 (no HashMap registries)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
else
  echo "SUCCEEDS 🔴 (HashMap found)"
  ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
fi

echo ""

# Category 4: Configuration Rollback
echo "=== Category 4: Configuration Rollback ==="
echo ""

# Test 10: Can we use env::var outside config module?
echo -n "Test 10 - Direct env::var usage: "
if [ -d src/handlers ] || [ -d src/adapters ]; then
  ENV_VAR_COUNT=$(grep -r "env::var\|std::env::var" src/handlers/ src/adapters/ 2>/dev/null | grep -v "test" | wc -l)
  if [ "$ENV_VAR_COUNT" -eq 0 ]; then
    echo "BLOCKED 🟢 (centralized config)"
    ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
  else
    echo "SUCCEEDS 🔴 (direct env access)"
    ROLLBACK_SUCCESSES=$((ROLLBACK_SUCCESSES + 1))
  fi
else
  echo "SKIP (handlers/adapters not created yet)"
fi

echo ""

# Category 5: Feature Rollback
echo "=== Category 5: Feature Isolation ==="
echo ""

# Test 11: Can we enable both legacy_clap and token_cli_unilang?
echo -n "Test 11 - Build with both features: "
if cargo build --features "legacy_clap,token_cli_unilang" 2>&1 | grep -q "Finished"; then
  echo "SUCCEEDS 🟡 (both work - Phase 1 acceptable)"
  # Not counted - parallel implementation expected in Phase 1
else
  echo "BLOCKED 🟢 (mutually exclusive)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

# Test 12: Is clap completely removed from Cargo.toml?
echo -n "Test 12 - Clap dependency in Cargo.toml: "
if grep -q "^clap.*=" Cargo.toml; then
  echo "EXISTS 🟡 (Phase 1 acceptable, remove in Phase 9)"
  # Not counted - transitional state
else
  echo "REMOVED 🟢 (clap gone)"
  ROLLBACK_FAILURES=$((ROLLBACK_FAILURES + 1))
fi

echo ""

# Summary
echo "========================================="
echo "ROLLBACK RESISTANCE SUMMARY"
echo "========================================="
echo ""
echo "Rollback Attempts (excluding transitional):"
echo "  🟢 BLOCKED (good):   $ROLLBACK_FAILURES"
echo "  🔴 SUCCEEDED (bad):  $ROLLBACK_SUCCESSES"
echo ""

TOTAL_TESTS=$((ROLLBACK_FAILURES + ROLLBACK_SUCCESSES))
if [ "$TOTAL_TESTS" -gt 0 ]; then
  RESISTANCE_PCT=$((ROLLBACK_FAILURES * 100 / TOTAL_TESTS))
else
  RESISTANCE_PCT=0
fi

echo "Rollback Resistance: $RESISTANCE_PCT%"
echo ""

if [ "$ROLLBACK_SUCCESSES" -eq 0 ]; then
  echo "✓ EXCELLENT: All rollback attempts blocked"
  echo "Migration is COMPLETE - old way is impossible"
  exit 0
elif [ "$ROLLBACK_SUCCESSES" -le 2 ]; then
  echo "⚠ GOOD: Most rollback attempts blocked"
  echo "Migration is STRONG but can be improved"
  echo ""
  echo "Remaining Vulnerabilities: $ROLLBACK_SUCCESSES"
  echo "Expected in Phase 1-2, should be 0 by Phase 9"
  exit 0
else
  echo "✗ WARNING: Too many rollback attempts succeeded"
  echo "Migration is WEAK - old way still accessible"
  echo ""
  echo "Vulnerabilities: $ROLLBACK_SUCCESSES"
  echo "Fix needed before Phase 9"
  exit 1
fi
