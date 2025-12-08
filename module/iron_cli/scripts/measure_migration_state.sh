#!/bin/bash
# Measure current migration state with quantitative metrics

echo "========================================="
echo "MIGRATION STATE MEASUREMENT"
echo "========================================="
echo ""

# Category 1: Dependency Patterns
echo "=== Category 1: Dependency Patterns ==="
echo ""

OLD_CLAP_IMPORTS=$(grep -r "use clap::" src/ --include="*.rs" 2>/dev/null | grep -v "cfg(feature = \"legacy_clap\")" | wc -l)
echo "OLD_CLAP_IMPORTS: $OLD_CLAP_IMPORTS (target: 0 after Phase 1)"

NEW_UNILANG_IMPORTS=$(grep -r "use unilang::" src/ --include="*.rs" 2>/dev/null | wc -l)
echo "NEW_UNILANG_IMPORTS: $NEW_UNILANG_IMPORTS (target: 10+ by Phase 9)"

OLD_PARSER_DERIVES=$(grep -r "#\[derive(.*Parser.*)\]" src/ --include="*.rs" 2>/dev/null | grep -v "cfg(feature = \"legacy_clap\")" | wc -l)
echo "OLD_PARSER_DERIVES: $OLD_PARSER_DERIVES (target: 0 after Phase 2)"

OLD_TABLED_USAGE=$(grep -r "use tabled::\|#\[derive(.*Tabled.*)\]" src/ --include="*.rs" 2>/dev/null | grep -v "cfg(feature = \"legacy_clap\")" | wc -l)
echo "OLD_TABLED_USAGE: $OLD_TABLED_USAGE (target: 0 after Phase 3)"

echo ""

# Category 2: Architecture Patterns
echo "=== Category 2: Architecture Patterns ==="
echo ""

if [ -d src/handlers ]; then
  ASYNC_IN_HANDLERS=$(grep -r "async fn" src/handlers/ --include="*.rs" 2>/dev/null | wc -l)
  echo "ASYNC_IN_HANDLERS: $ASYNC_IN_HANDLERS (target: 0 always)"

  PURE_HANDLERS=$(grep -r "pub fn.*HashMap.*String" src/handlers/ --include="*.rs" 2>/dev/null | wc -l)
  echo "PURE_HANDLERS: $PURE_HANDLERS (target: 22 by Phase 2)"

  IO_IN_HANDLERS=$(grep -r "\.await\|tokio::\|reqwest::\|sqlx::" src/handlers/ --include="*.rs" 2>/dev/null | wc -l)
  echo "IO_IN_HANDLERS: $IO_IN_HANDLERS (target: 0 by Phase 4)"
else
  echo "PURE_HANDLERS: 0 (handlers/ not created yet)"
  echo "ASYNC_IN_HANDLERS: N/A"
  echo "IO_IN_HANDLERS: N/A"
fi

if [ -d src/adapters ]; then
  ADAPTER_FUNCTIONS=$(grep -r "pub async fn.*_adapter\|pub async fn" src/adapters/ --include="*.rs" 2>/dev/null | wc -l)
  echo "ADAPTER_FUNCTIONS: $ADAPTER_FUNCTIONS (target: 22 by Phase 4)"
else
  echo "ADAPTER_FUNCTIONS: 0 (adapters/ not created yet)"
fi

echo ""

# Category 3: Command Definitions
echo "=== Category 3: Command Definitions ==="
echo ""

YAML_COMMANDS=$(grep -h "^  - name:" commands/*.yaml 2>/dev/null | wc -l)
echo "YAML_COMMANDS: $YAML_COMMANDS (target: 22 by Phase 1)"

OLD_SUBCOMMANDS=$(grep -r "#\[derive(Subcommand)\]" src/ --include="*.rs" 2>/dev/null | grep -v "cfg(feature = \"legacy_clap\")" | wc -l)
echo "OLD_SUBCOMMANDS: $OLD_SUBCOMMANDS (target: 0 after Phase 2)"

HANDLER_REGISTRATIONS=$(grep -r "registry\.register\|CommandRegistry.*insert" src/ --include="*.rs" 2>/dev/null | wc -l)
echo "HANDLER_REGISTRATIONS: $HANDLER_REGISTRATIONS (target: 22+ by Phase 2)"

echo ""

# Category 4: Documentation Patterns
echo "=== Category 4: Documentation Patterns ==="
echo ""

OLD_FLAG_EXAMPLES=$(grep -r "\-\-username\|\-\-password\|\-\-format" docs/ readme.md 2>/dev/null | wc -l)
echo "OLD_FLAG_EXAMPLES: $OLD_FLAG_EXAMPLES (target: 0 by Phase 8)"

NEW_KEYWORD_EXAMPLES=$(grep -r "username::\|password::\|format::" docs/ readme.md 2>/dev/null | wc -l)
echo "NEW_KEYWORD_EXAMPLES: $NEW_KEYWORD_EXAMPLES (target: >50 by Phase 8)"

echo ""

# Category 5: Test Patterns
echo "=== Category 5: Test Patterns ==="
echo ""

if [ -d tests ]; then
  OLD_CLAP_TESTS=$(grep -r "use clap::" tests/ --include="*.rs" 2>/dev/null | wc -l)
  echo "OLD_CLAP_TESTS: $OLD_CLAP_TESTS (target: 0 by Phase 6)"

  NEW_UNILANG_TESTS=$(grep -r "use unilang::\|\.process_command" tests/ --include="*.rs" 2>/dev/null | wc -l)
  echo "NEW_UNILANG_TESTS: $NEW_UNILANG_TESTS (target: 44+ by Phase 6)"

  PARITY_TESTS=$(grep -r "parity" tests/ --include="*.rs" 2>/dev/null | wc -l)
  echo "PARITY_TESTS: $PARITY_TESTS (target: 22 by Phase 6)"
else
  echo "Tests directory exists but no patterns found yet"
fi

echo ""

# Category 6: File Structure
echo "=== Category 6: File Structure ==="
echo ""

LEGACY_GATED_FILES=$(grep -rl "#\[cfg(feature = \"legacy_clap\")\]" src/ --include="*.rs" 2>/dev/null | wc -l)
echo "LEGACY_GATED_FILES: $LEGACY_GATED_FILES (Phase 1: 3-5, Phase 9: 0)"

YAML_FILES=$(ls commands/*.yaml 2>/dev/null | wc -l)
echo "YAML_FILES: $YAML_FILES (target: 7 by Phase 1)"

BUILD_GENERATED=$(find target/debug/build/iron_cli-*/out -name "*.rs" 2>/dev/null | wc -l)
echo "BUILD_GENERATED_FILES: $BUILD_GENERATED (target: 1+ by Phase 1)"

echo ""

# Category 7: Binary Targets
echo "=== Category 7: Binary Targets ==="
echo ""

BINARY_TARGETS=$(grep -c "^\[\[bin\]\]" Cargo.toml 2>/dev/null || echo 0)
echo "BINARY_TARGETS: $BINARY_TARGETS (Phase 0: 2, Phase 1: 3, Phase 9: 2)"

REQUIRED_FEATURES=$(grep -c "required-features" Cargo.toml 2>/dev/null || echo 0)
echo "REQUIRED_FEATURES: $REQUIRED_FEATURES (Phase 0: 0, Phase 1: 2, Phase 9: 1)"

echo ""
echo "========================================="
echo "PATTERN RATIO ANALYSIS"
echo "========================================="
echo ""

# Calculate Old vs New CLI Patterns
OLD_PATTERNS=$((OLD_CLAP_IMPORTS + OLD_PARSER_DERIVES + OLD_SUBCOMMANDS))
NEW_PATTERNS=$((NEW_UNILANG_IMPORTS + YAML_COMMANDS + HANDLER_REGISTRATIONS))
TOTAL_PATTERNS=$((OLD_PATTERNS + NEW_PATTERNS))

if [ "$TOTAL_PATTERNS" -gt 0 ]; then
  OLD_RATIO=$((OLD_PATTERNS * 100 / TOTAL_PATTERNS))
  NEW_RATIO=$((NEW_PATTERNS * 100 / TOTAL_PATTERNS))

  echo "CLI Pattern Ratio:"
  echo "  OLD: $OLD_PATTERNS patterns ($OLD_RATIO%)"
  echo "  NEW: $NEW_PATTERNS patterns ($NEW_RATIO%)"
  echo ""

  # Determine phase based on ratio
  if [ "$NEW_RATIO" -eq 0 ]; then
    echo "Phase Estimate: 0 (Before migration)"
  elif [ "$NEW_RATIO" -lt 10 ]; then
    echo "Phase Estimate: 1 (Setup)"
  elif [ "$NEW_RATIO" -lt 40 ]; then
    echo "Phase Estimate: 2 (Handlers)"
  elif [ "$NEW_RATIO" -lt 60 ]; then
    echo "Phase Estimate: 3 (Formatters)"
  elif [ "$NEW_RATIO" -lt 80 ]; then
    echo "Phase Estimate: 4-5 (Adapters/Config)"
  elif [ "$NEW_RATIO" -lt 95 ]; then
    echo "Phase Estimate: 6-8 (Tests/Docs)"
  else
    echo "Phase Estimate: 9 (Complete)"
  fi
  echo ""

  # Red flags
  if [ "$OLD_RATIO" -gt 50 ] && [ "$YAML_COMMANDS" -eq 22 ]; then
    echo "⚠️  RED FLAG: YAML complete but old patterns still dominant"
  fi

  if [ "$NEW_RATIO" -lt 10 ] && [ "$YAML_COMMANDS" -gt 0 ]; then
    echo "⚠️  RED FLAG: Migration started but ratio barely shifted"
  fi
else
  echo "No patterns detected yet"
fi

# Architecture Purity
if [ -d src/handlers ]; then
  TOTAL_FUNCTIONS=$((PURE_HANDLERS + ASYNC_IN_HANDLERS + IO_IN_HANDLERS))
  if [ "$TOTAL_FUNCTIONS" -gt 0 ]; then
    PURITY=$((PURE_HANDLERS * 100 / TOTAL_FUNCTIONS))
    echo ""
    echo "Architecture Purity: $PURITY%"

    if [ "$PURITY" -lt 90 ] && [ "$PURE_HANDLERS" -ge 22 ]; then
      echo "⚠️  RED FLAG: All handlers written but purity <90%"
    fi
  fi
fi

# Documentation Coverage
TOTAL_EXAMPLES=$((OLD_FLAG_EXAMPLES + NEW_KEYWORD_EXAMPLES))
if [ "$TOTAL_EXAMPLES" -gt 0 ]; then
  DOC_COVERAGE=$((NEW_KEYWORD_EXAMPLES * 100 / TOTAL_EXAMPLES))
  echo ""
  echo "Documentation Migration: $DOC_COVERAGE%"

  if [ "$DOC_COVERAGE" -lt 100 ] && [ "$PURE_HANDLERS" -ge 22 ]; then
    echo "⚠️  RED FLAG: Implementation complete but docs not updated"
  fi
fi

echo ""
echo "========================================="
echo "MEASUREMENT COMPLETE"
echo "========================================="
